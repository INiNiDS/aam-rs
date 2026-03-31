#[cfg(test)]
mod tests {
    use aam_rs::aam::AAM;

    #[test]
    fn color_not_treated_as_comment() {
        let doc = AAM::parse("tint = #ff6600").expect("parse should succeed");
        assert_eq!(doc.get("tint"), Some("#ff6600"));
    }

    #[test]
    fn comment_after_space_is_ignored() {
        let doc = AAM::parse("key = value # comment").expect("parse should succeed");
        assert_eq!(doc.get("key"), Some("value"));
    }

    #[test]
    fn quoted_hash_is_preserved() {
        let doc = AAM::parse(r#"key = "val # not comment""#).expect("parse should succeed");
        assert_eq!(doc.get("key"), Some("\"val # not comment\""));
    }

    #[test]
    fn inline_object_and_list_values_parse() {
        let content = "obj = { x = 1, y = 2 }\nitems = [a, b, c]";
        let doc = AAM::parse(content).expect("parse should succeed");

        assert_eq!(doc.get("obj"), Some("{ x = 1, y = 2 }"));
        assert_eq!(doc.get("items"), Some("[a, b, c]"));
    }
}
