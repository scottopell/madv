#include <stdio.h>
#include <stdlib.h>
#include <sys/mman.h>
#include <unistd.h>
#include <termios.h>

#define DEFAULT_ALLOC_SIZE (8 * 1024 * 1024)

static int q = 113;
static int f = 102;
static int d = 100;

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

void up_front_mode(size_t alloc_size, int num_allocs, int initial_sleep) {
    printf("Operating in special mode with %d allocations of %zu bytes each and an initial sleep of %d seconds.\n",
           num_allocs, alloc_size, initial_sleep);

    printf("Sleeping for %d seconds...\n", initial_sleep);
    sleep(initial_sleep);

    void *ptrs[num_allocs];

    for (int i = 0; i < num_allocs; i++) {
        ptrs[i] = mmap(0, alloc_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
        if (ptrs[i] == MAP_FAILED) {
            perror("Memory allocation failed");
            exit(1);
        }

        for (size_t j = 0; j < alloc_size / sizeof(int); j++) {
            ((int *)ptrs[i])[j] = 55;
        }

        printf("Allocated %p (size %zu) and dirtied it\n", ptrs[i], alloc_size);
    }

    // Sleep indefinitely until a signal is received
    pause();

    printf("Termination signal received. Cleaning up and exiting...\n");

    for (int i = 0; i < num_allocs; i++) {
        munmap(ptrs[i], alloc_size);
    }
}

// Function to handle the interactive mode
void interactive_mode() {
    printf("Press a to allocate more memory, f to MADV_FREE, d to MADV_DONTNEED, q to quit. Any other key defaults to allocate.\n");

    int alloc_size = DEFAULT_ALLOC_SIZE;
    char key_code = 0;
    void *addrs[10000];
    size_t addr_idx = 0;
    size_t madvised_idx = 0;

    while (1) {
        key_code = getch();
        if (key_code == q || key_code == EOF) {
            break;
        }
        if (key_code == f || key_code == d) {
            if (madvised_idx < addr_idx) {
                void *ptr = addrs[madvised_idx++];
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
            void *ptr = mmap(0, alloc_size, PROT_READ | PROT_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
            if (ptr == MAP_FAILED) {
                perror("Memory allocation failed");
                break;
            }
            addrs[addr_idx++] = ptr;

            for (int i = 0; i < alloc_size / sizeof(int); i++) {
                ((int *)ptr)[i] = 55;
            }
            fprintf(stderr, "Allocated %p (size %d) and dirtied it\n", ptr, alloc_size);
        }
    }
}

int main() {
    // Fetch environment variables for allocation size, number of allocations, and initial sleep
    char *alloc_env = getenv("ALLOC_SIZE");
    char *num_allocs_env = getenv("NUM_ALLOCS");
    char *sleep_env = getenv("INITIAL_SLEEP");

    if (alloc_env && num_allocs_env && sleep_env) {
        size_t alloc_size = strtoul(alloc_env, NULL, 10);
        int num_allocs = atoi(num_allocs_env);
        int initial_sleep = atoi(sleep_env);

        up_front_mode(alloc_size, num_allocs, initial_sleep);
    } else {
        interactive_mode();
    }

    return 0;
}

