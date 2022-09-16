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

```
hwatch 'ps e -o "rss,vsz,pid" -p $(pidof gomadv)'
```

In the C version, this appears to be pretty immediate and correctly reduces the
RSS.

In the Go version, the RSS goes down immediately post `FreeOsMemory`.
It seems to take ~1min for the scavenger to return the memory otherwise.

