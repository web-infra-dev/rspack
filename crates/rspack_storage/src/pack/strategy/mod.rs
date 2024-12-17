mod split;

use async_trait::async_trait;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
pub use split::SplitPackStrategy;

use super::data::{
  Pack, PackContents, PackFileMeta, PackKeys, PackOptions, PackScope, RootMeta, RootOptions,
};
use crate::{
  error::{StorageResult, ValidateResult},
  StorageItemKey, StorageItemValue,
};

pub struct UpdatePacksResult {
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
  async fn before_load(&self) -> StorageResult<()>;
  async fn read_root_meta(&self) -> StorageResult<Option<RootMeta>>;
  async fn write_root_meta(&self, root_meta: &RootMeta) -> StorageResult<()>;
  async fn validate_root(&self, root_meta: &RootMeta) -> StorageResult<ValidateResult>;
  async fn clean_unused(
    &self,
    root_meta: &RootMeta,
    scopes: &HashMap<String, PackScope>,
    root_options: &RootOptions,
  ) -> StorageResult<()>;
}

#[async_trait]
pub trait PackReadStrategy {
  async fn read_pack_keys(&self, path: &Utf8Path) -> StorageResult<Option<PackKeys>>;
  async fn read_pack_contents(&self, path: &Utf8Path) -> StorageResult<Option<PackContents>>;
}

#[async_trait]
pub trait PackWriteStrategy {
  fn update_packs(
    &self,
    dir: Utf8PathBuf,
    options: &PackOptions,
    packs: HashMap<PackFileMeta, Pack>,
    updates: HashMap<StorageItemKey, Option<StorageItemValue>>,
  ) -> UpdatePacksResult;
  async fn write_pack(&self, pack: &Pack) -> StorageResult<()>;
}

#[async_trait]
pub trait ScopeReadStrategy {
  fn get_path(&self, sub: &str) -> Utf8PathBuf;
  async fn ensure_meta(&self, scope: &mut PackScope) -> StorageResult<()>;
  async fn ensure_packs(&self, scope: &mut PackScope) -> StorageResult<()>;
  async fn ensure_keys(&self, scope: &mut PackScope) -> StorageResult<()>;
  async fn ensure_contents(&self, scope: &mut PackScope) -> StorageResult<()>;
}

#[async_trait]
pub trait ScopeValidateStrategy {
  async fn validate_meta(&self, scope: &mut PackScope) -> StorageResult<ValidateResult>;
  async fn validate_packs(&self, scope: &mut PackScope) -> StorageResult<ValidateResult>;
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

pub type ScopeUpdate = HashMap<StorageItemKey, Option<StorageItemValue>>;
#[async_trait]
pub trait ScopeWriteStrategy {
  fn update_scope(&self, scope: &mut PackScope, updates: ScopeUpdate) -> StorageResult<()>;
  async fn before_all(&self, scopes: &mut HashMap<String, PackScope>) -> StorageResult<()>;
  async fn write_packs(&self, scope: &mut PackScope) -> StorageResult<WriteScopeResult>;
  async fn write_meta(&self, scope: &mut PackScope) -> StorageResult<WriteScopeResult>;
  async fn merge_changed(&self, changed: WriteScopeResult) -> StorageResult<()>;
  async fn after_all(&self, scopes: &mut HashMap<String, PackScope>) -> StorageResult<()>;
}
