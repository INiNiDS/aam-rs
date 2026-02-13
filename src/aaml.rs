use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::found_value::FoundValue;

pub struct AAML {
    map: HashMap<String, String>,
}

impl AAML {
    pub fn parse(content: &str) -> Self {
        let mut map = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            if let Some((name, value)) = line.split_once('=') {
                map.insert(name.trim().to_string(), value.trim().to_string());
            }
        }

        AAML { map }
    }
    
    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        Ok(Self::parse(&content))
    }

    pub fn find_obj(&self, key: &str) -> Option<FoundValue> {
        self.map.get(key).map(|v| FoundValue::new(&*v.clone()))
    }
}