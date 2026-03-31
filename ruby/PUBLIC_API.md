# Public API (Ruby)

Source of truth: `ruby/ext/aam_rs/src/lib.rs`.

## Main Namespace

- `AamRb::AAM`

## Class Methods

- `AamRb::AAM.new`
- `AamRb::AAM.parse(content)`
- `AamRb::AAM.load(path)`

## Instance Methods

- `get(key)`
- `keys`
- `to_map`
- `find(query)`
- `deep_search(pattern)`
- `reverse_search(value)`
- `schema_names`
- `type_names`

## Error Model

- Parse and load failures raise runtime exceptions from native layer.

## Compatibility Note

- Existing README examples use a lightweight helper style.
- Native extension API is centered on `AamRb::AAM`.
