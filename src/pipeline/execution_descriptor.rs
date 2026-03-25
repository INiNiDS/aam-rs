//! Enhanced ExecutionDescriptor serving as the comprehensive execution manifest.
//!
//! The ExecutionDescriptor replaces direct AAML struct dependency in the Executer.
//! It bundles all necessary context, tasks, and metadata for clean execution.

use crate::pipeline::parser::AstNode;
use crate::pipeline::tasks::{ExecutionTask, ParseTask, ValidationTask, ExecutionStats};
use std::collections::HashMap;

#[cfg(feature = "perf-hash")]
type Hasher = ahash::RandomState;

#[cfg(not(feature = "perf-hash"))]
type Hasher = std::collections::hash_map::RandomState;

/// A comprehensive execution manifest that completely replaces AAML struct usage.
///
/// ExecutionDescriptor aggregates all necessary information for the Executer to
/// materialize the configuration without requiring the legacy AAML struct.
#[derive(Debug, Clone)]
pub struct ExecutionDescriptor {
    /// Original line numbers from source, indexed by AST node
    pub line_numbers: Vec<usize>,

    /// Execution context containing parsed configuration data
    pub context: ExecutionContext,

    /// Original parsed AST nodes for reference and diagnostics
    pub parsed_outputs: Vec<AstNode>,

    /// Lazy tasks for the parsing phase
    pub parse_tasks: Vec<ParseTask>,

    /// Lazy tasks for the validation phase
    pub validation_tasks: Vec<ValidationTask>,

    /// Lazy tasks for the execution phase
    pub execution_tasks: Vec<ExecutionTask>,

    /// Execution statistics
    pub stats: ExecutionStats,
}

/// Encapsulates all runtime context needed during execution.
///
/// This struct holds the accumulated state that would normally be scattered
/// across the AAML struct and various registries.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Source file path or identifier (for error reporting)
    pub source: String,

    /// Key-value map accumulated during parsing
    pub map: HashMap<Box<str>, Box<str>, Hasher>,

    /// Schema definitions accumulated from @schema directives
    pub schemas: HashMap<String, SchemaInfo>,

    /// Type definitions accumulated from @type directives
    pub types: HashMap<String, TypeInfo>,

    /// Registered commands (directives)
    pub commands: HashMap<String, CommandInfo>,

    /// Line number map: key → line number where it was defined
    pub key_line_map: HashMap<Box<str>, usize>,

    /// Scope tracking for nested configurations
    pub scope_stack: Vec<String>,

    /// Circular reference detection set
    pub visited_keys: std::collections::HashSet<Box<str>>,

    /// Import cache to prevent re-importing the same file
    pub imported_files: std::collections::HashSet<String>,
}

/// Information about a registered schema.
#[derive(Debug, Clone)]
pub struct SchemaInfo {
    /// Schema name
    pub name: String,

    /// Field name → (type_name, is_optional)
    pub fields: HashMap<String, (String, bool)>,

    /// Line number where schema was defined
    pub line: usize,
}

/// Information about a registered type.
#[derive(Debug, Clone)]
pub struct TypeInfo {
    /// Type name
    pub name: String,

    /// Type specification (e.g., "i32", "list<string>", "vector2")
    pub spec: String,

    /// Custom validation rules (if any)
    pub validator: Option<String>,

    /// Line number where type was defined
    pub line: usize,
}

/// Information about a registered command (directive).
#[derive(Debug, Clone)]
pub struct CommandInfo {
    /// Command name
    pub name: String,

    /// Expected argument pattern
    pub arg_pattern: String,

    /// Line number where command was registered
    pub line: usize,
}

impl ExecutionContext {
    /// Creates a new empty execution context.
    pub fn new(source: String) -> Self {
        Self {
            source,
            map: HashMap::with_hasher(Hasher::new()),
            schemas: HashMap::new(),
            types: HashMap::new(),
            commands: HashMap::new(),
            key_line_map: HashMap::new(),
            scope_stack: vec!["root".to_string()],
            visited_keys: std::collections::HashSet::new(),
            imported_files: std::collections::HashSet::new(),
        }
    }

    /// Returns the current scope as a string path.
    pub fn current_scope(&self) -> String {
        self.scope_stack.join("::")
    }

    /// Enters a new nested scope.
    pub fn push_scope(&mut self, scope_name: String) {
        self.scope_stack.push(scope_name);
    }

    /// Exits the current scope.
    pub fn pop_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
        }
    }

    /// Sets a key-value pair in the map with line tracking.
    pub fn set_value(&mut self, key: String, value: String, line: usize) {
        let key_box = key.into_boxed_str();
        self.map.insert(key_box.clone(), value.into_boxed_str());
        self.key_line_map.insert(key_box, line);
    }

    /// Gets a value from the map.
    pub fn get_value(&self, key: &str) -> Option<&str> {
        self.map.get(key).map(|v| v.as_ref())
    }

    /// Registers a schema definition.
    pub fn register_schema(&mut self, schema: SchemaInfo) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Registers a type definition.
    pub fn register_type(&mut self, type_def: TypeInfo) {
        self.types.insert(type_def.name.clone(), type_def);
    }

    /// Registers a command (directive).
    pub fn register_command(&mut self, command: CommandInfo) {
        self.commands.insert(command.name.clone(), command);
    }

    /// Checks if a key has been visited (for cycle detection).
    pub fn mark_visited(&mut self, key: &str) {
        self.visited_keys.insert(key.into());
    }

    /// Checks if a key has been visited.
    pub fn is_visited(&self, key: &str) -> bool {
        self.visited_keys.contains(key)
    }

    /// Clears visited set for a new traversal.
    pub fn reset_visited(&mut self) {
        self.visited_keys.clear();
    }

    /// Records an imported file.
    pub fn record_import(&mut self, file_path: String) {
        self.imported_files.insert(file_path);
    }

    /// Checks if a file has already been imported.
    pub fn is_imported(&self, file_path: &str) -> bool {
        self.imported_files.contains(file_path)
    }

    /// Returns the line number where a key was defined.
    pub fn get_line_for_key(&self, key: &str) -> Option<usize> {
        self.key_line_map.get(key).copied()
    }
}

impl ExecutionDescriptor {
    /// Creates a new execution descriptor from parsed AST.
    pub fn new(parsed_outputs: Vec<AstNode>, source: String) -> Self {
        let mut line_numbers = Vec::new();
        for node in &parsed_outputs {
            line_numbers.push(node.line());
        }

        Self {
            line_numbers,
            context: ExecutionContext::new(source),
            parsed_outputs,
            parse_tasks: Vec::new(),
            validation_tasks: Vec::new(),
            execution_tasks: Vec::new(),
            stats: ExecutionStats::default(),
        }
    }

    /// Adds a parse task to the manifest.
    pub fn add_parse_task(&mut self, task: ParseTask) {
        self.parse_tasks.push(task);
    }

    /// Adds multiple parse tasks.
    pub fn add_parse_tasks(&mut self, tasks: Vec<ParseTask>) {
        self.parse_tasks.extend(tasks);
    }

    /// Adds a validation task.
    pub fn add_validation_task(&mut self, task: ValidationTask) {
        self.validation_tasks.push(task);
    }

    /// Adds multiple validation tasks.
    pub fn add_validation_tasks(&mut self, tasks: Vec<ValidationTask>) {
        self.validation_tasks.extend(tasks);
    }

    /// Adds an execution task.
    pub fn add_execution_task(&mut self, task: ExecutionTask) {
        self.execution_tasks.push(task);
    }

    /// Adds multiple execution tasks.
    pub fn add_execution_tasks(&mut self, tasks: Vec<ExecutionTask>) {
        self.execution_tasks.extend(tasks);
    }

    /// Returns the total number of tasks.
    pub fn task_count(&self) -> usize {
        self.parse_tasks.len() + self.validation_tasks.len() + self.execution_tasks.len()
    }

    /// Returns a summary of tasks by type.
    pub fn task_summary(&self) -> String {
        format!(
            "Parse tasks: {}, Validation tasks: {}, Execution tasks: {}",
            self.parse_tasks.len(),
            self.validation_tasks.len(),
            self.execution_tasks.len()
        )
    }

    /// Updates execution statistics.
    pub fn update_stats(&mut self, stats: ExecutionStats) {
        self.stats = stats;
    }

    /// Retrieves the source identifier.
    pub fn source(&self) -> &str {
        &self.context.source
    }

    /// Returns a mutable reference to the execution context.
    pub fn context_mut(&mut self) -> &mut ExecutionContext {
        &mut self.context
    }

    /// Returns an immutable reference to the execution context.
    pub fn context(&self) -> &ExecutionContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_scope_management() {
        let mut ctx = ExecutionContext::new("test.aam".to_string());
        assert_eq!(ctx.current_scope(), "root");

        ctx.push_scope("section".to_string());
        assert_eq!(ctx.current_scope(), "root::section");

        ctx.pop_scope();
        assert_eq!(ctx.current_scope(), "root");
    }

    #[test]
    fn test_execution_context_key_value_operations() {
        let mut ctx = ExecutionContext::new("test.aam".to_string());
        ctx.set_value("key1".to_string(), "value1".to_string(), 1);

        assert_eq!(ctx.get_value("key1"), Some("value1"));
        assert_eq!(ctx.get_line_for_key("key1"), Some(1));
    }

    #[test]
    fn test_visited_keys_tracking() {
        let mut ctx = ExecutionContext::new("test.aam".to_string());
        assert!(!ctx.is_visited("key1"));

        ctx.mark_visited("key1");
        assert!(ctx.is_visited("key1"));

        ctx.reset_visited();
        assert!(!ctx.is_visited("key1"));
    }

    #[test]
    fn test_execution_descriptor_task_management() {
        let mut desc = ExecutionDescriptor::new(vec![], "test.aam".to_string());

        desc.add_validation_task(ValidationTask::VerifySchemaExists {
            schema_name: "MySchema".to_string(),
            line: 1,
        });

        assert_eq!(desc.validation_tasks.len(), 1);
        assert_eq!(desc.task_count(), 1);
    }
}


