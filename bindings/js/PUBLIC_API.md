# Public API (Node.js)

Source of truth: `js/index.d.ts`.

## Top-Level Functions

- `parse(content: string): AAM`
- `load(path: string): AAM`
- `format(content: string): string`
- `lspAssist(content: string): JsLspResult`
- `reconstructSchema(name: string, contents: string[]): string`
- `splitAam(content: string): Record<string, AAMBuilder>`
- `version(): string`

## Main Class

- `class AAM`
- `class AAMBuilder`

## Instance Methods

- `format(content: string): string`
- `formatRange(content: string, startLine: number, endLine: number): string`
- `get(key: string): string | null`
- `find(query: string): Record<string, string>`
- `deepSearch(pattern: string): Record<string, string>`
- `reverseSearch(value: string): string[]`
- `keys(): string[]`
- `toMap(): Record<string, string>`
- `schemaNames(): string[]`
- `typeNames(): string[]`
- `close(): void`
- `isClosed(): boolean`

## AAMBuilder Methods

- `new AAMBuilder()`
- `addLine(key: string, value: string): void`
- `comment(text: string): void`
- `schema(name: string, fields: string[]): void` (`"field: type"` or `"field*: type"`)
- `schemaMultiline(name: string, fields: string[]): void`
- `derive(path: string, schemas: string[]): void`
- `import(path: string): void`
- `typeAlias(alias: string, typeName: string): void`
- `asString(): string`

## Aliases

- `JsAam` and `JsAamBuilder` are type aliases for compatibility.
