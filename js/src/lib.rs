use aam_rs::aaml::AAML;
use aam_rs::error::AamlError;
use napi::{Error, Result, Status};
use napi_derive::napi;
use std::collections::BTreeMap;

fn to_napi_error(err: AamlError) -> Error {
    Error::new(Status::GenericFailure, err.to_string())
}

fn closed_error() -> Error {
    Error::new(Status::GenericFailure, "AAML instance is closed".to_owned())
}

#[napi(js_name = "AAML")]
pub struct JsAaml {
    inner: Option<AAML>,
}

impl JsAaml {
    fn inner_ref(&self) -> Result<&AAML> {
        self.inner.as_ref().ok_or_else(closed_error)
    }

    fn inner_mut(&mut self) -> Result<&mut AAML> {
        self.inner.as_mut().ok_or_else(closed_error)
    }
}

#[napi]
impl JsAaml {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Some(AAML::new()),
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

    #[napi(js_name = "findObj")]
    pub fn find_obj(&self, key: String) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.find_obj(&key).map(|value| value.as_str().to_string()))
    }

    #[napi(js_name = "findKey")]
    pub fn find_key(&self, value: String) -> Option<String> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.find_key(&value).map(|key| key.as_str().to_string()))
    }

    #[napi(js_name = "findDeep")]
    pub fn find_deep(&self, key: String) -> Option<String> {
        self.inner_ref().ok().and_then(|inner| {
            inner
                .find_deep(&key)
                .map(|value| value.as_str().to_string())
        })
    }

    #[napi(js_name = "findList")]
    pub fn find_list(&self, key: String) -> Option<Vec<String>> {
        self.inner_ref()
            .ok()
            .and_then(|inner| inner.find_obj(&key).and_then(|value| value.as_list()))
    }

    #[napi(js_name = "findObject")]
    pub fn find_object(&self, key: String) -> Option<BTreeMap<String, String>> {
        self.inner_ref().ok().and_then(|inner| {
            inner
                .find_obj(&key)
                .and_then(|value| value.as_object().map(|map| map.into_iter().collect()))
        })
    }

    #[napi]
    pub fn keys(&self) -> Vec<String> {
        match self.inner_ref() {
            Ok(inner) => inner.keys().iter().map(|key| key.to_string()).collect(),
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
        self.inner_ref()?
            .validate_value(&type_name, &value)
            .map_err(to_napi_error)
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
pub fn parse(content: String) -> Result<JsAaml> {
    AAML::parse(&content)
        .map(|inner| JsAaml { inner: Some(inner) })
        .map_err(to_napi_error)
}

#[napi]
pub fn load(path: String) -> Result<JsAaml> {
    AAML::load(path)
        .map(|inner| JsAaml { inner: Some(inner) })
        .map_err(to_napi_error)
}

#[napi]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
