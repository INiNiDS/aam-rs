use aam_rs::aam::{AamDocument as CoreAamDocument, AamLspAssist};
use aam_rs::builder::{AAMBuilder, InlineObject, SchemaField};
use aam_rs::error::AamlError;
use aam_rs::pipeline::formatter::{FormatRange, FormattingOptions as FormatterRules};
use aam_rs::translator::TOMLTranslator;
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

#[wasm_bindgen]
pub struct AamDocument {
    inner: CoreAamDocument,
}

#[wasm_bindgen(js_name = AAMBuilder)]
pub struct WasmAamBuilder {
    inner: AAMBuilder,
}

impl Default for WasmAamBuilder {
    fn default() -> Self {
        Self::new()
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

fn parse_section_header(line: &str) -> Option<&str> {
    if !line.starts_with('#') {
        return None;
    }
    let rest = line[1..].trim();
    rest.ends_with(".aam").then_some(rest)
}

fn parse_assignment(line: &str) -> Option<(&str, &str)> {
    let idx = line.find('=')?;
    if idx == 0 {
        return None;
    }
    let key = line[..idx].trim();
    if key.is_empty() {
        return None;
    }
    Some((key, line[idx + 1..].trim()))
}

#[wasm_bindgen]
impl WasmAamBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmAamBuilder {
        Self {
            inner: AAMBuilder::new(),
        }
    }

    #[wasm_bindgen(js_name = withCapacity)]
    pub fn with_capacity(capacity: usize) -> WasmAamBuilder {
        Self {
            inner: AAMBuilder::with_capacity(capacity),
        }
    }

    #[wasm_bindgen(js_name = addLine)]
    pub fn add_line(&mut self, key: &str, value: &str) {
        self.inner.add_line(key, value);
    }

    pub fn comment(&mut self, text: &str) {
        self.inner.comment(text);
    }

    pub fn schema(&mut self, name: &str, fields: Vec<String>) {
        self.inner.schema(name, parse_schema_fields(fields));
    }

    #[wasm_bindgen(js_name = schemaMultiline)]
    pub fn schema_multiline(&mut self, name: &str, fields: Vec<String>) {
        self.inner
            .schema_multiline(name, parse_schema_fields(fields));
    }

    pub fn derive(&mut self, path: &str, schemas: Vec<String>) {
        self.inner.derive(path, schemas);
    }

    #[wasm_bindgen(js_name = import)]
    pub fn import_path(&mut self, path: &str) {
        self.inner.import(path);
    }

    #[wasm_bindgen(js_name = typeAlias)]
    pub fn type_alias(&mut self, alias: &str, type_name: &str) {
        self.inner.type_alias(alias, type_name);
    }

    #[wasm_bindgen(js_name = asString)]
    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}

// ── Wasm InlineObject ────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = InlineObject)]
pub struct WasmInlineObject {
    inner: InlineObject,
}

impl Default for WasmInlineObject {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmInlineObject {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmInlineObject {
        WasmInlineObject {
            inner: InlineObject::new(),
        }
    }

    #[wasm_bindgen(js_name = add)]
    pub fn add_field(&mut self, key: &str, value: &str) {
        self.inner.add_field(key, value);
    }

    #[wasm_bindgen(js_name = toString)]
    pub fn to_js_string(&self) -> String {
        self.inner.to_string()
    }
}

/// Parse an inline object string into a JS object.
#[wasm_bindgen(js_name = parseInlineToMap)]
pub fn wasm_parse_inline_to_map(content: &str) -> Result<js_sys::Object, JsValue> {
    let map = aam_rs::builder::parse_inline_to_map(content)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let obj = js_sys::Object::new();
    for (k, v) in map {
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str(&k), &JsValue::from_str(&v));
    }
    Ok(obj)
}

#[wasm_bindgen]
impl AamDocument {
    #[wasm_bindgen(constructor)]
    pub fn new(content: &str) -> Result<AamDocument, JsValue> {
        let inner = CoreAamDocument::parse(content).map_err(first_js_error)?;
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
        let assist: AamLspAssist = CoreAamDocument::lsp_assist(content, &rules);

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

#[wasm_bindgen(js_name = splitAam)]
pub fn split_aam(content: &str) -> js_sys::Object {
    let result = js_sys::Object::new();
    let mut current_name: Option<String> = None;
    let mut current_builder = WasmAamBuilder::new();

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(filename) = parse_section_header(line) {
            if let Some(prev_name) = current_name.take() {
                let _ = js_sys::Reflect::set(
                    &result,
                    &JsValue::from_str(&prev_name),
                    &JsValue::from_str(&current_builder.as_string()),
                );
            }
            current_name = Some(filename.to_owned());
            current_builder = WasmAamBuilder::new();
            continue;
        }

        if current_name.is_some()
            && let Some((key, value)) = parse_assignment(line)
        {
            current_builder.add_line(key, value);
        }
    }

    if let Some(prev_name) = current_name {
        let _ = js_sys::Reflect::set(
            &result,
            &JsValue::from_str(&prev_name),
            &JsValue::from_str(&current_builder.as_string()),
        );
    }

    result
}

// ── TOMLTranslator ───────────────────────────────────────────────────────────

#[wasm_bindgen(js_name = TOMLTranslator)]
pub struct WasmTOMLTranslator;

#[wasm_bindgen]
impl WasmTOMLTranslator {
    #[wasm_bindgen(js_name = tomlToAAM)]
    pub fn toml_to_aam(toml_source: &str) -> Result<js_sys::Array, JsValue> {
        let builders = TOMLTranslator::toml_to_aam(toml_source)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        let arr = js_sys::Array::new();
        for builder in builders {
            arr.push(&JsValue::from_str(&builder.build()));
        }
        Ok(arr)
    }
}
