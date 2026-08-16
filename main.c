#include <stdio.h>
#include <stdlib.h>
#include "runtime/novac_rtm.h"

int main(void) {
    /* Novac Never-Value CodeGen v.0.0.1-rc0.1 */
    nv_string string1 = nv_string_from_cstring("Hello, world!");
    nv_print_string(&string1);
    return 0;
}
