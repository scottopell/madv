package main

import (
	"fmt"
	"io"
	"os"
	"runtime"
	"runtime/debug"

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

	for true {
		char, err := getChar()
		if err == io.EOF {
			break
		} else if err != nil {
			fmt.Println("Error getting input from terminal, err: ", err)
			continue
		}

		if char == 113 || char == 3 {
			break
		}
		if char == f {
			if len(allocations) > 0 {
				var alloc []byte
				alloc, allocations = allocations[0], allocations[1:]
				fmt.Printf("Popped off memory, %d should now be free\n", &alloc[0])
			} else {
				fmt.Println("Removed all allocations, can't free anything.")
			}
		} else if char == a {
			alloc := allocateMb(allocSizeInMb)
			fmt.Printf("Allocated memory at %d\n", &alloc[0])
			allocations = append(allocations, alloc)
		} else if char == g {
			fmt.Println("Forcing garbage collection")
			runtime.GC()
			fmt.Println("GC Done.")
		} else if char == o {
			fmt.Println("Invoking FreeOsMemory")
			debug.FreeOSMemory()
			fmt.Println("Freed.")
		} else {
			//fmt.Println("Unknown key code: ", char)
		}
	}
}
