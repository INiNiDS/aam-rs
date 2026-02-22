//! Internal type registry helpers used by [`AAML`](super::AAML).

use crate::error::AamlError;
use crate::types::{resolve_builtin, Type};

/// Validates `value` against a built-in or registered type identified by `type_name`.
///
/// Resolution order:
/// 1. `registered` — the caller's own type map (e.g. from `@type` aliases).
/// 2. Built-in primitives and module paths via [`resolve_builtin`].
///
/// # Errors
/// - [`AamlError::SchemaValidationError`] — value is invalid for the type.
/// - [`AamlError::SchemaValidationError`] with `"Unknown type"` — type not found anywhere.
pub(super) fn validate_field_type(
    registered: Option<&Box<dyn Type>>,
    type_name: &str,
    value: &str,
    schema_name: &str,
    field: &str,
) -> Result<(), AamlError> {
    if let Some(type_def) = registered {
        return type_def.validate(value).map_err(|e| AamlError::SchemaValidationError {
            schema: schema_name.to_string(),
            field: field.to_string(),
            type_name: type_name.to_string(),
            details: e.to_string(),
        });
    }

    match resolve_builtin(type_name) {
        Ok(type_def) => type_def.validate(value).map_err(|e| AamlError::SchemaValidationError {
            schema: schema_name.to_string(),
            field: field.to_string(),
            type_name: type_name.to_string(),
            details: e.to_string(),
        }),
        Err(_) => Err(AamlError::SchemaValidationError {
            schema: schema_name.to_string(),
            field: field.to_string(),
            type_name: type_name.to_string(),
            details: format!("Unknown type '{}'", type_name),
        }),
    }
}

