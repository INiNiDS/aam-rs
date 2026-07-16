# Public API (Ruby)

Source of truth: `ruby/ext/aam_rs/src/lib.rs`.

## Main Namespace

- `AamRb::AAM`
- `AamRb::AAMBuilder`
- `AamRb::SchemaField`

## Class Methods

- `AamRb::AAM.new`
- `AamRb::AAM.parse(content)`
- `AamRb::AAM.load(path)`

### Reload

- `AamRb::AAM#update` — reload from the original on-disk source file (loaded via `load`); raises `RuntimeError` if not loaded from a path.
- `AamRb::AAM#update_from_text(content)` — replace contents by reparsing raw text; clears the remembered source path so a subsequent `update` fails.
- `AamRb::AAM.reconstruct_schema(name, contents)`
- `AamRb::AAM.format(content)`
- `AamRb.split_aam(content)`
- `AamRb::AAMBuilder.new`
- `AamRb::AAMBuilder.with_capacity(capacity)`
- `AamRb::SchemaField.required(name, type_name)`
- `AamRb::SchemaField.optional(name, type_name)`

## Instance Methods

- `get(key)`
- `keys`
- `to_map`
- `find(query)`
- `deep_search(pattern)`
- `reverse_search(value)`
- `schema_names`
- `type_names`

## AAMBuilder Instance Methods

- `add_line(key, value)`
- `comment(text)`
- `schema(name, fields)`
- `schema_multiline(name, fields)`
- `derive(path, schemas)`
- `import(path)`
- `type_alias(alias, type_name)`
- `as_string`

## Error Model

- Parse and load failures raise runtime exceptions from native layer.

## Compatibility Note

- Existing README examples use a lightweight helper style.
- Native extension API is centered on `AamRb::AAM`.
