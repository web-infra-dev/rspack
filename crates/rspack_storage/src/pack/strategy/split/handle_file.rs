use std::{
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use futures::{future::join_all, TryFutureExt};
use rspack_error::{error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  pack::data::{PackScope, RootMeta, RootOptions, ScopeMeta},
  PackFS,
};

pub async fn remove_files(files: HashSet<Utf8PathBuf>, fs: Arc<dyn PackFS>) -> Result<()> {
  let tasks = files.into_iter().map(|path| {
    let fs = fs.clone();
    tokio::spawn(async move { fs.remove_file(&path).await })
      .map_err(|e| error!("remove files failed: {}", e))
  });

  join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<Result<()>>>>()?;

  Ok(())
}

pub async fn move_temp_files(
  files: HashSet<Utf8PathBuf>,
  fs: Arc<dyn PackFS>,
  src: &Utf8Path,
  dist: &Utf8Path,
) -> Result<()> {
  let mut candidates = vec![];
  for to in files {
    let from = redirect_to_path(&to, src, dist)?;
    candidates.push((from, to));
  }

  let tasks = candidates.into_iter().map(|(from, to)| {
    let fs = fs.clone();
    tokio::spawn(async move { fs.move_file(&from, &to).await })
      .map_err(|e| error!("move temp files failed: {}", e))
  });

  join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<Result<()>>>>()?;

  Ok(())
}

pub async fn walk_dir(root: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<HashSet<Utf8PathBuf>> {
  let mut files = HashSet::default();
  let mut stack = vec![root.to_owned()];
  while let Some(path) = stack.pop() {
    let meta = fs.metadata(&path).await?;
    if meta.is_directory {
      stack.append(
        &mut fs
          .read_dir(&path)
          .await?
          .into_iter()
          .map(|name| path.join(name))
          .collect::<Vec<_>>(),
      );
    } else {
      files.insert(path);
    }
  }
  Ok(files)
}

pub fn redirect_to_path(path: &Utf8Path, src: &Utf8Path, dist: &Utf8Path) -> Result<Utf8PathBuf> {
  let relative_path = path
    .strip_prefix(src)
    .map_err(|e| error!("get relative path failed: {}", e))?;
  Ok(dist.join(relative_path))
}

async fn clean_scope(scope: &PackScope, fs: Arc<dyn PackFS>) -> Result<()> {
  let scope_root = &scope.path;
  let scope_meta_file = ScopeMeta::get_path(scope_root);
  let mut scope_files = scope
    .packs
    .expect_value()
    .iter()
    .flatten()
    .map(|pack| &pack.path)
    .collect::<HashSet<_>>();

  scope_files.insert(&scope_meta_file);

  let all_files = walk_dir(scope_root, fs.clone()).await?;
  let mut unrelated_files = HashSet::default();
  for file in all_files {
    if !scope_files.contains(&file) {
      unrelated_files.insert(file);
    }
  }

  remove_files(unrelated_files, fs).await?;

  Ok(())
}

pub async fn clean_scopes(scopes: &HashMap<String, PackScope>, fs: Arc<dyn PackFS>) -> Result<()> {
  let clean_scope_tasks = scopes.values().map(|scope| clean_scope(scope, fs.clone()));

  join_all(clean_scope_tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<()>>>()?;

  Ok(())
}

pub async fn clean_root(root: &Utf8Path, root_meta: &RootMeta, fs: Arc<dyn PackFS>) -> Result<()> {
  let dirs = fs.read_dir(root).await?;
  let tasks = dirs.difference(&root_meta.scopes).map(|name| {
    let dir = root.join(name);
    let fs = fs.clone();
    tokio::spawn(async move { fs.remove_dir(&dir).await })
      .map_err(|e| error!("remove unused scope failed: {}", e))
  });

  join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<Result<()>>>>()?;

  Ok(())
}

async fn remove_expired_version(
  root_dir: &Utf8Path,
  expire: u64,
  fs: Arc<dyn PackFS>,
) -> Result<()> {
  let meta = RootMeta::get_path(root_dir);
  if !fs.exists(&meta).await? {
    return fs.remove_dir(root_dir).await;
  }
  let mut reader = fs.read_file(&meta).await?;
  let last_modified = reader
    .read_line()
    .await?
    .parse::<u64>()
    .map_err(|e| error!("parse option meta failed: {}", e))?;
  let current = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .expect("should get current time")
    .as_millis() as u64;

  if current - last_modified > expire {
    fs.remove_dir(root_dir).await
  } else {
    Ok(())
  }
}

pub async fn clean_versions(root_options: &RootOptions, fs: Arc<dyn PackFS>) -> Result<()> {
  let dirs = fs.read_dir(&root_options.root).await?;
  let tasks = dirs.iter().map(|name| {
    let version_dir = root_options.root.join(name);
    let fs = fs.clone();
    let expire = root_options.expire;
    tokio::spawn(async move { remove_expired_version(&version_dir, expire, fs).await })
      .map_err(|e| error!("remove expired version failed: {}", e))
  });

  join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<Result<()>>>>()?;

  Ok(())
}
