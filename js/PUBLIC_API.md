# Public API (Node.js)

Source of truth: `js/index.d.ts`.

## Top-Level Functions

- `parse(content: string): AAM`
- `load(path: string): AAM`
- `recoverSimple(content: string): AAM`
- `version(): string`

## Main Class

- `class AAM`
- `class AAMBuilder`

## Instance Methods

- `merge(content: string): void`
- `mergeContent(content: string): void`
- `mergeFile(path: string): void`
- `findObj(key: string): string | null`
- `findKey(value: string): string | null`
- `findDeep(key: string): string | null`
- `findList(key: string): string[] | null`
- `findObject(key: string): Record<string, string> | null`
- `keys(): string[]`
- `toMap(): Record<string, string>`
- `validateValue(typeName: string, value: string): void`
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

- `AAML`, `JsAam`, and `JsAaml` are type aliases to `AAM`.
