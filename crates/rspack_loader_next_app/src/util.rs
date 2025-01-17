use std::{
  collections::{HashMap, HashSet},
  fs::Metadata,
  path::{Path, PathBuf},
  sync::LazyLock,
};

use rspack_core::CompilationId;
use rspack_error::{error, Result};
use rspack_paths::Utf8Path;
use rspack_util::fx_hash::{BuildFxHasher, FxDashMap};
use tokio::sync::RwLock;

static READ_DIR_CACHE: LazyLock<
  RwLock<(
    Option<CompilationId>,
    FxDashMap<String, HashMap<String, Metadata, BuildFxHasher>>,
  )>,
> = LazyLock::new(|| RwLock::new((None, FxDashMap::default())));

pub fn normalize_app_path(route: &str) -> String {
  let segments = route.split('/');
  let segments_len = segments.clone().count();
  let mut pathname = String::new();

  for (index, segment) in segments.enumerate() {
    // Empty segments are ignored.
    if segment.is_empty() {
      continue;
    }

    // Groups are ignored.
    if is_group_segment(segment) {
      continue;
    }

    // Parallel segments are ignored.
    if segment.starts_with('@') {
      continue;
    }

    // The last segment (if it's a leaf) should be ignored.
    if (segment == "page" || segment == "route") && index == segments_len - 1 {
      continue;
    }

    pathname.push('/');
    pathname.push_str(segment);
  }

  ensure_leading_slash(&pathname)
}

pub fn ensure_leading_slash(path: &str) -> String {
  if path.starts_with('/') {
    path.to_string()
  } else {
    format!("/{}", path)
  }
}

pub fn is_group_segment(segment: &str) -> bool {
  segment.starts_with('(') && segment.ends_with(')')
}

pub fn normalize_underscore(pathname: &str) -> String {
  pathname.replace("%5F", "_")
}

pub fn normalize_parallel_key(key: &str) -> &str {
  key.strip_prefix('@').unwrap_or(key)
}

pub fn is_app_builtin_not_found_page(page: &str) -> bool {
  let re = lazy_regex::regex!(r"next[\\/]dist[\\/]client[\\/]components[\\/]not-found-error");
  re.is_match(page)
}

pub fn create_absolute_path(app_dir: &str, path_to_turn_absolute: &str) -> String {
  let p = path_to_turn_absolute.replace("/", std::path::MAIN_SEPARATOR_STR);
  if let Some(p) = p.strip_prefix("private-next-app-dir") {
    format!("{}{}", app_dir, p)
  } else {
    p
  }
}

pub async fn metadata_resolver(
  dirname: &str,
  filename: &str,
  exts: &[&str],
  app_dir: &str,
  compilation_id: CompilationId,
) -> Result<(Option<String>, HashSet<PathBuf, BuildFxHasher>)> {
  let absolute_dir = create_absolute_path(app_dir, dirname);

  let mut result: Option<String> = None;
  let mut missing_dependencies = HashSet::default();

  for ext in exts {
    // Compared to `resolver` above the exts do not have the `.` included already, so it's added here.
    let filename_with_ext = format!("{}.{}", filename, ext);
    let absolute_path_with_extension = format!(
      "{}{}{}",
      absolute_dir,
      std::path::MAIN_SEPARATOR,
      filename_with_ext
    );
    if result.is_none()
      && file_exists_in_directory(dirname, &filename_with_ext, compilation_id).await?
    {
      result = Some(absolute_path_with_extension.clone());
    }
    // Call `add_missing_dependency` for all files even if they didn't match,
    // because they might be added or removed during development.
    missing_dependencies.insert(PathBuf::from(absolute_path_with_extension));
  }

  Ok((result, missing_dependencies))
}

pub async fn resolver(
  pathname: &str,
  app_dir: &str,
  extensions: &[String],
  compilation_id: CompilationId,
) -> Result<(Option<String>, HashSet<PathBuf, BuildFxHasher>)> {
  let absolute_path = create_absolute_path(app_dir, pathname);

  let filename_index = absolute_path.rfind(std::path::MAIN_SEPARATOR).unwrap_or(0);
  let dirname = &absolute_path[..filename_index];
  let filename = &absolute_path[filename_index + 1..];

  let mut result: Option<String> = None;
  let mut missing_dependencies = HashSet::default();

  for ext in extensions {
    let absolute_path_with_extension = format!("{}.{}", absolute_path, ext);
    if result.is_none()
      && file_exists_in_directory(dirname, &format!("{}.{}", filename, ext), compilation_id).await?
    {
      result = Some(absolute_path_with_extension.clone());
    }
    // Call `add_missing_dependency` for all files even if they didn't match,
    // because they might be added or removed during development.
    missing_dependencies.insert(PathBuf::from(absolute_path_with_extension));
  }

  Ok((result, missing_dependencies))
}

pub async fn read_dir_with_compilation_cache<R>(
  dir: &str,
  compilation_id: CompilationId,
  f: impl FnOnce(&HashMap<String, Metadata, BuildFxHasher>) -> R,
) -> Result<R> {
  let cache = READ_DIR_CACHE.read().await;
  let cache = if cache.0 != Some(compilation_id) {
    cache.1.clear();
    drop(cache);
    let mut cache = READ_DIR_CACHE.write().await;
    cache.0 = Some(compilation_id);
    drop(cache);
    READ_DIR_CACHE.read().await
  } else {
    cache
  };
  let r = if let Some(results) = cache.1.get(dir) {
    f(&results)
  } else {
    let mut results = HashMap::default();
    let mut entries = tokio::fs::read_dir(dir)
      .await
      .map_err(|e| error!("{dir} {e}"))?;
    while let Some(entry) = entries.next_entry().await.map_err(|e| error!(e))? {
      results.insert(
        entry
          .file_name()
          .into_string()
          .map_err(|e| error!("failed to convert OsString to String"))?,
        entry.metadata().await.map_err(|e| error!(e))?,
      );
    }
    let r = f(&results);
    cache.1.insert(dir.to_string(), results);
    r
  };
  Ok(r)
}

pub async fn file_exists_in_directory(
  dirname: &str,
  filename: &str,
  compilation_id: CompilationId,
) -> Result<bool> {
  read_dir_with_compilation_cache(dirname, compilation_id, |results| {
    results
      .get(filename)
      .map(|metadata| metadata.is_file())
      .unwrap_or(false)
  })
  .await
}

pub async fn is_directory(path: &str) -> bool {
  tokio::fs::metadata(path)
    .await
    .map(|m| m.is_dir())
    .unwrap_or(false)
}
