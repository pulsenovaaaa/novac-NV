CC = cc
CFLAGS = -O2 -Wall -Wextra -fPIC
AR = ar

SRC = $(wildcard src/*.c)
OBJ = $(SRC:.c=.o)
LIB = libnovac_rtm.a
INCLUDE = -Iinclude

all: $(LIB)

$(LIB): $(OBJ)
	$(AR) rcs $@ $^

src/%.o: src/%.c include/novac_rtm.h
	$(CC) $(CFLAGS) $(INCLUDE) -c $< -o $@

clean:
	rm -f $(OBJ) $(LIB)

.PHONY: all clean
