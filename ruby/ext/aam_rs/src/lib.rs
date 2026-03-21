use aam_rs::aaml::AAML;
use magnus::{Error, Ruby, exception, function, prelude::*};

fn parse_find_obj(content: String, key: String) -> Result<Option<String>, Error> {
    let doc = AAML::parse(&content)
        .map_err(|err| Error::new(exception::runtime_error(), err.to_string()))?;
    let value = doc.find_obj(&key).map(|found| found.to_string());
    Ok(value)
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AamRs")?;
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
