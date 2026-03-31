use aam_rs::aam::AAM;

#[test]
fn parse_accepts_valid_builtin_types() {
    let content = "@schema Device { id: i32, active: bool, ratio: f64 }\nid = 42\nactive = true\nratio = 3.14";
    assert!(AAM::parse(content).is_ok());
}

#[test]
fn parse_rejects_invalid_i32() {
    let content = "@schema Device { id: i32 }\nid = not_a_number";
    assert!(AAM::parse(content).is_err());
}

#[test]
fn parse_accepts_optional_schema_fields() {
    let content = "@schema Server { host: string, port*: i32 }\nhost = localhost";
    assert!(AAM::parse(content).is_ok());
}
