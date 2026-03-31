# Public API (C FFI)

Source of truth: `include/aam.h`.

## Handle Lifecycle

- `AamlHandle *aam_new(void)`
- `void aam_free(AamlHandle *handle)`

## Parse and Load

- `int aam_parse(AamlHandle *handle, const char *content)`
- `int aam_load(AamlHandle *handle, const char *path)`
- `int aam_merge(AamlHandle *handle, const char *content)`
- `int aam_recover_simple(AamlHandle *handle, const char *content)`

Return convention: `0` on success, non-zero on error.

## Lookup

- `char *aam_find_obj(const AamlHandle *handle, const char *key)`
- `char *aam_find_key(const AamlHandle *handle, const char *value)`
- `char *aam_find_deep(const AamlHandle *handle, const char *key)`

Returned strings are owned by Rust and must be released via `aam_string_free`.

## Memory and Errors

- `void aam_string_free(char *s)`
- `const char *aam_last_error(const AamlHandle *handle)`

`aam_last_error` returns an internal borrowed pointer. Do not free it.
