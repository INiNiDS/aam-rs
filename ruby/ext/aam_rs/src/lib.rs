use aam_rs::aam::AAM;
use aam_rs::builder::{AAMBuilder, SchemaField};
use aam_rs::error::AamlError;
use aam_rs::pipeline::formatter::FormattingOptions as FormatterRules;
use magnus::{Error, Module, Object, Ruby, function, method};
use std::collections::BTreeMap;

fn ruby_runtime_error(message: String) -> Error {
    let ruby = Ruby::get().expect("Ruby VM must be initialized");
    Error::new(ruby.exception_runtime_error(), message)
}

fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    })
}

#[magnus::wrap(class = "AamRb::AAM", free_immediately, size)]
pub struct RubyAam {
    inner: AAM,
}

#[magnus::wrap(class = "AamRb::SchemaField", free_immediately, size)]
pub struct RubySchemaField {
    inner: SchemaField,
}

impl RubySchemaField {
    pub fn required(name: String, type_name: String) -> Self {
        Self {
            inner: SchemaField::required(name, type_name),
        }
    }

    pub fn optional(name: String, type_name: String) -> Self {
        Self {
            inner: SchemaField::optional(name, type_name),
        }
    }
}

#[magnus::wrap(class = "AamRb::AAMBuilder", free_immediately, size)]
pub struct RubyAamBuilder {
    inner: AAMBuilder,
}

impl RubyAamBuilder {
    pub fn new() -> Self {
        Self {
            inner: AAMBuilder::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: AAMBuilder::with_capacity(capacity),
        }
    }

    pub fn add_line(&mut self, key: String, value: String) {
        self.inner.add_line(&key, &value);
    }

    pub fn comment(&mut self, text: String) {
        self.inner.comment(&text);
    }

    pub fn schema(&mut self, name: String, fields: Vec<&RubySchemaField>) {
        self.inner
            .schema(&name, fields.into_iter().map(|field| field.inner.clone()));
    }

    pub fn schema_multiline(&mut self, name: String, fields: Vec<&RubySchemaField>) {
        self.inner
            .schema_multiline(&name, fields.into_iter().map(|field| field.inner.clone()));
    }

    pub fn derive(&mut self, path: String, schemas: Vec<String>) {
        self.inner.derive(&path, schemas);
    }

    pub fn import(&mut self, path: String) {
        self.inner.import(&path);
    }

    pub fn type_alias(&mut self, alias: String, type_name: String) {
        self.inner.type_alias(&alias, &type_name);
    }

    pub fn as_string(&self) -> String {
        self.inner.as_string()
    }
}

impl RubyAam {
    pub fn new() -> Self {
        Self { inner: AAM::new() }
    }

    pub fn parse(content: String) -> Result<Self, Error> {
        let doc = AAM::parse(&content)
            .map_err(first_error)
            .map_err(|err| ruby_runtime_error(err.to_string()))?;
        Ok(Self { inner: doc })
    }

    pub fn load(path: String) -> Result<Self, Error> {
        let doc = AAM::load(&path)
            .map_err(first_error)
            .map_err(|err| ruby_runtime_error(err.to_string()))?;
        Ok(Self { inner: doc })
    }

    pub fn format(content: String) -> Result<String, Error> {
        let doc = AAM::new();
        let rules = FormatterRules::default();
        doc.format(&content, &rules)
            .map_err(|err| ruby_runtime_error(err.to_string()))
    }

    pub fn get(&self, key: String) -> Option<String> {
        self.inner.get(&key).map(ToString::to_string)
    }

    pub fn keys(&self) -> Vec<String> {
        self.inner
            .keys()
            .into_iter()
            .map(ToString::to_string)
            .collect()
    }

    pub fn to_map(&self) -> BTreeMap<String, String> {
        self.inner.to_map().into_iter().collect()
    }

    pub fn find(&self, query: String) -> Vec<(String, String)> {
        self.inner
            .find(&query)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    pub fn deep_search(&self, pattern: String) -> Vec<(String, String)> {
        self.inner
            .deep_search(&pattern)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    pub fn reverse_search(&self, value: String) -> Vec<String> {
        self.inner
            .reverse_search(&value)
            .into_iter()
            .map(ToString::to_string)
            .collect()
    }

    pub fn schema_names(&self) -> Vec<String> {
        self.inner
            .schemas()
            .map(|schemas| schemas.keys().map(ToString::to_string).collect())
            .unwrap_or_default()
    }

    pub fn type_names(&self) -> Vec<String> {
        self.inner
            .types()
            .map(|types| types.keys().map(ToString::to_string).collect())
            .unwrap_or_default()
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AamRb")?;
    let aam_class = module.define_class("AAM", ruby.class_object())?;
    let schema_field_class = module.define_class("SchemaField", ruby.class_object())?;
    let builder_class = module.define_class("AAMBuilder", ruby.class_object())?;

    aam_class.define_singleton_method("new", function!(RubyAam::new, 0))?;
    aam_class.define_singleton_method("parse", function!(RubyAam::parse, 1))?;
    aam_class.define_singleton_method("load", function!(RubyAam::load, 1))?;
    aam_class.define_singleton_method("format", function!(RubyAam::format, 1))?;
    aam_class.define_method("get", method!(RubyAam::get, 1))?;

    aam_class.define_method("keys", method!(RubyAam::keys, 0))?;
    aam_class.define_method("to_map", method!(RubyAam::to_map, 0))?;

    aam_class.define_method("find", method!(RubyAam::find, 1))?;
    aam_class.define_method("deep_search", method!(RubyAam::deep_search, 1))?;
    aam_class.define_method("reverse_search", method!(RubyAam::reverse_search, 1))?;

    aam_class.define_method("schema_names", method!(RubyAam::schema_names, 0))?;
    aam_class.define_method("type_names", method!(RubyAam::type_names, 0))?;

    schema_field_class
        .define_singleton_method("required", function!(RubySchemaField::required, 2))?;
    schema_field_class
        .define_singleton_method("optional", function!(RubySchemaField::optional, 2))?;

    builder_class.define_singleton_method("new", function!(RubyAamBuilder::new, 0))?;
    builder_class
        .define_singleton_method("with_capacity", function!(RubyAamBuilder::with_capacity, 1))?;
    builder_class.define_method("add_line", method!(RubyAamBuilder::add_line, 2))?;
    builder_class.define_method("comment", method!(RubyAamBuilder::comment, 1))?;
    builder_class.define_method("schema", method!(RubyAamBuilder::schema, 2))?;
    builder_class.define_method(
        "schema_multiline",
        method!(RubyAamBuilder::schema_multiline, 2),
    )?;
    builder_class.define_method("derive", method!(RubyAamBuilder::derive, 2))?;
    builder_class.define_method("import", method!(RubyAamBuilder::import, 1))?;
    builder_class.define_method("type_alias", method!(RubyAamBuilder::type_alias, 2))?;
    builder_class.define_method("as_string", method!(RubyAamBuilder::as_string, 0))?;

    Ok(())
}
