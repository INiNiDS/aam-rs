use aam_rs::aam::{AAM, AAML};
use aam_rs::pipeline::formatter::FormatterRules;
use magnus::{exception, function, method, prelude::*, Error, Ruby};

// Wrapper for the new AAM logic
#[magnus::wrap(class = "AamRb::AAM", free_immediately, size)]
pub struct RubyAam {
    inner: AAM,
}

impl RubyAam {
    pub fn parse(content: String) -> Result<Self, Error> {
        let doc = AAM::parse(&content)
            .map_err(|err| Error::new(Ruby::exception_runtime_error(), err.to_string()))?;
        Ok(Self { inner: doc })
    }

    pub fn format(content: String) -> Result<String, Error> {
        let mut aam = AAM::new();
        aam.format(&content, &FormatterRules::default())
            .map_err(|err| Error::new(Ruby::exception_runtime_error(), err.to_string()))
    }

    pub fn find_obj(&self, key: String) -> Option<String> {
        self.inner.find_obj(&key).map(|found| found.as_str().to_string())
    }
}

// Deprecated AAML wrapper for backward compatibility
#[magnus::wrap(class = "AamRb::AAML", free_immediately, size)]
pub struct RubyAaml {
    inner: AAM,
}

impl RubyAaml {
    #[deprecated(since = "1.0.0", note = "Use AamRb::AAM instead")]
    pub fn parse(content: String) -> Result<Self, Error> {
        let doc = AAM::parse(&content)
            .map_err(|err| Error::new(Ruby::exception_runtime_error(), err.to_string()))?;
        Ok(Self { inner: doc })
    }

    #[deprecated(since = "1.0.0", note = "Use AamRb::AAM instead")]
    pub fn find_obj(&self, key: String) -> Option<String> {
        self.inner.find_obj(&key).map(|found| found.as_str().to_string())
    }
}

// Retain the old global method for maximum backwards compatibility
#[deprecated(since = "1.0.0", note = "Use AamRb::AAM.parse instead")]
fn parse_find_obj(content: String, key: String) -> Result<Option<String>, Error> {
    let doc = AAM::parse(&content)
        .map_err(|err| Error::new(Ruby::exception_runtime_error(), err.to_string()))?;
    Ok(doc.find_obj(&key).map(|found| found.as_str().to_string()))
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AamRb")?;

    // Register new AAM class
    let aam_class = module.define_class("AAM", ruby.class_object())?;
    aam_class.define_singleton_method("parse", function!(RubyAam::parse, 1))?;
    aam_class.define_singleton_method("format", function!(RubyAam::format, 1))?;
    aam_class.define_method("find_obj", method!(RubyAam::find_obj, 1))?;

    // Register deprecated AAML class
    let aaml_class = module.define_class("AAML", ruby.class_object())?;
    aaml_class.define_singleton_method("parse", function!(RubyAaml::parse, 1))?;
    aaml_class.define_method("find_obj", method!(RubyAaml::find_obj, 1))?;

    // Global deprecated method
    module.define_singleton_method("parse_find_obj", function!(parse_find_obj, 2))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_find_obj;

    #[test]
    fn parse_find_obj_smoke() {
        let value = parse_find_obj("host = localhost".to_string(), "host".to_string())
            .expect("parse should succeed");
        assert_eq!(value.as_deref(), Some("localhost"));
    }
}
