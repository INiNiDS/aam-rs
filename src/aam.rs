use crate::error::AamlError;
use crate::found_value::FoundValue;
use crate::pipeline::{
    new_pipeline_hash_map, DefaultFormatter, DefaultLexer, DefaultParser, ExecutionContext,
    Formatter, FormattingOptions, Lexer, Parser, Pipeline, PipelineBuildHasher, PipelineHashMap,
    PipelineOutput, SchemaInfo, TypeInfo,
};
use smol_str::SmolStr;
use std::collections::{HashMap, HashSet};
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
        let pair_count = mapped.archived().nodes.len().saturating_sub(1);
        let mut map = PipelineHashMap::with_capacity_and_hasher(
            pair_count,
            PipelineBuildHasher::default(),
        );
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
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pipeline_output.map.get(key).map(|v| v.as_ref())
    }

    /// Returns direct access to the internal key-value map without allocations.
    pub fn map(&self) -> &PipelineHashMap<SmolStr, SmolStr> {
        &self.pipeline_output.map
    }

    /// Iterates over all key-value pairs without allocating.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.pipeline_output
            .map
            .iter()
            .map(|(k, v)| (k.as_ref(), v.as_ref()))
    }

    /// Returns all keys currently stored in the map.
    /// Prefer [`AAM::iter`] for zero-allocation iteration.
    #[inline]
    pub fn keys(&self) -> Vec<&str> {
        self.pipeline_output.map.keys().map(|k| &**k).collect()
    }

    /// Returns all key-value pairs as a standard `FxHashMap<String, String>`.
    /// Prefer [`AAM::iter`] for zero-allocation iteration.
    #[inline]
    pub fn to_map(&self) -> PipelineHashMap<String, String> {
        self.pipeline_output
            .map
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    // ── Schema & Type Accessors ──────────────────────────────────────────────

    /// Returns a reference to all registered schemas.
    /// Prefer [`AAM::iter`] for zero-allocation iteration.
    #[inline]
    pub fn schemas(&self) -> &PipelineHashMap<SmolStr, SchemaInfo> {
        &self.pipeline_output.schemas
    }

    /// Returns a specific schema by name, if it exists.
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.pipeline_output.schemas.get(name)
    }

    /// Returns a reference to all registered types.
    /// Prefer [`AAM::iter`] for zero-allocation iteration.
    #[inline]
    pub fn types(&self) -> &PipelineHashMap<SmolStr, TypeInfo> {
        &self.pipeline_output.types
    }

    /// Returns a specific type info by name, if it exists.
    pub fn get_type(&self, name: &str) -> Option<&TypeInfo> {
        self.pipeline_output.types.get(name)
    }
}
