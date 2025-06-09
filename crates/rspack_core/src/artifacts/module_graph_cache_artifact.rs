use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc, Mutex,
};

pub use determine_export_assignments::DetermineExportAssignmentsKey;
use determine_export_assignments::*;
use get_mode::*;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::atoms::Atom;

use crate::{DependencyId, ExportInfo, RuntimeKey};
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
}

impl ModuleGraphCacheArtifactInner {
  pub fn freeze(&self) {
    self.get_mode_cache.freeze();
    self.determine_export_assignments_cache.freeze();
    self.freezed.store(true, Ordering::Relaxed);
  }

  pub fn unfreeze(&self) {
    self.freezed.store(false, Ordering::Relaxed);
  }

  pub fn cached_get_mode<F: FnOnce() -> ExportMode>(
    &self,
    key: GetModeCacheKey,
    f: F,
  ) -> ExportMode {
    if !self.freezed.load(Ordering::Relaxed) {
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
    if !self.freezed.load(Ordering::Relaxed) {
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
}

pub(super) mod get_mode {
  use super::*;

  pub type GetModeCacheKey = (DependencyId, Option<RuntimeKey>);

  #[derive(Debug, Default)]
  pub struct GetModeCache {
    cache: Mutex<HashMap<GetModeCacheKey, ExportMode>>,
  }

  impl GetModeCache {
    pub fn freeze(&self) {
      self.cache.lock().expect("should get lock").clear();
    }

    pub fn get(&self, key: &GetModeCacheKey) -> Option<ExportMode> {
      let inner = self.cache.lock().expect("should get lock");
      inner.get(key).cloned()
    }

    pub fn set(&self, key: GetModeCacheKey, value: ExportMode) {
      self
        .cache
        .lock()
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
    cache: Mutex<HashMap<DetermineExportAssignmentsKey, DetermineExportAssignmentsValue>>,
  }

  impl DetermineExportAssignmentsCache {
    pub fn freeze(&self) {
      self.cache.lock().expect("should get lock").clear();
    }

    pub fn get(
      &self,
      key: &DetermineExportAssignmentsKey,
    ) -> Option<DetermineExportAssignmentsValue> {
      let inner = self.cache.lock().expect("should get lock");
      inner.get(key).cloned()
    }

    pub fn set(&self, key: DetermineExportAssignmentsKey, value: DetermineExportAssignmentsValue) {
      self
        .cache
        .lock()
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ExportModeType {
  Missing,
  Unused,
  EmptyStar,
  ReexportDynamicDefault,
  ReexportNamedDefault,
  ReexportNamespaceObject,
  ReexportFakeNamespaceObject,
  ReexportUndefined,
  NormalReexport,
  DynamicReexport,
}

#[derive(Debug, Clone)]
pub struct ExportMode {
  /// corresponding to `type` field in webpack's `EpxortMode`
  pub ty: ExportModeType,
  pub items: Option<Vec<NormalReexportItem>>,
  pub name: Option<Atom>,
  pub fake_type: u8,
  pub partial_namespace_export_info: Option<ExportInfo>,
  pub ignored: Option<HashSet<Atom>>,
  pub hidden: Option<HashSet<Atom>>,
}

impl ExportMode {
  pub fn new(ty: ExportModeType) -> Self {
    Self {
      ty,
      items: None,
      name: None,
      fake_type: 0,
      partial_namespace_export_info: None,
      ignored: None,
      hidden: None,
    }
  }
}

#[derive(Debug, Default)]
pub struct StarReexportsInfo {
  pub exports: Option<HashSet<Atom>>,
  pub checked: Option<HashSet<Atom>>,
  pub ignored_exports: HashSet<Atom>,
  pub hidden: Option<HashSet<Atom>>,
}
