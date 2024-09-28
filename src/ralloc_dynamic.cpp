#include "BaseMeta.hpp"
#include <atomic>
#include <bit>
#include <cstddef>
#include <cstdint>
#include <pthread.h>

void init_thread();

extern "C" void *malloc(size_t size) {
  init_thread();
  return nullptr;
}

static std::atomic_uint64_t IDS = 0xFFFFFFFFFFFFFFFE;
static pthread_key_t KEY;
static pthread_once_t ONCE = PTHREAD_ONCE_INIT;
static Regions REGIONS;

__thread uint64_t ID = 0;

void destroy_thread(void *value) { IDS.fetch_or(ID); }

void init_process() {
  pthread_key_create(&KEY, destroy_thread);

  ralloc::_rgs = &REGIONS;
}

void init_thread() {
  pthread_once(&ONCE, init_process);
  pthread_setspecific(KEY, (void *)1);

  if (ID > 0) {
    return;
  }

  uint64_t ids = IDS.load();
  uint64_t id;
  do {
    id = std::countr_zero(IDS.load());
  } while (!IDS.compare_exchange_strong(ids, ids & !(1 << id),
                                        std::memory_order::acq_rel,
                                        std::memory_order::acquire));

  ID = id;
}
