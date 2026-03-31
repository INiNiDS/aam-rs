# Public API (Python)

Source of truth: `src/python.rs` (PyO3 class exported as `aam_py.AAM`).

## Main Class

- `AAM`

## Construction

- `AAM()`
- `AAM.parse(content: str) -> AAM`
- `AAM.load(path: str) -> AAM`

## Utility Static Method

- `AAM.lsp_assist(content: str) -> tuple[list[str], str | None]`

## Instance Methods

- `format(content: str) -> str`
- `format_range(content: str, start_line: int, end_line: int) -> str`
- `get(key: str) -> str | None`
- `keys() -> list[str]`
- `to_dict() -> dict[str, str]`
- `find(query: str) -> dict[str, str]`
- `deep_search(pattern: str) -> dict[str, str]`
- `reverse_search(target_value: str) -> list[str]`
- `schema_names() -> list[str]`
- `type_names() -> list[str]`
- `close() -> None`
- `is_closed() -> bool`

## Python Protocol Methods

- `__repr__`, `__len__`, `__contains__`, `__getitem__`
