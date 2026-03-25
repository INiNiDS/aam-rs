//! Executer stage: materializes validated tasks into final runtime state.
//!
//! The Executer receives an ExecutionDescriptor containing pre-computed tasks
//! and executes them in a completely decoupled manner, WITHOUT any AAML struct dependency.
//! This enables clean separation of concerns, better LSP integration, and potential
//! parallel execution in future iterations.

use crate::error::AamlError;
use crate::pipeline::execution_descriptor::ExecutionDescriptor;
use crate::pipeline::tasks::ExecutionTask;
use crate::found_value::FoundValue;
use std::collections::HashMap;

#[cfg(feature = "perf-hash")]
type Hasher = ahash::RandomState;

#[cfg(not(feature = "perf-hash"))]
type Hasher = std::collections::hash_map::RandomState;

/// Trait for executing the final materialization of configuration state.
///
/// The Executer operates purely on ExecutionDescriptor and task queues.
/// It NO LONGER instantiates or depends on the AAML struct.
pub trait Executer: Send + Sync {
    /// Executes the manifest to produce a final FoundValue.
    ///
    /// # Arguments
    /// - `manifest`: Comprehensive execution manifest with all tasks and context
    ///
    /// # Returns
    /// - `Ok(FoundValue)` on successful execution
    /// - `Err(AamlError)` if any execution phase fails
    fn execute(&self, manifest: &mut ExecutionDescriptor) -> Result<(), AamlError>;
}

/// Default implementation of the Executer stage.
///
/// This executor handles all execution tasks in a clean, isolated manner:
/// 1. Processes SetValue and MergeValue tasks to populate the output map
/// 2. Handles ApplySchema tasks for schema validation and enforcement
/// 3. Manages ExecuteInheritance for configuration inheritance
/// 4. Resolves cross-references via ResolveReference tasks
/// 5. Handles ImportFile tasks for external configuration merging
pub struct DefaultExecuter {
    // Can hold shared registries or execution strategies
}

impl DefaultExecuter {
    pub fn new() -> Self {
        Self {}
    }

    /// Executes a single execution task within the context.
    ///
    /// # Arguments
    /// - `task`: The execution task to perform
    /// - `output_map`: Mutable reference to the output map being built
    ///
    /// # Returns
    /// - `Ok(())` on success
    /// - `Err(AamlError)` on failure
    // High Complexity
    fn execute_task(
        task: &ExecutionTask,
        output_map: &mut HashMap<Box<str>, Box<str>, Hasher>,
        context: &crate::pipeline::execution_descriptor::ExecutionContext,
    ) -> Result<(), AamlError> {
        match task {
            ExecutionTask::SetValue { key, value, .. } => {
                // Set or overwrite a key-value pair
                output_map.insert(
                    key.clone().into_owned().into_boxed_str(),
                    value.clone().into_owned().into_boxed_str(),
                );
                Ok(())
            }

            ExecutionTask::MergeValue { key, value, .. } => {
                // Merge a value with existing entry (e.g., for lists/objects)
                let existing = output_map
                    .get(key.as_ref())
                    .map(|v| v.to_string())
                    .unwrap_or_default();

                // Simple merge strategy: concatenate with separator
                let merged: std::borrow::Cow<'_, str> = if existing.is_empty() {
                    value.clone()
                } else {
                    format!("{} {}", existing, value).into()
                };

                output_map.insert(
                    key.clone().into_owned().into_boxed_str(),
                    merged.into_owned().into_boxed_str(),
                );
                Ok(())
            }

            ExecutionTask::ApplySchema {
                schema_name,
                root_keys,
                line,
            } => {
                let schema = context.schemas.get(schema_name).ok_or_else(|| {
                    AamlError::NotFound {
                        key: schema_name.to_string(),
                        context: "schema registry".to_string(),
                        diagnostics: Some(crate::error::ErrorDiagnostics::new(
                            "Schema not found",
                            format!("Schema '{}' does not exist", schema_name),
                            "Check your @schema definitions",
                        )),
                    }
                })?;

                for key in root_keys {
                    for (field, (type_name, is_optional)) in &schema.fields {
                        let full_key = if key.is_empty() {
                            field.to_string()
                        } else {
                            format!("{}.{}", key, field)
                        };

                        if !output_map.contains_key(full_key.as_str()) {
                            if !is_optional {
                                return Err(AamlError::SchemaValidationError {
                                    schema: schema_name.to_string(),
                                    field: field.to_string(),
                                    type_name: type_name.to_string(),
                                    details: format!("Missing required field '{}'", field),
                                    diagnostics: None,
                                });
                            }
                        } else {
                            let value = output_map.get(full_key.as_str()).unwrap().to_string();
                            if let Err(err_msg) = crate::pipeline::utils::validate_type_value(&value, type_name, context) {
                                return Err(AamlError::SchemaValidationError {
                                    schema: schema_name.to_string(),
                                    field: field.to_string(),
                                    type_name: type_name.to_string(),
                                    details: format!("Type mismatch for field '{}': {}", field, err_msg),
                                    diagnostics: None,
                                });
                            }
                        }
                    }
                }
                Ok(())
            }

            ExecutionTask::ExecuteInheritance {
                derive_path,
                child_key,
                line: _,
            } => {
                let parts: Vec<&str> = derive_path.split("::").collect();
                if parts.len() < 2 {
                    return Ok(()); // Basic inheritance from file only handles types/schemas
                }
                
                let schema_names = &parts[1..];
                
                for schema_name in schema_names {
                    if let Some(schema) = context.schemas.get(*schema_name) {
                        for (field, (type_name, _is_optional)) in &schema.fields {
                            let full_child_key = if child_key.is_empty() {
                                field.to_string()
                            } else {
                                format!("{}.{}", child_key, field)
                            };
                            
                            // Only insert if the child doesn't already have this field defined
                            // This implements the expected "child values win" inheritance
                            if !output_map.contains_key(full_child_key.as_str()) {
                                // Provide a default representation for the inherited field based on its type
                                // In a full implementation, we'd look up the default value from the schema or parent context
                                let default_val = match type_name.as_ref() {
                                    "i32" | "f64" => "0",
                                    "bool" => "false",
                                    "string" => "\"\"",
                                    "vector2" => "[0.0, 0.0]",
                                    "vector3" => "[0.0, 0.0, 0.0]",
                                    "vector4" => "[0.0, 0.0, 0.0, 0.0]",
                                    "matrix4x4" => "[1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]",
                                    "color" => "#000000",
                                    "kilogram" => "0.0kg",
                                    "datetime" => "1970-01-01T00:00:00Z",
                                    list_type if list_type.starts_with("list<") => "[]",
                                    _ => {
                                        if context.schemas.contains_key(type_name.as_ref()) {
                                            "{}"
                                        } else {
                                            "\"\""
                                        }
                                    }
                                };
                                
                                output_map.insert(
                                    full_child_key.into_boxed_str(),
                                    default_val.to_string().into_boxed_str(),
                                );
                            }
                        }
                    }
                }
                Ok(())
            }

            ExecutionTask::ImportFile {
                file_path,
                merge_strategy,
                line: _,
            } => {
                let content = std::fs::read_to_string(file_path.as_ref()).map_err(|e| AamlError::IoError {
                    details: e.to_string(),
                    diagnostics: Some(crate::error::ErrorDiagnostics::new(
                        "I/O operation failed",
                        format!("Could not read imported file '{}': {}", file_path, e),
                        "Check file permissions and ensure the path exists"
                    )),
                })?;
                
                // Spin up a new pipeline instance to process the imported file
                let sub_pipeline = crate::pipeline::Pipeline::new();
                let sub_output = match sub_pipeline.process(&content) {
                    Ok(out) => out,
                    Err(mut errors) => {
                        return Err(errors.pop().unwrap_or(AamlError::DirectiveError {
                            directive: "import".to_string(),
                            message: "Unknown error in imported file".to_string(),
                            diagnostics: None,
                        }));
                    }
                };

                // Merge based on strategy
                for (k, v) in sub_output.map {
                    if merge_strategy == "override" || !output_map.contains_key(&*k) {
                        output_map.insert(k, v);
                    }
                }
                Ok(())
            }

            ExecutionTask::ResolveReference {
                source_key,
                target_key,
                ..
            } => {
                // Reference resolution: copy target value to source (or resolve interpolation)
                if let Some(target_value) = output_map.get(target_key.as_ref()) {
                    output_map.insert(
                        source_key.clone().into_owned().into_boxed_str(),
                        target_value.clone(),
                    );
                } else {
                    return Err(AamlError::NotFound {
                        key: target_key.to_string(),
                        context: format!(
                            "Reference target '{}' not found when resolving '{}'",
                            target_key, source_key
                        ),
                        diagnostics: None,
                    });
                }
                Ok(())
            }
        }
    }

    /// Converts the execution output to a FoundValue.
    fn output_to_found_value(
        output_map: &HashMap<Box<str>, Box<str>, Hasher>,
    ) -> FoundValue {
        // Serialize the final output map as a formatted string
        let mut entries: Vec<String> = output_map
            .iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect();
        entries.sort();

        FoundValue::new(&entries.join("\n"))
    }
}

impl Default for DefaultExecuter {
    fn default() -> Self {
        Self::new()
    }
}

impl Executer for DefaultExecuter {
    fn execute(&self, manifest: &mut ExecutionDescriptor) -> Result<(), AamlError> {
        // Initialize or retrieve the output map from context
        let mut output_map = manifest.context_mut().map.clone();
        
        let context = manifest.context.clone();

        // Execute all execution tasks in order
        for task in &manifest.execution_tasks {
            Self::execute_task(task, &mut output_map, &context)?;
        }

        // Update the manifest context with final output
        manifest.context.map = output_map.clone();

        Ok(())
    }
}
