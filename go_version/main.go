package main

import (
	"fmt"
	"io"
	"os"
	"runtime"
	"runtime/debug"
	"strconv"
	"time"

	"github.com/dustin/go-humanize"
	"golang.org/x/term"
)

const (
	a = 97
	q = 113
	f = 102
	o = 111
	g = 103
)

func allocateMb(s int) []byte {
	a := make([]byte, s*1024*1024)
	for i := 0; i < len(a); i += 4096 {
		a[i] = 'x'
	}
	return a
}

func allocateBytes(s int) []byte {
	a := make([]byte, s)
	for i := 0; i < len(a); i += 4096 {
		a[i] = 'x'
	}
	return a
}

func getChar() (byte, error) {
	// switch stdin into 'raw' mode
	oldState, err := term.MakeRaw(int(os.Stdin.Fd()))
	if err != nil {
		return 0, err
	}
	defer term.Restore(int(os.Stdin.Fd()), oldState)

	b := make([]byte, 1)
	_, err = os.Stdin.Read(b)
	if err != nil {
		return 0, err
	}

	return b[0], nil
}

func interactiveMode() {
	fmt.Println("Press a to allocate more memory, f to get rid of the reference, o to FreeOSMemory, g to force GC. Any other key defaults to allocate.")
	fmt.Println("FreeOSMemory does a GC followed by a forced release of memory to the OS.")

	allocSizeInMb := 8

	allocations := [][]byte{}
	mstats := runtime.MemStats{}

	for {
		char, err := getChar()
		if err == io.EOF {
			break
		} else if err != nil {
			fmt.Println("Error getting input from terminal, err: ", err)
			continue
		}

		switch char {
		case 3, 4:
			// ctrl+c and ctrl+d
			return
		case f:
			if len(allocations) > 0 {
				var alloc []byte
				alloc = allocations[0]
				// If the pointer is left in the backing array, then the allocation won't be
				// released, that reference will keep it alive.
				allocations[0] = nil
				allocations = allocations[1:]
				fmt.Printf("Popped off memory, 0x%x should now be free\n", &alloc[0])
			} else {
				fmt.Println("Removed all allocations, can't free anything.")
			}
		case a:
			alloc := allocateMb(allocSizeInMb)
			fmt.Printf("Allocated memory at 0x%x\n", &alloc[0])
			allocations = append(allocations, alloc)
		case g:
			fmt.Println("Forcing garbage collection")
			runtime.GC()
			fmt.Println("GC Done.")
		case o:
			fmt.Println("Invoking FreeOsMemory")
			debug.FreeOSMemory()
			fmt.Println("Freed.")
		case q:
			fmt.Println("Exiting.")
			return
		default:
			//fmt.Println("Unknown key code: ", char)
		}

		runtime.ReadMemStats(&mstats)
		fmt.Println("HeapInUse", humanize.Bytes(mstats.HeapInuse), "HeapAlloc", humanize.Bytes(mstats.HeapAlloc), "HeapSys", humanize.Bytes(mstats.HeapSys), "HeapReleased", humanize.Bytes(mstats.HeapReleased), "Sys", humanize.Bytes(mstats.Sys))
	}
}

func upfrontMode(allocSizeInBytes int, numAllocs int, initialSleepSeconds int) {
	mstats := runtime.MemStats{}
	humanAllocSize := humanize.Bytes(uint64(allocSizeInBytes))
	fmt.Printf("Operating in up-front mode with %d allocations of %d B (%s) each.\n", numAllocs, allocSizeInBytes, humanAllocSize)

	fmt.Printf("Sleeping for %d seconds before allocations...\n", initialSleepSeconds)
	time.Sleep(time.Duration(initialSleepSeconds) * time.Second)

	allocations := make([][]byte, numAllocs)

	for i := 0; i < numAllocs; i++ {
		alloc := allocateBytes(allocSizeInBytes)
		allocations[i] = alloc
		fmt.Printf("Allocated %s memory at 0x%x\n", humanAllocSize, &alloc[0])
	}

	runtime.ReadMemStats(&mstats)
	fmt.Println("HeapInUse", humanize.Bytes(mstats.HeapInuse), "HeapAlloc", humanize.Bytes(mstats.HeapAlloc), "HeapSys", humanize.Bytes(mstats.HeapSys), "HeapReleased", humanize.Bytes(mstats.HeapReleased), "Sys", humanize.Bytes(mstats.Sys))

	fmt.Println("All allocations done. Waiting for termination...")

	// Sleep indefinitely
	for {
		time.Sleep(time.Hour)
	}
}

func main() {
	allocSizeEnv := os.Getenv("ALLOC_SIZE")
	numAllocsEnv := os.Getenv("NUM_ALLOCS")
	initialSleepEnv := os.Getenv("INITIAL_SLEEP")

	if allocSizeEnv != "" && numAllocsEnv != "" && initialSleepEnv != "" {
		allocSizeInBytes, err := strconv.Atoi(allocSizeEnv)
		if err != nil {
			fmt.Println("Invalid ALLOC_SIZE, must be an integer")
			return
		}

		numAllocs, err := strconv.Atoi(numAllocsEnv)
		if err != nil {
			fmt.Println("Invalid NUM_ALLOCS, must be an integer")
			return
		}

		initialSleep, err := strconv.Atoi(initialSleepEnv)
		if err != nil {
			fmt.Println("Invalid INITIAL_SLEEP, must be an integer")
			return
		}

		upfrontMode(allocSizeInBytes, numAllocs, initialSleep)
	} else {
		interactiveMode()
	}
}
