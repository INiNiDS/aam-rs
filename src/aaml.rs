use std::collections::HashMap;
use std::path::Path;
use crate::found_value::FoundValue;

pub struct AAML {
    map: HashMap<String, String>,
}

impl AAML {
    pub fn load<P: AsRef<Path>>(file_path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let mut map = HashMap::new();

        for line in content.lines() {
            if line.trim().is_empty() || line.starts_with("#") {
                continue; // Skip empty lines and comments
            }

            if let Some((name, value)) = line.split_once('=') {
                map.insert(name.trim().to_string(), value.trim().to_string());
            }
        }

        Ok(AAML { map })
    }

    pub fn find_obj(&self, key: &str) -> Option<FoundValue> {
        self.map.get(key).map(|v| FoundValue::new(&*v.clone()))
    }
}