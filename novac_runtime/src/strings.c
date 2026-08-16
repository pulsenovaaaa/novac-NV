#include "novac_rtm.h"
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>


/*
 * Аналог String::new() из Rust.
 * Создаёт пустую структуру саморасщиряющейся строки
*/
nv_string nv_string_new(void) {
    return (nv_string){ .data = NULL, .len = 0, .capacity = 0 };
}

/*
 * Превращает Си-строку в умную строку рантайма NNV
*/
nv_string nv_string_from_cstring(const char *s) {
    size_t len = strlen(s);
    nv_string str = nv_string_new();
    str.data = nv_alloc(len + 1);
    memcpy(str.data, s, len + 1);
    str.len = len;
    str.capacity = len + 1;
    return str;
}

/*
 * Добавляет символ в конец строки
*/
void nv_string_push(nv_string *s, char c) {
    if (s->len + 1 > s->capacity) {
        s->capacity = s->capacity == 0 ? 16 : s->capacity * 2;
        s->data = nv_alloc(s->capacity);
    }
    s->data[s->len++] = c;
    s->data[s->len] = '\0';
}

/*
 * Освобождает память, занятую строкой
*/
void nv_string_free(nv_string *s) {
    free(s->data);
    s->data = NULL;
    s->len = s->capacity = 0;
}

void nv_print_string(const nv_string *str) {
    if (str->data == NULL) {
        return;
    }
    printf("%s", str->data);
}

void nv_stringinfo(const nv_string *str) {
    printf("String: len=%zu, capacity=%zu\n", str->len, str->capacity);
}
