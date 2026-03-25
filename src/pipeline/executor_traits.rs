//! Executor trait implementations for task-based validation and parsing.
//!
//! These executors consume declarative tasks and perform the actual execution,
//! completely decoupled from AAML struct manipulation.

use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::tasks::{ValidationTask, ParseTask, TaskExecutionResult, TaskError};
use crate::pipeline::execution_descriptor::ExecutionContext;
use crate::pipeline::lexer::Lexer;
use crate::pipeline::parser::Parser;
use std::collections::HashMap;

/// Trait for executing validation tasks.
///
/// A ValidateExecutor takes a stream of ValidationTasks and executes them,
/// returning aggregated results suitable for LSP integration.
pub trait ValidateExecutor: Send + Sync {
    /// Executes a single validation task within a given context.
    ///
    /// # Arguments
    /// - `task`: The validation task to execute
    /// - `context`: The current execution context
    ///
    /// # Returns
    /// - `Ok(true)` if validation passed
    /// - `Err(AamlError)` if validation failed
    fn execute_validation(
        &self,
        task: &ValidationTask,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError>;

    /// Executes multiple validation tasks and aggregates results.
    ///
    /// This method is the main entry point for validation execution.
    /// It collects errors from multiple failing tasks rather than stopping
    /// at the first error, which is critical for LSP support.
    ///
    /// # Arguments
    /// - `tasks`: All validation tasks to execute
    /// - `context`: The execution context
    ///
    /// # Returns
    /// - `TaskExecutionResult` containing success status and error collection
    fn execute_batch(
        &self,
        tasks: &[ValidationTask],
        context: &ExecutionContext,
    ) -> TaskExecutionResult {
        let mut errors = Vec::new();
        let mut successful = 0;

        for task in tasks {
            match self.execute_validation(task, context) {
                Ok(true) => {
                    successful += 1;
                }
                Ok(false) => {
                    // Task executed but validation failed
                    errors.push(TaskError {
                        line: task.line(),
                        message: format!("Validation failed: {}", task.description()),
                        task_description: task.description(),
                        aaml_error: None,
                    });
                }
                Err(e) => {
                    errors.push(TaskError {
                        line: task.line(),
                        message: format!("Validation error: {}", e),
                        task_description: task.description(),
                        aaml_error: Some(format!("{:?}", e)),
                    });
                }
            }
        }

        TaskExecutionResult {
            success: errors.is_empty(),
            errors,
            stats: Default::default(),
        }
    }
}

/// Trait for executing parsing tasks.
///
/// A ParserExecutor takes parse tasks and performs parsing operations,
/// managing scopes, variables, and directive registration.
pub trait ParserExecutor: Send + Sync {
    /// Executes a single parsing task.
    ///
    /// # Arguments
    /// - `task`: The parsing task to execute
    /// - `context`: Mutable execution context (may be modified)
    ///
    /// # Returns
    /// - `Ok(())` if parsing succeeded
    /// - `Err(AamlError)` if parsing failed
    fn execute_parse(
        &self,
        task: &ParseTask,
        context: &mut ExecutionContext,
    ) -> Result<(), AamlError>;

    /// Executes multiple parsing tasks in sequence.
    ///
    /// # Arguments
    /// - `tasks`: All parsing tasks to execute
    /// - `context`: Mutable execution context
    ///
    /// # Returns
    /// - `TaskExecutionResult` with aggregated results
    fn execute_batch(
        &self,
        tasks: &[ParseTask],
        context: &mut ExecutionContext,
    ) -> TaskExecutionResult {
        let mut errors = Vec::new();
        let mut successful = 0;

        for task in tasks {
            match self.execute_parse(task, context) {
                Ok(()) => {
                    successful += 1;
                }
                Err(e) => {
                    errors.push(TaskError {
                        line: task.line(),
                        message: format!("Parse error: {}", e),
                        task_description: task.description(),
                        aaml_error: Some(format!("{:?}", e)),
                    });
                }
            }
        }

        TaskExecutionResult {
            success: errors.is_empty(),
            errors,
            stats: Default::default(),
        }
    }
}

/// Default implementation of ValidateExecutor.
///
/// This provides basic validation task execution with type checking and schema verification.
pub struct DefaultValidateExecutor {
    // Can hold shared registries or validation strategies
}

impl DefaultValidateExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// Checks if a type exists in the context's type registry.
    fn type_exists(&self, context: &ExecutionContext, type_name: &str) -> bool {
        context.types.contains_key(type_name)
            || Self::is_builtin_type(type_name)
    }

    /// Checks if a built-in type is recognized.
    fn is_builtin_type(type_name: &str) -> bool {
        matches!(
            type_name,
            "string"
                | "i32"
                | "f64"
                | "bool"
                | "color"
                | "vector2"
                | "vector3"
                | "vector4"
                | "matrix4x4"
                | "kilogram"
                | "datetime"
        )
    }

    /// Validates a value against a type (basic implementation).
    fn validate_type_value(&self, value: &str, type_name: &str, context: &ExecutionContext) -> Result<(), String> {
        crate::pipeline::utils::validate_type_value(value, type_name, context)
    }
}

impl Default for DefaultValidateExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidateExecutor for DefaultValidateExecutor {
    // High Complexity
    fn execute_validation(
        &self,
        task: &ValidationTask,
        context: &ExecutionContext,
    ) -> Result<bool, AamlError> {
        match task {
            ValidationTask::CheckTypeMatch {
                key,
                value,
                type_name,
                line,
            } => {
                if !self.type_exists(context, type_name) {
                    return Err(AamlError::InvalidType {
                        type_name: type_name.clone(),
                        details: format!("Type not found in registry for key '{}'", key),
                        provided: value.clone(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Unknown type",
                            format!("Type '{}' is not registered", type_name),
                            "Register the type using @type directive".to_string(),
                        )),
                    });
                }

                if let Err(e) = self.validate_type_value(value, type_name, context) {
                    return Err(AamlError::InvalidType {
                        type_name: type_name.clone(),
                        details: format!("Validation failed for key '{}'", key),
                        provided: value.clone(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Type validation failed",
                            e,
                            format!("Ensure '{}' conforms to type '{}'", value, type_name),
                        )),
                    });
                }

                Ok(true)
            }

            ValidationTask::VerifySchemaExists { schema_name, line } => {
                if context.schemas.contains_key(schema_name) {
                    Ok(true)
                } else {
                    Err(AamlError::NotFound {
                        key: schema_name.clone(),
                        context: "Schema not found in registry".to_string(),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Schema not defined",
                            format!("Schema '{}' referenced but not defined", schema_name),
                            "Define it using @schema directive".to_string(),
                        )),
                    })
                }
            }

            ValidationTask::VerifyFileExists { path, line } => {
                if std::path::Path::new(path).exists() {
                    Ok(true)
                } else {
                    Err(AamlError::IoError {
                        details: format!("Imported file '{}' not found", path),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "File missing",
                            format!("The file '{}' does not exist.", path),
                            "Check the file path in your import directive.",
                        )),
                    })
                }
            }

            ValidationTask::CheckNoCircularReference { key, line } => {
                let mut current_key: &str = key;
                let mut visited = std::collections::HashSet::new();
                
                while let Some(next_val) = context.map.get(current_key) {
                    if !visited.insert(current_key) {
                        return Err(AamlError::CircularDependency {
                            path: format!("{} -> {}", key, next_val),
                            diagnostics: Some(ErrorDiagnostics::new(
                                "Circular reference detected",
                                format!("Key '{}' references itself directly or indirectly", key),
                                "Break the reference loop".to_string()
                            ))
                        });
                    }
                    if context.map.contains_key(&**next_val) {
                        current_key = next_val;
                    } else {
                        break;
                    }
                }
                
                Ok(true)
            }

            ValidationTask::CheckDeriveCompleteness { derive_path, current_key, line: _ } => {
                let parts: Vec<&str> = derive_path.split("::").collect();
                if parts.len() < 2 {
                    return Ok(true); // Nothing to validate if no schemas are specified
                }

                // Skip the first part (filename)
                let schema_names = &parts[1..];

                for schema_name in schema_names {
                    let schema = context.schemas.get(*schema_name).ok_or_else(|| {
                        AamlError::NotFound {
                            key: schema_name.to_string(),
                            context: "schema derivation".to_string(),
                            diagnostics: Some(ErrorDiagnostics::new(
                                "Schema not defined",
                                format!("Schema '{}' referenced in derive chain but not defined", schema_name),
                                "Ensure the file being derived from defines this schema",
                            )),
                        }
                    })?;

                    for (field, (type_name, is_optional)) in &schema.fields {
                        if !is_optional {
                            let full_key = if current_key.is_empty() {
                                field.clone()
                            } else {
                                format!("{}.{}", current_key, field)
                            };

                            // Check if the current context has this key defined
                            // The AST might not have been fully executed into map yet during validation,
                            // but `context.map` and `context.parsed_outputs` track what we have so far.
                            // For a robust implementation, `current_key` is typically the scope,
                            // and we need to check if a value for `full_key` exists.
                            // Currently `context.map` is populated during parsing (ProcessVariable).
                            if !context.map.contains_key(full_key.as_str()) {
                                return Err(AamlError::SchemaValidationError {
                                    schema: schema_name.to_string(),
                                    field: field.clone(),
                                    type_name: type_name.clone(),
                                    details: format!("Missing required field '{}' from derived schema '{}'", field, schema_name),
                                    diagnostics: Some(ErrorDiagnostics::new(
                                        "Incomplete derivation",
                                        format!("Derived object missing required field: {}", field),
                                        "Add the field to satisfy the derived schema",
                                    )),
                                });
                            }
                        }
                    }
                }
                Ok(true)
            }

            ValidationTask::ValidateAgainstSchema { schema_name, key, value, line: _ } => {
                let schema_info = context.schemas.get(schema_name).ok_or_else(|| {
                    AamlError::SchemaValidationError {
                        schema: schema_name.clone(),
                        field: key.clone(),
                        type_name: "schema".to_string(),
                        details: format!("Schema '{}' not found", schema_name),
                        diagnostics: None,
                    }
                })?;

                if let Err(e) = crate::pipeline::utils::validate_inline_object_against_schema(value, schema_info, context) {
                    return Err(AamlError::SchemaValidationError {
                        schema: schema_name.clone(),
                        field: key.clone(),
                        type_name: "schema".to_string(),
                        details: e,
                        diagnostics: None,
                    });
                }

                Ok(true)
            }

            ValidationTask::CheckSchemaCompleteness {
                schema_name,
                missing_fields,
                line,
            } => {
                if missing_fields.is_empty() {
                    Ok(true)
                } else {
                    Err(AamlError::SchemaValidationError {
                        schema: schema_name.clone(),
                        field: missing_fields.join(", "),
                        type_name: "required".to_string(),
                        details: format!(
                            "Schema incomplete: missing required fields: {}",
                            missing_fields.join(", ")
                        ),
                        diagnostics: None,
                    })
                }
            }
            ValidationTask::ValidateListElements {
                key,
                items,
                element_type,
                line,
            } => {
                let mut all_valid = true;
                for item in items.iter() {
                    if let Err(e) = self.validate_type_value(&item.to_string(), element_type, context) {
                        all_valid = false;
                        return Err(AamlError::InvalidType {
                            type_name: element_type.clone(),
                            details: format!("List element invalid in '{}'", key),
                            provided: item.to_string(),
                            diagnostics: Some(ErrorDiagnostics::new(
                                "List element validation failed",
                                e,
                                format!(
                                    "All elements in list must be of type '{}'",
                                    element_type
                                ),
                            )),
                        });
                    }
                }
                Ok(true)
            }

            ValidationTask::ValidateObjectStructure { key, pairs, line } => {
                if pairs.is_empty() {
                    return Err(AamlError::InvalidValue {
                        details: format!("Empty object for key '{}'", key),
                        expected: "non-empty object".to_string(),
                        diagnostics: None,
                    });
                }
                Ok(true)
            }
        }
    }
}

/// Default implementation of ParserExecutor.
///
/// This processes parsing tasks like variable registration, scope management,
/// and directive execution.
pub struct DefaultParserExecutor {
    // Can hold shared registries and command handlers
}

impl DefaultParserExecutor {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for DefaultParserExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserExecutor for DefaultParserExecutor {
    // High Complexity
    fn execute_parse(
        &self,
        task: &ParseTask,
        context: &mut ExecutionContext,
    ) -> Result<(), AamlError> {
        match task {
            ParseTask::ProcessVariable {
                variable_name,
                value,
                scope,
                line,
            } => {
                // Record the variable in the current scope
                context.set_value(variable_name.clone(), value.clone(), *line);
                Ok(())
            }

            ParseTask::ManageScope { scope, is_entry, line } => {
                if *is_entry {
                    context.push_scope(scope.clone());
                } else {
                    context.pop_scope();
                }
                Ok(())
            }

            ParseTask::ExecuteDirective {
                directive_name,
                arguments,
                line,
            } => {
                match directive_name.as_str() {
                    "import" | "derive" => {
                        // Handled in Execution Phase
                        Ok(())
                    },
                    _ => Err(AamlError::ParseError {
                        line: *line,
                        content: format!("@{} {}", directive_name, arguments),
                        details: format!("Unknown directive: @{}", directive_name),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Unknown directive",
                            format!("Directive '@{}' is not recognized", directive_name),
                            "Known directives: @import, @derive, @schema, @type",
                        )),
                    }),
                }
            }

            ParseTask::RegisterType {
                type_name,
                type_spec,
                line,
            } => {
                context.register_type(crate::pipeline::execution_descriptor::TypeInfo {
                    name: type_name.clone(),
                    spec: type_spec.clone(),
                    validator: None,
                    line: *line,
                });
                Ok(())
            }

            ParseTask::RegisterSchema {
                schema_name,
                fields,
                line,
            } => {
                // Parse fields (simplified: "field1:type1,field2:type2")
                let mut schema_fields = HashMap::new();
                for field_def in fields.split(',') {
                    let parts: Vec<&str> = field_def.trim().split(':').collect();
                    if parts.len() == 2 {
                        let field_name = parts[0].trim().to_string();
                        let type_name = parts[1].trim().to_string();
                        let is_optional = field_name.ends_with('*');
                        let clean_name = if is_optional {
                            field_name.trim_end_matches('*').to_string()
                        } else {
                            field_name
                        };
                        schema_fields.insert(clean_name, (type_name, is_optional));
                    }
                }

                context.register_schema(crate::pipeline::execution_descriptor::SchemaInfo {
                    name: schema_name.clone(),
                    fields: schema_fields,
                    line: *line,
                });
                Ok(())
            }

            ParseTask::ResolveDeriveImport { derive_path, line } => {
                let parts: Vec<&str> = derive_path.split("::").collect();
                if parts.is_empty() {
                    return Err(AamlError::DirectiveError {
                        directive: "derive".to_string(),
                        message: "Empty derive path".to_string(),
                        diagnostics: None,
                    });
                }
                
                let file_path = parts[0].to_string();
                if !context.is_imported(&file_path) {
                    let content = std::fs::read_to_string(&file_path).map_err(|e| {
                        AamlError::IoError {
                            details: format!("Failed to read imported file '{}': {}", file_path, e),
                            diagnostics: Some(ErrorDiagnostics::new(
                                "Import failed",
                                format!("Could not read file '{}'", file_path),
                                "Check if the file exists and is readable",
                            )),
                        }
                    })?;

                    // Run localized parsing on imported content
                    let lexer = crate::pipeline::lexer::DefaultLexer::new();
                    let parser = crate::pipeline::parser::DefaultParser::new();
                    
                    let tokens = lexer.tokenize(&content)?;
                    let ast = parser.parse(tokens)?;
                    let sub_tasks = parser.generate_parse_tasks(&ast);
                    
                    // Note: We're executing these tasks in the same context to merge types/schemas.
                    // Assignments from the imported file are currently ignored as derive only imports types/schemas,
                    // but we could filter or apply them if needed. For now, we apply them all.
                    for sub_task in sub_tasks {
                        self.execute_parse(&sub_task, context)?;
                    }

                    context.record_import(file_path);
                }
                
                Ok(())
            }

            ParseTask::ResolveModuleReference {
                module_name,
                scope,
                line,
            } => {
                // If it's a known module (cached import), ok, otherwise report not found
                if !context.imported_files.contains(module_name) && !context.schemas.contains_key(module_name) {
                    return Err(AamlError::NotFound {
                        key: module_name.clone(),
                        context: format!("module reference in scope '{}'", scope),
                        diagnostics: Some(ErrorDiagnostics::new(
                            "Module not found",
                            format!("The module '{}' has not been imported or defined", module_name),
                            "Check for a missing @import directive"
                        )),
                    });
                }
                Ok(())
            }
        }
    }
}

