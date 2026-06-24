# Public API (WASM)

Source of truth: `wasm/src/lib.rs`.

## Exported Class

- `AamDocument`
- `AAMBuilder` (JS name for `WasmAamBuilder`)

## Construction

- `new AamDocument(content: string)`

## Instance Methods

- `format(content: string): string`
- `formatRange(content: string, startLine: number, endLine: number): string`
- `get(key: string): string | undefined`
- `keys(): string[]`
- `toMap(): Record<string, string>`
- `find(query: string): Record<string, string>`
- `deepSearch(pattern: string): Record<string, string>`
- `reverseSearch(value: string): string[]`
- `schemaNames(): string[]`
- `typeNames(): string[]`

## Static Helper

- `AamDocument.lspAssist(content: string): { diagnostics: string[]; formatted: string | null }`
- `splitAam(content: string): Record<string, string>`

## Builder Methods

- `new AAMBuilder()`
- `AAMBuilder.withCapacity(capacity: number): AAMBuilder`
- `addLine(key: string, value: string): void`
- `comment(text: string): void`
- `schema(name: string, fields: string[]): void` (`"field: type"` or `"field*: type"`)
- `schemaMultiline(name: string, fields: string[]): void`
- `derive(path: string, schemas: string[]): void`
- `import(path: string): void`
- `typeAlias(alias: string, typeName: string): void`
- `asString(): string`

