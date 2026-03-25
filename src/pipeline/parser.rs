//! Parser stage: builds an Abstract Syntax Tree from tokens.
//!
//! The Parser consumes tokens from the Lexer and produces an AST,
//! preserving line number information for error diagnostics.

use crate::error::{AamlError, ErrorDiagnostics};
use crate::pipeline::lexer::Token;
use crate::pipeline::tasks::{ParseTask, ExecutionTask};
use crate::pipeline::lexer::TokenKind;
use std::sync::Arc;

/// Represents a value in the AST, supporting nested structures.
#[derive(Debug, Clone)]
#[derive(PartialEq, Eq)]
pub enum ValueNode<'a> {
    Literal(std::borrow::Cow<'a, str>),
    Object(std::sync::Arc<[(std::borrow::Cow<'a, str>, ValueNode<'a>)]>),
    List(std::sync::Arc<[ValueNode<'a>]>),
}

impl<'a> ValueNode<'a> {
    /// Converts the value node back to a string representation
    pub fn to_string(&self) -> String {
        match self {
            ValueNode::Literal(s) => s.to_string(),
            ValueNode::Object(pairs) => {
                let formatted_pairs: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{} = {}", k, v.to_string()))
                    .collect();
                format!("{{ {} }}", formatted_pairs.join(", "))
            }
            ValueNode::List(items) => {
                let formatted_items: Vec<String> = items
                    .iter()
                    .map(|v| v.to_string())
                    .collect();
                format!("[{}]", formatted_items.join(", "))
            }
        }
    }
}

/// A node in the Abstract Syntax Tree.
#[derive(Debug, Clone)]
pub enum AstNode<'a> {
    /// Key-value assignment: `key = value`
    Assignment {
        key: std::borrow::Cow<'a, str>,
        value: ValueNode<'a>,
        line: usize,
    },
    /// Directive: `@directive_name arguments`
    Directive {
        name: std::borrow::Cow<'a, str>,
        args: std::borrow::Cow<'a, str>,
        body: Option<ValueNode<'a>>,
        line: usize,
    },
}

impl<'a> AstNode<'a> {
    /// Returns the line number where this node appears
    pub fn line(&self) -> usize {
        match self {
            AstNode::Assignment { line, .. } => *line,
            AstNode::Directive { line, .. } => *line,
        }
    }
}

/// Trait for parsing tokens into an AST.
pub trait Parser: Send + Sync {
    /// Parses a token stream into an AST.
    ///
    /// # Errors
    /// Returns `AamlError::ParseError` if the token stream is malformed.
    fn parse<'a>(&self, tokens: &[Token<'a>]) -> Result<Vec<AstNode<'a>>, AamlError>;

    /// Generates parse tasks from an AST
    fn generate_parse_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ParseTask<'a>>;

    /// Generates execution tasks from an AST
    fn generate_execution_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ExecutionTask<'a>>;
}

/// Default implementation of the Parser stage.
pub struct DefaultParser;

impl DefaultParser {
    pub fn new() -> Self {
        Self
    }

    /// Filters out comment and newline tokens
    fn filter_tokens<'a, 'b>(tokens: &'b [Token<'a>]) -> Vec<&'b Token<'a>> {
        use crate::pipeline::lexer::TokenKind;
        tokens
            .iter()
            .filter(|t| {
                t.kind != TokenKind::Comment
            })
            .collect()
    }

    /// Parses assignment tokens: `identifier = value`
    fn parse_assignment<'a>(tokens: &[&Token<'a>], start: usize) -> Result<(std::borrow::Cow<'a, str>, ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        if tokens.len() < start + 3 {
            return Err(AamlError::ParseError {
                line: tokens.get(start).map(|t| t.line).unwrap_or(1),
                content: "incomplete assignment".to_string(),
                details: "Expected: key = value".to_string(),
                diagnostics: Some(ErrorDiagnostics::new(
                    "Incomplete assignment",
                    "Assignment must have at least 3 tokens: key, =, value".to_string(),
                    "Check format: key = value".to_string(),
                )),
            });
        }

        let key = match &tokens[start].kind {
            TokenKind::Identifier => tokens[start].text.clone(),
            _ => {
                return Err(AamlError::ParseError {
                    line: tokens[start].line,
                    content: format!("Expected identifier, got {:?}", tokens[start].kind),
                    details: "First token of assignment must be an identifier".to_string(),
                    diagnostics: None,
                });
            }
        };

        if tokens[start + 1].kind != TokenKind::Assign {
            return Err(AamlError::ParseError {
                line: tokens[start + 1].line,
                content: format!("Expected '=', got '{}'", tokens[start + 1].text),
                details: "Assignment operator '=' expected after key".to_string(),
                diagnostics: None,
            });
        }

        let (value, consumed) = Self::parse_value(tokens, start + 2)?;
        Ok((key, value, consumed))
    }

    /// Parses a value (which may be literal, inline object, or inline list)
    fn parse_value<'a, 'b>(tokens: &'b [&'b Token<'a>], start: usize) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        if start >= tokens.len() {
            return Err(AamlError::ParseError {
                line: tokens.len(),
                content: "unexpected end of input".to_string(),
                details: "Expected a value after '='".to_string(),
                diagnostics: None,
            });
        }

        match &tokens[start].kind {
            TokenKind::LeftBrace => {
                // Inline object
                let (obj, consumed) = Self::parse_inline_object(tokens, start)?;
                Ok((obj, consumed))
            }
            TokenKind::LeftBracket => {
                // Inline list
                let (list, consumed) = Self::parse_inline_list(tokens, start)?;
                Ok((list, consumed))
            }
            _ => {
                // Literal value
                let value = tokens[start].text.clone();
                Ok((ValueNode::Literal(value), start + 1))
            }
        }
    }

    /// Parses an inline object: `{ key = val, key = val, ... }`
    // High Complexity
    fn parse_inline_object<'a, 'b>(tokens: &'b [&'b Token<'a>], start: usize) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        let mut pairs = Vec::new();
        let mut pos = start + 1;

        while pos < tokens.len() {
            match tokens[pos].kind {
                TokenKind::RightBrace => return Ok((ValueNode::Object(pairs.into()), pos + 1)),
                TokenKind::Identifier => {
                    let key: std::borrow::Cow<'a, str> = tokens[pos].text.clone();
                    if pos + 2 < tokens.len() && tokens[pos + 1].kind == TokenKind::Assign {
                        let (value, next_pos) = Self::parse_value(tokens, pos + 2)?;
                        pairs.push((key, value));
                        pos = next_pos;

                        // Optional comma separation
                        if pos < tokens.len() && tokens[pos].kind == TokenKind::Comma {
                            pos += 1;
                        }
                    } else {
                        // Syntax error, didn't find =
                        return Err(AamlError::ParseError {
                            line: tokens[pos].line,
                            content: "invalid inline object format".to_string(),
                            details: "Expected '=' after key".to_string(),
                            diagnostics: None,
                        });
                    }
                }
                TokenKind::Comma => {
                    pos += 1; // skip stray commas
                }
                _ => {
                    return Err(AamlError::ParseError {
                        line: tokens[pos].line,
                        content: "invalid inline object format".to_string(),
                        details: "Expected identifier or closing brace".to_string(),
                        diagnostics: None,
                    });
                }
            }
        }

        Err(AamlError::ParseError {
            line: tokens[start].line,
            content: "unclosed brace".to_string(),
            details: "Expected '}' to close inline object".to_string(),
            diagnostics: None,
        })
    }

    /// Parses an inline list: `[item, item, ...]`
    fn parse_inline_list<'a, 'b>(tokens: &'b [&'b Token<'a>], start: usize) -> Result<(ValueNode<'a>, usize), AamlError> {
        use crate::pipeline::lexer::TokenKind;

        let mut items = Vec::new();
        let mut pos = start + 1;

        while pos < tokens.len() {
            if tokens[pos].kind == TokenKind::RightBracket {
                return Ok((ValueNode::List(items.into()), pos + 1));
            }

            if tokens[pos].kind == TokenKind::Comma {
                pos += 1;
                continue;
            }

            let (value, next_pos) = Self::parse_value(tokens, pos)?;
            items.push(value);
            pos = next_pos;
        }

        Err(AamlError::ParseError {
            line: tokens[start].line,
            content: "unclosed bracket".to_string(),
            details: "Expected ']' to close inline list".to_string(),
            diagnostics: None,
        })
    }
}

impl Default for DefaultParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for DefaultParser {
    // VERY HIGH COMPLEXITY
    fn parse<'a>(&self, tokens: &[Token<'a>]) -> Result<Vec<AstNode<'a>>, AamlError> {
        use crate::pipeline::lexer::TokenKind;

        let mut ast: Vec<AstNode<'a>> = Vec::new();
        let tokens_filtered = Self::filter_tokens(tokens);
        let mut pos = 0;

        while pos < tokens_filtered.len() {
            let token = tokens_filtered[pos];

            match &token.kind {
                TokenKind::At => {
                    // Directive: @name args...
                    if pos + 1 >= tokens_filtered.len() {
                        return Err(AamlError::ParseError {
                            line: token.line,
                            content: "@".to_string(),
                            details: "Directive name expected after '@'".to_string(),
                            diagnostics: Some(ErrorDiagnostics::new(
                                "Missing directive name",
                                "Directive requires a name after '@'".to_string(),
                                "Use format: @directive_name arguments".to_string(),
                            )),
                        });
                    }

                    let dir_name: std::borrow::Cow<'a, str> = tokens_filtered[pos + 1].text.clone();
                    let line = token.line;

                    // Collect remaining tokens on this line as args
                    let mut args = String::new();
                    let mut arg_pos = pos + 2;
                    let mut brace_count = 0;
                    let mut bracket_count = 0;

                    while arg_pos < tokens_filtered.len() {
                        let tk = tokens_filtered[arg_pos];

                        if tk.kind == TokenKind::LeftBrace {
                            brace_count += 1;
                        } else if tk.kind == TokenKind::RightBrace {
                            brace_count -= 1;
                        } else if tk.kind == TokenKind::LeftBracket {
                            bracket_count += 1;
                        } else if tk.kind == TokenKind::RightBracket {
                            bracket_count -= 1;
                        }

                        // Stop condition: token is At or Newline, AND we are not inside braces/brackets
                        if brace_count == 0 && bracket_count == 0 {
                            if tk.kind == TokenKind::At || tk.kind == TokenKind::Newline {
                                // Important: We should break but if it's newline, we don't include it.
                                // The At token will be processed next iteration.
                                break;
                            }
                        }

                        if !args.is_empty() {
                            args.push_str(" ");
                        }
                        args.push_str(&tk.text);
                        arg_pos += 1;
                    }

                    ast.push(AstNode::Directive {
                        name: dir_name,
                        args: args.trim().to_string().into(),
                        body: None,
                        line,
                    });

                    pos = arg_pos;
                }
                TokenKind::Identifier => {
                    // Assignment: identifier = value
                    let (key, value, new_pos) = Self::parse_assignment(&tokens_filtered, pos)?;
                    ast.push(AstNode::Assignment {
                        key,
                        value,
                        line: token.line,
                    });
                    pos = new_pos;
                }
                _ => {
                    pos += 1;
                }
            }
        }

        Ok(ast)
    }

    fn generate_parse_tasks<'a, 'b>(&self, ast: &'b [AstNode<'a>]) -> Vec<ParseTask<'a>> {
        let mut tasks = Vec::new();
        /*
        // A complete implementation would track scope transitions based on braces
        // let mut scope_stack = vec!["root".to_string()];
        // When encountering an opening brace `{` in assignment:
        // scope_stack.push(new_scope_name);
        // When encountering `}`:
        // scope_stack.pop();
        // let current_scope = scope_stack.last().unwrap().clone();
        */
        let current_scope = std::borrow::Cow::Borrowed("root");

        for node in ast {
            match node {
                AstNode::Assignment { key, value, line } => {
                    tasks.push(ParseTask::ProcessVariable {
                        variable_name: key.clone(),
                        value: value.to_string().into(),
                        scope: current_scope.clone(),
                        line: *line,
                    });
                }
                AstNode::Directive { name, args, line, .. } => {
                    if &**name == "type" {
                        // Assuming args contains the full type definition
                        tasks.push(ParseTask::RegisterType {
                            type_name: args.split_whitespace().next().unwrap_or("").to_string().into(),
                            type_spec: args.clone(),
                            line: *line,
                        });
                    } else if &**name == "schema" {
                        let name_part = args.split_whitespace().next().unwrap_or("").to_string();
                        let body = args.split_once('{').and_then(|(_, b)| b.rsplit_once('}')).map(|(b, _)| b).unwrap_or("");
                        let parsed_fields = body.replace(',', " ")
                            .split_whitespace()
                            .filter_map(|t| t.split_once(':'))
                            .map(|(k, v)| format!("{}:{}", k, v))
                            .collect::<Vec<_>>()
                            .join(",");

                        tasks.push(ParseTask::RegisterSchema {
                            schema_name: name_part.into(),
                            fields: parsed_fields.into(),
                            line: *line,
                        });
                    } else if &**name == "derive" {
                        tasks.push(ParseTask::ResolveDeriveImport {
                            derive_path: args.clone(),
                            line: *line,
                        });
                    } else {
                        tasks.push(ParseTask::ExecuteDirective {
                            directive_name: name.clone(),
                            arguments: args.clone(),
                            line: *line,
                        });
                    }
                }
            }
        }
        tasks
    }

    fn generate_execution_tasks<'a>(&self, ast: &[AstNode<'a>]) -> Vec<ExecutionTask<'a>> {
        let mut tasks = Vec::new();

        for node in ast {
            match node {
                AstNode::Assignment { key, value, line } => {
                    tasks.push(ExecutionTask::SetValue {
                        key: key.clone(),
                        value: value.to_string().into(),
                        line: *line,
                    });
                }
                // Directives translated to execution tasks...
                AstNode::Directive { name, args, line, .. } => {
                    if &**name == "import" {
                        tasks.push(ExecutionTask::ImportFile {
                            file_path: args.clone(),
                            merge_strategy: std::borrow::Cow::Borrowed("merge"),
                            line: *line,
                        });
                    } else if &**name == "derive" {
                        // Assuming current_key is tracked or can be determined (using scope string temporarily for simple configs)
                        // This would need a more sophisticated tracking to know the "current key" exactly if nested
                        tasks.push(ExecutionTask::ExecuteInheritance {
                            derive_path: args.clone(),
                            child_key: std::borrow::Cow::Borrowed(""), // In a robust parser we track standard object parent scope
                            line: *line,
                        });
                    }
                }
            }
        }
        tasks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::lexer::{DefaultLexer, Lexer};

    #[test]
    fn test_parse_simple_assignment() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("key = value").unwrap();
        let parser = DefaultParser::new();
        let ast = parser.parse(tokens).unwrap();

        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Assignment { key, value, .. } => {
                assert_eq!(&**key, "key");
                if let ValueNode::Literal(s) = value {
                    assert_eq!(&**s, "value");
                } else {
                    panic!("Expected ValueNode::Literal");
                }
            }
            _ => panic!("Expected assignment"),
        }
    }

    #[test]
    fn test_parse_directive() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("@import base.aam").unwrap();
        let parser = DefaultParser::new();
        let ast = parser.parse(tokens).unwrap();

        assert_eq!(ast.len(), 1);
        match &ast[0] {
            AstNode::Directive { name, args: _, .. } => {
                assert_eq!(&**name, "import");
            }
            _ => panic!("Expected directive"),
        }
    }

    #[test]
    fn test_parse_multiple_assignments() {
        let lexer = DefaultLexer::new();
        let tokens = lexer.tokenize("a = b\nc = d").unwrap();
        let parser = DefaultParser::new();
        let ast = parser.parse(tokens).unwrap();

        assert_eq!(ast.len(), 2);
    }
}