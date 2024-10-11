#include <cstddef>

extern "C" void *RP_malloc(size_t size);

extern "C" void RP_free(void *ptr);

extern "C" void *RP_realloc(void *ptr, size_t size);

extern "C" size_t RP_malloc_usable_size(void *ptr);

extern "C" void *RP_memalign(size_t align, size_t size);

extern "C" int RP_posix_memalign(void **ptr, size_t align, size_t size);
