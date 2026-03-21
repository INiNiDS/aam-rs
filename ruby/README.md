# aam-rs Ruby bindings

Minimal Ruby extension built with `magnus`.

## Build extension

```bash
cargo build --manifest-path ruby/ext/aam_rs/Cargo.toml --release
```

## Run tests

```bash
cargo test --manifest-path ruby/ext/aam_rs/Cargo.toml
ruby ruby/tests/test_aam_rs.rb
```

## Ruby usage

```ruby
require_relative 'lib/aam_rs'

value = AamRs.parse_find_obj("host = localhost\nport = 8080", 'host')
puts value # localhost
```

