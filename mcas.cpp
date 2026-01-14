#include "ralloc.hpp"
#include <iostream>

int main() {
  RP_init_thread(7);
  RP_init("test");
  uint64_t *foo = (uint64_t *)RP_malloc(8);
  *foo = 5;
  RP_free(foo);
  return 0;
}
