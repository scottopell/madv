# Memory Playgrounds

Goal of these two programs is to allocate memory and free it in various ways.
C version is making a rough attempt to allocate and de-allocate in a similar way to the go runtime.

## Modes
There are 2 modes that this program operates in.
### Interactive
The default (ie, without arguments) is to run in "interactive" mode.
This version starts and waits for keyboard input.

See the printed help messages for the commands accepted.

### Up-front
There are 3 arguments taken via env var, and each must be specified and valid in
order for the program to run in "up-front" mode.

- `NUM_ALLOCS` - (int) - The number of allocations to perform of the given size
- `ALLOC_SIZE` - (int) - The number of bytes to request during each allocation
- `INITIAL_SLEEP` - (int) - The number of seconds to sleep _before_ allocating
  anything. This can be useful to determine a "baseline" for the process.

Both the C and Go version take the same arguments.

## C Usage
### Build
```
cd c_version
make
./cmadv
```

### Run via docker
```
# To run in interactive mode
docker run --rm -it ghcr.io/scottopell/madv:c-latest

# To run in up-front mode
docker run --rm -e ALLOC_SIZE=10000000 -e NUM_ALLOCS=5 -e INITIAL_SLEEP=5 ghcr.io/scottopell/madv:c-latest
```

## Go Usage
### Build
```
cd go_version
go build -o gomadv main.go
./gomadv
```

### Run via docker
```
# To run in interactive mode
docker run --rm -it ghcr.io/scottopell/madv:c-latest

# To run in up-front mode
docker run --rm -e ALLOC_SIZE=10000000 -e NUM_ALLOCS=5 -e INITIAL_SLEEP=5 ghcr.io/scottopell/madv:go-latest
```

## Results
To "view the results" the idea is to check the OS level RSS reported.

```
hwatch 'ps e -o "rss,vsz,pid" -p $(pidof gomadv)'
```

In the C version, a free operation appears to be pretty immediate and correctly reduces the
RSS.

In the Go version, the RSS goes down immediately post `FreeOsMemory`.
It seems to take ~1min for the scavenger to return the memory otherwise.

