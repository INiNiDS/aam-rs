# Public API (PHP)

Source of truth: `php/src/AamPhp.php`.

## Main Classes

- `AamDocument`
- `AamBuilder`
- `SchemaField`

## AamDocument Construction

- `new AamDocument(string $content, ?string $libPath = null)`
- `AamDocument::parse(string $content, ?string $libPath = null)`
- `AamDocument::splitAam(string $content)`

## AamDocument Methods

- `update(): void` — reload from the original on-disk source file (loaded via a file path); throws `RuntimeException` if not loaded from a path.
- `reload(string $content): void` — replace contents by reparsing raw text; clears the remembered source path so a subsequent `update()` fails.
- `format(string $content): string`
- `get(string $key): ?string`
- `find(string $query): array`
- `deepSearch(string $pattern): array`
- `reverseSearch(string $value): array`
- `schemaNames(): array`
- `typeNames(): array`
- `reconstructSchema(string $name, array $contents): string`

## Builder API

- `SchemaField::required(string $name, string $typeName): SchemaField`
- `SchemaField::optional(string $name, string $typeName): SchemaField`
- `SchemaField::toAam(): string`
- `AamBuilder::addLine(string $key, string $value): AamBuilder`
- `AamBuilder::comment(string $text): AamBuilder`
- `AamBuilder::schema(string $name, array $fields): AamBuilder`
- `AamBuilder::derive(string $path, array $schemas = []): AamBuilder`
- `AamBuilder::import(string $path): AamBuilder`
- `AamBuilder::typeAlias(string $alias, string $typeName): AamBuilder`
- `AamBuilder::build(): string`
- `AamBuilder::toFile(string $path): void`

## Runtime Notes

- Uses PHP FFI and requires native `aam_rs` shared library.
- Library path can be passed explicitly or resolved from env/default path.
