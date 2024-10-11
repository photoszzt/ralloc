#include "BaseMeta.hpp"
#include "SizeClass.hpp"
#include "ralloc.hpp"
#include <cstddef>
#include <cstdint>
#include <pthread.h>

static void RP_init_thread();

extern "C" void *RP_malloc(size_t size) {
  RP_init_thread();
  return RP_malloc(size);
}

extern "C" void RP_free(void *ptr) {
  RP_init_thread();
  return RP_free(ptr);
}

extern "C" void *RP_realloc(void *ptr, size_t size) {
  RP_init_thread();
  return RP_realloc(ptr, size);
}

extern "C" size_t RP_malloc_usable_size(void *ptr) {
  RP_init_thread();
  return RP_malloc_size(ptr);
}

// https://github.com/ricleite/lrmalloc/blob/c5c322e5378555dd4f87095e4935efcb9a5f239b/lrmalloc.cpp#L526
extern "C" void *RP_memalign(size_t alignment, size_t size) {
  RP_init_thread();

  size = ALIGN_VAL(size, alignment);

  // allocations smaller than PAGE will be correctly aligned
  // this is because size >= alignment, and size will map to a small class
  // size with the formula 2^X + A*2^(X-1) + C*2^(X-2)
  // since size is a multiple of alignment, the lowest size class power of
  // two is already >= alignment
  // this does not work if allocation > PAGE even if it's a small class size,
  // because the superblock for those allocations is only guaranteed
  // to be page aligned
  // force such allocations to become large block allocs
  if (UNLIKELY(size > 4096)) {
    // hotfix solution for this case is to force allocation to be large
    size = std::max<size_t>(size, MAX_SZ + 1);

    // large blocks are page-aligned
    // if user asks for a diabolical alignment, need more pages to
    // fulfil it
    bool const needsMorePages = (alignment > 4096);
    assert(!needsMorePages);
  }

  return RP_malloc(size);
}

extern "C" int RP_posix_memalign(void **pointer, size_t align, size_t size) {
  *pointer = RP_memalign(align, size);
  return *pointer != nullptr;
}

// Hack to get around static initializer clobbering fields
// after we call init_process earlier, during some other
// initializer function that allocates.
static uint8_t MANAGERS[sizeof(RegionManager[LAST_IDX])];
static uint8_t REGIONS[sizeof(Regions)];

static void RP_init_process() {
  ralloc::_rgs = (Regions *)REGIONS;
  new ((Regions *)REGIONS) Regions((RegionManager(&)[LAST_IDX])MANAGERS);
  new (&ralloc::sizeclass) SizeClass();

  size_t size = 1ull << 34;
  uint64_t num_sb = size / SBSIZE;

  for (int i = 0; i < LAST_IDX; i++) {
    switch (i) {
    case DESC_IDX:
      ralloc::_rgs->create("", num_sb * DESCSIZE, false, true);
      break;
    case SB_IDX:
      ralloc::_rgs->create("", num_sb * SBSIZE, false, false);
      break;
    case META_IDX:
      ralloc::base_md =
          ralloc::_rgs->create_for<BaseMeta>("", sizeof(BaseMeta), false);
      break;
    } // switch
  }

  ralloc::initialized = true;
}

static void RP_init_thread() {
  if (ralloc::initialized) {
    return;
  }

  RP_init_process();
}
