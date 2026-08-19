#include <stdint.h>
#include <stdio.h>
#include <string.h>
// #include "runtime/novac_rtm.h"

typedef uint32_t u32;

void u32_to_string(char buf[32], u32 value) {
    int i = 0;

    if (value == 0) {
        buf[0] = '0';
        buf[1] = '\0';
        return;
    }

    while (value > 0) {
        buf[i++] = (char)(value % 10) + '0';
        value /= 10;
    }
    buf[i] = '\0';

    int st = 0;
    int end = i - 1;
    while (st < end) {
        char t = buf[st];
        buf[st] = buf[end];
        buf[end] = t;
        st++;
        end--;
    }
}

int main(void) {
    char buf[32];
    u32_to_string(buf, 12345);
    printf("%s\n", buf);
}
