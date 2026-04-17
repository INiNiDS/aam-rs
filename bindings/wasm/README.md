# aam-wasm

WebAssembly bindings for `aam-rs` powered by `wasm-bindgen` and `wasm-pack`.

## Build package

```bash
cd wasm
wasm-pack build --release --target nodejs --out-dir pkg
```

## Run tests

```bash
cargo test --manifest-path wasm/Cargo.toml
wasm-pack test --node wasm
```

## Basic usage

```javascript
const wasm = require('./pkg/aam_wasm.js')

const doc = new wasm.AAM('host = localhost\nport = 8080')
console.log(doc.get('host'))
```

## More examples

```javascript
const wasm = require('./pkg/aam_wasm.js')

const doc = new wasm.AAM('root_path = srv_app\nactive_path = root_path\nmode = active')
console.log(doc.deepSearch('path'))
console.log(doc.reverseSearch('active'))
```

## Notes

- The package is published to npm as `aam-wasm`.
- Generated JS and `.wasm` files are emitted to `wasm/pkg/`.

