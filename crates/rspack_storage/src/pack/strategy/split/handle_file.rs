use std::sync::Arc;

use futures::future::join_all;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::task::JoinError;

use crate::{
  FSResult, FileSystem,
  fs::{BatchFSError, BatchFSResult, FSError, FSOperation},
  pack::data::{PackScope, RootMeta, RootOptions, ScopeMeta, current_time},
};

pub async fn prepare_scope(
  scope_path: &Utf8Path,
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> FSResult<()> {
  let temp_path = redirect_to_path(scope_path, root, temp_root)?;
  fs.remove_dir(&temp_path).await?;
  fs.ensure_dir(&temp_path).await?;
  fs.ensure_dir(scope_path).await?;
  Ok(())
}

pub async fn prepare_scope_dirs(
  scopes: &HashMap<String, PackScope>,
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let tasks = scopes.values().map(|scope| {
    let fs = fs.clone();
    let scope_path = scope.path.clone();
    let root_path = root.to_path_buf();
    let temp_root_path = temp_root.to_path_buf();
    tokio::spawn(async move { prepare_scope(&scope_path, &root_path, &temp_root_path, fs).await })
  });

  BatchFSError::try_from_joined_result(
    "prepare scopes directories failed",
    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>, JoinError>>(),
  )
  .map(|_| ())
}

pub async fn remove_files(files: HashSet<Utf8PathBuf>, fs: Arc<FileSystem>) -> BatchFSResult<()> {
  let tasks = files.into_iter().map(|path| {
    let fs = fs.clone();
    tokio::spawn(async move { fs.remove_file(&path).await })
  });

  BatchFSError::try_from_joined_result(
    "remove files failed",
    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>, JoinError>>(),
  )
  .map(|_| ())
}

pub async fn write_lock(
  lock_file: &str,
  files: &HashSet<Utf8PathBuf>,
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> FSResult<()> {
  let lock_file = root.join(lock_file);
  let mut lock_writer = fs.write_file(&lock_file).await?;
  let mut contents = vec![root.to_string(), temp_root.to_string()];
  contents.extend(files.iter().map(|path| path.to_string()));

  lock_writer
    .write_all(contents.join("\n").as_bytes())
    .await?;
  lock_writer.flush().await?;
  Ok(())
}

pub async fn remove_lock(lock_file: &str, root: &Utf8Path, fs: Arc<FileSystem>) -> FSResult<()> {
  let lock_file = root.join(lock_file);
  fs.remove_file(&lock_file).await?;
  Ok(())
}

pub async fn move_files(
  files: HashSet<Utf8PathBuf>,
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let mut candidates = vec![];
  for to in files {
    let from = redirect_to_path(&to, root, temp_root)?;
    candidates.push((from, to));
  }

  let tasks = candidates.into_iter().map(|(from, to)| {
    let fs = fs.clone();
    tokio::spawn(async move { fs.move_file(&from, &to).await })
  });

  BatchFSError::try_from_joined_result(
    "move temp files failed",
    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>, JoinError>>(),
  )
  .map(|_| ())
}

async fn recovery_lock(
  lock: &str,
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> FSResult<Vec<String>> {
  let lock_file = root.join(lock);
  if !fs.exists(&lock_file).await? {
    return Ok(vec![]);
  }
  let mut lock_reader = fs.read_file(&lock_file).await?;
  let lock_file_content = String::from_utf8(lock_reader.read_to_end().await?).map_err(|e| {
    FSError::from_message(
      &lock_file,
      FSOperation::Read,
      format!("parse utf8 failed: {e}"),
    )
  })?;
  let files = lock_file_content
    .split("\n")
    .map(|i| i.to_owned())
    .collect::<Vec<_>>();
  fs.remove_file(&lock_file).await?;

  if files.len() < 2 {
    return Err(FSError::from_message(
      &lock_file,
      FSOperation::Read,
      "incomplete storage due to illegal `move.lock`".to_string(),
    ));
  }
  if files.first().is_some_and(|p: &String| !p.eq(root)) {
    return Err(FSError::from_message(
      &lock_file,
      FSOperation::Read,
      "incomplete storage due to `move.lock` to an unexpected directory".to_string(),
    ));
  }
  if files.get(1).is_some_and(|p| !p.eq(temp_root)) {
    return Err(FSError::from_message(
      &lock_file,
      FSOperation::Read,
      "incomplete storage due to `move.lock` from an unexpected directory".to_string(),
    ));
  }
  Ok(files[2..].to_vec())
}

pub async fn recovery_move_lock(
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let moving_files = recovery_lock("move.lock", root, temp_root, fs.clone()).await?;
  if moving_files.is_empty() {
    return Ok(());
  }
  move_files(
    moving_files
      .iter()
      .map(Utf8PathBuf::from)
      .collect::<HashSet<_>>(),
    root,
    temp_root,
    fs,
  )
  .await?;
  Ok(())
}

pub async fn recovery_remove_lock(
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let removing_files = recovery_lock("remove.lock", root, temp_root, fs.clone()).await?;
  if removing_files.is_empty() {
    return Ok(());
  }
  remove_files(
    removing_files
      .iter()
      .map(Utf8PathBuf::from)
      .collect::<HashSet<_>>(),
    fs,
  )
  .await?;
  Ok(())
}

pub async fn walk_dir(root: &Utf8Path, fs: Arc<FileSystem>) -> BatchFSResult<HashSet<Utf8PathBuf>> {
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
          .filter_map(|name| {
            if name.starts_with(".") {
              None
            } else {
              Some(path.join(name))
            }
          })
          .collect::<Vec<_>>(),
      );
    } else {
      files.insert(path);
    }
  }
  Ok(files)
}

pub fn redirect_to_path(path: &Utf8Path, src: &Utf8Path, dist: &Utf8Path) -> FSResult<Utf8PathBuf> {
  let relative_path = path
    .strip_prefix(src)
    .map_err(|e| FSError::from_message(path, FSOperation::Redirect, format!("{e}")))?;
  Ok(dist.join(relative_path))
}

async fn try_remove_scope_files(scope: &PackScope, fs: Arc<FileSystem>) -> BatchFSResult<()> {
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

pub async fn remove_unused_scope_files(
  scopes: &HashMap<String, PackScope>,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let clean_scope_tasks = scopes
    .values()
    .map(|scope| try_remove_scope_files(scope, fs.clone()));

  BatchFSError::try_from_results("clean scopes failed", join_all(clean_scope_tasks).await)
    .map(|_| ())
}

async fn try_remove_scope(name: &str, dir: &Utf8Path, fs: Arc<FileSystem>) -> FSResult<()> {
  // do not remove hidden dirs
  if name.starts_with(".") {
    return Ok(());
  }

  // do not remove files
  if !(fs.metadata(dir).await?.is_directory) {
    return Ok(());
  }

  fs.remove_dir(dir).await?;

  Ok(())
}

pub async fn remove_unused_scopes(
  root: &Utf8Path,
  root_meta: &RootMeta,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let dirs = fs.read_dir(root).await?;
  let tasks = dirs.difference(&root_meta.scopes).map(|name| {
    let fs = fs.clone();
    let scope_dir = root.join(name);
    let scope_name = name.clone();
    tokio::spawn(async move { try_remove_scope(&scope_name, &scope_dir, fs).await })
  });

  BatchFSError::try_from_joined_result(
    "remove unused scopes failed",
    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>, JoinError>>(),
  )
  .map(|_| ())
}

async fn try_remove_version(
  version: &str,
  dir: &Utf8Path,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  // do not remove hidden dirs and lock files
  if version.starts_with(".") || version.contains(".lock") {
    return Ok(());
  }

  // do not remove files
  if !(fs.metadata(dir).await?.is_directory) {
    return Ok(());
  }

  // remove unknown directories
  let meta = RootMeta::get_path(dir);
  if !fs.exists(&meta).await? {
    fs.remove_dir(dir).await?;
    return Ok(());
  }

  // remove direcotires of expired versions
  let mut reader = fs.read_file(&meta).await?;
  let expire_time = reader.read_line().await?.parse::<u64>().map_err(|e| {
    FSError::from_message(
      &meta,
      FSOperation::Read,
      format!("parse option meta failed: {e}"),
    )
  })?;
  let current = current_time();

  if current > expire_time {
    fs.remove_dir(dir).await?;
    Ok(())
  } else {
    Ok(())
  }
}

pub async fn remove_expired_versions(
  root: &Utf8Path,
  root_options: &RootOptions,
  fs: Arc<FileSystem>,
) -> BatchFSResult<()> {
  let dirs = fs.read_dir(&root_options.root).await?;
  let tasks = dirs.into_iter().filter_map(|version| {
    let version_dir = root_options.root.join(&version);
    if version_dir == root {
      None
    } else {
      let fs = fs.clone();
      Some(tokio::spawn(async move {
        try_remove_version(&version, &version_dir, fs).await
      }))
    }
  });

  BatchFSError::try_from_joined_result(
    "remove expired versions failed",
    join_all(tasks)
      .await
      .into_iter()
      .collect::<Result<Vec<_>, JoinError>>(),
  )
  .map(|_| ())
}
