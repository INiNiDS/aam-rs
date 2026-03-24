use aam_rs::aaml::AAML;
use aam_rs::aaml::parsing::{parse_inline_object, strip_comment};

#[test]
fn matrix_assignments_and_reverse_lookup_cases() {
    let mut covered_cases = 0usize;

    for i in 0..180 {
        let key = format!("key_{i}");
        let (raw_value, expected) = match i % 5 {
            0 => (format!("value_{i}"), format!("value_{i}")),
            1 => (format!("\"quoted value {i}\""), format!("quoted value {i}")),
            2 => (format!("'single quoted {i}'"), format!("single quoted {i}")),
            3 => {
                let hex = format!("{:06x}", i * 113 + 17);
                (format!("#{hex}"), format!("#{hex}"))
            }
            _ => (
                format!("list = [{i}, {}]", i + 1),
                format!("list = [{i}, {}]", i + 1),
            ),
        };

        let content = format!("{key} = {raw_value}");
        let doc = AAML::parse(&content).expect("matrix parse must succeed");
        assert_eq!(doc.find_obj(&key).unwrap().as_str(), expected);
        assert_eq!(doc.find_obj(&expected).unwrap().as_str(), key);
        covered_cases += 1;
    }

    assert_eq!(covered_cases, 180);
}

#[test]
fn matrix_comment_parsing_cases() {
    let mut covered_cases = 0usize;

    for i in 0..140 {
        if i % 4 == 0 {
            let line = format!("field_{i} = data_{i} # comment {i}");
            let stripped = strip_comment(&line).trim();
            assert_eq!(stripped, format!("field_{i} = data_{i}"));
        } else if i % 4 == 1 {
            let line = format!("tint_{i}=#ff{i:04x}");
            assert_eq!(strip_comment(&line), line);
        } else if i % 4 == 2 {
            let line = format!("key_{i} = value_{i}#tail");
            assert_eq!(strip_comment(&line), line);
        } else {
            let line = format!("key_{i} = \"value # keep {i}\"");
            assert_eq!(strip_comment(&line), line);
        }
        covered_cases += 1;
    }

    assert_eq!(covered_cases, 140);
}

#[test]
fn matrix_find_deep_chain_and_cycle_cases() {
    let mut covered_cases = 0usize;

    for len in 2..102 {
        let mut lines = String::new();
        for i in 0..(len - 1) {
            lines.push_str(&format!("k{i}=k{}\n", i + 1));
        }
        lines.push_str(&format!("k{}=terminal_{len}\n", len - 1));

        let doc = AAML::parse(&lines).expect("chain parse must succeed");
        assert_eq!(
            doc.find_deep("k0").unwrap().as_str(),
            format!("terminal_{len}")
        );
        covered_cases += 1;
    }

    for len in 3..103 {
        let mut lines = String::new();
        for i in 0..(len - 1) {
            lines.push_str(&format!("c{i}=c{}\n", i + 1));
        }
        lines.push_str(&format!("c{}=c{}\n", len - 1, len - 2));

        let doc = AAML::parse(&lines).expect("cycle parse must succeed");
        assert_eq!(
            doc.find_deep("c0").unwrap().as_str(),
            format!("c{}", len - 1)
        );
        covered_cases += 1;
    }

    assert_eq!(covered_cases, 200);
}

#[test]
fn matrix_inline_object_parser_cases() {
    let mut covered_cases = 0usize;

    for i in 0..120 {
        let src = format!(
            "{{ name = \"user {i}\", level = {i}, nested = {{ x = {i}, y = {} }}, tags = [a, b, c] }}",
            i + 1
        );

        let fields = parse_inline_object(&src).expect("inline object parse must succeed");
        assert!(
            fields
                .iter()
                .any(|(k, v)| k == "name" && v == &format!("user {i}"))
        );
        assert!(
            fields
                .iter()
                .any(|(k, v)| k == "level" && v == &format!("{i}"))
        );
        assert!(
            fields
                .iter()
                .any(|(k, v)| k == "nested" && v == &format!("{{ x = {i}, y = {} }}", i + 1))
        );
        assert!(fields.iter().any(|(k, v)| k == "tags" && v == "[a, b, c]"));
        covered_cases += 1;
    }

    for i in 0..30 {
        let broken = format!("{{ good = 1, broken_field_{i} }}");
        assert!(parse_inline_object(&broken).is_err());
        covered_cases += 1;
    }

    assert_eq!(covered_cases, 150);
}

#[test]
fn matrix_builtin_type_validation_cases() {
    let doc = AAML::new();
    let mut covered_cases = 0usize;

    for i in -80..80 {
        assert!(doc.validate_value("i32", &i.to_string()).is_ok());
        covered_cases += 1;
    }

    for i in 0..80 {
        let value = format!("{}.{}", i * 3 + 1, i % 10);
        assert!(doc.validate_value("f64", &value).is_ok());
        covered_cases += 1;
    }

    for val in ["true", "false", "1", "0", "TRUE", "False"] {
        assert!(doc.validate_value("bool", val).is_ok());
        covered_cases += 1;
    }

    for val in ["yes", "no", "2", "", "truthy", "f"] {
        assert!(doc.validate_value("bool", val).is_err());
        covered_cases += 1;
    }

    for i in 0..30 {
        let rgb = format!("#{:06x}", i * 1000 + 77);
        assert!(doc.validate_value("color", &rgb).is_ok());
        covered_cases += 1;
    }

    for i in 0..12 {
        let rgba = format!("#{:08x}", i * 5000 + 99);
        assert!(doc.validate_value("color", &rgba).is_ok());
        covered_cases += 1;
    }

    for bad in ["#123", "#abcd", "123456", "#gggggg", "#12345z", "#1"] {
        assert!(doc.validate_value("color", bad).is_err());
        covered_cases += 1;
    }

    for i in 0..30 {
        let vec3 = format!("{i}, {}, {}", i + 1, i + 2);
        assert!(doc.validate_value("math::vector3", &vec3).is_ok());
        covered_cases += 1;
    }

    for i in 0..15 {
        let invalid_vec = format!("{i}, {}", i + 1);
        assert!(doc.validate_value("math::vector3", &invalid_vec).is_err());
        covered_cases += 1;
    }

    for i in 0..25 {
        let list = format!("[{i}, {}, {}]", i + 1, i + 2);
        assert!(doc.validate_value("list<i32>", &list).is_ok());
        covered_cases += 1;
    }

    for i in 0..20 {
        let invalid = format!("[{i}, bad_{i}, {}]", i + 2);
        assert!(doc.validate_value("list<i32>", &invalid).is_err());
        covered_cases += 1;
    }

    assert_eq!(covered_cases, 390);
}

#[test]
fn matrix_schema_validation_cases() {
    let mut covered_cases = 0usize;

    for i in 0..120 {
        let content = format!(
            "@schema Device {{ id: i32, name: string, enabled: bool }}\nid = {i}\nname = dev_{i}\nenabled = {}\n",
            if i % 2 == 0 { "true" } else { "false" }
        );
        let doc = AAML::parse(&content).expect("valid schema instance should parse");
        assert_eq!(doc.find_obj("id").unwrap().as_str(), format!("{i}"));
        covered_cases += 1;
    }

    for i in 0..60 {
        let content =
            format!("@schema Device {{ id: i32, name: string }}\nid = bad_{i}\nname = dev_{i}\n");
        assert!(AAML::parse(&content).is_err());
        covered_cases += 1;
    }

    assert_eq!(covered_cases, 180);
}
