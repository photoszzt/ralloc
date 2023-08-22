#include <stddef.h>
#include <stdint.h>

void* RP_get_root_c(uint64_t i);
int RP_init(const char* _id, uint64_t size, uint8_t process_id, uint8_t process_count);
void RP_recover();
void RP_close();
void* RP_malloc(size_t sz);
void RP_free(void* ptr);
void* RP_set_root(void* ptr, uint64_t i);
size_t RP_malloc_size(void* ptr);
void* RP_calloc(size_t num, size_t size);
void* RP_realloc(void* ptr, size_t new_size);
int RP_in_prange(void* ptr);
int RP_region_range(int idx, void** start_addr, void** end_addr);
