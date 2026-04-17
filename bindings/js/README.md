# aam-nodejs

Node.js bindings for the `aam-rs` AAM parser, powered by N-API.

## Installation

```bash
npm install aam-nodejs
```

Prebuilt binaries are published for:

- Linux x64 (GNU libc)
- macOS x64
- macOS arm64
- Windows x64

## Usage

```js
const {AAM, parse, version} = require('aam-nodejs')

const cfg = parse(`
host = localhost
port = 8080
paths = [assets, cache]
point = { x = 10, y = 20 }
`)

console.log(version())
console.log(cfg.get('host'))
console.log(cfg.find('host'))
console.log(cfg.deepSearch('po'))

const runtime = new AAM()
console.log(runtime.toMap())
```

## More examples

```js
const {parse} = require('aam-nodejs')

const cfg = parse(`
root = /srv/app
active = root
roles = [api, worker]
meta = { owner = team-a, region = eu }
`)

console.log(cfg.find('root'))               // { root: '/srv/app' }
console.log(cfg.find('team-a'))             // { owner: 'team-a' } when matched by value
console.log(cfg.deepSearch('act'))          // { active: 'root' }
console.log(cfg.reverseSearch('/srv/app'))  // ['root']
```

## Local build & test

```bash
cd js
npm install
npm run build
npm test
```

## API

### `new AAM()`

Creates an empty configuration.

### `parse(content)`

Parses an AAM string and returns an `AAM` instance.

### `load(path)`

Loads a `.aam` file from disk and returns an `AAM` instance.

### Instance methods

- `format(content)`
- `formatRange(content, startLine, endLine)`
- `get(key)`
- `find(query)`
- `deepSearch(pattern)`
- `reverseSearch(value)`
- `keys()`
- `toMap()`
- `schemaNames()`
- `typeNames()`
- `close()`
- `isClosed()`

