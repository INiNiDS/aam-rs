use aam_rs::aam::{AAM, AamLspAssist};
use aam_rs::builder::{AAMBuilder as CoreAamBuilder, InlineObject, SchemaField};
use aam_rs::error::AamlError;
use aam_rs::pipeline::formatter::{FormatRange, FormattingOptions as FormatterRules};
#[cfg(feature = "reconstructer")]
use aam_rs::reconstructer;
use aam_rs::translator::TOMLTranslator;
use napi::{Error, Result, Status};
use napi_derive::napi;
use std::collections::HashMap;

fn to_napi_error(err: AamlError) -> Error {
    Error::new(Status::GenericFailure, err.to_string())
}

fn first_napi_error(errors: Vec<AamlError>) -> Error {
    let err = errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    });
    to_napi_error(err)
}

fn closed_error() -> Error {
    Error::new(Status::GenericFailure, "AAM instance is closed".to_owned())
}

#[napi(object)]
pub struct JsLspResult {
    pub diagnostics: Vec<String>,
    pub formatted: Option<String>,
}

#[napi(js_name = "AAMBuilder")]
pub struct JsAamBuilder {
    inner: CoreAamBuilder,
}

impl Default for JsAamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl JsAamBuilder {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: CoreAamBuilder::new(),
        }
    }

    #[napi(js_name = "addLine")]
    pub fn add_line(&mut self, key: String, value: String) {
        self.inner.add_line(&key, &value);
    }

    #[napi]
    pub fn comment(&mut self, text: String) {
        self.inner.comment(&text);
    }

    #[napi]
    pub fn schema(&mut self, name: String, fields: Vec<String>) {
        self.inner.schema(&name, parse_schema_fields(fields));
    }

    #[napi(js_name = "schemaMultiline")]
    pub fn schema_multiline(&mut self, name: String, fields: Vec<String>) {
        self.inner
            .schema_multiline(&name, parse_schema_fields(fields));
    }

    #[napi]
    pub fn derive(&mut self, path: String, schemas: Vec<String>) {
        self.inner.derive(&path, schemas);
    }

    #[napi]
    pub fn import(&mut self, path: String) {
        self.inner.import(&path);
    }

    #[napi(js_name = "typeAlias")]
    pub fn type_alias(&mut self, alias: String, type_name: String) {
        self.inner.type_alias(&alias, &type_name);
    }

    #[napi(js_name = "asString")]
    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}

fn parse_schema_fields(fields: Vec<String>) -> Vec<SchemaField> {
    fields
        .into_iter()
        .filter_map(|field| {
            let mut parts = field.splitn(2, ':');
            let name = parts.next()?.trim();
            let type_name = parts.next()?.trim();
            if let Some(optional_name) = name.strip_suffix('*') {
                Some(SchemaField::optional(optional_name.trim(), type_name))
            } else {
                Some(SchemaField::required(name, type_name))
            }
        })
        .collect()
}

impl From<AamLspAssist> for JsLspResult {
    fn from(value: AamLspAssist) -> Self {
        Self {
            diagnostics: value
                .diagnostics
                .into_iter()
                .map(|err| err.to_string())
                .collect(),
            formatted: value.formatted,
        }
    }
}

#[napi(js_name = "AAM")]
pub struct JsAam {
    inner: Option<AAM>,
}

impl JsAam {
    fn inner_ref(&self) -> Result<&AAM> {
        self.inner.as_ref().ok_or_else(closed_error)
    }

    fn inner_mut(&mut self) -> Result<&mut AAM> {
        self.inner.as_mut().ok_or_else(closed_error)
    }
}

impl Default for JsAam {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl JsAam {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(AAM::new()),
        }
    }

    #[napi(js_name = "format")]
    pub fn format(&self, content: String) -> Result<String> {
        let rules = FormatterRules::default();
        self.inner_ref()?
            .format(&content, &rules)
            .map_err(to_napi_error)
    }

    #[napi(js_name = "formatRange")]
    pub fn format_range(&self, content: String, start_line: u32, end_line: u32) -> Result<String> {
        let rules = FormatterRules::default();
        let range = FormatRange {
            start_line: start_line as usize,
            end_line: end_line as usize,
        };

        self.inner_ref()?
            .format_range(&content, range, &rules)
            .map_err(to_napi_error)
    }

    #[napi]
    pub fn get(&self, key: String) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.get(&key).map(ToString::to_string))
    }

    #[napi]
    pub fn keys(&self) -> Vec<String> {
        match self.inner_ref() {
            Ok(inner) => inner.keys().iter().map(|k| k.to_string()).collect(),
            Err(_) => Vec::new(),
        }
    }

    #[napi(js_name = "toMap")]
    pub fn to_map(&self) -> HashMap<String, String> {
        self.inner_ref().map_or_else(
            |_| HashMap::new(),
            |inner| inner.to_map().into_iter().collect(),
        )
    }

    #[napi]
    pub fn find(&self, query: String) -> HashMap<String, String> {
        self.inner_ref().map_or_else(
            |_| HashMap::new(),
            |inner| {
                inner
                    .find(&query)
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            },
        )
    }

    #[napi(js_name = "deepSearch")]
    pub fn deep_search(&self, pattern: String) -> HashMap<String, String> {
        self.inner_ref().map_or_else(
            |_| HashMap::new(),
            |inner| {
                inner
                    .deep_search(&pattern)
                    .into_iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect()
            },
        )
    }

    #[napi(js_name = "reverseSearch")]
    pub fn reverse_search(&self, target_value: String) -> Vec<String> {
        self.inner_ref().map_or_else(
            |_| Vec::new(),
            |inner| {
                inner
                    .reverse_search(&target_value)
                    .into_iter()
                    .map(ToString::to_string)
                    .collect()
            },
        )
    }

    #[napi(js_name = "schemaNames")]
    pub fn schema_names(&self) -> Vec<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.schemas())
            .map(|schemas| schemas.keys().map(|k| k.to_string()).collect())
            .unwrap_or_default()
    }

    #[napi(js_name = "typeNames")]
    pub fn type_names(&self) -> Vec<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.types())
            .map(|types| types.keys().map(|k| k.to_string()).collect())
            .unwrap_or_default()
    }

    #[napi]
    pub fn close(&mut self) {
        self.inner = None;
    }

    #[napi(js_name = "isClosed")]
    pub fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    /// Reload the configuration from its original on-disk source file (the path
    /// captured at `load`/`parse`-of-a-path time). Rejects if the instance was
    /// not loaded from a file path.
    #[napi]
    pub fn update(&mut self) -> Result<()> {
        self.inner_mut()?.update().map_err(first_napi_error)
    }

    /// Replace the entire backing configuration by re-parsing `content`.
    /// Clears any remembered on-disk source path.
    #[napi(js_name = "updateFromText")]
    pub fn update_from_text(&mut self, content: String) -> Result<()> {
        self.inner_mut()?.update_from_text(&content).map_err(first_napi_error)
    }
}

#[napi]
pub fn parse(content: String) -> Result<JsAam> {
    AAM::parse(&content)
        .map(|inner| JsAam { inner: Some(inner) })
        .map_err(first_napi_error)
}

#[napi]
pub fn load(path: String) -> Result<JsAam> {
    AAM::load(path)
        .map(|inner| JsAam { inner: Some(inner) })
        .map_err(first_napi_error)
}

#[cfg(feature = "reconstructer")]
#[napi(js_name = "reconstructSchema")]
pub fn reconstruct_schema(name: String, contents: Vec<String>) -> Result<String> {
    let refs: Vec<&str> = contents.iter().map(String::as_str).collect();
    reconstructer::reconstruct_schema(&name, &refs).map_err(|e| Error::new(Status::GenericFailure, e))
}

#[napi]
pub fn format(content: String) -> Result<String> {
    let aam = AAM::new();
    let rules = FormatterRules::default();
    aam.format(&content, &rules).map_err(to_napi_error)
}

#[napi(js_name = "lspAssist")]
pub fn lsp_assist(content: String) -> JsLspResult {
    let rules = FormatterRules::default();
    JsLspResult::from(AAM::lsp_assist(&content, &rules))
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// ── InlineObject ─────────────────────────────────────────────────────────────

#[napi(js_name = "InlineObject")]
pub struct JsInlineObject {
    inner: InlineObject,
}

impl Default for JsInlineObject {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl JsInlineObject {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: InlineObject::new(),
        }
    }

    #[napi]
    pub fn add(&mut self, key: String, value: String) {
        self.inner.add_field(&key, &value);
    }

    #[napi(js_name = "toString")]
    pub fn as_string(&self) -> String {
        self.inner.to_string()
    }
}

/// Parse an inline object string into a JavaScript object.
#[napi(js_name = "parseInlineToMap")]
pub fn js_parse_inline_to_map(content: String) -> Result<HashMap<String, String>> {
    aam_rs::builder::parse_inline_to_map(&content).map_err(to_napi_error)
}

// ── TOMLTranslator ──────────────────────────────────────────────────────────

#[napi(js_name = "TOMLTranslator")]
pub struct JsTOMLTranslator;

#[napi]
impl JsTOMLTranslator {
    #[napi(js_name = "tomlToAAM")]
    pub fn toml_to_aam(toml_source: String) -> Result<Vec<String>> {
        TOMLTranslator::toml_to_aam(&toml_source)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
            .map(|builders| builders.into_iter().map(|b| b.build()).collect())
    }
}
