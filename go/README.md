# aam-rs Go bindings

CGo bindings for the [aam-rs](https://github.com/INiNiDS/aam-rs) AAML parser.

## Prerequisites

Build the Rust library first (from the repository root):

```sh
cargo build --release --features ffi
```

This produces `target/release/libaam_rs.so` (Linux), `libaam_rs.dylib` (macOS),
or `aam_rs.dll` (Windows) as well as the static archive `libaam_rs.a`.

The CGo flags in `aam/aam.go` default to `../../target/release` (relative to
the package source), so building from the repository root works out of the box.
Override via environment variables if you install the library elsewhere:

```sh
export CGO_CFLAGS="-I/usr/local/include"
export CGO_LDFLAGS="-L/usr/local/lib -laam_rs -ldl -lpthread -lm"
```

## Installation

```sh
go get github.com/INiNiDS/aam-rs/go/aam
```

## Quick start

```go
package main

import (
    "fmt"
    "log"

    "github.com/INiNiDS/aam-rs/go/aam"
)

func main() {
    doc, err := aam.Parse("host = localhost\nport = 8080\n")
    if err != nil {
        log.Fatal(err)
    }
    defer doc.Close()

    if val, ok := doc.FindObj("host"); ok {
        fmt.Println("host:", val) // host: localhost
    }

    if key, ok := doc.FindKey("8080"); ok {
        fmt.Println("key for 8080:", key) // key for 8080: port
    }
}
```

## API

| Function / Method                              | Description                            |
|------------------------------------------------|----------------------------------------|
| `New() (*AAML, error)`                         | Creates an empty AAML handle           |
| `Parse(content string) (*AAML, error)`         | Parses AAML content from a string      |
| `Load(path string) (*AAML, error)`             | Loads and parses a `.aam` file         |
| `(*AAML) Merge(content string) error`          | Merges additional content (child-wins) |
| `(*AAML) FindObj(key string) (string, bool)`   | Forward then reverse key lookup        |
| `(*AAML) FindKey(value string) (string, bool)` | Reverse lookup (value → key)           |
| `(*AAML) FindDeep(key string) (string, bool)`  | Follows reference chain to terminal    |
| `(*AAML) LastError() string`                   | Returns last error message             |
| `(*AAML) Close()`                              | Frees the native handle (idempotent)   |

## Running tests

```sh
# From repository root
cargo build --release --features ffi

# Then run Go tests
cd go
go test -v ./...
```

