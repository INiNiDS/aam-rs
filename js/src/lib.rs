use aam_rs::aam::{AAM, AamLspAssist};
use aam_rs::error::AamlError;
use aam_rs::found_value::FoundValue;
use aam_rs::pipeline::formatter::{FormatRange, FormattingOptions as FormatterRules};
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

    #[napi(js_name = "findList")]
    pub fn find_list(&self, key: String) -> Option<Vec<String>> {
        self.get(key).and_then(|v| FoundValue::new(&v).as_list())
    }

    #[napi(js_name = "findObject")]
    pub fn find_object(&self, key: String) -> Option<HashMap<String, String>> {
        self.get(key)
            .and_then(|v| FoundValue::new(&v).as_object())
            .map(|m| m.into_iter().collect())
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