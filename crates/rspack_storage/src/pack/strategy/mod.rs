mod split;

use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
pub use split::SplitPackStrategy;

use super::data::{
  Pack, PackContents, PackFileMeta, PackGenerations, PackKeys, PackOptions, PackScope, RootMeta,
  RootOptions,
};
use crate::{
  ItemKey, ItemValue,
  error::{Result, ValidateResult},
};

pub struct UpdatePacksResult {
  pub new_packs: Vec<(PackFileMeta, Pack)>,
  pub remain_packs: Vec<(PackFileMeta, Pack)>,
  pub removed_files: Vec<Utf8PathBuf>,
}

#[derive(Debug, Default)]
pub struct PackMainContents {
  pub contents: PackContents,
  pub generations: PackGenerations,
}

#[derive(Debug, Default, Clone)]
pub struct WriteScopeResult {
  pub wrote_files: HashSet<Utf8PathBuf>,
  pub removed_files: HashSet<Utf8PathBuf>,
}

impl WriteScopeResult {
  pub fn extend(&mut self, other: Self) {
    self.wrote_files.extend(other.wrote_files);
    self.removed_files.extend(other.removed_files);
  }
}

pub type ScopeUpdate = HashMap<ItemKey, Option<ItemValue>>;
