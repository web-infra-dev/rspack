use std::collections::hash_map::Entry;
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use anymap::CloneAny;
use rspack_error::{internal_error, Result};
use rspack_identifier::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;
use xxhash_rust::xxh3::Xxh3;

use crate::{
  AssetInfo, AstOrSource, ChunkInitFragments, ModuleIdentifier, RuntimeGlobals, RuntimeSpec,
  RuntimeSpecMap, SourceType,
};

#[derive(Debug, Clone)]
pub struct GenerationResult {
  pub ast_or_source: AstOrSource,
}

impl From<AstOrSource> for GenerationResult {
  fn from(ast_or_source: AstOrSource) -> Self {
    GenerationResult { ast_or_source }
  }
}

#[derive(Clone, Debug)]
pub struct CodeGenerationDataUrl {
  inner: String,
}

impl CodeGenerationDataUrl {
  pub fn new(inner: String) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &str {
    &self.inner
  }
}

#[derive(Clone, Debug)]
pub struct CodeGenerationDataFilename {
  inner: String,
}

impl CodeGenerationDataFilename {
  pub fn new(inner: String) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &str {
    &self.inner
  }
}

#[derive(Clone, Debug)]
pub struct CodeGenerationDataAssetInfo {
  inner: AssetInfo,
}

impl CodeGenerationDataAssetInfo {
  pub fn new(inner: AssetInfo) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &AssetInfo {
    &self.inner
  }
}

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationData {
  inner: anymap::Map<dyn CloneAny + Send + Sync>,
}

impl Deref for CodeGenerationData {
  type Target = anymap::Map<dyn CloneAny + Send + Sync>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl DerefMut for CodeGenerationData {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResult {
  inner: HashMap<SourceType, GenerationResult>,
  /// [definition in webpack](https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L75)
  pub data: CodeGenerationData,
  pub chunk_init_fragments: ChunkInitFragments,
  pub runtime_requirements: RuntimeGlobals,
  pub hash: u64,
}

impl CodeGenerationResult {
  pub fn with_javascript(mut self, generation_result: impl Into<GenerationResult>) -> Self {
    self
      .inner
      .insert(SourceType::JavaScript, generation_result.into());
    self
  }

  pub fn with_css(mut self, generation_result: impl Into<GenerationResult>) -> Self {
    self.inner.insert(SourceType::Css, generation_result.into());
    self
  }

  pub fn with_asset(mut self, generation_result: impl Into<GenerationResult>) -> Self {
    self
      .inner
      .insert(SourceType::Asset, generation_result.into());
    self
  }

  pub fn inner(&self) -> &HashMap<SourceType, GenerationResult> {
    &self.inner
  }

  pub fn get(&self, source_type: &SourceType) -> Option<&GenerationResult> {
    self.inner.get(source_type)
  }

  pub fn add(&mut self, source_type: SourceType, generation_result: impl Into<GenerationResult>) {
    let result = self.inner.insert(source_type, generation_result.into());
    debug_assert!(result.is_none());
  }

  pub fn set_hash(&mut self) {
    let mut state = Xxh3::default();
    for (source_type, generation_result) in &self.inner {
      source_type.hash(&mut state);
      if let Some(source) = generation_result.ast_or_source.as_source() {
        source.hash(&mut state);
      }
    }
    for (k, v) in &self.chunk_init_fragments {
      k.hash(&mut state);
      v.hash(&mut state);
    }
    self.hash = state.finish();
  }
}

#[derive(Default, Debug)]
pub struct CodeGenerationResults {
  // TODO: This should be a map of ModuleIdentifier to CodeGenerationResult
  pub module_generation_result_map: IdentifierMap<CodeGenerationResult>,
  map: IdentifierMap<RuntimeSpecMap<ModuleIdentifier>>,
}

impl CodeGenerationResults {
  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<&CodeGenerationResult> {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry
          .get(runtime)
          .and_then(|m| {
            // dbg!(self.module_generation_result_map.contains_key(m));
            self.module_generation_result_map.get(m)
          })
          .ok_or_else(|| {
            internal_error!(
              "Failed to code generation result for {module_identifier} with runtime {runtime:?} \n {entry:?}"
            )
          })
      } else {
        if entry.size() > 1 {
          let results = entry.get_values();
          if results.len() != 1 {
            return Err(internal_error!(
              "No unique code generation entry for unspecified runtime for {module_identifier} ",
            ));
          }

          return results
            .first()
            .copied()
            .and_then(|m| self.module_generation_result_map.get(m))
            .ok_or_else(|| internal_error!("Expected value exists"));
        }

        entry
          .get_values()
          .first()
          .copied()
          .and_then(|m| self.module_generation_result_map.get(m))
          .ok_or_else(|| internal_error!("Expected value exists"))
      }
    } else {
      Err(internal_error!(
        "No code generation entry for {} (existing entries: {:?})",
        module_identifier,
        self.map.keys().collect::<Vec<_>>()
      ))
    }
  }

  pub fn add(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: RuntimeSpec,
    result: ModuleIdentifier,
  ) {
    match self.map.entry(module_identifier) {
      Entry::Occupied(mut record) => {
        record.get_mut().set(runtime, result);
      }
      Entry::Vacant(record) => {
        let mut spec_map = RuntimeSpecMap::default();
        spec_map.set(runtime, result);
        record.insert(spec_map);
      }
    };
  }

  pub fn get_runtime_requirements(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> RuntimeGlobals {
    match self.get(module_identifier, runtime) {
      Ok(result) => result.runtime_requirements,
      Err(_) => {
        eprint!("Failed to get runtime requirements for {module_identifier}");
        Default::default()
      }
    }
  }

  #[allow(clippy::unwrap_in_result)]
  pub fn get_hash(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<u64> {
    let code_generation_result = self
      .get(module_identifier, runtime)
      .expect("should have code generation result");

    Some(code_generation_result.hash)
  }
}
