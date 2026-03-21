# aam-rs PHP bindings

Minimal PHP wrapper using FFI on top of the stable `aam-rs` C API.

## Build native library

```bash
cargo build --release --features ffi
```

## Run tests

```bash
AAM_RS_LIB=target/release/libaam_rs.so php php/tests/smoke.php
```

## PHP usage

```php
require_once 'php/src/AamRs.php';

$aam = new AamRs('/absolute/path/to/libaam_rs.so');
echo $aam->parseFindObj("host = localhost\nport = 8080", 'host');
```

