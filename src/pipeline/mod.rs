//! The new five-stage architecture pipeline for AAML parsing.
//!
//! Pipeline stages:
//! 1. **Lexer** — tokenizes raw text into `Token` stream
//! 2. **Parser** — builds AST from tokens, manages scope
//! 3. **Validator** — applies schema and type checks to AST
//! 4. **Executer** — executes directives and populates final map
//! 5. **Output** — final key-value map, schemas, types
//!
//! Each stage is independent and can be tested in isolation.

pub mod utils;
pub mod lexer;
pub mod parser;
pub mod scope_manager;
pub mod validator;
pub mod executer;
pub mod tasks;
pub mod execution_descriptor;
pub mod executor_traits;
pub mod formatter;

pub use lexer::{Lexer, Token, DefaultLexer};
pub use parser::{Parser, AstNode, DefaultParser};
pub use scope_manager::ScopeManager;
pub use validator::{Validator, DefaultValidator};
pub use executer::{Executer, DefaultExecuter};
pub use tasks::{ValidationTask, ParseTask, ExecutionTask, TaskExecutionResult, TaskError};
pub use execution_descriptor::{ExecutionDescriptor, ExecutionContext, SchemaInfo, TypeInfo, CommandInfo};
pub use executor_traits::{ValidateExecutor, ParserExecutor, DefaultValidateExecutor, DefaultParserExecutor};
pub use formatter::{Formatter, DefaultFormatter, FormattingOptions, FormatRange};

use crate::error::AamlError;
use crate::commands::schema::SchemaDef;
use crate::types::Type;
use std::collections::HashMap;

#[cfg(feature = "perf-hash")]
type Hasher = ahash::RandomState;

#[cfg(not(feature = "perf-hash"))]
type Hasher = std::collections::hash_map::RandomState;

type AamlString = Box<str>;

/// Output produced by the full pipeline after all stages complete successfully.
pub struct PipelineOutput {
    /// Final key-value map with all directives executed
    pub map: HashMap<AamlString, AamlString, Hasher>,
    /// Registered schema definitions
    pub schemas: HashMap<String, SchemaInfo, Hasher>,
    /// Registered custom types
    pub types: HashMap<String, TypeInfo, Hasher>,
}

impl std::fmt::Debug for PipelineOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineOutput")
            .field("map", &self.map)
            .field("schemas", &self.schemas)
            .field("types_count", &self.types.len())
            .finish()
    }
}

impl PipelineOutput {
    /// Creates a new empty pipeline output
    pub fn new() -> Self {
        Self {
            map: HashMap::with_hasher(Hasher::new()),
            schemas: HashMap::new(),
            types: HashMap::new(),
        }
    }
}

impl Default for PipelineOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// The complete pipeline orchestrator that coordinates all five stages with task-based architecture.
///
/// This pipeline implements strict Separation of Concerns:
/// 1. Lexer tokenizes input
/// 2. Parser builds AST
/// 3. Validator generates declarative ValidationTasks
/// 4. ValidateExecutor executes tasks and aggregates errors
/// 5. ParserExecutor populates ExecutionContext
/// 6. Executer runs ExecutionTasks to produce final output
///
/// The pipeline is LSP-ready with support for:
/// - Error aggregation (all errors returned, not just first)
/// - Independent formatting (via Formatter)
/// - No AAML struct dependency in execution stage
pub struct Pipeline {
    lexer: Box<dyn Lexer>,
    parser: Box<dyn Parser>,
    validator: Box<dyn Validator>,
    validate_executor: Box<dyn ValidateExecutor>,
    parser_executor: Box<dyn ParserExecutor>,
    executer: Box<dyn Executer>,
    formatter: Box<dyn Formatter>,
}

impl Pipeline {
    /// Creates a new pipeline with default implementations for all stages
    pub fn new() -> Self {
        Self {
            lexer: Box::new(DefaultLexer::new()),
            parser: Box::new(DefaultParser::new()),
            validator: Box::new(DefaultValidator::new()),
            validate_executor: Box::new(DefaultValidateExecutor::new()),
            parser_executor: Box::new(DefaultParserExecutor::new()),
            executer: Box::new(DefaultExecuter::new()),
            formatter: Box::new(DefaultFormatter::new()),
        }
    }

    /// Process AAML content through all pipeline stages with task-based architecture.
    ///
    /// # Flow
    /// 1. Lexer: Raw text → Tokens
    /// 2. Parser: Tokens → AST
    /// 3. Validator: AST → ValidationTasks (+ syntax check)
    /// 4. ValidateExecutor: Tasks → Aggregated validation results
    /// 5. ParserExecutor: Parse tasks → ExecutionContext
    /// 6. Executer: ExecutionDescriptor → Final FoundValue
    ///
    /// # Returns
    /// - `Ok(PipelineOutput)` on success with final map, schemas, and types
    /// - `Err(AamlError)` on failure (from any stage)
    ///
    /// # Note
    /// The Executer never instantiates an AAML struct. All execution is
    /// task-based and stateless beyond the ExecutionDescriptor.
    pub fn process(&self, content: &str) -> Result<PipelineOutput, Vec<AamlError>> {
        let mut all_errors = Vec::new();

        // Stage 1: Lexer
        let tokens = match self.lexer.tokenize(content) {
            Ok(t) => t,
            Err(e) => {
                all_errors.push(e);
                return Err(all_errors); // Lexer failure is fatal as we can't parse
            }
        };

        // Stage 2: Parser
        let ast = match self.parser.parse(tokens) {
            Ok(a) => a,
            Err(e) => {
                all_errors.push(e);
                // Even if AST fails partially, we might want to continue in a real LSP,
                // but currently parse returns a single error on failure.
                // We'll proceed with an empty AST to gather more errors if possible.
                Vec::new()
            }
        };

        // Create ExecutionDescriptor to hold all information
        let mut descriptor = ExecutionDescriptor::new(ast.clone(), "inline".to_string());

        // Stage 3: Parser to generate ParseTasks
        let parse_tasks = self.parser.generate_parse_tasks(&ast);
        descriptor.add_parse_tasks(parse_tasks.clone());

        // Execute ParseTasks to populate descriptor.context
        let parse_result = self.parser_executor.execute_batch(&parse_tasks, descriptor.context_mut());
        if !parse_result.success {
            for err in parse_result.errors {
                all_errors.push(AamlError::ParseError {
                    line: err.line,
                    content: "".to_string(),
                    details: err.message,
                    diagnostics: None,
                });
            }
        }

        // Stage 4: Validator (generates tasks based on AST)
        let validation_tasks = match self.validator.validate(&ast) {
            Ok(t) => t,
            Err(e) => {
                all_errors.push(e);
                Vec::new()
            }
        };
        descriptor.add_validation_tasks(validation_tasks.clone());

        // Stage 5: ValidateExecutor (aggregates errors, reading from context if needed)
        let validation_result = self.validate_executor.execute_batch(&validation_tasks, descriptor.context());
        if !validation_result.success {
            for err in validation_result.errors {
                all_errors.push(AamlError::DirectiveError {
                    directive: "validation".to_string(),
                    message: err.message,
                    diagnostics: None,
                });
            }
        }

        if !all_errors.is_empty() {
            return Err(all_errors);
        }

        // Stage 6: Generate internal execution tasks from AST
        let execution_tasks = self.parser.generate_execution_tasks(&ast);
        descriptor.add_execution_tasks(execution_tasks);

        // Stage 7: Executer (task-based, no AAML struct)
        if let Err(e) = self.executer.execute(&mut descriptor) {
            all_errors.push(e);
            return Err(all_errors);
        }

        // Return final output
        Ok(PipelineOutput {
            map: descriptor.context.map,
            schemas: descriptor.context.schemas,
            types: descriptor.context.types,
        })
    }

    /// Format a document using the Formatter stage (no execution required).
    ///
    /// This is useful for LSP "Format Document" commands that don't need
    /// to execute the full pipeline.
    pub fn format(
        &self,
        nodes: &[AstNode],
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        self.formatter.format_document(nodes, options)
    }

    /// Format a specific range in a document.
    ///
    /// Used for LSP "Format Range" commands.
    pub fn format_range(
        &self,
        nodes: &[AstNode],
        range: FormatRange,
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        self.formatter.format_range(nodes, range, options)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
