mod split;

use async_trait::async_trait;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
pub(super) use split::SplitPackStrategy;

use super::data::{
  Pack, PackContents, PackFileMeta, PackGenerations, PackKeys, PackOptions, PackScope, RootMeta,
  RootOptions,
};
use crate::{
  ItemKey, ItemValue,
  error::{Result, ValidateResult},
};

pub(super) struct UpdatePacksResult {
  pub new_packs: Vec<(PackFileMeta, Pack)>,
  pub remain_packs: Vec<(PackFileMeta, Pack)>,
  pub removed_files: Vec<Utf8PathBuf>,
}

#[async_trait]
pub trait ScopeStrategy:
  RootStrategy
  + ScopeReadStrategy
  + ScopeWriteStrategy
  + ScopeValidateStrategy
  + std::fmt::Debug
  + Sync
  + Send
{
}

#[async_trait]
pub trait RootStrategy {
  async fn before_load(&self) -> Result<()>;
  async fn read_root_meta(&self) -> Result<Option<RootMeta>>;
  async fn write_root_meta(&self, root_meta: &RootMeta) -> Result<()>;
  async fn validate_root(&self, root_meta: &RootMeta) -> Result<ValidateResult>;
  async fn clean(
    &self,
    root_meta: &RootMeta,
    scopes: &HashMap<String, PackScope>,
    root_options: &RootOptions,
  ) -> Result<()>;
  async fn reset(&self);
}

#[derive(Debug, Default)]
pub(super) struct PackMainContents {
  pub contents: PackContents,
  pub generations: PackGenerations,
}

#[async_trait]
pub(super) trait PackReadStrategy {
  async fn read_pack_keys(&self, path: &Utf8Path) -> Result<Option<PackKeys>>;
  async fn read_pack_contents(&self, path: &Utf8Path) -> Result<Option<PackMainContents>>;
}

#[async_trait]
pub(super) trait PackWriteStrategy {
  async fn update_packs(
    &self,
    dir: Utf8PathBuf,
    generation: usize,
    options: &PackOptions,
    packs: HashMap<PackFileMeta, Pack>,
    updates: HashMap<ItemKey, Option<ItemValue>>,
  ) -> Result<UpdatePacksResult>;
  async fn optimize_packs(
    &self,
    dir: Utf8PathBuf,
    options: &PackOptions,
    packs: Vec<(PackFileMeta, Pack)>,
  ) -> Result<UpdatePacksResult>;
  async fn write_pack(&self, pack: &Pack) -> Result<()>;
}

#[async_trait]
pub trait ScopeReadStrategy {
  fn get_path(&self, sub: &str) -> Utf8PathBuf;
  async fn ensure_meta(&self, scope: &mut PackScope) -> Result<()>;
  async fn ensure_packs(&self, scope: &mut PackScope) -> Result<()>;
  async fn ensure_keys(&self, scope: &mut PackScope) -> Result<()>;
  async fn ensure_contents(&self, scope: &mut PackScope) -> Result<()>;
}

#[async_trait]
pub trait ScopeValidateStrategy {
  async fn validate_meta(&self, scope: &mut PackScope) -> Result<ValidateResult>;
  async fn validate_packs(&self, scope: &mut PackScope) -> Result<ValidateResult>;
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

pub(super) type ScopeUpdate = HashMap<ItemKey, Option<ItemValue>>;
#[async_trait]
pub trait ScopeWriteStrategy {
  async fn update_scope(&self, scope: &mut PackScope, updates: ScopeUpdate) -> Result<()>;
  async fn before_all(&self, scopes: &mut HashMap<String, PackScope>) -> Result<()>;
  async fn optimize_scope(&self, scope: &mut PackScope) -> Result<()>;
  async fn release_scope(&self, scope: &mut PackScope) -> Result<()>;
  async fn write_packs(&self, scope: &mut PackScope) -> Result<WriteScopeResult>;
  async fn write_meta(&self, scope: &mut PackScope) -> Result<WriteScopeResult>;
  async fn merge_changed(&self, changed: WriteScopeResult) -> Result<()>;
  async fn after_all(&self, scopes: &mut HashMap<String, PackScope>) -> Result<()>;
}
