use std::sync::{
  Arc, RwLock,
  atomic::{AtomicBool, Ordering},
};

use concatenated_module_entries::*;
pub use determine_export_assignments::DetermineExportAssignmentsKey;
use determine_export_assignments::*;
use get_exports_type::*;
use get_mode::*;
use get_side_effects_connection_state::*;
use module_graph_hash::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::atoms::Atom;

use crate::{
  ConcatenationEntry, ConnectionState, DependencyId, ExportInfo, ExportsInfo, ExportsType,
  ModuleIdentifier, RuntimeKey,
};
pub type ModuleGraphCacheArtifact = Arc<ModuleGraphCacheArtifactInner>;

/// This is a rust port of `ModuleGraph.cached` and `ModuleGraph.dependencyCacheProvide` in webpack.
/// We use this to cache the result of functions with high computational overhead.
#[derive(Debug, Default)]
pub struct ModuleGraphCacheArtifactInner {
  /// Webpack enables module graph caches by creating new cache maps and disable them by setting them to undefined.
  /// But in rust I think it's better to use a bool flag to avoid memory reallocation.
  freezed: AtomicBool,
  get_mode_cache: GetModeCache,
  determine_export_assignments_cache: DetermineExportAssignmentsCache,
  get_exports_type_cache: GetExportsTypeCache,
  get_side_effects_connection_state_cache: GetSideEffectsConnectionStateCache,
  concatenated_module_entries: ConcatenatedModuleEntriesCache,
  module_graph_hash_cache: ModuleGraphHashCache,
}

impl ModuleGraphCacheArtifactInner {
  pub fn freeze(&self) {
    self.get_mode_cache.freeze();
    self.determine_export_assignments_cache.freeze();
    self.get_exports_type_cache.freeze();
    self.get_side_effects_connection_state_cache.freeze();
    self.concatenated_module_entries.freeze();
    self.module_graph_hash_cache.freeze();
    self.freezed.store(true, Ordering::Release);
  }

  pub fn unfreeze(&self) {
    self.freezed.store(false, Ordering::Release);
  }

  pub fn cached_get_exports_type<F: FnOnce() -> ExportsType>(
    &self,
    key: GetExportsTypeCacheKey,
    f: F,
  ) -> ExportsType {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.get_exports_type_cache.get(&key) {
      Some(value) => value,
      None => {
        let value = f();
        self.get_exports_type_cache.set(key, value);
        value
      }
    }
  }

  pub fn cached_get_mode<F: FnOnce() -> ExportMode>(
    &self,
    key: GetModeCacheKey,
    f: F,
  ) -> ExportMode {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.get_mode_cache.get(&key) {
      Some(value) => value,
      None => {
        let value = f();
        self.get_mode_cache.set(key, value.clone());
        value
      }
    }
  }

  pub fn cached_determine_export_assignments<F: FnOnce() -> DetermineExportAssignmentsValue>(
    &self,
    key: DetermineExportAssignmentsKey,
    f: F,
  ) -> DetermineExportAssignmentsValue {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.determine_export_assignments_cache.get(&key) {
      Some(value) => value,
      None => {
        let value = f();
        self
          .determine_export_assignments_cache
          .set(key, value.clone());
        value
      }
    }
  }

  pub fn cached_get_side_effects_connection_state<F: FnOnce() -> ConnectionState>(
    &self,
    key: ModuleIdentifier,
    f: F,
  ) -> ConnectionState {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.get_side_effects_connection_state_cache.get(key) {
      Some(value) => value,
      None => {
        let value = f();
        self.get_side_effects_connection_state_cache.set(key, value);
        value
      }
    }
  }

  pub fn cached_concatenated_module_entries<F: FnOnce() -> Vec<ConcatenationEntry>>(
    &self,
    key: ConcatenatedModuleEntriesCacheKey,
    f: F,
  ) -> Vec<ConcatenationEntry> {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.concatenated_module_entries.get(&key) {
      Some(value) => value,
      None => {
        let value = f();
        self.concatenated_module_entries.set(key, value.clone());
        value
      }
    }
  }

  pub fn cached_module_graph_hash<F: FnOnce() -> (u64, ExportsInfo, Vec<ExportsInfo>)>(
    &self,
    key: ModuleIdentifier,
    f: F,
  ) -> (u64, ExportsInfo, Vec<ExportsInfo>) {
    if !self.freezed.load(Ordering::Acquire) {
      return f();
    }

    match self.module_graph_hash_cache.get(key) {
      Some(value) => value,
      None => {
        let value = f();
        self.module_graph_hash_cache.set(key, value.clone());
        value
      }
    }
  }
}

pub(super) mod module_graph_hash {
  use rspack_collections::IdentifierDashMap;

  use crate::{ExportsInfo, ModuleIdentifier};

  #[derive(Debug, Default)]
  pub struct ModuleGraphHashCache {
    cache: IdentifierDashMap<(u64, ExportsInfo, Vec<ExportsInfo>)>,
  }

  impl ModuleGraphHashCache {
    pub fn freeze(&self) {
      self.cache.clear();
    }

    pub fn get(&self, key: ModuleIdentifier) -> Option<(u64, ExportsInfo, Vec<ExportsInfo>)> {
      self.cache.get(&key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: ModuleIdentifier, value: (u64, ExportsInfo, Vec<ExportsInfo>)) {
      self.cache.insert(key, value);
    }
  }
}
pub(super) mod concatenated_module_entries {
  use super::*;
  use crate::ModuleIdentifier;

  pub type ConcatenatedModuleEntriesCacheKey = (ModuleIdentifier, Option<RuntimeKey>);

  #[derive(Debug, Default)]
  pub struct ConcatenatedModuleEntriesCache {
    cache: dashmap::DashMap<ConcatenatedModuleEntriesCacheKey, Vec<ConcatenationEntry>>,
  }

  impl ConcatenatedModuleEntriesCache {
    pub fn freeze(&self) {
      self.cache.clear();
    }

    pub fn get(&self, key: &ConcatenatedModuleEntriesCacheKey) -> Option<Vec<ConcatenationEntry>> {
      self.cache.get(key).map(|v| v.value().clone())
    }

    pub fn set(&self, key: ConcatenatedModuleEntriesCacheKey, value: Vec<ConcatenationEntry>) {
      self.cache.insert(key, value);
    }
  }
}

pub(super) mod get_side_effects_connection_state {
  use rspack_collections::IdentifierDashMap;

  use crate::{ConnectionState, ModuleIdentifier};

  #[derive(Debug, Default)]
  pub struct GetSideEffectsConnectionStateCache {
    cache: IdentifierDashMap<ConnectionState>,
  }

  impl GetSideEffectsConnectionStateCache {
    pub fn freeze(&self) {
      self.cache.clear();
    }

    pub fn get(&self, key: ModuleIdentifier) -> Option<ConnectionState> {
      self.cache.get(&key).map(|v| *v.value())
    }

    pub fn set(&self, key: ModuleIdentifier, value: ConnectionState) {
      self.cache.insert(key, value);
    }
  }
}

pub(super) mod get_exports_type {
  use crate::{ExportsType, ModuleIdentifier};

  pub type GetExportsTypeCacheKey = (ModuleIdentifier, bool);

  #[derive(Debug, Default)]
  pub struct GetExportsTypeCache {
    cache: dashmap::DashMap<GetExportsTypeCacheKey, ExportsType>,
  }

  impl GetExportsTypeCache {
    pub fn freeze(&self) {
      self.cache.clear();
    }

    pub fn get(&self, key: &GetExportsTypeCacheKey) -> Option<ExportsType> {
      self.cache.get(key).map(|x| *x)
    }

    pub fn set(&self, key: GetExportsTypeCacheKey, value: ExportsType) {
      self.cache.insert(key, value);
    }
  }
}

pub(super) mod get_mode {
  use super::*;

  pub type GetModeCacheKey = (DependencyId, Option<RuntimeKey>);

  #[derive(Debug, Default)]
  pub struct GetModeCache {
    cache: RwLock<HashMap<GetModeCacheKey, ExportMode>>,
  }

  impl GetModeCache {
    pub fn freeze(&self) {
      self.cache.write().expect("should get lock").clear();
    }

    pub fn get(&self, key: &GetModeCacheKey) -> Option<ExportMode> {
      let inner = self.cache.read().expect("should get lock");
      inner.get(key).cloned()
    }

    pub fn set(&self, key: GetModeCacheKey, value: ExportMode) {
      self
        .cache
        .write()
        .expect("should get lock")
        .insert(key, value);
    }
  }
}

pub(super) mod determine_export_assignments {
  use super::*;
  use crate::ModuleIdentifier;

  /// Webpack cache the result of `determineExportAssignments` with the keys of dependencies arraris of `allStarExports.dependencies` and `otherStarExports` + `this`(DependencyId).
  /// See: https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/dependencies/HarmonyExportImportedSpecifierDependency.js#L645
  ///
  /// However, we can simplify the cache key since dependencies under the same parent module share `allStarExports` and copy their own `otherStarExports`.
  #[derive(Debug, PartialEq, Eq, Hash)]
  pub enum DetermineExportAssignmentsKey {
    All(ModuleIdentifier),
    Other(DependencyId),
  }
  pub type DetermineExportAssignmentsValue = (Vec<Atom>, Vec<usize>);

  #[derive(Debug, Default)]
  pub struct DetermineExportAssignmentsCache {
    cache: RwLock<HashMap<DetermineExportAssignmentsKey, DetermineExportAssignmentsValue>>,
  }

  impl DetermineExportAssignmentsCache {
    pub fn freeze(&self) {
      self.cache.write().expect("should get lock").clear();
    }

    pub fn get(
      &self,
      key: &DetermineExportAssignmentsKey,
    ) -> Option<DetermineExportAssignmentsValue> {
      let inner = self.cache.read().expect("should get lock");
      inner.get(key).cloned()
    }

    pub fn set(&self, key: DetermineExportAssignmentsKey, value: DetermineExportAssignmentsValue) {
      self
        .cache
        .write()
        .expect("should get lock")
        .insert(key, value);
    }
  }
}

#[derive(Debug, Clone)]
pub struct NormalReexportItem {
  pub name: Atom,
  pub ids: Vec<Atom>,
  pub hidden: bool,
  pub checked: bool,
  pub export_info: ExportInfo,
}

#[derive(Debug, Clone)]
pub enum ExportMode {
  Missing,
  LazyMake,
  Unused(ExportModeUnused),
  EmptyStar(ExportModeEmptyStar),
  ReexportDynamicDefault(ExportModeReexportDynamicDefault),
  ReexportNamedDefault(ExportModeReexportNamedDefault),
  ReexportNamespaceObject(ExportModeReexportNamespaceObject),
  ReexportFakeNamespaceObject(ExportModeFakeNamespaceObject),
  ReexportUndefined(ExportModeReexportUndefined),
  NormalReexport(ExportModeNormalReexport),
  DynamicReexport(Box<ExportModeDynamicReexport>),
}

#[derive(Debug, Clone)]
pub struct ExportModeUnused {
  pub name: Atom,
}

#[derive(Debug, Clone)]
pub struct ExportModeEmptyStar {
  pub hidden: Option<HashSet<Atom>>,
}

#[derive(Debug, Clone)]
pub struct ExportModeReexportDynamicDefault {
  pub name: Atom,
}

#[derive(Debug, Clone)]
pub struct ExportModeReexportNamedDefault {
  pub name: Atom,
  pub partial_namespace_export_info: ExportInfo,
}

#[derive(Debug, Clone)]
pub struct ExportModeReexportNamespaceObject {
  pub name: Atom,
  pub partial_namespace_export_info: ExportInfo,
}

#[derive(Debug, Clone)]
pub struct ExportModeFakeNamespaceObject {
  pub name: Atom,
  pub fake_type: u8,
  pub partial_namespace_export_info: ExportInfo,
}

#[derive(Debug, Clone)]
pub struct ExportModeReexportUndefined {
  pub name: Atom,
}

#[derive(Debug, Clone)]
pub struct ExportModeNormalReexport {
  pub items: Vec<NormalReexportItem>,
}

#[derive(Debug, Clone)]
pub struct ExportModeDynamicReexport {
  pub ignored: HashSet<Atom>,
  pub hidden: Option<HashSet<Atom>>,
}

#[derive(Debug, Default)]
pub struct StarReexportsInfo {
  pub exports: Option<HashSet<Atom>>,
  pub checked: Option<HashSet<Atom>>,
  pub ignored_exports: HashSet<Atom>,
  pub hidden: Option<HashSet<Atom>>,
}
