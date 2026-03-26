    let report = AAM::recover_simple(&content);
    let mut aam = AAM::new();
    let rules = FormatterRules::default();
    aam.format(&content, &rules).map_err(to_napi_error)
        .map(|inner| JsAam { inner: Some(inner) })
        .map(|inner| JsAam { inner: Some(inner) })
        self.inner_ref()?.validate_value(&type_name, &value).map_err(to_napi_error)
            inner.find_obj(&key).and_then(|v| v.as_object().map(|m| m.into_iter().collect()))
        self.inner_ref().ok().and_then(|inner| inner.find_obj(&key).and_then(|v| v.as_list()))
        self.inner_ref().ok().and_then(|inner| inner.find_deep(&key).map(|v| v.as_str().to_string()))
            .and_then(|inner| inner.find_key(&value).map(|k| k.as_str().to_string()))
            .and_then(|inner| inner.find_obj(&key).map(|v| v.as_str().to_string()))
        let rules = FormatterRules::default();
        self.inner_ref()?
            .format(&content, &rules)
            .map_err(to_napi_error)
        self.inner_mut()?.merge_file(path).map_err(to_napi_error)
        self.inner_mut()?
            .merge_content(&content)
            .map_err(to_napi_error)
            inner: Some(AAM::new()),
use aam_rs::pipeline::formatter::FormattingOptions as FormatterRules;
use aam_rs::aam::AAM;
use aam_rs::pipeline::formatter::FormattingOptions as FormatterRules;
use napi::{Error, Result, Status};
use napi_derive::napi;
use std::collections::BTreeMap;

fn to_napi_error(err: AamError) -> Error {
    Error::new(Status::GenericFailure, err.to_string())
}

fn first_napi_error(errors: Vec<AamError>) -> Error {
    let err = errors.into_iter().next().unwrap_or(AamError::ParseError {
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

#[napi]
impl JsAam {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(AAM::new()),
        }
    }

    #[napi]
    pub fn merge(&mut self, content: String) -> Result<()> {
        self.inner_mut()?
            .merge_content(&content)
            .map_err(to_napi_error)
    }

    #[napi(js_name = "mergeContent")]
    pub fn merge_content(&mut self, content: String) -> Result<()> {
        self.merge(content)
    }

    #[napi(js_name = "mergeFile")]
    pub fn merge_file(&mut self, path: String) -> Result<()> {
        self.inner_mut()?.merge_file(path).map_err(to_napi_error)
    }

    #[napi(js_name = "format")]
    pub fn format(&self, content: String) -> Result<String> {
        let rules = FormatterRules::default();
        self.inner_ref()?
            .format(&content, &rules)
            .map_err(to_napi_error)
    }

    #[napi(js_name = "findObj")]
    pub fn find_obj(&self, key: String) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.find_obj(&key).map(|v| v.as_str().to_string()))
    }

    #[napi(js_name = "findKey")]
    pub fn find_key(&self, value: String) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.find_key(&value).map(|k| k.as_str().to_string()))
    }

    #[napi(js_name = "findDeep")]
    pub fn find_deep(&self, key: String) -> Option<String> {
        self.inner_ref().ok().and_then(|inner| inner.find_deep(&key).map(|v| v.as_str().to_string()))
    }

    #[napi(js_name = "findList")]
    pub fn find_list(&self, key: String) -> Option<Vec<String>> {
        self.inner_ref().ok().and_then(|inner| inner.find_obj(&key).and_then(|v| v.as_list()))
    }

    #[napi(js_name = "findObject")]
    pub fn find_object(&self, key: String) -> Option<BTreeMap<String, String>> {
        self.inner_ref().ok().and_then(|inner| {
            inner.find_obj(&key).and_then(|v| v.as_object().map(|m| m.into_iter().collect()))
        })
    }

    #[napi]
    pub fn keys(&self) -> Vec<String> {
        match self.inner_ref() {
            Ok(inner) => inner.keys().iter().map(|k| k.to_string()).collect(),
            Err(_) => Vec::new(),
        }
    }

    #[napi(js_name = "toMap")]
    pub fn to_map(&self) -> BTreeMap<String, String> {
        self.inner_ref().map_or_else(
            |_| BTreeMap::new(),
            |inner| inner.to_map().into_iter().collect(),
        )
    }

    #[napi(js_name = "validateValue")]
    pub fn validate_value(&self, type_name: String, value: String) -> Result<()> {
        self.inner_ref()?.validate_value(&type_name, &value).map_err(to_napi_error)
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
    let mut aam = AAM::new();
    let rules = FormatterRules::default();
    aam.format(&content, &rules).map_err(to_napi_error)
}

#[napi(js_name = "recoverSimple")]
pub fn recover_simple(content: String) -> Result<JsAam> {
    let report = AAM::recover_simple(&content);
    Ok(JsAam {
        inner: Some(report.recovered),
    })
}


#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
