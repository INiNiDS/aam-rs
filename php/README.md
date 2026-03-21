# aam-php

PHP bindings for `aam-rs` using `FFI` on top of the stable C API.

## What you get

- Parse AAML from strings.
- Forward lookup (`key -> value`) and reverse lookup fallback (`value -> key`).
- Simple zero-dependency wrapper class for scripting and prototyping.

## Build native library

```bash
cargo build --release --features ffi
```

By default `AamPhp` reads the library from:

- `AAM_PHP_LIB` env var, if present.
- `target/release/libaam_rs.so` as fallback on Linux.

## Basic usage

```php
<?php

require_once 'php/src/AamPhp.php';

$aam = new AamPhp('/absolute/path/to/libaam_rs.so');
echo $aam->parseFindObj("host = localhost\nport = 8080", 'host');
// localhost
```

## More examples

```php
<?php

require_once 'php/src/AamPhp.php';

$aam = new AamPhp();

// direct lookup
var_dump($aam->parseFindObj("env = production", 'env')); // "production"

// reverse lookup fallback via find_obj semantics
var_dump($aam->parseFindObj("env = production", 'production')); // "env"

// missing key
var_dump($aam->parseFindObj("env = production", 'missing')); // null
```

## Tests

```bash
AAM_RS_LIB=target/release/libaam_rs.so php php/tests/smoke.php
```

The smoke script includes multiple assertions for valid parsing, reverse lookup, missing keys, and invalid input errors.

