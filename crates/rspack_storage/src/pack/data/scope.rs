use std::sync::Arc;

use itertools::Itertools;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashSet as HashSet;

use super::{Pack, PackOptions, ScopeMeta};
use crate::StorageContent;

#[derive(Debug, Default)]
pub enum ScopeMetaState {
  #[default]
  Pending,
  Value(ScopeMeta),
}

impl ScopeMetaState {
  pub fn loaded(&self) -> bool {
    matches!(self, Self::Value(_))
  }
  pub fn set_value(&mut self, value: ScopeMeta) {
    *self = ScopeMetaState::Value(value);
  }
  pub fn expect_value(&self) -> &ScopeMeta {
    match self {
      ScopeMetaState::Value(v) => v,
      ScopeMetaState::Pending => panic!("should have scope meta"),
    }
  }
  pub fn expect_value_mut(&mut self) -> &mut ScopeMeta {
    match self {
      ScopeMetaState::Value(ref mut v) => v,
      ScopeMetaState::Pending => panic!("should have scope meta"),
    }
  }
  pub fn take_value(&mut self) -> Option<ScopeMeta> {
    match self {
      ScopeMetaState::Value(v) => Some(std::mem::take(&mut *v)),
      _ => None,
    }
  }
}

pub type ScopePacks = Vec<Vec<Pack>>;

#[derive(Debug, Default)]
pub enum ScopePacksState {
  #[default]
  Pending,
  Value(ScopePacks),
}

impl ScopePacksState {
  pub fn loaded(&self) -> bool {
    matches!(self, Self::Value(_))
  }
  pub fn set_value(&mut self, value: ScopePacks) {
    *self = ScopePacksState::Value(value);
  }
  pub fn expect_value(&self) -> &ScopePacks {
    match self {
      ScopePacksState::Value(v) => v,
      ScopePacksState::Pending => panic!("scope meta is not ready"),
    }
  }
  pub fn expect_value_mut(&mut self) -> &mut ScopePacks {
    match self {
      ScopePacksState::Value(v) => v,
      ScopePacksState::Pending => panic!("scope meta is not ready"),
    }
  }
  pub fn take_value(&mut self) -> Option<ScopePacks> {
    match self {
      ScopePacksState::Value(v) => Some(std::mem::take(&mut *v)),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub struct PackScope {
  pub path: Utf8PathBuf,
  pub options: Arc<PackOptions>,
  pub meta: ScopeMetaState,
  pub packs: ScopePacksState,
  pub removed: HashSet<Utf8PathBuf>,
}

impl PackScope {
  pub fn new(path: Utf8PathBuf, options: Arc<PackOptions>) -> Self {
    Self {
      path,
      options,
      meta: ScopeMetaState::Pending,
      packs: ScopePacksState::Pending,
      removed: HashSet::default(),
    }
  }

  pub fn empty(path: Utf8PathBuf, options: Arc<PackOptions>) -> Self {
    let mut scope = Self::new(path, options);
    scope.clear();
    scope
  }

  pub fn loaded(&self) -> bool {
    matches!(self.meta, ScopeMetaState::Value(_))
      && matches!(self.packs, ScopePacksState::Value(_))
      && self
        .packs
        .expect_value()
        .iter()
        .flatten()
        .all(|pack| pack.loaded())
  }

  pub fn get_contents(&self) -> StorageContent {
    self
      .packs
      .expect_value()
      .iter()
      .flatten()
      .filter_map(|pack| {
        if let (Some(keys), Some(contents)) = (pack.keys.get_value(), pack.contents.get_value()) {
          if keys.len() == contents.len() {
            return Some(
              keys
                .iter()
                .enumerate()
                .map(|(index, key)| (key.clone(), contents[index].clone()))
                .collect_vec(),
            );
          }
        }
        None
      })
      .flatten()
      .collect_vec()
  }

  pub fn clear(&mut self) {
    self.meta = ScopeMetaState::Value(ScopeMeta::new(&self.path, &self.options));
    self.packs =
      ScopePacksState::Value((0..self.options.bucket_size).map(|_| vec![]).collect_vec());
    self.removed = HashSet::default();
  }
}
