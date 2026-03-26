        self.inner.find_deep(key).map(|v| v.to_string())
        self.inner.find_key(value).map(|v| v.to_string())
        self.inner.find_obj(key).map(|v| v.to_string())
        let rules = FormatterRules::default();
        self.inner.format(content, &rules)
        self.inner
            .merge_content(content)
            .map_err(|e| JsValue::from_str(&e.to_string()))
        let report = AAM::recover_simple(content);
        Ok(Self { inner })
use aam_rs::pipeline::formatter::FormattingOptions as FormatterRules;
use aam_rs::aam::AAM;
use aam_rs::pipeline::formatter::FormattingOptions as FormatterRules;
use wasm_bindgen::prelude::*;

fn first_js_error(errors: Vec<AamlError>) -> JsValue {
    let err = errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    });
    JsValue::from_str(&err.to_string())
}

/// JavaScript-facing document wrapper for AAM parsing and lookups.
#[wasm_bindgen]
pub struct AamDocument {
    inner: AAM,
}

#[wasm_bindgen]
impl AamDocument {
    /// Parses AAM text and returns a new document instance.
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str) -> Result<AamDocument, JsValue> {
        let inner = AAM::parse(content).map_err(first_js_error)?;
        Ok(Self { inner })
    }

    /// Loads and parses content into the current document, replacing existing state.
    pub fn parse(&mut self, content: &str) -> Result<(), JsValue> {
        self.inner = AAM::parse(content).map_err(first_js_error)?;
        Ok(())
    }

    /// Attempts to recover simple malformed input by dropping invalid lines.
    #[wasm_bindgen(js_name = recoverSimple)]
    pub fn recover_simple(content: &str) -> AamDocument {
        let report = AAM::recover_simple(content);
        AamDocument {
            inner: report.recovered,
        }
    }

    /// Merges additional AAM text into the current document.
    pub fn merge(&mut self, content: &str) -> Result<(), JsValue> {
        self.inner
            .merge_content(content)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
    
    /// Formats an AAM string to standardized style.
    pub fn format(&self, content: &str) -> Result<String, JsValue> {
        let rules = FormatterRules::default();
        self.inner.format(content, &rules)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Returns the value for a key, or null when missing.
    #[wasm_bindgen(js_name = findObj)]
    pub fn find_obj(&self, key: &str) -> Option<String> {
        self.inner.find_obj(key).map(|v| v.to_string())
    }

    /// Returns the key for a value, or null when missing.
    #[wasm_bindgen(js_name = findKey)]
    pub fn find_key(&self, value: &str) -> Option<String> {
        self.inner.find_key(value).map(|v| v.to_string())
    }

    /// Resolves deep references and returns the terminal value.
    #[wasm_bindgen(js_name = findDeep)]
    pub fn find_deep(&self, key: &str) -> Option<String> {
        self.inner.find_deep(key).map(|v| v.to_string())
    }
}


#[cfg(test)]
mod tests {
    use crate::AamDocument;

    #[test]
    fn parse_and_lookup_smoke() {
        let doc = AamDocument::new("host = localhost\nport = 8080").expect("should parse");
        assert_eq!(doc.find_obj("host").as_deref(), Some("localhost"));
    }
}
