# Public API (PHP)

Source of truth: `php/src/AamPhp.php`.

## Main Class

- `AamPhp`

## Construction

- `new AamPhp(?string $libraryPath = null)`

## Exposed Method

- `parseFindObj(string $content, string $query): ?string`

Behavior:

- Parses input content and performs `find_obj` style lookup.
- Supports reverse lookup fallback when query is not a key.

## Runtime Notes

- Uses PHP FFI and requires native `aam_rs` shared library.
- Library path can be passed explicitly or resolved from env/default path.
