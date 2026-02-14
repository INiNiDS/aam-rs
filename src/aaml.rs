use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use crate::error::AamlError;
use crate::found_value::FoundValue;

#[derive(Debug, Clone)]
pub struct AAML {
    map: HashMap<String, String>,
}

impl AAML {
    pub fn parse(content: &str) -> Result<Self, AamlError> {
        let mut map = HashMap::with_capacity(content.lines().count());

        for (index, line) in content.lines().enumerate() {
            let clean_line = Self::strip_comment(line);
            let clean_line = clean_line.trim();

            if clean_line.is_empty() {
                continue;
            }

            if let Some((name, value)) = clean_line.split_once('=') {
                let key = name.trim();
                let mut val = value.trim();

                if key.is_empty() {
                    return Err(AamlError::ParseError {
                        line: index + 1,
                        content: line.to_string(),
                        details: "Key cannot be empty".to_string(),
                    });
                }

                if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                    if val.len() >= 2 {
                        val = &val[1..val.len()-1];
                    }
                }

                map.insert(key.to_string(), val.to_string());
            } else {
                return Err(AamlError::ParseError {
                    line: index + 1,
                    content: line.to_string(),
                    details: "Missing assignment operator '='".to_string(),
                });
            }
        }

        map.shrink_to_fit();

        Ok(AAML { map })
    }

    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, AamlError> {
        let content = fs::read_to_string(file_path)?;

        Self::parse(&content)
    }

    pub fn find_obj(&self, key: &str) -> Option<FoundValue> {
        match self.map.get(key).map(|v| FoundValue::new(&*v)) {
            Some(val) => Some(val),
            None => self.find_key(key),
        }
    }

    pub fn find_deep(&self, key: &str) -> Option<FoundValue> {
        let mut current_key = key.to_string();
        let mut last_found = None;
        let mut visited = HashSet::new();

        while let Some(next_val) = self.map.get(&current_key) {
            if !visited.insert(current_key.clone()) {
                break;
            }

            if visited.contains(next_val) {
                if last_found.is_none() {
                    last_found = Some(FoundValue::new(next_val));
                }
                break;
            }

            last_found = Some(FoundValue::new(&*next_val.clone()));
            current_key = next_val.clone();
        }

        last_found
    }

    pub fn find_key(&self, value: &str) -> Option<FoundValue> {
        self.map.iter()
            .find_map(|(k, v)| {
                if v == value {
                    Some(FoundValue::new(k))
                } else {
                    None
                }
            })
    }

    fn strip_comment(line: &str) -> String {
        let mut result = String::with_capacity(line.len());
        let mut in_quote = false;
        let mut quote_char = '\0';

        for c in line.chars() {
            if c == '"' || c == '\'' {
                if in_quote {
                    if c == quote_char {
                        in_quote = false;
                    }
                } else {
                    in_quote = true;
                    quote_char = c;
                }

                if c == '#' && !in_quote {
                    break;
                }

                result.push(c);
            }
        }

        result
    }
}