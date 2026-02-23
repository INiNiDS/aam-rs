//! `@schema` directive — defines a named struct-like schema with typed fields.
//!
//! # Syntax
//! ```text
//! @schema Name { field1: type1, field2: type2, ... }
//! ```
//!
//! # Semantics
//! After a schema is registered any `key = value` assignment whose key matches
//! a schema field is automatically validated against the declared type.
//! Use [`AAML::apply_schema`] to validate a complete data map programmatically.

use std::collections::HashMap;
use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

/// A parsed schema definition: maps field names to their declared type strings.
///
/// Type strings can be primitives (`i32`, `f64`, `string`, `bool`, `color`),
/// built-in module paths (`math::vector3`, `physics::kilogram`, `time::datetime`),
/// or custom aliases registered via `@type`.
#[derive(Clone, Debug)]
pub struct SchemaDef {
    /// Map of `field_name → type_name`.
    pub fields: HashMap<String, String>,
}

/// Command handler for the `@schema` directive.
pub struct SchemaCommand;

impl SchemaCommand {
    /// Splits `args` into the schema name and the raw body between `{` and `}`.
    fn parse_header(args: &str) -> Result<(&str, &str), AamlError> {
        let (name_part, body_part) = args
            .split_once('{')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '{'".into()))?;

        let name = name_part.trim();
        if name.is_empty() {
            return Err(AamlError::DirectiveError("schema".into(), "Schema name is empty".into()));
        }

        let body = body_part
            .rsplit_once('}')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '}'".into()))?
            .0;

        Ok((name, body))
    }

    /// Parses a single `field:type` token pair, consuming an extra token from
    /// `tokens` when the colon is at the end (`"field:"`).
    fn parse_field<'a>(
        token: &'a str,
        tokens: &mut impl Iterator<Item = &'a str>,
    ) -> Result<(String, String), AamlError> {
        let (field, ty) = token
            .split_once(':')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), format!("Bad field: '{token}'")))?;

        // "field:type" or "field:" — type may follow as the next token.
        let ty = if ty.is_empty() {
            tokens.next().ok_or_else(|| {
                AamlError::DirectiveError("schema".into(), format!("Bad field: '{field}:' has no type"))
            })?
        } else {
            ty
        };

        if field.is_empty() || ty.is_empty() {
            return Err(AamlError::DirectiveError(
                "schema".into(),
                format!("Bad field: '{field}: {ty}'"),
            ));
        }

        Ok((field.to_string(), ty.to_string()))
    }

    /// Parses the raw argument string into a `(name, SchemaDef)` pair.
    ///
    /// Expected format: `Name { field: type, ... }`
    fn parse(args: &str) -> Result<(String, SchemaDef), AamlError> {
        let (name, body) = Self::parse_header(args.trim())?;

        // Normalize: commas and whitespace are both valid field separators.
        // Replace commas with spaces so we can use split_whitespace uniformly.
        let normalized = body.replace(',', " ");
        let mut tokens = normalized.split_whitespace();
        let mut fields = HashMap::new();

        while let Some(token) = tokens.next() {
            let (field, ty) = Self::parse_field(token, &mut tokens)?;
            fields.insert(field, ty);
        }

        Ok((name.to_string(), SchemaDef { fields }))
    }
}

impl Command for SchemaCommand {
    fn name(&self) -> &str { "schema" }

    /// Parses the schema definition and registers it in the current [`AAML`] instance.
    ///
    /// If a schema with the same name already exists it is **replaced**.
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let (name, schema) = Self::parse(args)?;
        aaml.get_schemas_mut().insert(name, schema);
        Ok(())
    }
}
