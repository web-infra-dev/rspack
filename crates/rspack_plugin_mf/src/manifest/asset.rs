use std::path::Path;

use rspack_core::{BoxModule, Compilation, ModuleGraph, ModuleIdentifier, NormalModule};
use rspack_util::fx_hash::FxHashSet as HashSet;

use super::{
  data::{AssetsSplit, StatsAssetsGroup},
  utils::is_hot_file,
};

pub(super) fn collect_assets_from_chunk(
  compilation: &Compilation,
  chunk_key: &rspack_core::ChunkUkey,
  entry_point_names: &HashSet<String>,
) -> StatsAssetsGroup {
  let mut js_sync = HashSet::<String>::default();
  let mut js_async = HashSet::<String>::default();
  let mut css_sync = HashSet::<String>::default();
  let mut css_async = HashSet::<String>::default();
  let Some(chunk) = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .get(chunk_key)
  else {
    return empty_assets_group();
  };

  for file in chunk.files() {
    if file.ends_with(".css") {
      css_sync.insert(file.clone());
    } else if !is_hot_file(file) {
      js_sync.insert(file.clone());
    }
  }

  for cg in chunk.groups() {
    let group = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(cg);
    let skip = group
      .name()
      .is_some_and(|name| entry_point_names.contains(name));
    if !skip {
      for chunk_ukey in &group.chunks {
        let group_chunk = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey);
        if let Some(group_chunk_name) = group_chunk.name()
          && let Some(chunk_name) = chunk.name()
          && group_chunk_name == chunk_name
          && chunk_ukey != chunk_key
        {
          continue;
        }
        for file in group_chunk.files() {
          if file.ends_with(".css") {
            css_sync.insert(file.clone());
          } else if !is_hot_file(file) {
            js_sync.insert(file.clone());
          }
        }
      }
    }
  }

  for async_chunk_key in
    chunk.get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
  {
    let async_chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(&async_chunk_key);
    for file in async_chunk.files() {
      if file.ends_with(".css") {
        css_async.insert(file.clone());
      } else if !is_hot_file(file) {
        js_async.insert(file.clone());
      }
    }
    for cg in async_chunk.groups() {
      let group = compilation
        .build_chunk_graph_artifact
        .chunk_group_by_ukey
        .expect_get(cg);
      let skip = group
        .name()
        .is_some_and(|name| entry_point_names.contains(name));
      if !skip {
        for file in group.get_files(&compilation.build_chunk_graph_artifact.chunk_by_ukey) {
          if file.ends_with(".css") {
            css_async.insert(file.clone());
          } else if !is_hot_file(&file) {
            js_async.insert(file);
          }
        }
      }
    }
  }

  StatsAssetsGroup {
    js: AssetsSplit {
      sync: js_sync.into_iter().collect(),
      r#async: js_async.into_iter().collect(),
    },
    css: AssetsSplit {
      sync: css_sync.into_iter().collect(),
      r#async: css_async.into_iter().collect(),
    },
  }
}

pub(super) fn merge_assets_group(target: &mut StatsAssetsGroup, source: StatsAssetsGroup) {
  target.js.sync.extend(source.js.sync);
  target.js.r#async.extend(source.js.r#async);
  target.css.sync.extend(source.css.sync);
  target.css.r#async.extend(source.css.r#async);
}

pub(super) fn empty_assets_group() -> StatsAssetsGroup {
  StatsAssetsGroup {
    js: AssetsSplit::default(),
    css: AssetsSplit::default(),
  }
}

pub(super) fn normalize_assets_group(group: &mut StatsAssetsGroup) {
  group.js.sync.sort();
  group.js.sync.dedup();
  group.js.r#async.sort();
  group.js.r#async.dedup();
  group.css.sync.sort();
  group.css.sync.dedup();
  group.css.r#async.sort();
  group.css.r#async.dedup();
}

pub(super) fn collect_assets_for_module(
  compilation: &Compilation,
  module_identifier: &ModuleIdentifier,
  entry_point_names: &HashSet<String>,
) -> Option<StatsAssetsGroup> {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  if chunk_graph.get_number_of_module_chunks(*module_identifier) == 0 {
    return None;
  }
  let mut result = empty_assets_group();
  for chunk_ukey in chunk_graph.get_module_chunks(*module_identifier) {
    let chunk_assets = collect_assets_from_chunk(compilation, chunk_ukey, entry_point_names);
    merge_assets_group(&mut result, chunk_assets);
  }
  normalize_assets_group(&mut result);
  Some(result)
}

pub(super) fn collect_usage_files_for_module(
  compilation: &Compilation,
  module_graph: &ModuleGraph,
  module_identifier: &ModuleIdentifier,
  entry_point_names: &HashSet<String>,
) -> Vec<String> {
  let mut files = HashSet::default();
  for connection in module_graph.get_incoming_connections(module_identifier) {
    let origin_identifier = connection
      .original_module_identifier
      .or(connection.resolved_original_module_identifier);
    let Some(origin) = origin_identifier else {
      continue;
    };
    if let Some(path) = module_graph
      .module_by_identifier(&origin)
      .and_then(|module| module_source_path(module, compilation))
    {
      files.insert(path);
      continue;
    }
    if let Some(assets) = collect_assets_for_module(compilation, &origin, entry_point_names) {
      files.extend(assets.js.sync);
      files.extend(assets.js.r#async);
    } else if let Some(origin_module) = module_graph.module_by_identifier(&origin) {
      files.insert(origin_module.identifier().to_string());
    }
  }
  let mut collected: Vec<String> = files.into_iter().collect();
  collected.sort();
  collected
}

pub(super) fn module_source_path(module: &BoxModule, compilation: &Compilation) -> Option<String> {
  if let Some(normal_module) = module.as_ref().as_any().downcast_ref::<NormalModule>()
    && let Some(path) = normal_module.resource_resolved_data().path()
  {
    let context_path = compilation.options.context.as_path();
    let relative = Path::new(path.as_str())
      .strip_prefix(context_path)
      .unwrap_or_else(|_| Path::new(path.as_str()));
    let mut display = relative.to_string_lossy().into_owned();
    if display.is_empty() {
      display = path.as_str().to_string();
    }
    if display.starts_with("./") {
      display.drain(..2);
    } else if display.starts_with('/') {
      display = display.trim_start_matches('/').to_string();
    }
    if display.is_empty() {
      return None;
    }
    let normalized: String = display
      .chars()
      .map(|c| if c == '\\' { '/' } else { c })
      .collect();
    if normalized.is_empty() {
      return None;
    }
    return Some(normalized);
  }

  let mut identifier = module
    .readable_identifier(&compilation.options.context)
    .to_string();
  if identifier.is_empty() {
    return None;
  }
  if let Some(pos) = identifier.rfind('!') {
    identifier = identifier.split_off(pos + 1);
  }
  if let Some(pos) = identifier.find('?') {
    identifier.truncate(pos);
  }
  // strip aggregated suffix like " + 1 modules"
  if let Some((before, _)) = identifier.split_once(" + ") {
    identifier = before.to_string();
  }
  if identifier.starts_with("./") {
    identifier.drain(..2);
  }
  if identifier.is_empty() {
    return None;
  }
  let normalized: String = identifier
    .chars()
    .map(|c| if c == '\\' { '/' } else { c })
    .collect();
  if normalized.is_empty() {
    None
  } else {
    Some(normalized)
  }
}
