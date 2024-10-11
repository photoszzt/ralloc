#include "BaseMeta.hpp"
#include "SizeClass.hpp"
#include "ralloc.hpp"
#include <cstddef>
#include <cstdint>
#include <pthread.h>

static void ralloc_init_thread();

extern "C" void *ralloc_malloc(size_t size) {
  ralloc_init_thread();
  return RP_malloc(size);
}

extern "C" void ralloc_free(void *ptr) {
  ralloc_init_thread();
  return RP_free(ptr);
}

// Hack to get around static initializer clobbering fields
// after we call init_process earlier, during some other
// initializer function that allocates.
static uint8_t MANAGERS[sizeof(RegionManager[LAST_IDX])];
static uint8_t REGIONS[sizeof(Regions)];

static void ralloc_init_process() {
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

static void ralloc_init_thread() {
  if (ralloc::initialized) {
    return;
  }

  ralloc_init_process();
}
