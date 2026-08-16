#include "novac_rtm.h"
#include <stdlib.h>
#include <stdio.h>

void *nv_alloc(size_t size) {
    void *ptr = malloc(size);
    if (!ptr) {
        fprintf(stderr, "Novac Runtime: Out of memory (%zu bytes)", size);
        exit(1);
    }
    return ptr;
}

void nv_free(void *ptr) {
    free(ptr);
}
