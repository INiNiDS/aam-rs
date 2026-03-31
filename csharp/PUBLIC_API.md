# Public API (C#)

Source of truth: `csharp/src/AamDocument.cs`.

## Main Type

- `AamDocument : IDisposable`

## Construction

- `new AamDocument()`
- `AamDocument.Parse(string content)`
- `AamDocument.Load(string path)`

## Core Methods

- `string Format(string content)`
- `void Merge(string content)`
- `void RecoverSimple(string content)`
- `string? FindObj(string key)`
- `string? FindKey(string value)`
- `string? FindDeep(string key)`

## Lifecycle

- `bool IsClosed`
- `void Dispose()`

## Error Model

- Throws `AamException` for native parse/load/merge/format/recovery failures.
- Throws `ObjectDisposedException` when using a closed instance.
