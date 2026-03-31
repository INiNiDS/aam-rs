use aam_rs::aam::{AAM, AamLspAssist};
use aam_rs::error::AamlError;
use aam_rs::pipeline::formatter::{FormatRange, FormattingOptions as FormatterRules};
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

#[wasm_bindgen(js_name = AAM)]
pub struct AamDocument {
    inner: AAM,
}

#[wasm_bindgen]
impl AamDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str) -> Result<AamDocument, JsValue> {
        let inner = AAM::parse(content).map_err(first_js_error)?;
        Ok(Self { inner })
    }

    pub fn format(&self, content: &str) -> Result<String, JsValue> {
        let rules = FormatterRules::default();
        self.inner
            .format(content, &rules)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    #[wasm_bindgen(js_name = formatRange)]
    pub fn format_range(
        &self,
        content: &str,
        start_line: u32,
        end_line: u32,
    ) -> Result<String, JsValue> {
        let rules = FormatterRules::default();
        let range = FormatRange {
            start_line: start_line as usize,
            end_line: end_line as usize,
        };
        self.inner
            .format_range(content, range, &rules)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    pub fn get(&self, key: &str) -> Option<String> {
        self.inner.get(key).map(ToString::to_string)
    }

    pub fn keys(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        for key in self.inner.keys() {
            arr.push(&JsValue::from_str(key));
        }
        arr
    }

    #[wasm_bindgen(js_name = toMap)]
    pub fn to_map(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();
        for (k, v) in self.inner.iter() {
            let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(k), &JsValue::from_str(v));
        }
        obj
    }

    pub fn find(&self, query: &str) -> js_sys::Object {
        let obj = js_sys::Object::new();
        for (k, v) in self.inner.find(query) {
            let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(k), &JsValue::from_str(v));
        }
        obj
    }

    #[wasm_bindgen(js_name = deepSearch)]
    pub fn deep_search(&self, pattern: &str) -> js_sys::Object {
        let obj = js_sys::Object::new();
        for (k, v) in self.inner.deep_search(pattern) {
            let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(k), &JsValue::from_str(v));
        }
        obj
    }

    #[wasm_bindgen(js_name = reverseSearch)]
    pub fn reverse_search(&self, value: &str) -> js_sys::Array {
        let arr = js_sys::Array::new();
        for key in self.inner.reverse_search(value) {
            arr.push(&JsValue::from_str(key));
        }
        arr
    }

    #[wasm_bindgen(js_name = schemaNames)]
    pub fn schema_names(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        if let Some(schemas) = self.inner.schemas() {
            for key in schemas.keys() {
                arr.push(&JsValue::from_str(key.as_str()));
            }
        }
        arr
    }

    #[wasm_bindgen(js_name = typeNames)]
    pub fn type_names(&self) -> js_sys::Array {
        let arr = js_sys::Array::new();
        if let Some(types) = self.inner.types() {
            for key in types.keys() {
                arr.push(&JsValue::from_str(key.as_str()));
            }
        }
        arr
    }

    #[wasm_bindgen(js_name = lspAssist)]
    pub fn lsp_assist(content: &str) -> JsValue {
        let rules = FormatterRules::default();
        let assist: AamLspAssist = AAM::lsp_assist(content, &rules);

        let obj = js_sys::Object::new();
        let diagnostics = js_sys::Array::new();
        for err in assist.diagnostics {
            diagnostics.push(&JsValue::from_str(&err.to_string()));
        }

        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("diagnostics"), &diagnostics);
        let formatted = assist
            .formatted
            .map_or(JsValue::NULL, |text| JsValue::from_str(&text));
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("formatted"), &formatted);
        JsValue::from(obj)
    }
}