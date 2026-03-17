# aam-rs for Node.js

Node.js bindings for the `aam-rs` AAML parser, powered by N-API.

## Installation

```bash
npm install aam-rs
```

Prebuilt binaries are published for:

- Linux x64 (GNU libc)
- macOS x64
- macOS arm64
- Windows x64

## Usage

```js
const { AAML, parse, version } = require('aam-rs')

const cfg = parse(`
host = localhost
port = 8080
paths = [assets, cache]
point = { x = 10, y = 20 }
`)

console.log(version())
console.log(cfg.findObj('host'))
console.log(cfg.findList('paths'))
console.log(cfg.findObject('point'))

const runtime = new AAML()
runtime.merge('theme = dark')
console.log(runtime.toMap())
```

## API

### `new AAML()`

Creates an empty configuration.

### `parse(content)`

Parses an AAML string and returns an `AAML` instance.

### `load(path)`

Loads a `.aam` file from disk and returns an `AAML` instance.

### Instance methods

- `merge(content)` / `mergeContent(content)`
- `mergeFile(path)`
- `findObj(key)`
- `findKey(value)`
- `findDeep(key)`
- `findList(key)`
- `findObject(key)`
- `keys()`
- `toMap()`
- `validateValue(typeName, value)`
- `close()`
- `isClosed()`

