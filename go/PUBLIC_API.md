# Public API (Go)

Source of truth: `go/aam/aam.go`.

## Main Type

- `type AAM struct`

## Constructors

- `New() (*AAM, error)`
- `Parse(content string) (*AAM, error)`
- `Load(path string) (*AAM, error)`

## Instance Methods

- `Format(content string) (string, error)`
- `Get(key string) (string, bool)`
- `Find(query string) map[string]string`
- `DeepSearch(pattern string) map[string]string`
- `ReverseSearch(value string) []string`
- `SchemaNames() []string`
- `TypeNames() []string`
- `LastError() string`
- `Close()`

## Notes

- `Close()` is idempotent and should be called when done.
- Methods on a closed handle return empty values or errors, depending on method.
