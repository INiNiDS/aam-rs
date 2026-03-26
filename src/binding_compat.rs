use crate::aam::AAM;
use crate::error::AamlError;
use crate::found_value::FoundValue;
use crate::pipeline::execution_descriptor::ExecutionContext;
use crate::pipeline::formatter::{DefaultFormatter, Formatter, FormattingOptions};
use crate::pipeline::lexer::{DefaultLexer, Lexer};
use crate::pipeline::parser::{DefaultParser, Parser};
use crate::pipeline::Pipeline;

pub fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    })
}

pub fn empty_aam() -> AAM {
    AAM::parse("").unwrap_or_else(|_| {
        AAM::from_pipeline(Pipeline::new(), "")
            .unwrap_or_else(|_| panic!("failed to initialize empty AAM"))
    })
}

pub fn parse_single(content: &str) -> Result<AAM, AamlError> {
    AAM::parse(content).map_err(first_error)
}

pub fn load_single(path: &str) -> Result<AAM, AamlError> {
    AAM::load(path).map_err(first_error)
}

pub fn merge_sources(base: &str, fragment: &str) -> String {
    if base.is_empty() {
        return fragment.to_string();
    }
    if fragment.is_empty() {
        return base.to_string();
    }
    format!("{base}\n{fragment}")
}

pub fn find_key(aam: &AAM, value: &str) -> Option<FoundValue> {
    aam.reverse_search(value).first().map(|k| FoundValue::new(k))
}

pub fn find_obj(aam: &AAM, key: &str) -> Option<FoundValue> {
    aam.get(key)
        .map(FoundValue::new)
        .or_else(|| find_key(aam, key))
}

pub fn find_deep(aam: &AAM, key: &str) -> Option<FoundValue> {
    let mut current = key;
    let mut last: Option<FoundValue> = None;
    let mut visited = std::collections::HashSet::new();

    while let Some(next) = aam.get(current) {
        if !visited.insert(current.to_string()) {
            break;
        }
        if visited.contains(next) {
            if last.is_none() {
                last = Some(FoundValue::new(next));
            }
            break;
        }
        last = Some(FoundValue::new(next));
        current = next;
    }

    last
}

pub fn validate_value(aam: &AAM, type_name: &str, value: &str) -> Result<(), AamlError> {
    let mut context = ExecutionContext::new("inline");
    context.map = aam.map().clone();
    context.schemas = aam.schemas().clone();
    context.types = aam.types().clone();
    crate::pipeline::utils::validate_type_value(value, type_name, &context)
}

pub fn format_content(content: &str) -> Result<String, AamlError> {
    let lexer = DefaultLexer::new();
    let parser = DefaultParser::new();
    let formatter = DefaultFormatter::new();
    let tokens = lexer.tokenize(content)?;
    let ast = parser.parse(&tokens)?;
    formatter.format_document(&ast, &FormattingOptions::default())
}

