mod split;

use async_trait::async_trait;
use rspack_error::Result;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
pub use split::SplitPackStrategy;

use super::data::{Pack, PackContents, PackFileMeta, PackKeys, PackOptions, PackScope};
use crate::{StorageItemKey, StorageItemValue};

pub struct UpdatePacksResult {
  pub new_packs: Vec<(PackFileMeta, Pack)>,
  pub remain_packs: Vec<(PackFileMeta, Pack)>,
  pub removed_files: Vec<Utf8PathBuf>,
}

#[async_trait]
pub trait ScopeStrategy:
  ScopeReadStrategy + ScopeWriteStrategy + ScopeValidateStrategy + std::fmt::Debug + Sync + Send
{
}

#[async_trait]
pub trait PackReadStrategy {
  async fn read_pack_keys(&self, path: &Utf8Path) -> Result<Option<PackKeys>>;
  async fn read_pack_contents(&self, path: &Utf8Path) -> Result<Option<PackContents>>;
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

#[derive(Debug)]
pub struct InvalidDetail {
  pub reason: String,
  pub packs: Vec<String>,
}

#[derive(Debug)]
pub enum ValidateResult {
  Valid,
  Invalid(InvalidDetail),
}

impl ValidateResult {
  pub fn invalid(reason: &str) -> Self {
    Self::Invalid(InvalidDetail {
      reason: reason.to_string(),
      packs: vec![],
    })
  }
  pub fn invalid_with_packs(reason: &str, packs: Vec<String>) -> Self {
    Self::Invalid(InvalidDetail {
      reason: reason.to_string(),
      packs,
    })
  }
  pub fn is_valid(&self) -> bool {
    matches!(self, ValidateResult::Valid)
  }
}

impl std::fmt::Display for ValidateResult {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ValidateResult::Valid => write!(f, "validation passed"),
      ValidateResult::Invalid(e) => {
        let mut pack_info_lines = e
          .packs
          .iter()
          .map(|p| format!("- {}", p))
          .collect::<Vec<_>>();
        if pack_info_lines.len() > 5 {
          pack_info_lines.truncate(5);
          pack_info_lines.push("...".to_string());
        }
        write!(
          f,
          "validation failed due to {}{}",
          e.reason,
          if pack_info_lines.is_empty() {
            "".to_string()
          } else {
            format!(":\n{}", pack_info_lines.join("\n"))
          }
        )
      }
    }
  }
}

#[async_trait]
pub trait ScopeValidateStrategy {
  async fn validate_meta(&self, scope: &mut PackScope) -> Result<ValidateResult>;
  async fn validate_packs(&self, scope: &mut PackScope) -> Result<ValidateResult>;
}

#[derive(Debug, Default)]
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
  fn update_scope(&self, scope: &mut PackScope, updates: ScopeUpdate) -> Result<()>;
  fn before_all(&self, scope: &mut PackScope) -> Result<()>;
  async fn before_write(&self, scope: &PackScope) -> Result<()>;
  async fn write_packs(&self, scope: &mut PackScope) -> Result<WriteScopeResult>;
  async fn write_meta(&self, scope: &mut PackScope) -> Result<WriteScopeResult>;
  async fn after_write(
    &self,
    scope: &PackScope,
    wrote_files: HashSet<Utf8PathBuf>,
    removed_files: HashSet<Utf8PathBuf>,
  ) -> Result<()>;
  fn after_all(&self, scope: &mut PackScope) -> Result<()>;
}
