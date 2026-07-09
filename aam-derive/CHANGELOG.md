# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.8.0](https://github.com/INiNiDS/aam-rs/releases/tag/2.8.0) - 2026-07-09

### Added

- enhance code quality with Clippy lints and improve formatting
- Enhance schema validation and improve parser error handling
- Update package names and improve Ruby gem publishing workflow
- Add C# bindings for aam-rs with support for AAM parsing and configuration management
- Add Node.js bindings for aam-rs with platform-specific packages and build configuration
- add C FFI bindings support and gh actions
- Added schema-in-schema, new examples and list

### Fixed

- use explicit rust-version instead of workspace inheritance
- update migration guide, build.yml, and README for clarity and accuracy
- update README and release-please config for version 2.0.0
- Update references from AAML to AAM and improve method names for consistency
- fixed mismatched types when use ahash
- Now derive automatically import all types that used in schema
- Fixed using schema as type in resolve_builtin()

### Other

- Bump version to 2.8.0 across all crates and bindings
- Add aam-derive proc-macro crate with #[derive(FromAam)] and schema_to_struct!
- Split single crate into workspace (aam-core, aam-derive, aam-rs)
- Update origin story in README.md for AAM
- Refactored to GitLab
- *(release)* now release please for non-Rust package versions and release-plz for Rust Packages.
- *(main)* release aam-rs 2.0.4
- *(main)* release aam-rs 2.0.3
- Enhance README with AAM origin story and purpose
- Expand README with AAM advantages and CLI details
- *(main)* release aam-rs 2.0.2
- *(main)* release aam-rs 2.0.1
- Full renamed packages.
- Rename aam_rs to aam_rb and update related references; add reverse lookup tests
- Update repository
- Update Cargo for categories and README
- Fix small mistackes
- Updated Cargo.toml and added README.md for crates.io
