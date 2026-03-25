import os
import re

def r(f, p, r_str):
    with open(f, 'r') as file:
        x = file.read()
    with open(f, 'w') as file:
        file.write(re.sub(p, r_str, x))

r('src/pipeline/lexer.rs', r'pub struct Token \{', 'pub struct Token<\'a> {')
r('src/pipeline/lexer.rs', r'pub text: String', 'pub text: std::borrow::Cow<\'a, str>')
r('src/pipeline/lexer.rs', r'impl Token \{', 'impl<\'a> Token<\'a> {')
r('src/pipeline/lexer.rs', r'fn new\(kind: TokenKind, line: usize, column: usize, text: String\)', 'fn new(kind: TokenKind, line: usize, column: usize, text: impl Into<std::borrow::Cow<\'a, str>>)')
r('src/pipeline/lexer.rs', r'\{ kind, line, column, text \}', '{ kind, line, column, text: text.into() }')
r('src/pipeline/lexer.rs', r'fn tokenize\(&self, content: &str\) -> Result<Vec<Token>, AamlError>', 'fn tokenize<\'a>(&self, content: &\'a str) -> Result<Vec<Token<\'a>>, AamlError>')

print("Running parsing substitutions..")
r('src/pipeline/parser.rs', r'pub enum ValueNode \{', 'pub enum ValueNode<\'a> {')
r('src/pipeline/parser.rs', r'Literal\(Arc<str>\)', 'Literal(std::borrow::Cow<\'a, str>)')
r('src/pipeline/parser.rs', r'Object\(Arc<\[\(Arc<str>, ValueNode\)\]>\)', 'Object(std::sync::Arc<[(std::borrow::Cow<\'a, str>, ValueNode<\'a>)]>)')
r('src/pipeline/parser.rs', r'List\(Arc<\[ValueNode\]>\)', 'List(std::sync::Arc<[ValueNode<\'a>]>)')
r('src/pipeline/parser.rs', r'impl ValueNode \{', 'impl<\'a> ValueNode<\'a> {')
r('src/pipeline/parser.rs', r'pub enum AstNode \{', 'pub enum AstNode<\'a> {')
r('src/pipeline/parser.rs', r'key: Arc<str>', 'key: std::borrow::Cow<\'a, str>')
r('src/pipeline/parser.rs', r'value: ValueNode', 'value: ValueNode<\'a>')
r('src/pipeline/parser.rs', r'name: Arc<str>', 'name: std::borrow::Cow<\'a, str>')
r('src/pipeline/parser.rs', r'args: Arc<str>', 'args: std::borrow::Cow<\'a, str>')
r('src/pipeline/parser.rs', r'body: Option<ValueNode>', 'body: Option<ValueNode<\'a>>')
r('src/pipeline/parser.rs', r'impl AstNode \{', 'impl<\'a> AstNode<\'a> {')
r('src/pipeline/parser.rs', r'fn parse\(&self, tokens: Vec<Token>\) -> Result<Vec<AstNode>, AamlError>', 'fn parse<\'a>(&self, tokens: Vec<Token<\'a>>) -> Result<Vec<AstNode<\'a>>, AamlError>')

print("Done")
