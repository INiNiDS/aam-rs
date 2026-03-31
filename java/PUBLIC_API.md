# Public API (Java/Kotlin JVM)

Source of truth: `java/src/main/kotlin/AamDocument.kt`.

## Main Type

- `com.rustgames.aam.AamDocument : AutoCloseable`

## Construction

- `AamDocument.parse(String content)`
- `AamDocument.load(String path)`

## Instance Methods

- `void reload(String content)`
- `String? get(String key)`
- `Map<String, String> deepSearch(String pattern)`
- `List<String> reverseSearch(String value)`
- `List<String> schemaNames()`
- `List<String> typeNames()`
- `void close()`

## Runtime Notes

- Native library is loaded from packaged resources at runtime.
- `IllegalStateException` is thrown on parse/load failures and closed-handle usage.
