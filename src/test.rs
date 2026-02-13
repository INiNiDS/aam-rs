#[cfg(test)]
mod tests {
    use crate::aaml::AAML;
    use crate::found_value::FoundValue;

    const TEST_CONFIG: &str = "
        a = b
        c = d
        e = f
        d = g
        loop1 = loop2
        loop2 = loop1
    ";

    #[test]
    fn test_simple_find() {
        let parser = AAML::parse(TEST_CONFIG);
        let res = parser.find_obj("a").expect("Должен найти 'a'");
        assert_eq!(res, "b");
    }

    #[test]
    fn test_not_found() {
        let parser = AAML::parse(TEST_CONFIG);
        assert!(parser.find_obj("unknown").is_none());
    }

    #[test]
    fn test_deref_behavior() {
        let parser = AAML::parse(TEST_CONFIG);
        let res = parser.find_obj("a").unwrap();

        assert_eq!(res.len(), 1);
        assert!(res.starts_with('b'));
    }

    #[test]
    fn test_display_trait() {
        let res = FoundValue::new(&*"hello".to_string());
        let formatted = format!("{}", res);
        assert_eq!(formatted, "hello");
    }
}