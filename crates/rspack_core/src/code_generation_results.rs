use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicU32;

use anymap::CloneAny;
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest};
use rspack_identifier::IdentifierMap;
use rspack_sources::BoxSource;
use rustc_hash::{FxHashMap as HashMap, FxHashSet};
use serde::Serialize;

use crate::{
  AssetInfo, ChunkInitFragments, ConcatenationScope, ModuleIdentifier, PublicPath, RuntimeGlobals,
  RuntimeMode, RuntimeSpec, RuntimeSpecMap, SourceType,
};

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
  filename: String,
  public_path: PublicPath,
}

impl CodeGenerationDataFilename {
  pub fn new(filename: String, public_path: PublicPath) -> Self {
    Self {
      filename,
      public_path,
    }
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }

  pub fn public_path(&self) -> &PublicPath {
    &self.public_path
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

#[derive(Clone, Debug)]
pub struct CodeGenerationDataTopLevelDeclarations {
  inner: FxHashSet<String>,
}

impl CodeGenerationDataTopLevelDeclarations {
  pub fn new(inner: FxHashSet<String>) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &FxHashSet<String> {
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
  pub inner: HashMap<SourceType, BoxSource>,
  /// [definition in webpack](https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L75)
  pub data: CodeGenerationData,
  pub chunk_init_fragments: ChunkInitFragments,
  pub runtime_requirements: RuntimeGlobals,
  pub hash: Option<RspackHashDigest>,
  pub id: CodeGenResultId,
  pub concatenation_scope: Option<ConcatenationScope>,
}

impl CodeGenerationResult {
  pub fn with_javascript(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::JavaScript, generation_result);
    self
  }

  pub fn with_css(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::Css, generation_result);
    self
  }

  pub fn with_asset(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::Asset, generation_result);
    self
  }

  pub fn inner(&self) -> &HashMap<SourceType, BoxSource> {
    &self.inner
  }

  pub fn get(&self, source_type: &SourceType) -> Option<&BoxSource> {
    self.inner.get(source_type)
  }

  pub fn add(&mut self, source_type: SourceType, generation_result: BoxSource) {
    let result = self.inner.insert(source_type, generation_result);
    debug_assert!(result.is_none());
  }

  pub fn set_hash(
    &mut self,
    hash_function: &HashFunction,
    hash_digest: &HashDigest,
    hash_salt: &HashSalt,
  ) {
    let mut hasher = RspackHash::with_salt(hash_function, hash_salt);
    for (source_type, source) in &self.inner {
      source_type.hash(&mut hasher);
      source.hash(&mut hasher);
    }
    self.chunk_init_fragments.hash(&mut hasher);
    self.hash = Some(hasher.digest(hash_digest));
  }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct CodeGenResultId(u32);

impl Default for CodeGenResultId {
  fn default() -> Self {
    Self(CODE_GEN_RESULT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
  }
}

pub static CODE_GEN_RESULT_ID: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResults {
  pub module_generation_result_map: HashMap<CodeGenResultId, CodeGenerationResult>,
  pub map: IdentifierMap<RuntimeSpecMap<CodeGenResultId>>,
}

impl CodeGenerationResults {
  pub fn get_one(&self, module_identifier: &ModuleIdentifier) -> Option<&CodeGenerationResult> {
    self
      .map
      .get(module_identifier)
      .and_then(|spec| match spec.mode {
        RuntimeMode::Empty => None,
        RuntimeMode::SingleEntry => spec
          .single_value
          .and_then(|result_id| self.module_generation_result_map.get(&result_id)),
        RuntimeMode::Map => spec
          .map
          .values()
          .next()
          .and_then(|result_id| self.module_generation_result_map.get(result_id)),
      })
  }

  pub fn clear_entry(
    &mut self,
    module_identifier: &ModuleIdentifier,
  ) -> Option<(ModuleIdentifier, RuntimeSpecMap<CodeGenResultId>)> {
    self.map.remove_entry(module_identifier)
  }

  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> &CodeGenerationResult {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry
          .get(runtime)
          .and_then(|m| {
            self.module_generation_result_map.get(m)
          })
          .unwrap_or_else(|| {
            panic!(
              "Failed to code generation result for {module_identifier} with runtime {runtime:?} \n {entry:?}"
            )
          })
      } else {
        if entry.size() > 1 {
          let results = entry.get_values();
          if results.len() != 1 {
            panic!(
              "No unique code generation entry for unspecified runtime for {module_identifier} ",
            );
          }

          return results
            .first()
            .copied()
            .and_then(|m| self.module_generation_result_map.get(m))
            .unwrap_or_else(|| panic!("Expected value exists"));
        }

        entry
          .get_values()
          .first()
          .copied()
          .and_then(|m| self.module_generation_result_map.get(m))
          .unwrap_or_else(|| panic!("Expected value exists"))
      }
    } else {
      panic!(
        "No code generation entry for {} (existing entries: {:?})",
        module_identifier,
        self.map.keys().collect::<Vec<_>>()
      )
    }
  }

  pub fn add(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: RuntimeSpec,
    result: CodeGenResultId,
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
    self.get(module_identifier, runtime).runtime_requirements
  }

  #[allow(clippy::unwrap_in_result)]
  pub fn get_hash(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<&RspackHashDigest> {
    let code_generation_result = self.get(module_identifier, runtime);

    code_generation_result.hash.as_ref()
  }
}
