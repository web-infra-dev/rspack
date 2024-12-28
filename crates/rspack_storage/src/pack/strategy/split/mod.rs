mod handle_file;
mod read_pack;
mod read_scope;
mod util;
mod validate_scope;
mod write_pack;
mod write_scope;

use std::{hash::Hasher, sync::Arc};

use handle_file::{
  recovery_move_lock, recovery_remove_lock, remove_expired_versions, remove_unused_scope_files,
  remove_unused_scopes,
};
use itertools::Itertools;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use util::get_name;

use super::{RootStrategy, ScopeStrategy};
use crate::{
  error::{Result, ValidateResult},
  fs::{FSError, FSOperation},
  pack::data::{
    current_time, PackContents, PackKeys, PackScope, RootMeta, RootMetaFrom, RootOptions,
  },
  FileSystem,
};

#[derive(Debug, Clone)]
pub struct SplitPackStrategy {
  pub fs: Arc<dyn FileSystem>,
  pub root: Arc<Utf8PathBuf>,
  pub temp_root: Arc<Utf8PathBuf>,
  pub fresh_generation: Option<usize>,
  pub release_generation: Option<usize>,
}

impl SplitPackStrategy {
  pub fn new(
    root: Utf8PathBuf,
    temp_root: Utf8PathBuf,
    fs: Arc<dyn FileSystem>,
    fresh_generation: Option<usize>,
    release_generation: Option<usize>,
  ) -> Self {
    Self {
      fs,
      root: Arc::new(root),
      temp_root: Arc::new(temp_root),
      fresh_generation,
      release_generation,
    }
  }

  pub async fn get_pack_hash(
    &self,
    path: &Utf8Path,
    keys: &PackKeys,
    contents: &PackContents,
  ) -> Result<String> {
    let mut hasher = FxHasher::default();
    let file_name = get_name(keys, contents);
    hasher.write(file_name.as_bytes());

    let meta = self.fs.metadata(path).await?;
    hasher.write_u64(meta.size);
    hasher.write_u64(meta.mtime_ms);

    Ok(format!("{:016x}", hasher.finish()))
  }
}

#[async_trait::async_trait]
impl RootStrategy for SplitPackStrategy {
  async fn before_load(&self) -> Result<()> {
    recovery_remove_lock(&self.root, &self.temp_root, self.fs.clone()).await?;
    recovery_move_lock(&self.root, &self.temp_root, self.fs.clone()).await?;
    Ok(())
  }
  async fn read_root_meta(&self) -> Result<Option<RootMeta>> {
    let meta_path = RootMeta::get_path(&self.root);
    if !self.fs.exists(&meta_path).await? {
      return Ok(None);
    }

    let mut reader = self.fs.read_file(&meta_path).await?;
    let expire_time = reader.read_line().await?.parse::<u64>().map_err(|e| {
      FSError::from_message(
        &meta_path,
        FSOperation::Read,
        format!("parse root meta failed: {}", e),
      )
    })?;
    let scopes = reader
      .read_line()
      .await?
      .split(',')
      .map(|s| s.to_string())
      .collect::<HashSet<_>>();

    Ok(Some(RootMeta {
      expire_time,
      scopes,
      from: RootMetaFrom::File,
    }))
  }
  async fn write_root_meta(&self, root_meta: &RootMeta) -> Result<()> {
    let meta_path = RootMeta::get_path(&self.root);
    let mut writer = self.fs.write_file(&meta_path).await?;

    writer
      .write_line(root_meta.expire_time.to_string().as_str())
      .await?;

    writer
      .write_line(root_meta.scopes.iter().join(",").as_str())
      .await?;

    writer.flush().await?;

    Ok(())
  }
  async fn validate_root(&self, root_meta: &RootMeta) -> Result<ValidateResult> {
    if matches!(root_meta.from, RootMetaFrom::New) {
      Ok(ValidateResult::Valid)
    } else {
      let now = current_time();
      if now > root_meta.expire_time {
        Ok(ValidateResult::invalid("expiration"))
      } else {
        Ok(ValidateResult::Valid)
      }
    }
  }

  async fn clean(
    &self,
    root_meta: &RootMeta,
    scopes: &HashMap<String, PackScope>,
    root_options: &RootOptions,
  ) -> Result<()> {
    if !root_options.clean {
      return Ok(());
    }

    let _ = tokio::try_join!(
      remove_unused_scope_files(scopes, self.fs.clone()),
      remove_unused_scopes(&self.root, root_meta, self.fs.clone()),
      remove_expired_versions(&self.root, root_options, self.fs.clone())
    );

    Ok(())
  }
}

impl ScopeStrategy for SplitPackStrategy {}
