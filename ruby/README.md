# aam-ruby

Ruby bindings for `aam-rs`, built with `magnus`.

## Features

- Parse AAML from string input.
- Direct lookup and reverse lookup fallback through `parse_find_obj`.
- Lightweight native extension with tiny Ruby API surface.

## Build extension

```bash
cargo build --manifest-path ruby/ext/aam_rb/Cargo.toml --release
```

## Build gem

```bash
cp ruby/ext/aam_rb/target/release/libaam_rs_ruby.so ruby/lib/aam_rs_ruby.so
gem build ruby/aam-ruby.gemspec
```

## Usage

```ruby
require_relative 'ruby/lib/aam_rs'

value = AamRs.parse_find_obj("host = localhost\nport = 8080", 'host')
puts value
```

## More examples

```ruby
require_relative 'ruby/lib/aam_rs'

# reverse lookup fallback
key = AamRs.parse_find_obj("host = localhost", 'localhost')
puts key # host

# missing values return nil
missing = AamRs.parse_find_obj("host = localhost", 'missing')
puts missing.nil?
```

## Tests

```bash
cargo test --manifest-path ruby/ext/aam_rb/Cargo.toml
ruby ruby/tests/test_aam_rs.rb
```

Ruby tests now cover successful lookups, reverse lookup behavior, missing key handling, and parse errors.

