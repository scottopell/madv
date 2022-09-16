package main

import (
	"fmt"
	"io"
	"os"
	"runtime"
	"runtime/debug"

	"github.com/dustin/go-humanize"
	"golang.org/x/term"
)

const a = 97
const q = 113
const f = 102
const o = 111
const g = 103

func allocateMb(s int) []byte {
	a := make([]byte, s*1024*1024)
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

func main() {
	fmt.Println("Press a to allocate more memory, f to get rid of the reference, o to FreeOSMemory, g to force GC. Any other key defaults to allocate.")

	allocSizeInMb := 8

	allocations := [][]byte{}
	mstats := runtime.MemStats{}

	for true {
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
		default:
			//fmt.Println("Unknown key code: ", char)
		}

		runtime.ReadMemStats(&mstats)
		fmt.Println("HeapInUse", humanize.Bytes(mstats.HeapInuse), "HeapAlloc", humanize.Bytes(mstats.HeapAlloc), "HeapSys", humanize.Bytes(mstats.HeapSys), "HeapReleased", humanize.Bytes(mstats.HeapReleased), "Sys", humanize.Bytes(mstats.Sys))

	}
}
