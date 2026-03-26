        self.inner.find_obj(&key).map(|found| found.as_str().to_string())
use aam_rs::binding_compat;
        aam.format(&content, &FormatterRules::default())
        let doc = AAM::parse(&content)
            .map_err(first_error)
fn first_error(errors: Vec<AamlError>) -> AamlError {
    errors.into_iter().next().unwrap_or(AamlError::ParseError {
        line: 1,
        content: String::new(),
use magnus::{function, method, prelude::*, Error, Ruby};

fn ruby_runtime_error(message: String) -> Error {
    let ruby = Ruby::get().expect("Ruby VM must be initialized");
    Error::new(ruby.exception_runtime_error(), message)
}

fn first_error(errors: Vec<AamlError>) -> AamlError {
        let doc = binding_compat::parse_single(&content)
        content: String::new(),
        details: "unexpected empty parse error list".to_string(),
        diagnostics: None,
    })
}
        binding_compat::format_content(&content)
#[magnus::wrap(class = "AamRb::AAM", free_immediately, size)]
pub struct RubyAam {
    inner: AAM,
}
        let report = aam_rs::recovery::recover_simple(&content);
impl RubyAam {
    pub fn parse(content: String) -> Result<Self, Error> {
        let doc = AAM::parse(&content)
            .map_err(first_error)
            .map_err(|err| ruby_runtime_error(err.to_string()))?;
        Ok(Self { inner: doc })
        binding_compat::find_obj(&self.inner, &key).map(|found| found.as_str().to_string())

    pub fn format(content: String) -> Result<String, Error> {
        let aam = AAM::new();
        aam.format(&content, &FormatterRules::default())
            .map_err(|err| ruby_runtime_error(err.to_string()))
    }

    pub fn recover_simple(content: String) -> Self {
        let report = AAM::recover_simple(&content);
        Self {
            inner: report.recovered,
        }
    }

    pub fn find_obj(&self, key: String) -> Option<String> {
        self.inner.find_obj(&key).map(|found| found.as_str().to_string())
    }
}


#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AamRb")?;

    // Register new AAM class
    let aam_class = module.define_class("AAM", ruby.class_object())?;
    aam_class.define_singleton_method("parse", function!(RubyAam::parse, 1))?;
    aam_class.define_singleton_method("format", function!(RubyAam::format, 1))?;
    aam_class.define_singleton_method("recover_simple", function!(RubyAam::recover_simple, 1))?;
    aam_class.define_method("find_obj", method!(RubyAam::find_obj, 1))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::RubyAam;

    #[test]
    fn parse_find_obj_smoke() {
        let value = RubyAam::parse("host = localhost".to_string())
            .expect("parse should succeed")
            .find_obj("host".to_string());
        assert_eq!(value.as_deref(), Some("localhost"));
    }
}
