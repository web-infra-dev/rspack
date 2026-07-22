use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use camino::{Utf8Path, Utf8PathBuf};
use rspack_fs::ReadableFileSystem;
use rustc_hash::{FxHashMap, FxHashSet};

pub mod collect_shared_entry_plugin;
pub mod consume_shared_fallback_dependency;
pub mod consume_shared_module;
pub mod consume_shared_plugin;
pub mod consume_shared_runtime_module;
pub mod provide_for_shared_dependency;
pub mod provide_shared_dependency;
pub mod provide_shared_module;
pub mod provide_shared_module_factory;
pub mod provide_shared_plugin;
pub mod share_runtime_module;
pub mod share_runtime_plugin;
pub mod shared_container_plugin;
pub mod shared_container_runtime_module;
pub mod shared_used_exports_optimizer_plugin;
pub mod shared_used_exports_optimizer_runtime_module;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct RequestMatchKey {
  request: String,
  layer: Option<String>,
}

impl RequestMatchKey {
  pub(crate) fn new(request: &str, layer: Option<&str>) -> Self {
    Self {
      request: request.to_string(),
      layer: layer.map(str::to_string),
    }
  }

  pub(crate) fn request(&self) -> &str {
    &self.request
  }

  fn matches_layer(&self, layer: Option<&str>) -> bool {
    self.layer.is_none() || self.layer.as_deref() == layer
  }
}

pub(crate) fn find_exact_match<'a, T>(
  matches: &'a FxHashMap<RequestMatchKey, T>,
  request: &str,
  layer: Option<&str>,
) -> Option<&'a T> {
  matches
    .get(&RequestMatchKey::new(request, layer))
    .or_else(|| layer.and_then(|_| matches.get(&RequestMatchKey::new(request, None))))
}

pub(crate) fn find_prefix_match<'rules, 'request, T>(
  matches: &'rules [(RequestMatchKey, T)],
  request: &'request str,
  layer: Option<&str>,
) -> Option<(&'rules T, &'request str)> {
  let (key, value) = matches
    .iter()
    .enumerate()
    .filter(|(_, (key, _))| key.matches_layer(layer) && request.starts_with(key.request()))
    .max_by_key(|(index, (key, _))| (key.request().len(), key.layer.is_some(), *index))?
    .1;
  Some((value, request.strip_prefix(key.request())?))
}

const DESCRIPTION_FILE_NAME: &str = "package.json";

fn is_node_modules_dir(dir: &Path) -> bool {
  dir.file_name().is_some_and(|name| name == "node_modules")
}

fn collect_description_file_paths(mut dir: &Path) -> Vec<PathBuf> {
  let mut description_file_paths = Vec::new();

  loop {
    if is_node_modules_dir(dir) {
      break;
    }

    description_file_paths.push(dir.join(DESCRIPTION_FILE_NAME));

    if let Some(parent) = dir.parent() {
      dir = parent;
    } else {
      break;
    }
  }

  description_file_paths
}

fn find_ancestor_description_data<T>(
  start_dir: &Path,
  mut matcher: impl FnMut(&Path, &serde_json::Value) -> Option<T>,
) -> Option<T> {
  for description_file in collect_description_file_paths(start_dir) {
    if let Ok(data) = std::fs::read(&description_file)
      && let Ok(data) = serde_json::from_slice::<serde_json::Value>(&data)
      && let Some(dir) = description_file.parent()
      && let Some(value) = matcher(dir, &data)
    {
      return Some(value);
    }
  }

  None
}

async fn get_description_file(
  fs: Arc<dyn ReadableFileSystem>,
  dir: &Utf8Path,
  satisfies_description_file_data: Option<impl Fn(&serde_json::Value) -> bool>,
) -> (Option<serde_json::Value>, Option<Vec<String>>) {
  let mut checked_file_paths = FxHashSet::default();

  for description_file in collect_description_file_paths(dir.as_std_path()) {
    let description_file = Utf8PathBuf::from_path_buf(description_file)
      .expect("description file path should remain utf8");
    let data = fs.read(&description_file).await;

    if let Ok(data) = data
      && let Ok(data) = serde_json::from_slice::<serde_json::Value>(&data)
    {
      if satisfies_description_file_data
        .as_ref()
        .is_some_and(|f| !f(&data))
      {
        checked_file_paths.insert(description_file.to_string());
      } else {
        return (Some(data), None);
      }
    }
  }

  let mut checked_file_paths = checked_file_paths.into_iter().collect::<Vec<_>>();
  checked_file_paths.sort_unstable();

  (None, Some(checked_file_paths))
}

#[cfg(test)]
mod tests {
  use rustc_hash::FxHashMap;

  use super::{RequestMatchKey, find_exact_match, find_prefix_match};

  #[test]
  fn request_match_keys_do_not_parse_layer_delimiters() {
    let mut matches = FxHashMap::default();
    matches.insert(RequestMatchKey::new("b)c", Some("a")), 1);
    matches.insert(RequestMatchKey::new("c", Some("a)b")), 2);

    assert_eq!(find_exact_match(&matches, "b)c", Some("a")), Some(&1));
    assert_eq!(find_exact_match(&matches, "c", Some("a)b")), Some(&2));
  }

  #[test]
  fn exact_match_prefers_layer_then_falls_back() {
    let mut matches = FxHashMap::default();
    matches.insert(RequestMatchKey::new("pkg", None), "fallback");
    matches.insert(RequestMatchKey::new("pkg", Some("rsc")), "layered");

    assert_eq!(
      find_exact_match(&matches, "pkg", Some("rsc")),
      Some(&"layered")
    );
    assert_eq!(
      find_exact_match(&matches, "pkg", Some("client")),
      Some(&"fallback")
    );
  }

  #[test]
  fn prefix_match_is_longest_then_layer_specific() {
    let matches = vec![
      (RequestMatchKey::new("pkg/", Some("rsc")), "layered"),
      (RequestMatchKey::new("pkg/", None), "fallback"),
      (RequestMatchKey::new("pkg/feature/", None), "feature"),
    ];

    assert_eq!(
      find_prefix_match(&matches, "pkg/component", Some("rsc")),
      Some((&"layered", "component"))
    );
    assert_eq!(
      find_prefix_match(&matches, "pkg/component", Some("client")),
      Some((&"fallback", "component"))
    );
    assert_eq!(
      find_prefix_match(&matches, "pkg/feature/button", Some("rsc")),
      Some((&"feature", "button"))
    );
  }
}
