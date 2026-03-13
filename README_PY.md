# AAM (Abstract Alias Mapping)

A robust and lightweight configuration parser for Python that supports key-value pairs, recursive dependency resolution,
file imports, and bidirectional lookups. Designed for applications that need flexible configuration files with
references, aliases, and a modular structure.

## Features

* Simple syntax: A key = value format that is easy to read and write.
* Import support: The @import directive lets you split configuration into multiple files.
* Comments support: Lines starting with # are treated as comments.
* Deep resolution (find_deep): Automatically resolves chains of references (e.g., A -> B -> C) to find the final value.
* Loop detection: Safely handles circular dependencies (e.g., A -> B -> A) without RecursionError.
* Bidirectional lookup (find_obj): Looks up a value by key, or performs a reverse lookup (finds a key by value) when the
  key is missing.

  Config builder (AAMBuilder): Programmatically generate and save .aam files.

  Configuration merging: Supports the + operator to combine two AAML instances.

  Typed errors: Detailed parsing and I/O error handling via AamlError exceptions.

Format

You can find documentation and examples for the format in the official documentation.
Installation

Install the library using pip:
Bash

pip install aam-py

Configuration syntax (.aam)

The format is line-based. Whitespace around keys and values is trimmed. Strings can be quoted.
Фрагмент кода

# This is a comment

host = "localhost"
port = 8080

# Import other configuration files

@import database.aam
@import theme.aam

# You can define aliases for deep lookup

base_path = /var/www
current_path = base_path

# Circular references are handled safely

loop_a = loop_b
loop_b = loop_a

Usage guide

1) Parsing and loading

You can parse configuration from a string, load it from a file, or merge multiple sources. Errors are handled via
exceptions inheriting from AamlError.
Python

from aam import AAML
from aam.error import AamlError

# 1. Parse from string

content = """
username = admin
timeout = 30
"""
try:
config = AAML.parse(content)
except AamlError as e:
print(f"Parsing error: {e}")

# 2. Load from file (supports @import directives)

file_config = AAML.load("config.aam")

2) Merging configurations

Combine different AAML objects using the addition operator.
Python

config1 = AAML.parse("a = 1")
config2 = AAML.parse("b = 2")

# Merge (config2 overwrites matching keys in config1)

config1 += config2

# or: config3 = config1 + config2

3) Smart lookup (find_obj)

find_obj is a hybrid lookup method. It first tries to find a value by the given key. If the key does not exist, it
searches for a key whose value matches the provided string.
Python

content = """

# Key = Value

app_mode = production
debug = false
"""
config = AAML.parse(content)

# Scenario A: Direct key lookup

mode = config.find_obj("app_mode")
assert mode == "production"

# Scenario B: Reverse lookup

# "production" is not a key, so it looks for a key with value "production"

key = config.find_obj("production")
assert key == "app_mode"

4) Deep recursive lookup (find_deep)

This is useful for aliasing. It follows values as keys until it reaches a value that is not present as a key, or until a
loop is detected.
Python

content = """
root = /usr/bin
executable = root
service = executable
"""
config = AAML.parse(content)

# Traces: "service" -> "executable" -> "root" -> "/usr/bin"

final_val = config.find_deep("service")
assert final_val == "/usr/bin"

Handling loops: If the configuration contains a loop (e.g., a=b, b=a), find_deep returns the last unique value visited
before the loop closes, preventing infinite recursion.

5) Building configurations (AAMBuilder)

Use AAMBuilder to generate configuration files programmatically.
Python

from aam.builder import AAMBuilder, SchemaField

builder = AAMBuilder()
(builder.comment("Server configuration")
.type_alias("port_t", "int")
.schema("Server", [
SchemaField.required("host", "string"),
SchemaField.required("port", "port_t"),
SchemaField.optional("debug", "bool"),
])
.add_line("host", "127.0.0.1")
.add_line("port", "8000"))

# Save to file

builder.to_file("generated_config.aam")

# Or convert to string

print(str(builder))

6) Working with FoundValue

Lookup results are wrapped in a FoundValue object (which behaves like a standard Python string). You can use it just
like a regular string, and it also provides helper methods for in-place modification.
Python

config = AAML.parse("greeting = Hello World")
value = config.find_obj("greeting")

# Use as a string

print(f"Original: {value}") # Prints: Hello World

# Modify in-place

value.remove(" World")
assert str(value) == "Hello"

API reference
AAML

    parse(content: str) -> 'AAML': Parses a string into an AAML object.

    load(file_path: str | Path) -> 'AAML': Loads and parses a file, handling imports.

    merge_content(self, content: str) -> None: Merges content into the current instance.

    merge_file(self, path: str | Path) -> None: Reads a file and merges it.

    find_obj(self, key: str) -> FoundValue | None: Smart bidirectional lookup.

    find_deep(self, key: str) -> FoundValue | None: Recursive lookup with loop detection.

    find_key(self, value: str) -> FoundValue | None: Strict reverse lookup (find key by value).

AAMBuilder

    __init__(self): Creates a new builder.

    add_line(self, key: str, value: str): Adds a key = value pair.

    comment(self, text: str): Adds a # text comment line.

    schema(self, name: str, fields: Iterable[SchemaField]): Adds a @schema Name { ... } directive (inline).

    schema_multiline(self, name: str, fields: Iterable[SchemaField]): Adds a @schema Name { ... } directive (one field per line).

    derive(self, path: str, schemas: Iterable[str]): Adds a @derive path[::Schema...] directive.

    import_(self, path: str): Adds an @import path directive (note the trailing _ since import is a Python keyword).

    type_alias(self, alias: str, type_name: str): Adds a @type alias = type_name directive.

    add_raw(self, raw_line: str) (deprecated): Adds a raw line as-is. Prefer the typed methods above.

    to_file(self, path: str | Path) -> None: Writes the buffer to a file.

AamlError

Base exception class for the library.

    IoError: Wraps standard I/O errors (e.g., FileNotFoundError).

    ParseError: Syntax errors (includes line number and details).

    NotFound: Key not found (mostly for internal use).

License

See the LICENSE file.
Full Documentation

Full API documentation is available at our website (note: this link can be updated to point to your Python documentation
hosting, like ReadTheDocs).