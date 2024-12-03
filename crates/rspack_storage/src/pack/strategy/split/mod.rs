mod read_pack;
mod read_scope;
mod util;
mod validate_scope;
mod write_pack;
mod write_scope;

use std::{hash::Hasher, sync::Arc};

use futures::{future::join_all, TryFutureExt};
use rspack_error::{error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashSet as HashSet, FxHasher};
use util::{get_name, walk_dir};

use super::ScopeStrategy;
use crate::pack::{
  data::{PackContents, PackKeys, PackScope, ScopeMeta},
  fs::PackFs,
};

#[derive(Debug, Clone)]
pub struct SplitPackStrategy {
  pub fs: Arc<dyn PackFs>,
  pub root: Arc<Utf8PathBuf>,
  pub temp_root: Arc<Utf8PathBuf>,
}

impl SplitPackStrategy {
  pub fn new(root: Utf8PathBuf, temp_root: Utf8PathBuf, fs: Arc<dyn PackFs>) -> Self {
    Self {
      fs,
      root: Arc::new(root),
      temp_root: Arc::new(temp_root),
    }
  }

  pub async fn move_temp_files(&self, files: HashSet<Utf8PathBuf>) -> Result<()> {
    let mut candidates = vec![];
    for to in files {
      let from = self.get_temp_path(&to)?;
      candidates.push((from, to));
    }

    let tasks = candidates.into_iter().map(|(from, to)| {
      let fs = self.fs.clone();
      tokio::spawn(async move { fs.move_file(&from, &to).await })
        .map_err(|e| error!("move temp files failed: {}", e))
    });

    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<Result<()>>>>()?;

    Ok(())
  }

  pub async fn remove_files(&self, files: HashSet<Utf8PathBuf>) -> Result<()> {
    let tasks = files.into_iter().map(|path| {
      let fs = self.fs.to_owned();
      tokio::spawn(async move { fs.remove_file(&path).await })
        .map_err(|e| error!("remove files failed: {}", e))
    });

    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<Result<()>>>>()?;

    Ok(())
  }

  pub async fn remove_unrelated_files(&self, scope: &PackScope) -> Result<()> {
    let scope_root = &scope.path;
    let scope_meta_file = ScopeMeta::get_path(scope_root);
    // let scope_related_files = vec![ScopeMeta::get_path(&scope_root)];
    let mut scope_files = scope
      .packs
      .expect_value()
      .iter()
      .flatten()
      .map(|pack| &pack.path)
      .collect::<HashSet<_>>();

    scope_files.insert(&scope_meta_file);

    let all_files = walk_dir(scope_root, self.fs.clone()).await?;
    let mut unrelated_files = HashSet::default();
    for file in all_files {
      if !scope_files.contains(&file) {
        unrelated_files.insert(file);
      }
    }

    self.remove_files(unrelated_files).await?;

    Ok(())
  }

  pub fn get_temp_path(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    let relative_path = path
      .strip_prefix(&*self.root)
      .map_err(|e| error!("get relative path failed: {}", e))?;
    Ok(self.temp_root.join(relative_path))
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
    hasher.write_u64(meta.mtime);

    Ok(format!("{:016x}", hasher.finish()))
  }
}

impl ScopeStrategy for SplitPackStrategy {}
