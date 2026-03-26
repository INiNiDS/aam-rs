//! The new five-stage architecture pipeline for AAML parsing.
//!
//! Pipeline stages:
//! 1. **Lexer** - tokenizes raw text into `Token` stream
//! 2. **Parser** - builds AST from tokens, manages scope
//! 3. **Validator** - applies schema and type checks to AST
//! 4. **Executer** - executes directives and populates final map
//! 5. **Output** - final key-value map, schemas, types

pub mod execution_descriptor;
pub mod formatter;
pub mod lexer;
pub mod parser;
pub mod scope_manager;
pub mod utils;

pub mod executer;
pub mod executor_traits;
pub mod tasks;
pub mod validator;

pub use executer::{DefaultExecuter, Executer};
pub use execution_descriptor::{
    CommandInfo, ExecutionContext, ExecutionDescriptor, SchemaInfo, TypeInfo,
};
pub use executor_traits::{
    DefaultParserExecutor, DefaultValidateExecutor, ParserExecutor, ValidateExecutor,
};
pub use formatter::{DefaultFormatter, FormatRange, Formatter, FormattingOptions};
pub use lexer::{DefaultLexer, Lexer, Token};
pub use parser::{AstNode, DefaultParser, Parser};
pub use scope_manager::ScopeManager;
pub use tasks::{ExecutionTask, ParseTask, TaskError, TaskExecutionResult, ValidationTask};
pub use validator::{DefaultValidator, Validator};

use crate::error::AamlError;
use bumpalo::Bump;
use smol_str::SmolStr;
use tinyvec::TinyVec;

#[cfg(all(feature = "hash-std", feature = "hash-fx"))]
compile_error!("Features 'hash-std' and 'hash-fx' are mutually exclusive for pipeline hashing.");

#[cfg(all(feature = "hash-std", feature = "hash-ahash"))]
compile_error!("Features 'hash-std' and 'hash-ahash' are mutually exclusive for pipeline hashing.");

#[cfg(all(feature = "hash-fx", feature = "hash-ahash"))]
compile_error!("Features 'hash-fx' and 'hash-ahash' are mutually exclusive for pipeline hashing.");

#[cfg(all(feature = "hash-std", feature = "hash-rapidhash"))]
compile_error!(
    "Features 'hash-std' and 'hash-rapidhash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-std", feature = "hash-ritehash"))]
compile_error!(
    "Features 'hash-std' and 'hash-ritehash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-std", feature = "hash-ripemd"))]
compile_error!(
    "Features 'hash-std' and 'hash-ripemd' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-fx", feature = "hash-rapidhash"))]
compile_error!(
    "Features 'hash-fx' and 'hash-rapidhash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-fx", feature = "hash-ritehash"))]
compile_error!(
    "Features 'hash-fx' and 'hash-ritehash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-fx", feature = "hash-ripemd"))]
compile_error!("Features 'hash-fx' and 'hash-ripemd' are mutually exclusive for pipeline hashing.");

#[cfg(all(feature = "hash-ahash", feature = "hash-rapidhash"))]
compile_error!(
    "Features 'hash-ahash' and 'hash-rapidhash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-ahash", feature = "hash-ritehash"))]
compile_error!(
    "Features 'hash-ahash' and 'hash-ritehash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-ahash", feature = "hash-ripemd"))]
compile_error!(
    "Features 'hash-ahash' and 'hash-ripemd' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-rapidhash", feature = "hash-ritehash"))]
compile_error!(
    "Features 'hash-rapidhash' and 'hash-ritehash' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-rapidhash", feature = "hash-ripemd"))]
compile_error!(
    "Features 'hash-rapidhash' and 'hash-ripemd' are mutually exclusive for pipeline hashing."
);

#[cfg(all(feature = "hash-ritehash", feature = "hash-ripemd"))]
compile_error!(
    "Features 'hash-ritehash' and 'hash-ripemd' are mutually exclusive for pipeline hashing."
);

#[cfg(feature = "hash-ripemd")]
#[derive(Default, Clone)]
pub struct RipemdBuildHasher;

#[cfg(feature = "hash-ripemd")]
#[derive(Default, Clone)]
pub struct RipemdHasher {
    bytes: Vec<u8>,
}

#[cfg(feature = "hash-ripemd")]
impl std::hash::BuildHasher for RipemdBuildHasher {
    type Hasher = RipemdHasher;

    fn build_hasher(&self) -> Self::Hasher {
        RipemdHasher::default()
    }
}

#[cfg(feature = "hash-ripemd")]
impl std::hash::Hasher for RipemdHasher {
    fn finish(&self) -> u64 {
        use ripemd::Digest;
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(&self.bytes);
        let digest = hasher.finalize();

        let mut out = [0_u8; 8];
        out.copy_from_slice(&digest[..8]);
        u64::from_le_bytes(out)
    }

    fn write(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
}

#[cfg(feature = "hash-std")]
pub type PipelineBuildHasher = std::collections::hash_map::RandomState;

#[cfg(all(not(feature = "hash-std"), feature = "hash-fx"))]
pub type PipelineBuildHasher = rustc_hash::FxBuildHasher;

#[cfg(all(
    not(feature = "hash-std"),
    not(feature = "hash-fx"),
    feature = "hash-ahash"
))]
pub type PipelineBuildHasher = ahash::RandomState;

#[cfg(all(
    not(feature = "hash-std"),
    not(feature = "hash-fx"),
    not(feature = "hash-ahash"),
    feature = "hash-rapidhash"
))]
pub type PipelineBuildHasher = rapidhash::fast::RandomState;

#[cfg(all(
    not(feature = "hash-std"),
    not(feature = "hash-fx"),
    not(feature = "hash-ahash"),
    not(feature = "hash-rapidhash"),
    not(feature = "hash-ritehash"),
    feature = "hash-ripemd"
))]
pub type PipelineBuildHasher = RipemdBuildHasher;

#[cfg(not(any(
    feature = "hash-std",
    feature = "hash-fx",
    feature = "hash-ahash",
    feature = "hash-rapidhash",
    feature = "hash-ritehash",
    feature = "hash-ripemd"
)))]
pub type PipelineBuildHasher = std::collections::hash_map::RandomState;

pub type PipelineHashMap<K, V> = std::collections::HashMap<K, V, PipelineBuildHasher>;

#[inline]
pub(crate) fn new_pipeline_hash_map<K, V>() -> PipelineHashMap<K, V> {
    PipelineHashMap::with_hasher(PipelineBuildHasher::default())
}

type AamlString = SmolStr;
type ErrorAccumulator = TinyVec<[Option<AamlError>; 4]>;

/// Output produced by the full pipeline after all stages complete successfully.
pub struct PipelineOutput {
    pub map: PipelineHashMap<AamlString, AamlString>,
    pub schemas: PipelineHashMap<AamlString, SchemaInfo>,
    pub types: PipelineHashMap<AamlString, TypeInfo>,
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
    pub fn new() -> Self {
        Self {
            map: new_pipeline_hash_map(),
            schemas: new_pipeline_hash_map(),
            types: new_pipeline_hash_map(),
        }
    }
}

impl Default for PipelineOutput {
    fn default() -> Self {
        Self::new()
    }
}

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
    /// Processes AAML content using the 5-stage arena-based pipeline.
    pub fn process(&self, content: &str) -> Result<PipelineOutput, Vec<AamlError>> {
        let arena = Bump::new();
        self.process_with_arena(content, &arena)
    }

    /// Creates a pipeline instance.
    ///
    /// Enable the `parallel` feature to run stateless parse-task pre-validation in parallel.
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

    #[cfg(feature = "parallel")]
    fn execute_parse_tasks_parallel(tasks: &[ParseTask<'_>]) -> Vec<TaskError> {
        use rayon::prelude::*;

        tasks
            .par_iter()
            .filter_map(|task| task.validate_stateless().err())
            .collect()
    }

    fn collect_parse_errors(all_errors: &mut ErrorAccumulator, errors: Vec<TaskError>) {
        for err in errors.into_iter() {
            all_errors.push(Some(AamlError::ParseError {
                line: err.line,
                content: String::new(),
                details: err.message,
                diagnostics: None,
            }));
        }
    }

    fn collect_validation_errors(all_errors: &mut ErrorAccumulator, errors: Vec<TaskError>) {
        for err in errors.into_iter() {
            all_errors.push(Some(AamlError::DirectiveError {
                directive: "validation".to_string(),
                message: err.message,
                diagnostics: None,
            }));
        }
    }

    fn parse_ast<'a>(
        &self,
        tokens: &[Token<'a>],
        all_errors: &mut ErrorAccumulator,
    ) -> Vec<AstNode<'a>> {
        let parse_output = self.parser.parse_with_recovery(tokens);
        for err in parse_output.errors {
            all_errors.push(Some(err));
        }
        parse_output.ast
    }

    fn run_parse_tasks<'a>(
        &self,
        ast: &[AstNode<'a>],
        arena: &'a Bump,
        descriptor: &mut ExecutionDescriptor<'a>,
        all_errors: &mut ErrorAccumulator,
    ) {
        let parse_tasks = self.parser.generate_parse_tasks(ast);
        descriptor.add_parse_tasks(parse_tasks.clone());

        #[cfg(feature = "parallel")]
        let stateless_errors = Self::execute_parse_tasks_parallel(&parse_tasks);

        #[cfg(feature = "parallel")]
        {
            if !stateless_errors.is_empty() {
                Self::collect_parse_errors(all_errors, stateless_errors);
            }
        }

        #[cfg(feature = "parallel")]
        let sequential_tasks: Vec<ParseTask<'a>> = parse_tasks
            .into_iter()
            .filter(|task| task.validate_stateless().is_ok())
            .collect();

        #[cfg(not(feature = "parallel"))]
        let sequential_tasks = parse_tasks;

        let parse_result = self.parser_executor.execute_batch(
            &sequential_tasks,
            arena,
            descriptor.context_mut(),
        );
        if !parse_result.success {
            Self::collect_parse_errors(all_errors, parse_result.errors);
        }
    }

    fn run_validation_tasks<'a>(
        &self,
        ast: &[AstNode<'a>],
        descriptor: &mut ExecutionDescriptor<'a>,
        all_errors: &mut ErrorAccumulator,
    ) {
        let validation_tasks = self.validator.validate(ast).unwrap_or_else(|e| {
            all_errors.push(Some(e));
            Vec::new()
        });
        descriptor.add_validation_tasks(validation_tasks.clone());

        let validation_result = self
            .validate_executor
            .execute_batch(&validation_tasks, descriptor.context());
        if !validation_result.success {
            Self::collect_validation_errors(all_errors, validation_result.errors);
        }
    }

    fn process_with_tasks<'a>(
        &self,
        content: &'a str,
        arena: &'a Bump,
    ) -> Result<PipelineOutput, Vec<AamlError>> {
        let mut all_errors: ErrorAccumulator = TinyVec::new();

        let tokens = match self.lexer.tokenize(content) {
            Ok(t) => t,
            Err(e) => return Err(vec![e]),
        };

        let ast = self.parse_ast(&tokens, &mut all_errors);
        let mut descriptor = ExecutionDescriptor::new(ast.clone(), "inline".to_string());

        self.run_parse_tasks(&ast, arena, &mut descriptor, &mut all_errors);
        self.run_validation_tasks(&ast, &mut descriptor, &mut all_errors);

        if let Some(errors) = finalize_error_accumulator(all_errors) {
            return Err(errors);
        }

        descriptor.add_execution_tasks(self.parser.generate_execution_tasks(&ast));

        if let Err(e) = self.executer.execute(&mut descriptor) {
            return Err(vec![e]);
        }

        Ok(PipelineOutput {
            map: descriptor.context.map,
            schemas: descriptor.context.schemas,
            types: descriptor.context.types,
        })
    }

    pub fn process_with_arena<'a>(
        &self,
        content: &'a str,
        arena: &'a Bump,
    ) -> Result<PipelineOutput, Vec<AamlError>> {
        self.process_with_tasks(content, arena)
    }

    pub fn format(
        &self,
        nodes: &[AstNode],
        options: &FormattingOptions,
    ) -> Result<String, AamlError> {
        self.formatter.format_document(nodes, options)
    }

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

#[inline]
fn finalize_error_accumulator(errors: ErrorAccumulator) -> Option<Vec<AamlError>> {
    if errors.is_empty() {
        return None;
    }
    Some(errors.into_iter().flatten().collect())
}
