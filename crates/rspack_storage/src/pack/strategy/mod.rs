mod split;

use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashSet as HashSet;
pub use split::SplitPackStrategy;

use super::data::{Pack, PackContents, PackFileMeta, PackGenerations};
use crate::{ItemKey, ItemValue};

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

pub type ScopeUpdate = rustc_hash::FxHashMap<ItemKey, Option<ItemValue>>;
