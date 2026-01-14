/*
 * Copyright (C) 2019 University of Rochester. All rights reserved.
 * Licenced under the MIT licence. See LICENSE file in the project root for
 * details. 
 */

#include "ralloc.hpp"

#include <string>
#include <sys/mman.h>
#include <functional>
#include <fcntl.h>
#include <atomic>
#include <vector>
#include <algorithm>
#include <cstring>
#include <unistd.h>

#include "RegionManager.hpp"
#include "BaseMeta.hpp"
#include "SizeClass.hpp"
#include "pm_config.hpp"

using namespace std;

namespace ralloc{
    bool initialized = false;
    /* persistent metadata and their layout */
    BaseMeta* base_md;
    Regions* _rgs;
    std::function<void(const CrossPtr<char, SB_IDX>&, GarbageCollection&)> roots_filter_func[MAX_ROOTS];
    extern SizeClass sizeclass;
};
using namespace ralloc;
extern void public_flush_cache();

#define CXL_PCIE_BAR_PATH  "/sys/devices/pci0000:16/0000:16:00.1/resource2"
#define CSR_RD_BUFF 13
#define CSR_WR_BUFF 14

int init_csr(uint64_t **pci_vaddr) {
    uint64_t *ptr;
    int fd;

    fd = open(CXL_PCIE_BAR_PATH, O_RDWR | O_SYNC);
    if(fd == -1){
        return -1;
    }

    ptr = (uint64_t*)mmap(0, (1 << 21), PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);     // 2MB
    if(ptr == (void *) -1){
        close(fd);
        return -1;
    }
    if(ptr == (void *) 0){
        close(fd);
        return -1;
    }

    *pci_vaddr = ptr;
    return 0;
}

int get_addr_from_kmod(
        const char* file_name,
        uint64_t byte_size,
        uint64_t* ret_pa_val,
        uint64_t** ret_va_ptr,
        uint64_t* pci_vaddr_ptr,
        int csr_num) {
    int         kmod_fd = 0;
    ssize_t     bytes_read = 0;

    if (file_name == NULL) {
        return -1;
    }
    kmod_fd = open(file_name, O_RDWR); // FIXED: we can't open the file in read-only mode if we want to map it for both read and write
    if (kmod_fd == -1) {
        goto FAILED;
    }
    if ((bytes_read = read(kmod_fd, ret_pa_val, sizeof(uint64_t))) < 0) {
        goto FAILED;
    }
    if (csr_num > 0) {
        pci_vaddr_ptr[csr_num] = *ret_pa_val;
    }
    *ret_va_ptr = (uint64_t*)mmap(NULL, byte_size, 
            PROT_READ | PROT_WRITE, MAP_SHARED, kmod_fd, 0); 

    if(*ret_va_ptr == (void *) -1){
        goto FAILED;
    }
    if(*ret_va_ptr == (void *) 0){
        goto FAILED;
    }
    return 0;
FAILED:
    return -1;
}

uint64_t* rd_buff_vaddr = nullptr;
uint64_t rd_buff_paddr = 0;

uint64_t* wr_buff_vaddr = nullptr;
uint64_t wr_buff_paddr = 0;

uint64_t* target_buff_vaddr = nullptr;
uint64_t target_buff_paddr = 0;

int init_mcas() {
    uint64_t TGT_SIZE = 64 * 1024 * 1024;
    uint64_t RD_SIZE = 64 * 1024;
    uint64_t WR_SIZE = 64 * 1024;

    int init_ok = 0;
    uint64_t* pci_vaddr = nullptr;

    init_ok = init_csr(&pci_vaddr);
    if (init_ok) {
        return -1;
    }

    init_ok =
        get_addr_from_kmod("/proc/mcas_target_buff", TGT_SIZE, &target_buff_paddr,
                            &target_buff_vaddr, pci_vaddr, -1);
    if (init_ok) {
        return -1;
    }

    rd_buff_paddr =
        (uint64_t)(((uint64_t) target_buff_paddr) + TGT_SIZE - RD_SIZE);
    rd_buff_vaddr =
        (uint64_t *)(((uint64_t) target_buff_vaddr) + TGT_SIZE - RD_SIZE);
    pci_vaddr[CSR_RD_BUFF] = rd_buff_paddr;

    wr_buff_paddr =
        (uint64_t)(((uint64_t) target_buff_paddr) + TGT_SIZE - RD_SIZE - WR_SIZE);
    wr_buff_vaddr = (uint64_t *)(((uint64_t) target_buff_vaddr) + TGT_SIZE -
                                    RD_SIZE - WR_SIZE);
    pci_vaddr[CSR_WR_BUFF] = wr_buff_paddr;
    return 0;
}

static inline void movdir64b_addr(uint64_t* content_addr, uint64_t* wr_addr) {
    asm volatile(
        "mov %[wr_addr], %%r10\n"
        "mov %[content_addr], %%r9\n"
        "movdir64b 0x0(%%r9), %%r10 \n"
        "sfence \n"
        :
        :[wr_addr] "r" (wr_addr), [content_addr] "r" (content_addr)
        :"r9", "r10"
    );
}

static inline void mcas_rd_nc(uint64_t* A, volatile uint64_t* B) {
    __asm__ __volatile__ (
        ".intel_syntax noprefix\n\t"
        "movdqu xmm0, [rdi]\n\t"     // Load 64 bits from A into xmm0
        "movdqu [rsi], xmm0\n\t"     // Store 64 bits from xmm0 into B
        ".att_syntax prefix\n"
        :
        : "D"(A), "S"(B)
        : "xmm0"
    );
}

uint64_t mcas(uint64_t tid, uint64_t* address, uint64_t* compare, uint64_t exchange) {
    uint64_t* wr_global = wr_buff_vaddr + (tid * 2 * 8);
    uint64_t* rd_global = rd_buff_vaddr + (tid * 2 * 8);

    alignas(64) uint64_t wr_local_buffer[8];
    wr_local_buffer[0] = *compare;
    wr_local_buffer[1] = exchange;
    wr_local_buffer[2] = ((uint64_t) address) - ((uint64_t) target_buff_vaddr) + ((uint64_t) target_buff_paddr);
    wr_local_buffer[3] = tid * 2;

    movdir64b_addr(wr_local_buffer, wr_global);

    alignas(16) uint64_t rd_local_buffer[2];
    mcas_rd_nc(rd_global, rd_local_buffer);

    uint64_t out = rd_local_buffer[0];
    uint64_t success = rd_local_buffer[1];

    if (success == 0 && out == 0) {
        *compare = *address;
    } else {
        *compare = out;
    }
    return success;
}

void mcas_store(uint64_t tid, uint64_t* address, uint64_t value) {
    uint64_t compare = *address;
    uint64_t success = 0;
    while (success == 0) {
        success = mcas(tid, address, &compare, value);
    }
}

int _RP_init(const char* id, uint64_t size){
    // thread_num = thd_num;

    // reinitialize global variables in case they haven't
    new (&sizeclass) SizeClass();

    assert(sizeof(Descriptor) == DESCSIZE); // check desc size
    assert(size < MAX_SB_REGION_SIZE && size >= MIN_SB_REGION_SIZE); // ensure user input is >=MAX_SB_REGION_SIZE
    uint64_t num_sb = size/SBSIZE;
    bool restart;
    _rgs = new Regions();

    init_mcas();
    // uint64_t dummy = 0;
    // mcas(0, target_buff_vaddr, &dummy, 1);

    std::cout << rd_buff_vaddr << std::endl;
    std::cout << rd_buff_paddr << std::endl;
    std::cout << wr_buff_vaddr << std::endl;
    std::cout << wr_buff_paddr << std::endl;
    std::cout << target_buff_vaddr << std::endl;
    std::cout << target_buff_paddr << std::endl;

    char* temp;

    for(int i=0; i<LAST_IDX;i++){
    switch(i){
    case DESC_IDX:
        // temp = (char*) malloc(1 + strlen(id) + 5 + 1);
        // strcpy(temp, "/");
        // strcat(temp, id);
        // strcat(temp, "_desc");
        //
        // _rgs->create(temp, num_sb*DESCSIZE, true, true);

        _rgs->set(num_sb*DESCSIZE, (char*) malloc(num_sb*DESCSIZE));
        break;
    case SB_IDX:
        temp = (char*) malloc(1 + strlen(id) + 3 + 1);
        strcpy(temp, "/");
        strcat(temp, id);
        strcat(temp, "_sb");

        _rgs->create(temp, num_sb*SBSIZE, true, false);
        break;
    case META_IDX:
        temp = (char*) malloc(1 + strlen(id) + 7 + 1);
        strcpy(temp, "/");
        strcat(temp, id);
        strcat(temp, "_basemd");
        restart = exists_test(temp);
        base_md = _rgs->create_for<BaseMeta>(temp, sizeof(BaseMeta), true);
        break;
    } // switch
    }
    initialized = true;
    return (int)restart;
}

struct RallocHolder{
    int init_ret_val;
    RallocHolder(const char* _id, uint64_t size){
        init_ret_val = _RP_init(_id,size);
    }
    ~RallocHolder(){
        // #ifndef MEM_CONSUME_TEST
        // flush_region would affect the memory consumption result (rss) and 
        // thus is disabled for benchmark testing. To enable, simply comment out
        // -DMEM_CONSUME_TEST flag in Makefile.
        _rgs->flush_region(DESC_IDX);
        _rgs->flush_region(SB_IDX);
        // #endif
        base_md->writeback();
        initialized = false;
        delete _rgs;
    }
};

/* 
 * mmap the existing heap file corresponding to id. aka restart,
 * 		and if multiple heaps exist, print out and let user select;
 * if such a heap doesn't exist, create one. aka start.
 * id is the distinguishable identity of applications.
 */
int RP_init(const char* _id, uint64_t size){
    static RallocHolder _holder(_id,size);
    return _holder.init_ret_val;
}

int RP_recover(){
    return (int) base_md->restart();
}

// we assume RP_close is called by the last exiting thread.
void RP_close(){
    // Wentao: this is a noop as the real function body is now i ~RallocHolder
}

void* RP_malloc(size_t sz){
    assert(initialized&&"RPMalloc isn't initialized!");
    return base_md->do_malloc(sz);
}

void RP_free(void* ptr){
    assert(initialized&&"RPMalloc isn't initialized!");
    base_md->do_free(ptr);
}

void* RP_set_root(void* ptr, uint64_t i){
    if(ralloc::initialized==false){
        RP_init("no_explicit_init");
    }
    return base_md->set_root(ptr,i);
}
void* RP_get_root_c(uint64_t i){
    assert(initialized);
    return (void*)base_md->get_root<char>(i);
}

// return the size of ptr in byte.
// No check for whether ptr is allocated or isn't null
size_t RP_malloc_size(void* ptr){
    const Descriptor* desc = base_md->desc_lookup(ptr);
    return (size_t)desc->block_size;
}

void* RP_realloc(void* ptr, size_t new_size){
    if(ptr == nullptr) return RP_malloc(new_size);
    if(!_rgs->in_range(SB_IDX, ptr)) return nullptr;
    size_t old_size = RP_malloc_size(ptr);
    if(old_size == new_size) {
        return ptr;
    }
    void* new_ptr = RP_malloc(new_size);
    if(UNLIKELY(new_ptr == nullptr)) return nullptr;
    memcpy(new_ptr, ptr, old_size);
    FLUSH(new_ptr);
    FLUSHFENCE;
    RP_free(ptr);
    return new_ptr;
}

void* RP_calloc(size_t num, size_t size){
    void* ptr = RP_malloc(num*size);
    if(UNLIKELY(ptr == nullptr)) return nullptr;
    size_t real_size = RP_malloc_size(ptr);
    memset(ptr, 0, real_size);
    FLUSH(ptr);
    FLUSHFENCE;
    return ptr;
}

int RP_in_prange(void* ptr){
    if(_rgs->in_range(SB_IDX,ptr)) return 1;
    else return 0;
}

int RP_region_range(int idx, void** start_addr, void** end_addr){
    if(start_addr == nullptr || end_addr == nullptr || idx>=_rgs->cur_idx){
        return 1;
    }
    *start_addr = (void*)_rgs->regions_address[idx];
    *end_addr = (void*) ((uint64_t)_rgs->regions_address[idx] + _rgs->regions[idx]->FILESIZE);
    return 0;
}

size_t RP_pointer_to_offset(void* pointer) {
    return (size_t) _rgs->untranslate(SB_IDX, (char*) pointer);
}

void* RP_offset_to_pointer(size_t offset) {
    return (void*) _rgs->translate(SB_IDX, (char*) offset);
}
