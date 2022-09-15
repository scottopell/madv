#include <stdio.h>
#include <sys/mman.h>
#include <stdlib.h>
#include <termios.h>

// https://stackoverflow.com/a/19317368
char getch(void) {
    /* get original settings */
    struct termios new, old;
    tcgetattr(0, &old);
    new = old;

    /* set new settings and flush out terminal */
    new.c_lflag &= ~ICANON;
    tcsetattr(0, TCSAFLUSH, &new);

    /* get char and reset terminal */
    char ch = getchar();
    tcsetattr(0, TCSAFLUSH, &old);

    return ch;
}

static int q = 113;
static int a = 95;
static int f = 102;
static int d = 100;

int main() {
    printf("Press a to allocate more memory, f to MADV_FREE, d to MADV_DONTNEED, q to quit. Any other key defaults to allocate.\n");
    int alloc_size = 8 * 1024 * 1024;
    char key_code = 0;
    void* addrs[10000];
    size_t addr_idx = 0;
    size_t madvised_idx = 0;
    while (1) {
        key_code = getch();
        fprintf(stderr, "Key code: %d\n", key_code);
        if (key_code == q || key_code == EOF) {
            break;
        }
        if (key_code == f || key_code == d) {
            if (madvised_idx < addr_idx) {
                void * ptr = addrs[madvised_idx++];
                int advice = MADV_FREE;
                if (key_code == d) {
                    advice = MADV_DONTNEED;
                    fprintf(stderr, "Issued MADV_DONTNEED for %p\n", ptr);
                } else {
                    fprintf(stderr, "Issued MADV_FREE for %p\n", ptr);
                }
                madvise(ptr, alloc_size, advice);
            } else {
                fprintf(stderr, "No memory left to free\n");
            }
        } else {
            void * ptr = mmap(0, alloc_size, PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS, -1, 0);
            addrs[addr_idx++] = ptr;

            for (int i = 0; i < alloc_size / sizeof(int); i++) {
                *(int*)(ptr + i) = 55;
            }
            fprintf(stderr, "Allocated %p and dirtied it\n", ptr);
        }
    }
    return 0;
}
