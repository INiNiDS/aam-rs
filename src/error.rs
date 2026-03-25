//! Error types for the AAML parser and validation pipeline with beautiful colored output.

use colored::Colorize;
use std::fmt;
use std::io;

/// Error diagnostics with "What", "Why", and "Fix" guidance (inspired by Cargo).
#[derive(Debug, Clone)]
pub struct ErrorDiagnostics {
    /// What went wrong (short title).
    pub what: String,
    /// Why it happened (detailed explanation).
    pub why: String,
    /// How to fix it (suggested resolution).
    pub fix: String,
}

impl ErrorDiagnostics {
    /// Create new diagnostics.
    pub fn new(what: impl Into<String>, why: impl Into<String>, fix: impl Into<String>) -> Self {
        Self {
            what: what.into(),
            why: why.into(),
            fix: fix.into(),
        }
    }

    /// Pretty-print with colors (like Cargo).
    pub fn pretty_print(&self) -> String {
        format!(
            "{}\n{}\n\n{}\n{}\n\n{}\n{}\n",
            "error".red().bold(),
            format!("  {}", self.what).red(),
            "why".cyan().bold(),
            format!("  {}", self.why).cyan(),
            "fix".green().bold(),
            format!("  {}", self.fix).green(),
        )
    }
}

/// All errors that can be produced while parsing or validating an AAML document.
#[derive(Debug)]
pub enum AamlError {
    /// An I/O error occurred while reading a file.
    IoError {
        details: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A line could not be parsed as a valid AAML statement.
    ParseError {
        /// 1-based line number in the source file.
        line: usize,
        /// Raw content of the offending line.
        content: String,
        /// Human-readable explanation of why parsing failed.
        details: String,
        /// Diagnostic guidance.
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A key or type name was not found in the registry or map.
    NotFound {
        key: String,
        context: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A value does not satisfy a basic type constraint (not schema-specific).
    InvalidValue {
        details: String,
        expected: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A value failed validation against a registered or built-in type.
    InvalidType {
        /// Name of the type that rejected the value.
        type_name: String,
        /// Details from the type validator.
        details: String,
        /// What was provided.
        provided: String,
        /// Diagnostic guidance.
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A directive (`@import`, `@derive`, …) encountered an error in its arguments.
    DirectiveError {
        directive: String,
        message: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// A schema constraint was violated during parsing or explicit validation.
    SchemaValidationError {
        /// Name of the schema that declared the field.
        schema: String,
        /// Name of the field that failed validation.
        field: String,
        /// Declared type of the field.
        type_name: String,
        /// Human-readable description of the failure.
        details: String,
        /// Diagnostic guidance.
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Missing required field in schema.
    MissingRequiredField {
        schema: String,
        field: String,
        field_type: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Circular dependency detected.
    CircularDependency {
        path: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Type registration conflict.
    TypeRegistrationConflict {
        type_name: String,
        existing: String,
        new: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Nesting depth exceeded (possible infinite loop).
    NestingDepthExceeded {
        depth: usize,
        context: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Malformed inline object or array literal.
    MalformedLiteral {
        literal_type: String,
        content: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Directive syntax is incorrect.
    DirectiveSyntaxError {
        directive: String,
        provided_syntax: String,
        expected_syntax: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Type conversion failed.
    TypeConversionError {
        from_type: String,
        to_type: String,
        value: String,
        diagnostics: Option<ErrorDiagnostics>,
    },

    /// Lexical analysis error (invalid character or token).
    LexError {
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
        /// The invalid character
        character: String,
        /// Diagnostic guidance
        diagnostics: Option<ErrorDiagnostics>,
    },
}

impl AamlError {
    /// Get the primary error message (short form).
    pub fn short_message(&self) -> String {
        match self {
            AamlError::IoError { details, .. } => format!("IO error: {}", details),
            AamlError::ParseError {
                line,
                content,
                details,
                ..
            } => {
                format!("Parse error at line {}: {} ({})", line, content, details)
            }
            AamlError::NotFound { key, context, .. } => {
                format!("Key '{}' not found ({})", key, context)
            }
            AamlError::InvalidValue {
                details, expected, ..
            } => {
                format!("Invalid value: {} (expected: {})", details, expected)
            }
            AamlError::InvalidType {
                type_name,
                provided,
                details,
                ..
            } => {
                format!(
                    "Invalid type '{}': {} (got: {})",
                    type_name, details, provided
                )
            }
            AamlError::DirectiveError {
                directive, message, ..
            } => {
                format!("Directive '@{}' error: {}", directive, message)
            }
            AamlError::SchemaValidationError {
                schema,
                field,
                type_name,
                details,
                ..
            } => {
                format!(
                    "Schema '{}' field '{}' ({}): {}",
                    schema, field, type_name, details
                )
            }
            AamlError::MissingRequiredField {
                schema,
                field,
                field_type,
                ..
            } => {
                format!(
                    "Missing required field '{}' in schema '{}' (type: {})",
                    field, schema, field_type
                )
            }
            AamlError::CircularDependency { path, .. } => {
                format!("Circular dependency detected: {}", path)
            }
            AamlError::TypeRegistrationConflict {
                type_name,
                existing,
                new,
                ..
            } => {
                format!(
                    "Type '{}' already defined as '{}', cannot redefine as '{}'",
                    type_name, existing, new
                )
            }
            AamlError::NestingDepthExceeded { depth, context, .. } => {
                format!("Nesting depth exceeded ({}): {}", depth, context)
            }
            AamlError::MalformedLiteral {
                literal_type,
                content,
                ..
            } => {
                format!("Malformed {} literal: {}", literal_type, content)
            }
            AamlError::DirectiveSyntaxError {
                directive,
                provided_syntax,
                expected_syntax,
                ..
            } => {
                format!(
                    "Directive '@{}' syntax error: got '{}', expected '{}'",
                    directive, provided_syntax, expected_syntax
                )
            }
            AamlError::TypeConversionError {
                from_type,
                to_type,
                value,
                ..
            } => {
                format!(
                    "Cannot convert '{}' from {} to {}",
                    value, from_type, to_type
                )
            }
            AamlError::LexError {
                line,
                column,
                character,
                ..
            } => {
                format!(
                    "Lexical error at {}:{}: invalid character '{}'",
                    line, column, character
                )
            }
        }
    }

    /// Get the detailed diagnostics if available.
    pub fn diagnostics(&self) -> Option<&ErrorDiagnostics> {
        match self {
            AamlError::IoError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::ParseError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::NotFound { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::InvalidValue { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::InvalidType { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::DirectiveError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::SchemaValidationError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::MissingRequiredField { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::CircularDependency { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::TypeRegistrationConflict { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::NestingDepthExceeded { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::MalformedLiteral { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::DirectiveSyntaxError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::TypeConversionError { diagnostics, .. } => diagnostics.as_ref(),
            AamlError::LexError { diagnostics, .. } => diagnostics.as_ref(),
        }
    }
}

impl fmt::Display for AamlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let short = self.short_message();
        write!(f, "{}", short.red().bold())?;

        if let Some(diag) = self.diagnostics() {
            write!(f, "\n{}", diag.pretty_print())?;
        }

        Ok(())
    }
}

impl std::error::Error for AamlError {}

impl From<io::Error> for AamlError {
    fn from(err: io::Error) -> Self {
        let details = err.to_string();
        let diagnostics = Some(ErrorDiagnostics::new(
            "I/O operation failed",
            format!("Could not read or write file: {}", details),
            "Check file permissions and ensure the path exists",
        ));
        AamlError::IoError {
            details,
            diagnostics,
        }
    }
}
