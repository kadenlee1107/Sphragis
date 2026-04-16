CC      ?= aarch64-linux-musl-gcc
CFLAGS  ?= -O1 -Wall -Wextra -static -nostartfiles -ffreestanding \
           -fno-stack-protector -fno-pie -no-pie
LDFLAGS ?= -static -Wl,--build-id=none

TARGET  := syscall_fuzz
SRC     := syscall_fuzz.c

$(TARGET): $(SRC) seeds.h
	$(CC) $(CFLAGS) $(SRC) -o $@ $(LDFLAGS)

clean:
	rm -f $(TARGET)

.PHONY: clean
