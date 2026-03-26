use crate::error::AamlError;
use crate::pipeline::{
    Pipeline, PipelineHashMap, PipelineOutput, SchemaInfo, TypeInfo, new_pipeline_hash_map,
};
use smol_str::SmolStr;
use std::path::Path;

#[cfg(feature = "aot")]
use crate::aot::{AamCompiler, AamLoader, MappedAam};

/// The main AAM configuration store.
///
/// Holds the final, validated output of the AAM pipeline, including
/// the key-value map, schemas, and registered types.
#[derive(Debug)]
pub struct AAM {
    pipeline_output: PipelineOutput,
}

impl AAM {
    /// Parses an AAML string using the default Pipeline and returns a new [`AAM`] instance.
    pub fn parse(text: &str) -> Result<Self, Vec<AamlError>> {
        let pipeline = Pipeline::new();
        let output = pipeline.process(text)?;

        Ok(Self {
            pipeline_output: output,
        })
    }

    /// Creates an [`AAM`] instance from a custom configured Pipeline.
    /// Use this if you need to register custom commands, parsers, or validators.
    pub fn from_pipeline(pipeline: Pipeline, text: &str) -> Result<Self, Vec<AamlError>> {
        let output = pipeline.process(text)?;
        Ok(Self {
            pipeline_output: output,
        })
    }

    /// Loads an `.aam` file from disk.
    ///
    /// With `aot` enabled (default), this uses cooked `.aam.bin` cache as the
    /// primary path and only invokes parsing/cooking when cache is missing/stale.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Vec<AamlError>> {
        #[cfg(feature = "aot")]
        {
            let mapped = AamLoader::load_fast(path)?;
            return Ok(Self {
                pipeline_output: Self::pipeline_output_from_mapped(&mapped),
            });
        }

        #[cfg(not(feature = "aot"))]
        {
            let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
                vec![AamlError::IoError {
                    details: format!("failed to read '{}': {e}", path.as_ref().display()),
                    diagnostics: None,
                }]
            })?;

            Self::parse(&content)
        }
    }

    /// Explicitly cooks an `.aam` file into `.aam.bin` cache.
    #[cfg(feature = "aot")]
    pub fn cook(path: impl AsRef<Path>) -> Result<std::path::PathBuf, Vec<AamlError>> {
        AamCompiler::cook(path)
    }

    /// Exposes zero-copy fast loading for advanced runtime integrations.
    #[cfg(feature = "aot")]
    pub fn load_fast(path: impl AsRef<Path>) -> Result<MappedAam, Vec<AamlError>> {
        AamLoader::load_fast(path)
    }

    #[cfg(feature = "aot")]
    fn pipeline_output_from_mapped(mapped: &MappedAam) -> PipelineOutput {
        let mut map = new_pipeline_hash_map();
        for (k, v) in mapped.iter_pairs() {
            map.insert(SmolStr::new(k), SmolStr::new(v));
        }

        PipelineOutput {
            map,
            schemas: new_pipeline_hash_map(),
            types: new_pipeline_hash_map(),
        }
    }

    // ── Search & Filtering ───────────────────────────────────────────

    /// Deep Search
    pub fn deep_search(&self, pattern: &str) -> Vec<(&str, &str)> {
        self.pipeline_output
            .map
            .iter()
            .filter(|(k, _)| k.contains(pattern))
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
            .collect()
    }

    /// Reserve Search
    pub fn reverse_search(&self, target_value: &str) -> Vec<&str> {
        self.pipeline_output
            .map
            .iter()
            .filter(|(_, v)| &**v == target_value)
            .map(|(k, _)| &**k)
            .collect()
    }

    /// Advanced search with predicate.
    pub fn find_by<F>(&self, predicate: F) -> Vec<(&str, &str)>
    where
        F: Fn(&str, &str) -> bool,
    {
        self.pipeline_output
            .map
            .iter()
            .filter(|(k, v)| predicate(&**k, &**v))
            .map(|(k, v)| (&**k, &**v))
            .collect()
    }

    /// Find by key or value with fallback. First tries to find by key, if not found, then tries to find by value.
    pub fn find(&self, query: &str) -> Vec<(&str, &str)> {
        if let Some((k, v)) = self.pipeline_output.map.get_key_value(query) {
            return vec![(&**k, &**v)];
        }

        self.pipeline_output
            .map
            .iter()
            .filter(|(_, v)| &**v == query)
            .map(|(k, v)| (&**k, &**v))
            .collect()
    }

    // ── Key-Value Data Accessors ─────────────────────────────────────────────

    /// Retrieves a string value by its key.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pipeline_output.map.get(key).map(|v| v.as_ref())
    }

    /// Returns all keys currently stored in the map.
    pub fn keys(&self) -> Vec<&str> {
        self.pipeline_output.map.keys().map(|k| &**k).collect()
    }

    /// Returns all key-value pairs as a standard `FxHashMap<String, String>`.
    pub fn to_map(&self) -> PipelineHashMap<String, String> {
        self.pipeline_output
            .map
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // ── Schema & Type Accessors ──────────────────────────────────────────────

    /// Returns a reference to all registered schemas.
    pub fn schemas(&self) -> &PipelineHashMap<SmolStr, SchemaInfo> {
        &self.pipeline_output.schemas
    }

    /// Returns a specific schema by name, if it exists.
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.pipeline_output.schemas.get(name)
    }

    /// Returns a reference to all registered types.
    pub fn types(&self) -> &PipelineHashMap<SmolStr, TypeInfo> {
        &self.pipeline_output.types
    }

    /// Returns a specific type info by name, if it exists.
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.pipeline_output.types.get(name)
    }
}
