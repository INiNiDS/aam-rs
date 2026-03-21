# aam-rs for C#

C# bindings for the `aam-rs` AAML parser using the stable C FFI.

## Features

- **AAML Parsing**: Parse and manipulate `.aam` configuration files
- **Cross-platform**: Supports Windows, macOS, and Linux (x64, ARM64)
- **Type-safe**: Strongly-typed C# API with proper resource management
- **Zero-copy**: Efficient bindings using P/Invoke
- **Schema Validation**: Built-in schema and type system support

## Installation

### Via NuGet

```bash
dotnet add package aam-csharp
```

Or via Package Manager:

```powershell
Install-Package aam-csharp
```

## Prerequisites

The managed package requires the native `aam_rs` library at runtime:

- Windows: `aam_rs.dll`
- macOS: `libaam_rs.dylib`
- Linux: `libaam_rs.so`

The NuGet package includes pre-built native libraries for:

- Windows (x64)
- macOS (x64, ARM64)
- Linux (x64)

### For local development

Build the native library from the repository:

```bash
cd aam-rs/
cargo build --release --features ffi
```

Then set the library path:

- **Linux/macOS**: `export LD_LIBRARY_PATH=/path/to/target/release:$LD_LIBRARY_PATH`
- **Windows**: Add the directory to your `PATH`

## Usage

### Basic Parsing

```csharp
using AamRs;

// Parse AAML content
using var doc = AamDocument.Parse("host = localhost\nport = 8080");

// Look up values
string host = doc.FindObj("host");      // "localhost"
string port = doc.FindObj("port");      // "8080"
```

### Merging Configurations

```csharp
using var doc = AamDocument.Parse("host = localhost\nport = 8080");

// Merge additional configuration
doc.Merge("port = 9090\ndebug = true");

var port = doc.FindObj("port");   // "9090"
var debug = doc.FindObj("debug"); // "true"
```

### Finding Keys by Value

```csharp
using var doc = AamDocument.Parse(@"
database = postgres
cache = redis
messaging = rabbitmq
");

// Search by value to find its key
string key = doc.FindKey("postgres"); // "database"
```

### Loading from Files

```csharp
using var doc = AamDocument.Load("config.aam");
```

## Building and Testing

### Solution Layout

- `aam-rs.csproj` - class library (NuGet package)
- `tests/AamRs.Tests.csproj` - test project
- `examples/AamRs.Examples.csproj` - runnable examples
- `apps/AamRs.Console/AamRs.Console.csproj` - console app
- `AamRs.sln` - solution that includes all projects

### Build

```bash
dotnet build AamRs.sln
```

### Run Tests

```bash
dotnet test AamRs.sln
```

### Run Examples

```bash
dotnet run --project examples/AamRs.Examples.csproj -- basic
dotnet run --project examples/AamRs.Examples.csproj -- load
```

### Run Console App

```bash
dotnet run --project apps/AamRs.Console/AamRs.Console.csproj -- parse "host = localhost"
dotnet run --project apps/AamRs.Console/AamRs.Console.csproj -- load ./examples/config.aam
```

Note: Some tests require the native library to be available. On CI systems without native artifacts, tests gracefully
skip with `DllNotFoundException`.

## Supported Platforms

| Platform | Arch   | Status |
|----------|--------|--------|
| Windows  | x86_64 | ✓      |
| macOS    | x86_64 | ✓      |
| macOS    | ARM64  | ✓      |
| Linux    | x86_64 | ✓      |

## API Reference

### `AamDocument`

Main class for AAML document handling.

#### `Parse(string content): AamDocument`

Parses AAML content from a string.

#### `Load(string path): AamDocument`

Loads an AAML file from disk.

#### `FindObj(string key): string?`

Finds an object/value by key.

#### `FindKey(string value): string?`

Finds a key by its value.

#### `FindDeep(string path): string?`

Finds a nested value using dot notation (e.g., "parent.child").

#### `Merge(string content): void`

Merges additional AAML content into the current document.

#### `Dispose(): void`

Releases native resources. Implement via `using` statement.

## Error Handling

The C# bindings use standard .NET exceptions:

```csharp
try
{
    using var doc = AamDocument.Parse("invalid content");
}
catch (InvalidOperationException ex)
{
    // Handle parse errors
    Console.WriteLine($"Parse error: {ex.Message}");
}
catch (DllNotFoundException ex)
{
    // Handle missing native library
    Console.WriteLine($"Native library not found: {ex.Message}");
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../LICENSE-MIT))

at your option.


