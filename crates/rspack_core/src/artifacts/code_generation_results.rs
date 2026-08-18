use std::{collections::hash_map::Entry, fmt::Debug, sync::Arc};

use dyn_clone::{DynClone, clone_trait_object};
use rspack_cacheable::{
  cacheable, cacheable_dyn,
  with::{AsCacheable, AsInner, AsMap, AsPreset, AsVec},
};
use rspack_collections::IdentifierMap;
use rspack_hash::{HashDigest, HashFunction, HashSalt, RspackHash, RspackHashDigest, RspackHasher};
use rspack_sources::BoxSource;
use rspack_util::{
  atom::Atom,
  ext::{AsAny, IntoAny},
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet};

use crate::{
  ArchivedRenderedInitFragments, ArtifactExt, AssetInfo, BindingCell, ChunkInitFragments,
  ConcatenationScope, ModuleIdentifier, RenderedInitFragments, RuntimeGlobals, RuntimeSpec,
  RuntimeSpecMap, SourceType, incremental::IncrementalPasses,
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
/// Describes a chunk-level default import referenced by a non-concatenated asset module.
#[derive(Clone, Debug)]
pub struct CodeGenerationDataPreservedAssetImport {
  request: String,
  #[cacheable(with=AsPreset)]
  binding: Atom,
}

impl CodeGenerationDataPreservedAssetImport {
  pub fn new(request: String, binding: Atom) -> Self {
    Self { request, binding }
  }

  pub fn request(&self) -> &str {
    &self.request
  }

  pub fn binding(&self) -> &Atom {
    &self.binding
  }
}

impl RspackHash for CodeGenerationDataPreservedAssetImport {
  fn hash(&self, state: &mut RspackHasher) {
    "preserved asset import".hash(state);
    self.request.hash(state);
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

#[cacheable_dyn]
pub trait CodeGenerationDataItem: Debug + DynClone + AsAny + IntoAny + Send + Sync {
  fn update_hash(&self, _hasher: &mut RspackHasher) {}
}

clone_trait_object!(CodeGenerationDataItem);

#[cacheable]
/// Typed [`CodeGenerationData`] entry for the digest of rendered init fragments.
///
/// `CodeGenerationData` is keyed by `TypeId`, so the newtype keeps this digest
/// distinct from other `RspackHashDigest` values stored as code generation data.
#[derive(Clone, Debug)]
pub struct RenderedInitFragmentsDigest(RspackHashDigest);

impl RenderedInitFragmentsDigest {
  pub fn new(inner: RspackHashDigest) -> Self {
    Self(inner)
  }
}

impl RspackHash for RenderedInitFragmentsDigest {
  fn hash(&self, state: &mut RspackHasher) {
    state.write(b"RenderedInitFragmentsDigest");
    self.0.hash(state);
  }
}

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

impl RspackHash for CodeGenerationDataChunkInitFragments {
  fn hash(&self, state: &mut RspackHasher) {
    self.inner.hash(state);
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
impl CodeGenerationDataItem for CodeGenerationDataPreservedAssetImport {
  fn update_hash(&self, hasher: &mut RspackHasher) {
    RspackHash::hash(self, hasher);
  }
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataTopLevelDeclarations {}

#[cacheable_dyn]
impl CodeGenerationDataItem for RenderedInitFragments {
  fn update_hash(&self, hasher: &mut RspackHasher) {
    if !self.is_empty() {
      RspackHash::hash(self, hasher);
    }
  }
}

#[cacheable_dyn]
impl CodeGenerationDataItem for RenderedInitFragmentsDigest {
  fn update_hash(&self, hasher: &mut RspackHasher) {
    RspackHash::hash(self, hasher);
  }
}

#[cacheable_dyn]
impl CodeGenerationDataItem for CodeGenerationDataChunkInitFragments {
  fn update_hash(&self, hasher: &mut RspackHasher) {
    RspackHash::hash(self, hasher);
  }
}

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

  pub fn update_hash(&self, hasher: &mut RspackHasher) {
    for item in &self.inner {
      item.update_hash(hasher);
    }
  }
}

#[cacheable]
#[derive(Debug, Default)]
struct CodeGenerationResultInner {
  #[cacheable(with=AsInner<AsMap<AsCacheable, AsPreset>>)]
  sources: BindingCell<HashMap<SourceType, BoxSource>>,
  /// [definition in webpack](https://github.com/webpack/webpack/blob/4b4ca3bb53f36a5b8fc6bc1bd976ed7af161bd80/lib/Module.js#L75)
  data: CodeGenerationData,
  runtime_requirements: RuntimeGlobals,
  hash: Option<RspackHashDigest>,
}

/// Immutable code generation output constructed by [`CodeGenerationResultBuilder`].
#[cacheable]
#[derive(Debug, Clone)]
pub struct CodeGenerationResult {
  value: Arc<CodeGenerationResultInner>,
}

impl CodeGenerationResult {
  pub fn sources(&self) -> &HashMap<SourceType, BoxSource> {
    &self.value.sources
  }

  pub fn sources_cell(&self) -> &BindingCell<HashMap<SourceType, BoxSource>> {
    &self.value.sources
  }

  pub fn data(&self) -> &CodeGenerationData {
    &self.value.data
  }

  pub fn runtime_requirements(&self) -> &RuntimeGlobals {
    &self.value.runtime_requirements
  }

  pub fn hash(&self) -> Option<&RspackHashDigest> {
    self.value.hash.as_ref()
  }

  pub fn get(&self, source_type: &SourceType) -> Option<&BoxSource> {
    self.value.sources.get(source_type)
  }

  fn has_same_value(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.value, &other.value)
  }
}

/// Mutable state used to construct a [`CodeGenerationResult`].
#[derive(Debug, Default)]
pub struct CodeGenerationResultBuilder {
  value: CodeGenerationResultInner,
}

impl CodeGenerationResultBuilder {
  pub fn sources(&self) -> &HashMap<SourceType, BoxSource> {
    &self.value.sources
  }

  pub fn data(&self) -> &CodeGenerationData {
    &self.value.data
  }

  pub fn data_mut(&mut self) -> &mut CodeGenerationData {
    &mut self.value.data
  }

  pub fn runtime_requirements(&self) -> &RuntimeGlobals {
    &self.value.runtime_requirements
  }

  pub fn runtime_requirements_mut(&mut self) -> &mut RuntimeGlobals {
    &mut self.value.runtime_requirements
  }

  pub fn get(&self, source_type: &SourceType) -> Option<&BoxSource> {
    self.value.sources.get(source_type)
  }

  pub fn add(&mut self, source_type: SourceType, generation_result: BoxSource) {
    let result = self.value.sources.insert(source_type, generation_result);
    debug_assert!(result.is_none());
  }

  pub fn set_hash(
    &mut self,
    hash_function: &HashFunction,
    hash_digest: &HashDigest,
    hash_salt: &HashSalt,
    concatenated_module_hash: Option<&RspackHashDigest>,
  ) {
    let mut hasher = RspackHasher::with_salt(hash_function, hash_salt);
    if let Some(concatenated_module_hash) = concatenated_module_hash {
      concatenated_module_hash.hash(&mut hasher);
      for source_type in self.value.sources.as_ref().keys() {
        source_type.hash(&mut hasher);
      }
    } else {
      for (source_type, source) in self.value.sources.as_ref() {
        source_type.hash(&mut hasher);
        std::hash::Hash::hash(source, &mut hasher);
      }
    }
    self.value.data.update_hash(&mut hasher);
    self.value.runtime_requirements.hash(&mut hasher);
    self.value.hash = Some(hasher.digest(hash_digest));
  }

  pub fn build(self) -> CodeGenerationResult {
    CodeGenerationResult {
      value: Arc::new(self.value),
    }
  }
}

#[derive(Debug, Default, Clone)]
pub struct CodeGenerationResults {
  map: IdentifierMap<RuntimeSpecMap<BindingCell<CodeGenerationResult>>>,
}

impl ArtifactExt for CodeGenerationResults {
  const PASS: IncrementalPasses = IncrementalPasses::MODULES_CODEGEN;
}

impl CodeGenerationResults {
  pub fn is_empty(&self) -> bool {
    self.map.is_empty()
  }

  pub fn insert(
    &mut self,
    module_identifier: ModuleIdentifier,
    codegen_res: CodeGenerationResult,
    runtimes: impl IntoIterator<Item = RuntimeSpec>,
  ) {
    for runtime in runtimes {
      self.add(
        module_identifier,
        runtime,
        BindingCell::from(codegen_res.clone()),
      );
    }
  }

  pub fn remove(&mut self, module_identifier: &ModuleIdentifier) -> Option<()> {
    self.map.remove(module_identifier).map(|_| ())
  }

  pub fn get(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> &BindingCell<CodeGenerationResult> {
    if let Some(entry) = self.map.get(module_identifier) {
      if let Some(runtime) = runtime {
        entry.get(runtime).unwrap_or_else(|| {
          panic!(
            "Failed to code generation result for {module_identifier} with runtime {runtime:?} \n {entry:?}"
          )
        })
      } else {
        let mut values = entry.values();
        let result = values
          .next()
          .unwrap_or_else(|| panic!("Expected value exists"));
        if values.any(|other| !result.has_same_value(other)) {
          panic!(
            "No unique code generation entry for unspecified runtime for {module_identifier} ",
          );
        }
        result
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
      .and_then(|entry| entry.values().next())
      .unwrap_or_else(|| panic!("No code generation result for {module_identifier}"))
  }

  fn add(
    &mut self,
    module_identifier: ModuleIdentifier,
    runtime: RuntimeSpec,
    result: BindingCell<CodeGenerationResult>,
  ) {
    match self.map.entry(module_identifier) {
      Entry::Occupied(mut record) => {
        record.get_mut().set(runtime, result);
      }
      Entry::Vacant(record) => {
        let mut spec_map = RuntimeSpecMap::new();
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
    *self.get(module_identifier, runtime).runtime_requirements()
  }

  pub fn get_hash(
    &self,
    module_identifier: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Option<&RspackHashDigest> {
    let code_generation_result = self.get(module_identifier, runtime);

    code_generation_result.hash()
  }

  pub fn inner(&self) -> &IdentifierMap<RuntimeSpecMap<BindingCell<CodeGenerationResult>>> {
    &self.map
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
