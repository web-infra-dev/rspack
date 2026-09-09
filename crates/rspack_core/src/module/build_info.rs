use std::sync::{
  Arc, RwLock, RwLockReadGuard,
  atomic::{AtomicBool, AtomicU8, Ordering},
};

use json::JsonValue;
use rspack_cacheable::{
  ContextGuard, cacheable,
  with::{As, AsConverter, AsOption, AsPreset, AsVec},
};
use rspack_hash::RspackHashDigest;
use rspack_intern::{Atom, AtomSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  AssetBuildInfo, BindingCell, CollectedTypeScriptInfo, CompilationAsset, CssBuildInfo,
  DeferredPureCheck, DependencyId, ExportsArgument, ImportPhase, IsolatedDts, ModuleArgument,
  OptimizationBailoutItem, RscMeta, Snapshot,
};

/// Build information with field-scoped mutation through shared references.
///
/// Scalar flags use atomics; collection updates take only that field's lock.
/// Most collection readers own immutable snapshots. Updating a collection copies
/// it only while a reader retains a snapshot. Emitted assets keep their binding
/// identity; assets and build bailouts use scoped read guards. Mutation is still
/// restricted to the compilation's permitted phases.
/// Module-build cache entries share this information with the module.
#[cacheable(with=BuildInfoCache)]
#[derive(Debug)]
pub struct BuildInfo {
  cacheable: AtomicBool,
  hash: RwLock<Option<RspackHashDigest>>,
  strict: AtomicBool,
  module_argument: AtomicU8,
  exports_argument: AtomicU8,
  dependencies: RwLock<Arc<crate::LoaderDependencies>>,
  snapshot: RwLock<Option<Snapshot>>,
  value_dependencies: RwLock<Arc<HashMap<String, String>>>,
  esm_named_exports: RwLock<Arc<HashSet<Atom>>>,
  // Keep Vec capacity while parsing appends exports; readers retain COW snapshots.
  #[allow(clippy::rc_buffer)]
  all_star_exports: RwLock<Arc<Vec<DependencyId>>>,
  need_create_require: AtomicBool,
  json_data: RwLock<Option<Arc<JsonValue>>>,
  asset: RwLock<Option<Arc<AssetBuildInfo>>>,
  css: RwLock<Option<Arc<CssBuildInfo>>>,
  side_effects_free: RwLock<Option<Arc<AtomSet>>>,
  top_level_declarations: RwLock<Option<Arc<AtomSet>>>,
  /// Bailouts produced during module builds, before compilation-specific optimizations.
  optimization_bailouts: RwLock<Vec<OptimizationBailoutItem>>,
  module_concatenation_bailout: RwLock<Option<String>>,
  assets: RwLock<BindingCell<HashMap<String, CompilationAsset>>>,
  module: AtomicBool,
  inline_exports: AtomicBool,
  collected_typescript_info: RwLock<Option<Arc<CollectedTypeScriptInfo>>>,
  rsc: RwLock<Option<Arc<RscMeta>>>,
  import_phase: AtomicU8,
  isolated_dts: RwLock<Option<Arc<IsolatedDts>>>,
  extras: RwLock<Arc<serde_json::Map<String, serde_json::Value>>>,
  deferred_pure_checks: RwLock<Arc<HashSet<DeferredPureCheck>>>,
}

type BuildInfoCache = As<BuildInfoSnapshot>;

impl Default for BuildInfo {
  fn default() -> Self {
    BuildInfoSnapshot::default().into()
  }
}

impl BuildInfo {
  pub fn cacheable(&self) -> bool {
    self.cacheable.load(Ordering::Relaxed)
  }

  pub fn set_cacheable(&self, value: bool) {
    self.cacheable.store(value, Ordering::Relaxed);
  }

  pub fn hash(&self) -> Option<RspackHashDigest> {
    self.hash.read().expect("should read hash").clone()
  }

  pub fn set_hash(&self, value: Option<RspackHashDigest>) {
    *self.hash.write().expect("should write hash") = value;
  }

  pub fn strict(&self) -> bool {
    self.strict.load(Ordering::Relaxed)
  }

  pub fn set_strict(&self, value: bool) {
    self.strict.store(value, Ordering::Relaxed);
  }

  pub fn module_argument(&self) -> ModuleArgument {
    match self.module_argument.load(Ordering::Relaxed) {
      1 => ModuleArgument::RspackModule,
      _ => ModuleArgument::Module,
    }
  }

  pub fn set_module_argument(&self, value: ModuleArgument) {
    self.module_argument.store(
      match value {
        ModuleArgument::Module => 0,
        ModuleArgument::RspackModule => 1,
      },
      Ordering::Relaxed,
    );
  }

  pub fn exports_argument(&self) -> ExportsArgument {
    match self.exports_argument.load(Ordering::Relaxed) {
      1 => ExportsArgument::RspackExports,
      _ => ExportsArgument::Exports,
    }
  }

  pub fn set_exports_argument(&self, value: ExportsArgument) {
    self.exports_argument.store(
      match value {
        ExportsArgument::Exports => 0,
        ExportsArgument::RspackExports => 1,
      },
      Ordering::Relaxed,
    );
  }

  pub fn dependencies(&self) -> Arc<crate::LoaderDependencies> {
    self
      .dependencies
      .read()
      .expect("should read dependencies")
      .clone()
  }

  pub fn set_dependencies(&self, value: crate::LoaderDependencies) {
    *self
      .dependencies
      .write()
      .expect("should write dependencies") = Arc::new(value);
  }

  pub fn update_dependencies<R>(
    &self,
    update: impl FnOnce(&mut crate::LoaderDependencies) -> R,
  ) -> R {
    let mut value = self
      .dependencies
      .write()
      .expect("should write dependencies");
    update(Arc::make_mut(&mut value))
  }

  pub fn snapshot(&self) -> Option<Snapshot> {
    self.snapshot.read().expect("should read snapshot").clone()
  }

  pub fn set_snapshot(&self, value: Option<Snapshot>) {
    *self.snapshot.write().expect("should write snapshot") = value;
  }

  pub fn value_dependencies(&self) -> Arc<HashMap<String, String>> {
    self
      .value_dependencies
      .read()
      .expect("should read value_dependencies")
      .clone()
  }

  pub fn set_value_dependencies(&self, value: HashMap<String, String>) {
    *self
      .value_dependencies
      .write()
      .expect("should write value_dependencies") = Arc::new(value);
  }

  pub fn update_value_dependencies<R>(
    &self,
    update: impl FnOnce(&mut HashMap<String, String>) -> R,
  ) -> R {
    let mut value = self
      .value_dependencies
      .write()
      .expect("should write value_dependencies");
    update(Arc::make_mut(&mut value))
  }

  pub fn esm_named_exports(&self) -> Arc<HashSet<Atom>> {
    self
      .esm_named_exports
      .read()
      .expect("should read esm_named_exports")
      .clone()
  }

  pub fn set_esm_named_exports(&self, value: HashSet<Atom>) {
    *self
      .esm_named_exports
      .write()
      .expect("should write esm_named_exports") = Arc::new(value);
  }

  pub fn update_esm_named_exports<R>(&self, update: impl FnOnce(&mut HashSet<Atom>) -> R) -> R {
    let mut value = self
      .esm_named_exports
      .write()
      .expect("should write esm_named_exports");
    update(Arc::make_mut(&mut value))
  }

  pub fn all_star_exports(&self) -> Arc<Vec<DependencyId>> {
    self
      .all_star_exports
      .read()
      .expect("should read all_star_exports")
      .clone()
  }

  pub fn set_all_star_exports(&self, value: Vec<DependencyId>) {
    *self
      .all_star_exports
      .write()
      .expect("should write all_star_exports") = Arc::new(value);
  }

  pub fn update_all_star_exports<R>(&self, update: impl FnOnce(&mut Vec<DependencyId>) -> R) -> R {
    let mut value = self
      .all_star_exports
      .write()
      .expect("should write all_star_exports");
    update(Arc::make_mut(&mut value))
  }

  pub fn need_create_require(&self) -> bool {
    self.need_create_require.load(Ordering::Relaxed)
  }

  pub fn set_need_create_require(&self, value: bool) {
    self.need_create_require.store(value, Ordering::Relaxed);
  }

  pub fn json_data(&self) -> Option<Arc<JsonValue>> {
    self
      .json_data
      .read()
      .expect("should read json_data")
      .clone()
  }

  pub fn set_json_data(&self, value: Option<JsonValue>) {
    *self.json_data.write().expect("should write json_data") = value.map(Arc::new);
  }

  pub fn asset(&self) -> Option<Arc<AssetBuildInfo>> {
    self.asset.read().expect("should read asset").clone()
  }

  pub fn set_asset(&self, value: Option<Box<AssetBuildInfo>>) {
    *self.asset.write().expect("should write asset") = value.map(Arc::from);
  }

  pub fn update_asset<R>(&self, update: impl FnOnce(&mut AssetBuildInfo) -> R) -> R {
    let mut value = self.asset.write().expect("should write asset");
    update(Arc::make_mut(
      value
        .as_mut()
        .expect("asset build info should exist for asset module"),
    ))
  }

  pub fn css(&self) -> Option<Arc<CssBuildInfo>> {
    self.css.read().expect("should read css").clone()
  }

  pub fn set_css(&self, value: Option<Box<CssBuildInfo>>) {
    *self.css.write().expect("should write css") = value.map(Arc::from);
  }

  pub fn update_css<R>(&self, update: impl FnOnce(&mut CssBuildInfo) -> R) -> R {
    let mut value = self.css.write().expect("should write css");
    update(Arc::make_mut(
      value.get_or_insert_with(|| Arc::new(Default::default())),
    ))
  }

  pub fn side_effects_free(&self) -> Option<Arc<AtomSet>> {
    self
      .side_effects_free
      .read()
      .expect("should read side_effects_free")
      .clone()
  }

  pub fn set_side_effects_free(&self, value: Option<AtomSet>) {
    *self
      .side_effects_free
      .write()
      .expect("should write side_effects_free") = value.map(Arc::new);
  }

  pub fn update_side_effects_free<R>(&self, update: impl FnOnce(&mut AtomSet) -> R) -> R {
    let mut value = self
      .side_effects_free
      .write()
      .expect("should write side_effects_free");
    update(Arc::make_mut(
      value.get_or_insert_with(|| Arc::new(Default::default())),
    ))
  }

  pub fn top_level_declarations(&self) -> Option<Arc<AtomSet>> {
    self
      .top_level_declarations
      .read()
      .expect("should read top_level_declarations")
      .clone()
  }

  pub fn set_top_level_declarations(&self, value: Option<AtomSet>) {
    *self
      .top_level_declarations
      .write()
      .expect("should write top_level_declarations") = value.map(Arc::new);
  }

  pub fn update_top_level_declarations<R>(&self, update: impl FnOnce(&mut AtomSet) -> R) -> R {
    let mut value = self
      .top_level_declarations
      .write()
      .expect("should write top_level_declarations");
    update(Arc::make_mut(
      value.get_or_insert_with(|| Arc::new(Default::default())),
    ))
  }

  /// Drop this field's read guard before mutation, hooks, or async work.
  pub fn optimization_bailouts(&self) -> RwLockReadGuard<'_, Vec<OptimizationBailoutItem>> {
    self
      .optimization_bailouts
      .read()
      .expect("should read optimization_bailouts")
  }

  pub fn update_optimization_bailouts<R>(
    &self,
    update: impl FnOnce(&mut Vec<OptimizationBailoutItem>) -> R,
  ) -> R {
    let mut value = self
      .optimization_bailouts
      .write()
      .expect("should write optimization_bailouts");
    update(&mut value)
  }

  pub fn module_concatenation_bailout(&self) -> Option<String> {
    self
      .module_concatenation_bailout
      .read()
      .expect("should read module_concatenation_bailout")
      .clone()
  }

  pub fn set_module_concatenation_bailout(&self, value: Option<String>) {
    *self
      .module_concatenation_bailout
      .write()
      .expect("should write module_concatenation_bailout") = value;
  }

  /// Drop this field's read guard before mutation, hooks, or async work.
  pub fn assets(&self) -> RwLockReadGuard<'_, BindingCell<HashMap<String, CompilationAsset>>> {
    self.assets.read().expect("should read assets")
  }

  pub fn update_assets<R>(
    &self,
    update: impl FnOnce(&mut BindingCell<HashMap<String, CompilationAsset>>) -> R,
  ) -> R {
    update(&mut self.assets.write().expect("should write assets"))
  }

  pub fn module(&self) -> bool {
    self.module.load(Ordering::Relaxed)
  }

  pub fn set_module(&self, value: bool) {
    self.module.store(value, Ordering::Relaxed);
  }

  pub fn inline_exports(&self) -> bool {
    self.inline_exports.load(Ordering::Relaxed)
  }

  pub fn set_inline_exports(&self, value: bool) {
    self.inline_exports.store(value, Ordering::Relaxed);
  }

  pub fn collected_typescript_info(&self) -> Option<Arc<CollectedTypeScriptInfo>> {
    self
      .collected_typescript_info
      .read()
      .expect("should read collected_typescript_info")
      .clone()
  }

  pub fn set_collected_typescript_info(&self, value: Option<CollectedTypeScriptInfo>) {
    *self
      .collected_typescript_info
      .write()
      .expect("should write collected_typescript_info") = value.map(Arc::new);
  }

  pub fn rsc(&self) -> Option<Arc<RscMeta>> {
    self.rsc.read().expect("should read rsc").clone()
  }

  pub fn set_rsc(&self, value: Option<RscMeta>) {
    *self.rsc.write().expect("should write rsc") = value.map(Arc::new);
  }

  pub fn update_rsc<R>(&self, update: impl FnOnce(&mut RscMeta) -> R) -> Option<R> {
    self
      .rsc
      .write()
      .expect("should write rsc")
      .as_mut()
      .map(|value| update(Arc::make_mut(value)))
  }

  pub fn import_phase(&self) -> ImportPhase {
    match self.import_phase.load(Ordering::Relaxed) {
      1 => ImportPhase::Source,
      2 => ImportPhase::Defer,
      _ => ImportPhase::Evaluation,
    }
  }

  pub fn set_import_phase(&self, value: ImportPhase) {
    self.import_phase.store(
      match value {
        ImportPhase::Evaluation => 0,
        ImportPhase::Source => 1,
        ImportPhase::Defer => 2,
      },
      Ordering::Relaxed,
    );
  }

  pub fn isolated_dts(&self) -> Option<Arc<IsolatedDts>> {
    self
      .isolated_dts
      .read()
      .expect("should read isolated_dts")
      .clone()
  }

  pub fn set_isolated_dts(&self, value: Option<Box<IsolatedDts>>) {
    *self
      .isolated_dts
      .write()
      .expect("should write isolated_dts") = value.map(Arc::from);
  }

  pub fn extras(&self) -> Arc<serde_json::Map<String, serde_json::Value>> {
    self.extras.read().expect("should read extras").clone()
  }

  pub fn set_extras(&self, value: serde_json::Map<String, serde_json::Value>) {
    *self.extras.write().expect("should write extras") = Arc::new(value);
  }

  pub fn update_extras<R>(
    &self,
    update: impl FnOnce(&mut serde_json::Map<String, serde_json::Value>) -> R,
  ) -> R {
    let mut value = self.extras.write().expect("should write extras");
    update(Arc::make_mut(&mut value))
  }

  pub fn deferred_pure_checks(&self) -> Arc<HashSet<DeferredPureCheck>> {
    self
      .deferred_pure_checks
      .read()
      .expect("should read deferred_pure_checks")
      .clone()
  }

  pub fn set_deferred_pure_checks(&self, value: HashSet<DeferredPureCheck>) {
    *self
      .deferred_pure_checks
      .write()
      .expect("should write deferred_pure_checks") = Arc::new(value);
  }

  pub fn update_deferred_pure_checks<R>(
    &self,
    update: impl FnOnce(&mut HashSet<DeferredPureCheck>) -> R,
  ) -> R {
    let mut value = self
      .deferred_pure_checks
      .write()
      .expect("should write deferred_pure_checks");
    update(Arc::make_mut(&mut value))
  }

  fn snapshot_data(&self) -> BuildInfoSnapshot {
    BuildInfoSnapshot {
      cacheable: self.cacheable(),
      hash: self.hash(),
      strict: self.strict(),
      module_argument: self.module_argument(),
      exports_argument: self.exports_argument(),
      dependencies: self.dependencies().as_ref().clone(),
      snapshot: self.snapshot(),
      value_dependencies: self.value_dependencies().as_ref().clone(),
      esm_named_exports: self.esm_named_exports().as_ref().clone(),
      all_star_exports: self.all_star_exports().as_ref().clone(),
      need_create_require: self.need_create_require(),
      json_data: self.json_data().as_deref().cloned(),
      asset: self.asset().as_deref().cloned().map(Box::new),
      css: self.css().as_deref().cloned().map(Box::new),
      side_effects_free: self.side_effects_free().as_deref().cloned(),
      top_level_declarations: self.top_level_declarations().as_deref().cloned(),
      optimization_bailouts: self.optimization_bailouts().clone(),
      module_concatenation_bailout: self.module_concatenation_bailout(),
      assets: self.assets().clone(),
      module: self.module(),
      inline_exports: self.inline_exports(),
      collected_typescript_info: self.collected_typescript_info().as_deref().cloned(),
      rsc: self.rsc().as_deref().cloned(),
      import_phase: self.import_phase(),
      isolated_dts: self.isolated_dts().as_deref().cloned().map(Box::new),
      extras: self.extras().as_ref().clone(),
      deferred_pure_checks: self.deferred_pure_checks().as_ref().clone(),
    }
  }
}

impl From<BuildInfoSnapshot> for BuildInfo {
  fn from(value: BuildInfoSnapshot) -> Self {
    Self {
      cacheable: AtomicBool::new(value.cacheable),
      hash: RwLock::new(value.hash),
      strict: AtomicBool::new(value.strict),
      module_argument: AtomicU8::new(match value.module_argument {
        ModuleArgument::Module => 0,
        ModuleArgument::RspackModule => 1,
      }),
      exports_argument: AtomicU8::new(match value.exports_argument {
        ExportsArgument::Exports => 0,
        ExportsArgument::RspackExports => 1,
      }),
      dependencies: RwLock::new(Arc::new(value.dependencies)),
      snapshot: RwLock::new(value.snapshot),
      value_dependencies: RwLock::new(Arc::new(value.value_dependencies)),
      esm_named_exports: RwLock::new(Arc::new(value.esm_named_exports)),
      all_star_exports: RwLock::new(Arc::new(value.all_star_exports)),
      need_create_require: AtomicBool::new(value.need_create_require),
      json_data: RwLock::new(value.json_data.map(Arc::new)),
      asset: RwLock::new(value.asset.map(Arc::from)),
      css: RwLock::new(value.css.map(Arc::from)),
      side_effects_free: RwLock::new(value.side_effects_free.map(Arc::new)),
      top_level_declarations: RwLock::new(value.top_level_declarations.map(Arc::new)),
      optimization_bailouts: RwLock::new(value.optimization_bailouts),
      module_concatenation_bailout: RwLock::new(value.module_concatenation_bailout),
      assets: RwLock::new(value.assets),
      module: AtomicBool::new(value.module),
      inline_exports: AtomicBool::new(value.inline_exports),
      collected_typescript_info: RwLock::new(value.collected_typescript_info.map(Arc::new)),
      rsc: RwLock::new(value.rsc.map(Arc::new)),
      import_phase: AtomicU8::new(match value.import_phase {
        ImportPhase::Evaluation => 0,
        ImportPhase::Source => 1,
        ImportPhase::Defer => 2,
      }),
      isolated_dts: RwLock::new(value.isolated_dts.map(Arc::from)),
      extras: RwLock::new(Arc::new(value.extras)),
      deferred_pure_checks: RwLock::new(Arc::new(value.deferred_pure_checks)),
    }
  }
}

#[cacheable]
#[derive(Debug)]
pub struct BuildInfoSnapshot {
  /// Whether the result is cacheable, i.e shared between builds.
  cacheable: bool,
  hash: Option<RspackHashDigest>,
  strict: bool,
  module_argument: ModuleArgument,
  exports_argument: ExportsArgument,
  dependencies: crate::LoaderDependencies,
  /// Snapshot used by full `need_build` validation. `NormalModule` populates
  /// this when the module build cache is enabled; other builds leave it empty
  /// to avoid snapshot creation overhead.
  snapshot: Option<Snapshot>,
  value_dependencies: HashMap<String, String>,
  #[cacheable(with=AsVec<AsPreset>)]
  esm_named_exports: HashSet<Atom>,
  all_star_exports: Vec<DependencyId>,
  need_create_require: bool,
  #[cacheable(with=AsOption<AsPreset>)]
  json_data: Option<JsonValue>,
  asset: Option<Box<AssetBuildInfo>>,
  css: Option<Box<CssBuildInfo>>,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  side_effects_free: Option<AtomSet>,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  top_level_declarations: Option<AtomSet>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
  module_concatenation_bailout: Option<String>,
  assets: BindingCell<HashMap<String, CompilationAsset>>,
  module: bool,
  inline_exports: bool,
  collected_typescript_info: Option<CollectedTypeScriptInfo>,
  rsc: Option<RscMeta>,
  import_phase: ImportPhase,
  isolated_dts: Option<Box<IsolatedDts>>,
  /// Stores external fields from the JS side (Record<string, any>),
  /// while other properties are stored in KnownBuildInfo.
  #[cacheable(with=AsPreset)]
  extras: serde_json::Map<String, serde_json::Value>,
  #[cacheable(with=AsVec)]
  deferred_pure_checks: HashSet<DeferredPureCheck>,
}

impl Default for BuildInfoSnapshot {
  fn default() -> Self {
    Self {
      cacheable: true,
      hash: None,
      strict: false,
      module_argument: Default::default(),
      exports_argument: Default::default(),
      dependencies: Default::default(),
      snapshot: None,
      value_dependencies: HashMap::default(),
      esm_named_exports: HashSet::default(),
      all_star_exports: Vec::default(),
      need_create_require: false,
      json_data: None,
      asset: None,
      css: None,
      side_effects_free: None,
      top_level_declarations: None,
      optimization_bailouts: Vec::new(),
      module_concatenation_bailout: None,
      assets: Default::default(),
      module: false,
      inline_exports: false,
      collected_typescript_info: None,
      rsc: None,
      import_phase: ImportPhase::Evaluation,
      isolated_dts: None,
      extras: Default::default(),
      deferred_pure_checks: HashSet::default(),
    }
  }
}

impl AsConverter<BuildInfo> for BuildInfoSnapshot {
  fn serialize(data: &BuildInfo, _guard: &ContextGuard) -> rspack_cacheable::Result<Self> {
    Ok(data.snapshot_data())
  }
  fn deserialize(self, _guard: &ContextGuard) -> rspack_cacheable::Result<BuildInfo> {
    Ok(self.into())
  }
}
