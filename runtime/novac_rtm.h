#ifndef NOVAC_RTM_H
#define NOVAC_RTM_H

#include <stddef.h>
#include <stdint.h>

#define NV_STDIN  0
#define NV_STDOUT 1
#define NV_STDERR 2

// Структура умной строки для NNV
typedef struct {
  char *data;
  size_t len;
  size_t capacity;
} nv_string;

// Строки
nv_string nv_string_new(void);
nv_string nv_string_from_cstring(const char *s);
void nv_string_free(nv_string *s);
void nv_string_push(nv_string *s, char c);
void nv_string_concat(nv_string *dst, const nv_string *src);

// I/O
void nv_putchar(char c);
void nv_print_string(const nv_string *str);
int nv_getchar(void);

// Память
void *nv_alloc(size_t size);
void nv_free(void *ptr);

#endif // NOVAC_RTM_H!
