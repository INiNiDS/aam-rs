use aam_rs::aam::AAM;
use aam_rs::pipeline::formatter::FormatterRules;
use wasm_bindgen::prelude::*;

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
        let inner = AAM::parse(content).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self { inner })
    }

    /// Loads and parses content into the current document, replacing existing state.
    pub fn parse(&mut self, content: &str) -> Result<(), JsValue> {
        self.inner = AAM::parse(content).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
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

// Deprecated AAML wrapper
#[wasm_bindgen]
pub struct AamlDocument {
    inner: AamDocument,
}

#[wasm_bindgen]
impl AamlDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str) -> Result<AamlDocument, JsValue> {
        let inner = AamDocument::new(content)?;
        Ok(Self { inner })
    }
    
    #[wasm_bindgen(js_name = findObj)]
    pub fn find_obj(&self, key: &str) -> Option<String> {
        self.inner.find_obj(key)
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
