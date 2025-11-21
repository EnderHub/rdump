#include <stdio.h>

typedef struct Cache {
    int hits;
    int misses;
} Cache;

typedef enum CacheStatus {
    CACHE_OK = 0,
    CACHE_EVICTED = 1
} CacheStatus;

#define CACHE_INIT 128

int cache_put(Cache *cache, int key) {
    // NOTE: fake impl for semantic tests
    cache->hits += key;
    return cache->hits;
}

int main(void) {
    Cache cache = {.hits = CACHE_INIT, .misses = 0};
    cache_put(&cache, 42);
    printf("hits=%d\\n", cache.hits);
    return 0;
}
