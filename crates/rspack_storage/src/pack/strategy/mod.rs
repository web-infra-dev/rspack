mod split;

use async_trait::async_trait;
use rspack_error::{miette, thiserror::Error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
pub use split::SplitPackStrategy;

use super::data::{
  Pack, PackContents, PackFileMeta, PackKeys, PackOptions, PackScope, RootMeta, RootOptions,
};
use crate::{StorageItemKey, StorageItemValue};

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
  async fn before_load(&self) -> Result<()>;
  async fn read_root_meta(&self) -> Result<Option<RootMeta>>;
  async fn write_root_meta(&self, root_meta: &RootMeta) -> Result<()>;
  async fn validate_root(&self, root_meta: &RootMeta) -> Result<ValidateResult>;
  async fn clean_unused(
    &self,
    root_meta: &RootMeta,
    scopes: &HashMap<String, PackScope>,
    root_options: &RootOptions,
  ) -> Result<()>;
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

pub type ScopeUpdate = HashMap<StorageItemKey, Option<StorageItemValue>>;
#[async_trait]
pub trait ScopeWriteStrategy {
  fn update_scope(&self, scope: &mut PackScope, updates: ScopeUpdate) -> Result<()>;
  async fn before_all(&self, scopes: &mut HashMap<String, PackScope>) -> Result<()>;
  async fn write_packs(&self, scope: &mut PackScope) -> Result<WriteScopeResult>;
  async fn write_meta(&self, scope: &mut PackScope) -> Result<WriteScopeResult>;
  async fn merge_changed(&self, changed: WriteScopeResult) -> Result<()>;
  async fn after_all(&self, scopes: &mut HashMap<String, PackScope>) -> Result<()>;
}

#[derive(Debug)]
pub struct InvalidDetail {
  pub reason: String,
  pub packs: Vec<String>,
}

#[derive(Debug)]
pub enum ValidateResult {
  NotExists,
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

#[derive(Debug)]
enum ScopeValidateErrorReason {
  Reason(String),
  Detail(InvalidDetail),
  Error(rspack_error::Error),
}

#[derive(Debug, Error)]
pub struct StorageValidateError {
  scope: Option<&'static str>,
  inner: ScopeValidateErrorReason,
}

impl StorageValidateError {
  pub fn from_detail(scope: Option<&'static str>, detail: InvalidDetail) -> Self {
    Self {
      scope,
      inner: ScopeValidateErrorReason::Detail(detail),
    }
  }
  pub fn from_error(scope: Option<&'static str>, error: rspack_error::Error) -> Self {
    Self {
      scope,
      inner: ScopeValidateErrorReason::Error(error),
    }
  }
  pub fn from_reason(scope: Option<&'static str>, reason: String) -> Self {
    Self {
      scope,
      inner: ScopeValidateErrorReason::Reason(reason),
    }
  }
}

impl std::fmt::Display for StorageValidateError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "validate ")?;
    if let Some(scope) = self.scope {
      write!(f, "scope `{}` ", scope)?;
    }
    write!(f, "failed due to")?;

    match &self.inner {
      ScopeValidateErrorReason::Detail(detail) => {
        write!(f, " {}", detail.reason)?;
        let mut pack_info_lines = detail
          .packs
          .iter()
          .map(|p| format!("- {}", p))
          .collect::<Vec<_>>();
        if pack_info_lines.len() > 5 {
          pack_info_lines.truncate(5);
          pack_info_lines.push("...".to_string());
        }
        if !pack_info_lines.is_empty() {
          write!(f, ":\n{}", pack_info_lines.join("\n"))?;
        }
      }
      ScopeValidateErrorReason::Error(e) => {
        write!(f, " {}", e)?;
      }
      ScopeValidateErrorReason::Reason(e) => {
        write!(f, " {}", e)?;
      }
    }
    Ok(())
  }
}

impl miette::Diagnostic for StorageValidateError {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new("StorageValidateError"))
  }
  fn severity(&self) -> Option<miette::Severity> {
    Some(miette::Severity::Warning)
  }
  fn diagnostic_source(&self) -> Option<&dyn miette::Diagnostic> {
    match &self.inner {
      ScopeValidateErrorReason::Error(e) => Some(e.as_ref()),
      _ => None,
    }
  }
}
