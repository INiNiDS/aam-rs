# Public API (WASM)

Source of truth: `wasm/src/lib.rs`.

## Exported Class

- `AAM` (JS name for `AamDocument`)

## Construction

- `new AAM(content: string)`

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

- `AAM.lspAssist(content: string): { diagnostics: string[]; formatted: string | null }`
