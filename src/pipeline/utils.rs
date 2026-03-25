use crate::pipeline::execution_descriptor::ExecutionContext;

/// Validates a value against a type, handling built-ins, registered custom types, and nested schemas.
pub fn validate_type_value(
    value: &str,
    type_name: &str,
    context: &ExecutionContext,
) -> Result<(), String> {
    // 1. Registered custom type alias
    if let Some(custom_type) = context.types.get(type_name) {
        return validate_type_value(value, &custom_type.spec, context);
    }

    // 2. Nested schema — type_name matches a registered schema name
    if let Some(schema_info) = context.schemas.get(type_name) {
        return validate_inline_object_against_schema(value, schema_info, context);
    }

    // 3. Built-in types
    match crate::types_aam::resolve_builtin(type_name) {
        Ok(validator) => validator.validate(value, context).map_err(|e| e.to_string()),
        Err(_) => Err(format!("Unknown type '{}'", type_name)),
    }
}

pub fn validate_inline_object_against_schema(
    value: &str,
    schema_info: &crate::pipeline::execution_descriptor::SchemaInfo,
    context: &ExecutionContext,
) -> Result<(), String> {
    if !crate::aaml::parsing::is_inline_object(value) {
        return Err(format!(
            "Field typed as schema '{}' must be an inline object '{{ k = v, ... }}'",
            schema_info.name
        ));
    }

    let pairs = crate::aaml::parsing::parse_inline_object(value).map_err(|e| e.to_string())?;

    let mut pair_map = std::collections::HashMap::new();
    for (k, v) in &pairs {
        pair_map.insert(k.as_str(), v.as_str());
    }

    for (field, (type_name, is_optional)) in &schema_info.fields {
        match pair_map.get(field.as_str()) {
            None => {
                if !is_optional {
                    return Err(format!(
                        "Missing required field '{}' in inline object for schema '{}'",
                        field, schema_info.name
                    ));
                }
            }
            Some(field_value) => {
                validate_type_value(field_value, type_name, context)?;
            }
        }
    }

    Ok(())
}
