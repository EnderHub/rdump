#include "util.h"
#include <stdio.h>

#define MAX_BUFFER 1024
#define LOG(msg) log_message(msg)
#define LOG_INT(x) log_message(#x)

typedef struct Server {
    int port;
} Server;

typedef union Packet {
    unsigned long bits;
    unsigned char bytes[8];
} Packet;

typedef enum Status {
    STATUS_OK = 0,
    STATUS_ERR = 1
} Status;

typedef int user_id;
typedef int (*handler_fn)(int);
typedef long number_array[4];

static void log_message(const char *msg) {
    printf("%s\n", msg);
}

int add(int a, int b) {
    // TODO: add validation
    LOG("adding");
    return a + b;
}

int main(void) {
    Packet p = {.bits = 0};
    log_message("hello");
    int total = add(1, 2);
    LOG_INT(total);
    use_util(total);
    return total;
}
