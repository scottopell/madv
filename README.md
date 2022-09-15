# Memory Playgrounds

Goal of these two programs is to allocate memory and free it in various ways.
C is obviously much simpler than Go, and C version is making some rough attempt
to allocate and de-allocate in a similar way to the go runtime.

## C Usage
```
cd c_version
gcc -o cmadv main.c
./cmadv
```

## Go Usage
```
cd go_version
go build -o gomadv
./gomadv
```

## Results
To "view the results" the idea is to check the OS level RSS reported.

In the C version, this appears to be pretty immediate and correctly reduces the
RSS.

In the Go version, this isn't working as seamlessly, even with calls to
`FreeOsMemory`.

TODO: Double check the logic in the go version

