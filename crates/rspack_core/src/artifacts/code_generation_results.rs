use std::{
  collections::hash_map::Entry,
  fmt::Debug,
  sync::atomic::{AtomicU32, Ordering},
};

use dyn_clone::{DynClone, clone_trait_object};
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsInner, AsMap, AsOption, AsPreset, AsVec, Unsupported},
};
use rspack_collections::IdentifierMap;
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest, RspackHasher};
use rspack_sources::BoxSource;
use rspack_util::{
  atom::Atom,
  ext::{AsAny, IntoAny},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet};
use serde::Serialize;

use crate::{
  ArtifactExt, AssetInfo, BindingCell, ChunkInitFragments, ConcatenationScope, ModuleIdentifier,
  RuntimeGlobals, RuntimeSpec, RuntimeSpecMap, SourceType, incremental::IncrementalPasses,
};

#[cacheable]
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

// For performance, mark the js modules containing AUTO_PUBLIC_PATH_PLACEHOLDER
#[cacheable]
#[derive(Clone, Debug)]
pub struct CodeGenerationPublicPathAutoReplace(pub bool);

#[cacheable]
#[derive(Clone, Debug)]
pub struct URLStaticMode;

#[cacheable]
#[derive(Clone, Debug)]
pub struct CodeGenerationDataFilename {
  filename: String,
  public_path: String,
}

impl CodeGenerationDataFilename {
  pub fn new(filename: String, public_path: String) -> Self {
    Self {
      filename,
      public_path,
    }
  }

  pub fn filename(&self) -> &str {
    &self.filename
  }

  pub fn public_path(&self) -> &str {
    &self.public_path
  }
}

#[cacheable]
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

#[cacheable]
#[derive(Clone, Debug)]
pub struct CodeGenerationDataTopLevelDeclarations {
  #[cacheable(with=AsVec<AsPreset>)]
  inner: FxHashSet<Atom>,
}

impl CodeGenerationDataTopLevelDeclarations {
  pub fn new(inner: FxHashSet<Atom>) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &FxHashSet<Atom> {
    &self.inner
  }
}

#[cacheable]
#[derive(Clone, Debug)]
pub struct CodeGenerationExportsFinalNames {
  inner: HashMap<String, String>,
}

impl CodeGenerationExportsFinalNames {
  pub fn new(inner: HashMap<String, String>) -> Self {
    Self { inner }
  }

  pub fn inner(&self) -> &HashMap<String, String> {
    &self.inner
  }
}

#[cacheable_dyn]
pub trait CodeGenerationDataItem: Debug + DynClone + AsAny + IntoAny + Send + Sync {}

clone_trait_object!(CodeGenerationDataItem);

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CodeGenerationDataChunkInitFragments {
  inner: ChunkInitFragments,
}

impl CodeGenerationDataChunkInitFragments {
  pub fn inner(&self) -> &ChunkInitFragments {
    &self.inner
  }

  pub fn inner_mut(&mut self) -> &mut ChunkInitFragments {
    &mut self.inner
  }
}

impl From<ChunkInitFragments> for CodeGenerationDataChunkInitFragments {
  fn from(inner: ChunkInitFragments) -> Self {
    Self { inner }
  }
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataUrl {}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationPublicPathAutoReplace {}

#[cacheable_dyn]
impl CodeGenerationDataItem for URLStaticMode {}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataFilename {}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataAssetInfo {}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataTopLevelDeclarations {}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationExportsFinalNames {}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataChunkInitFragments {}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CodeGenerationData {
  inner: Vec<Box<dyn CodeGenerationDataItem>>,
}

impl CodeGenerationData {
  pub fn insert<T: CodeGenerationDataItem + 'static>(&mut self, item: T) -> Option<T> {
    if let Some(index) = self
      .inner
      .iter()
      .position(|item| item.as_ref().as_any().is::<T>())
    {
      let old = std::mem::replace(&mut self.inner[index], Box::new(item));
      old.into_any().downcast::<T>().ok().map(|item| *item)
    } else {
      self.inner.push(Box::new(item));
      None
    }
  }

  pub fn get<T: CodeGenerationDataItem + 'static>(&self) -> Option<&T> {
    self
      .inner
      .iter()
      .find_map(|item| item.as_ref().as_any().downcast_ref::<T>())
  }

  pub fn get_mut<T: CodeGenerationDataItem + 'static>(&mut self) -> Option<&mut T> {
    self
      .inner
      .iter_mut()
      .find_map(|item| item.as_mut().as_any_mut().downcast_mut::<T>())
  }

  pub fn contains<T: CodeGenerationDataItem + 'static>(&self) -> bool {
    self.get::<T>().is_some()
  }

  pub fn len(&self) -> usize {
    self.inner.len()
  }

  pub fn is_empty(&self) -> bool {
    self.inner.is_empty()
  }
}

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResult {
  #[cacheable(with=AsInner<AsMap<AsCacheable, AsPreset>>)]
  pub inner: BindingCell<HashMap<SourceType, BoxSource>>,
  /// [definition in webpack](https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L75)
  pub data: CodeGenerationData,
  pub chunk_init_fragments: ChunkInitFragments,
  pub runtime_requirements: RuntimeGlobals,
  pub hash: Option<RspackHashDigest>,
  pub id: CodeGenResultId,
  #[cacheable(with=AsOption<Unsupported>)]
  pub concatenation_scope: Option<ConcatenationScope>,
}

impl CodeGenerationResult {
  pub fn with_javascript(mut self, generation_result: BoxSource) -> Self {
    self.inner.insert(SourceType::JavaScript, generation_result);
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
    let mut hasher = RspackHasher::with_salt(hash_function, hash_salt);
    for (source_type, source) in self.inner.as_ref() {
      source_type.hash(&mut hasher);
      std::hash::Hash::hash(source, &mut hasher);
    }
    self.chunk_init_fragments.hash(&mut hasher);
    self.runtime_requirements.hash(&mut hasher);
    self.hash = Some(hasher.digest(hash_digest));
  }

  /// Concatenated modules already encode the generated module bodies into
  /// `ConcatenatedModule::get_runtime_hash`, so we can reuse that digest here
  /// and only mix in codegen-specific metadata instead of hashing the large
  /// concatenated source again.
  pub fn set_hash_for_concatenated_module(
    &mut self,
    runtime_hash: &RspackHashDigest,
    hash_function: &HashFunction,
    hash_digest: &HashDigest,
    hash_salt: &HashSalt,
  ) {
    let mut hasher = RspackHasher::with_salt(hash_function, hash_salt);
    runtime_hash.hash(&mut hasher);
    for source_type in self.inner.as_ref().keys() {
      source_type.hash(&mut hasher);
    }
    self.chunk_init_fragments.hash(&mut hasher);
    self.runtime_requirements.hash(&mut hasher);
    self.hash = Some(hasher.digest(hash_digest));
  }
}

#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct CodeGenResultId(u32);

impl Default for CodeGenResultId {
  fn default() -> Self {
    Self(CODE_GEN_RESULT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
  }
}

pub static CODE_GEN_RESULT_ID: AtomicU32 = AtomicU32::new(0);

#[cacheable]
#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResults {
  #[cacheable(with=AsMap<AsCacheable, AsInner<AsCacheable>>)]
  module_generation_result_map: HashMap<CodeGenResultId, BindingCell<CodeGenerationResult>>,
  map: IdentifierMap<RuntimeSpecMap<CodeGenResultId>>,
}

impl ArtifactExt for CodeGenerationResults {
  const PASS: IncrementalPasses = IncrementalPasses::MODULES_CODEGEN;
}

impl CodeGenerationResults {
  pub fn is_empty(&self) -> bool {
    self.module_generation_result_map.is_empty() && self.map.is_empty()
  }

  pub fn insert(
    &mut self,
    module_identifier: ModuleIdentifier,
    codegen_res: CodeGenerationResult,
    runtimes: impl IntoIterator<Item = RuntimeSpec>,
  ) {
    let codegen_res_id = codegen_res.id;
    self
      .module_generation_result_map
      .insert(codegen_res_id, BindingCell::from(codegen_res));
    for runtime in runtimes {
      self.add(module_identifier, runtime, codegen_res_id);
    }
  }

  pub fn remove(&mut self, module_identifier: &ModuleIdentifier) -> Option<()> {
    let runtime_map = self.map.remove(module_identifier)?;
    for result in runtime_map.values() {
      self.module_generation_result_map.remove(result)?;
    }
    Some(())
  }

  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> &BindingCell<CodeGenerationResult> {
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
          let mut values = entry.values();
          let results: FxHashSet<_> = entry.values().collect();
          if results.len() > 1 {
            panic!(
              "No unique code generation entry for unspecified runtime for {module_identifier} ",
            );
          }

          return values
            .next()
            .and_then(|m| self.module_generation_result_map.get(m))
            .unwrap_or_else(|| panic!("Expected value exists"));
        }

        entry
          .values()
          .next()
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

  /**
   * This API should be used carefully, it will return one of the code generation result,
   * make sure the module has the same code generation result for all runtimes.
   */
  pub fn get_one(
    &self,
    module_identifier: &ModuleIdentifier,
  ) -> &BindingCell<CodeGenerationResult> {
    self
      .map
      .get(module_identifier)
      .and_then(|entry| {
        entry
          .values()
          .next()
          .and_then(|m| self.module_generation_result_map.get(m))
      })
      .unwrap_or_else(|| panic!("No code generation result for {module_identifier}"))
  }

  pub fn get_mut(
    &mut self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> &mut BindingCell<CodeGenerationResult> {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry
          .get(runtime)
          .and_then(|m| {
            self.module_generation_result_map.get_mut(m)
          })
          .unwrap_or_else(|| {
            panic!(
              "Failed to code generation result for {module_identifier} with runtime {runtime:?} \n {entry:?}"
            )
          })
      } else {
        if entry.size() > 1 {
          let mut values = entry.values();
          let results: FxHashSet<_> = entry.values().collect();
          if results.len() > 1 {
            panic!(
              "No unique code generation entry for unspecified runtime for {module_identifier} ",
            );
          }

          return values
            .next()
            .and_then(|m| self.module_generation_result_map.get_mut(m))
            .unwrap_or_else(|| panic!("Expected value exists"));
        }

        entry
          .values()
          .next()
          .and_then(|m| self.module_generation_result_map.get_mut(m))
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

  pub fn get_hash(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<&RspackHashDigest> {
    let code_generation_result = self.get(module_identifier, runtime);

    code_generation_result.hash.as_ref()
  }

  pub fn inner(
    &self,
  ) -> (
    &IdentifierMap<RuntimeSpecMap<CodeGenResultId>>,
    &HashMap<CodeGenResultId, BindingCell<CodeGenerationResult>>,
  ) {
    (&self.map, &self.module_generation_result_map)
  }

  pub(crate) fn sync_code_generation_result_id(&self) {
    if let Some(next) = self
      .module_generation_result_map
      .keys()
      .map(|id| id.0)
      .max()
      .and_then(|id| id.checked_add(1))
    {
      let mut current = CODE_GEN_RESULT_ID.load(Ordering::Relaxed);
      while current < next {
        match CODE_GEN_RESULT_ID.compare_exchange_weak(
          current,
          next,
          Ordering::Relaxed,
          Ordering::Relaxed,
        ) {
          Ok(_) => break,
          Err(value) => current = value,
        }
      }
    }
  }
}

#[derive(Debug)]
pub struct CodeGenerationJob {
  pub module: ModuleIdentifier,
  pub hash: RspackHashDigest,
  pub runtime: RuntimeSpec,
  pub runtimes: Vec<RuntimeSpec>,
  pub scope: Option<ConcatenationScope>,
}
