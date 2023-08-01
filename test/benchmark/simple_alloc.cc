#include "ralloc.hpp"

int main(void)
{
    int ret = RP_init("test", 1024UL*1024UL*1024UL*2UL);
    void* ptr = RP_malloc(50);
    RP_free(ptr);
    return 0;
}
