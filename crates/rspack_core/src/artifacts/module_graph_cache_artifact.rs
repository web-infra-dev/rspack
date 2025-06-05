use std::sync::{
  atomic::{AtomicBool, Ordering},
  Arc, Mutex,
};

use derive_more::derive::Debug;
use indexmap::IndexMap;
use rustc_hash::FxHashSet as HashSet;
use swc_core::atoms::Atom;

use crate::{DependencyId, ExportInfo, RuntimeSpec};

pub type ModuleGraphCacheArtifact = Arc<ModuleGraphCacheArtifactInner>;

#[derive(Debug, Default)]
pub struct ModuleGraphCacheArtifactInner {
  freezed: AtomicBool,
  get_mode_cache: GetModeCache,
}

impl ModuleGraphCacheArtifactInner {
  pub fn freeze(&self) {
    self.get_mode_cache.freeze();
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
      Some(mode) => mode,
      None => {
        let mode = f();
        self.get_mode_cache.set(key, mode.clone());
        mode
      }
    }
  }
}

type GetModeCacheKey = (DependencyId, Option<RuntimeSpec>);

#[derive(Debug, Default)]
pub struct GetModeCache {
  cache: Mutex<IndexMap<GetModeCacheKey, ExportMode>>,
}

impl GetModeCache {
  fn freeze(&self) {
    self.cache.lock().expect("should get lock").clear();
  }

  fn get(&self, key: &GetModeCacheKey) -> Option<ExportMode> {
    let inner = self.cache.lock().expect("should get lock");
    inner.get(key).cloned()
  }

  fn set(&self, key: GetModeCacheKey, value: ExportMode) {
    self
      .cache
      .lock()
      .expect("should get lock")
      .insert(key, value);
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
