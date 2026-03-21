# aam-wasm

Minimal WebAssembly wrapper for `aam-rs` built with `wasm-bindgen` and `wasm-pack`.

## Build

```bash
wasm-pack build --release --target nodejs --out-dir pkg
```

## Test

```bash
cargo test --manifest-path wasm/Cargo.toml
wasm-pack test --node wasm
```

## JavaScript usage

```js
const wasm = require('./pkg/aam_wasm.js');

const doc = new wasm.AamDocument('host = localhost\nport = 8080');
console.log(doc.findObj('host')); // localhost
```

