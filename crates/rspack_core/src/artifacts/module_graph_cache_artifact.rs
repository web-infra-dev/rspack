use std::sync::{Arc, Mutex};

use derive_more::derive::Debug;
use indexmap::IndexMap;
use rustc_hash::FxHashSet as HashSet;
use swc_core::atoms::Atom;

use crate::{DependencyId, ExportInfo, RuntimeSpec};

#[derive(Debug, Default, Clone)]
pub struct ModuleGraphCacheArtifact {
  pub get_mode_cache: GetModeCache,
}

impl ModuleGraphCacheArtifact {
  pub fn freeze(&self) {}

  pub fn unfreeze(&self) {}
}

type GetModeCacheKey = (DependencyId, Option<RuntimeSpec>);

#[derive(Debug, Clone, Default)]
pub struct GetModeCache {
  inner: Arc<Mutex<IndexMap<GetModeCacheKey, ExportMode>>>,
}

impl GetModeCache {
  pub fn get(&self, key: &GetModeCacheKey) -> Option<ExportMode> {
    let inner = self.inner.lock().expect("should get lock");
    inner.get(key).cloned()
  }

  pub fn set(&self, key: GetModeCacheKey, value: ExportMode) {
    self
      .inner
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
