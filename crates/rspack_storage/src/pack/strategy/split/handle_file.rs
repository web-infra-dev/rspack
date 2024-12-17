use std::sync::Arc;

use futures::{future::join_all, TryFutureExt};
use rspack_error::{error, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  pack::data::{current_time, PackScope, RootMeta, RootOptions, ScopeMeta},
  PackFS,
};

pub async fn remove_files(files: HashSet<Utf8PathBuf>, fs: Arc<dyn PackFS>) -> Result<()> {
  let tasks = files.into_iter().map(|path| {
    let fs = fs.clone();
    tokio::spawn(async move { fs.remove_file(&path).await }).map_err(|e| error!("{e}"))
  });

  let res = join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

  let mut errors = vec![];
  for task_result in res {
    if let Err(e) = task_result {
      errors.push(format!("- {}", e));
    }
  }

  if errors.is_empty() {
    Ok(())
  } else {
    Err(error!("remove files failed:\n{}", errors.join("\n")))
  }
}

pub async fn move_temp_files(
  files: HashSet<Utf8PathBuf>,
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<dyn PackFS>,
) -> Result<()> {
  let lock_file = root.join("move.lock");
  let mut lock_writer = fs.write_file(&lock_file).await?;
  let mut contents = vec![temp_root.to_string()];
  contents.extend(files.iter().map(|path| path.to_string()));

  lock_writer
    .write_all(contents.join("\n").as_bytes())
    .await?;
  lock_writer.flush().await?;

  let mut candidates = vec![];
  for to in files {
    let from = redirect_to_path(&to, root, temp_root)?;
    candidates.push((from, to));
  }

  let tasks = candidates.into_iter().map(|(from, to)| {
    let fs = fs.clone();
    tokio::spawn(async move { fs.move_file(&from, &to).await }).map_err(|e| error!("{e}"))
  });

  let res = join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<Result<()>>>>()?;

  let mut errors = vec![];
  for task_result in res {
    if let Err(e) = task_result {
      errors.push(format!("- {}", e));
    }
  }

  if errors.is_empty() {
    fs.remove_file(&lock_file).await?;

    Ok(())
  } else {
    Err(error!("move temp files failed:\n{}", errors.join("\n")))
  }
}

pub async fn recovery_move_lock(
  root: &Utf8Path,
  temp_root: &Utf8Path,
  fs: Arc<dyn PackFS>,
) -> Result<()> {
  let lock_file = root.join("move.lock");
  if !fs.exists(&lock_file).await? {
    return Ok(());
  }
  let mut lock_reader = fs.read_file(&lock_file).await?;
  let lock_file_content = String::from_utf8(lock_reader.read_to_end().await?)
    .map_err(|e| error!("parse utf8 failed: {}", e))?;
  let files = lock_file_content.split("\n").collect::<Vec<_>>();
  fs.remove_file(&lock_file).await?;

  if files.is_empty() {
    return Err(error!("incomplete storage due to empty `move.lock`"));
  }
  if files.first().is_some_and(|root| !root.eq(temp_root)) {
    return Err(error!(
      "incomplete storage due to `move.lock` from an unexpected directory"
    ));
  }
  move_temp_files(
    files[1..]
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

  let res = join_all(clean_scope_tasks).await;

  let mut errors = vec![];
  for task_result in res {
    if let Err(e) = task_result {
      errors.push(format!("- {}", e));
    }
  }

  if errors.is_empty() {
    Ok(())
  } else {
    Err(error!("clean scopes failed:\n{}", errors.join("\n")))
  }
}

async fn remove_unused_scope(name: &str, dir: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<()> {
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

pub async fn clean_root(root: &Utf8Path, root_meta: &RootMeta, fs: Arc<dyn PackFS>) -> Result<()> {
  let dirs = fs.read_dir(root).await?;
  let tasks = dirs.difference(&root_meta.scopes).map(|name| {
    let fs = fs.clone();
    let scope_dir = root.join(name);
    let scope_name = name.clone();
    tokio::spawn(async move { remove_unused_scope(&scope_name, &scope_dir, fs).await })
      .map_err(|e| error!("{e}"))
  });

  let res = join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

  let mut errors = vec![];
  for task_result in res {
    if let Err(e) = task_result {
      errors.push(format!("- {}", e));
    }
  }

  if errors.is_empty() {
    Ok(())
  } else {
    Err(error!(
      "remove unused scopes failed:\n{}",
      errors.join("\n")
    ))
  }
}

async fn remove_expired_version(version: &str, dir: &Utf8Path, fs: Arc<dyn PackFS>) -> Result<()> {
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
    return fs.remove_dir(dir).await;
  }

  // remove direcotires of expired versions
  let mut reader = fs.read_file(&meta).await?;
  let expire_time = reader
    .read_line()
    .await?
    .parse::<u64>()
    .map_err(|e| error!("parse option meta failed: {}", e))?;
  let current = current_time();

  if current > expire_time {
    fs.remove_dir(dir).await
  } else {
    Ok(())
  }
}

pub async fn clean_versions(
  root: &Utf8Path,
  root_options: &RootOptions,
  fs: Arc<dyn PackFS>,
) -> Result<()> {
  let dirs = fs.read_dir(&root_options.root).await?;
  let tasks = dirs.into_iter().filter_map(|version| {
    let version_dir = root_options.root.join(&version);
    if version_dir == root {
      None
    } else {
      let fs = fs.clone();
      Some(
        tokio::spawn(async move { remove_expired_version(&version, &version_dir, fs).await })
          .map_err(|e| error!("{e}")),
      )
    }
  });

  let res = join_all(tasks)
    .await
    .into_iter()
    .collect::<Result<Vec<_>>>()?;

  let mut errors = vec![];
  for task_result in res {
    if let Err(e) = task_result {
      errors.push(format!("- {}", e));
    }
  }

  if errors.is_empty() {
    Ok(())
  } else {
    Err(error!(
      "remove expired versions failed:\n{}",
      errors.join("\n")
    ))
  }
}
