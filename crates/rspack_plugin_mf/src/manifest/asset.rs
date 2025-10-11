use rspack_core::{Compilation, ModuleGraph, ModuleIdentifier};
use rspack_util::fx_hash::FxHashSet as HashSet;

use super::{
  data::{AssetsSplit, StatsAssetsGroup},
  utils::is_hot_file,
};

pub fn collect_assets_from_chunk(
  compilation: &Compilation,
  chunk_key: &rspack_core::ChunkUkey,
  entry_point_names: &HashSet<String>,
) -> StatsAssetsGroup {
  let mut js_sync = HashSet::<String>::default();
  let mut js_async = HashSet::<String>::default();
  let mut css_sync = HashSet::<String>::default();
  let mut css_async = HashSet::<String>::default();
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_key);

  for cg in chunk.groups() {
    let group = compilation.chunk_group_by_ukey.expect_get(cg);
    if group
      .name()
      .is_some_and(|name| !entry_point_names.contains(name))
    {
      for file in group.get_files(&compilation.chunk_by_ukey) {
        if file.ends_with(".css") {
          css_sync.insert(file.to_string());
        } else if !is_hot_file(&file) {
          js_sync.insert(file);
        }
      }
    }
  }

  for async_chunk_key in chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey) {
    let async_chunk = compilation.chunk_by_ukey.expect_get(&async_chunk_key);
    for file in async_chunk.files() {
      if file.ends_with(".css") {
        css_async.insert(file.to_string());
      } else if !is_hot_file(file) {
        js_async.insert(file.to_string());
      }
    }
    for cg in async_chunk.groups() {
      let group = compilation.chunk_group_by_ukey.expect_get(cg);
      if group
        .name()
        .is_some_and(|name| !entry_point_names.contains(name))
      {
        for file in group.get_files(&compilation.chunk_by_ukey) {
          if file.ends_with(".css") {
            css_async.insert(file.to_string());
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

pub fn merge_assets_group(target: &mut StatsAssetsGroup, source: StatsAssetsGroup) {
  target.js.sync.extend(source.js.sync);
  target.js.r#async.extend(source.js.r#async);
  target.css.sync.extend(source.css.sync);
  target.css.r#async.extend(source.css.r#async);
}

pub fn empty_assets_group() -> StatsAssetsGroup {
  StatsAssetsGroup {
    js: AssetsSplit::default(),
    css: AssetsSplit::default(),
  }
}

pub fn normalize_assets_group(group: &mut StatsAssetsGroup) {
  group.js.sync.sort();
  group.js.sync.dedup();
  group.js.r#async.sort();
  group.js.r#async.dedup();
  group.css.sync.sort();
  group.css.sync.dedup();
  group.css.r#async.sort();
  group.css.r#async.dedup();
}

pub fn collect_assets_for_module(
  compilation: &Compilation,
  module_identifier: &ModuleIdentifier,
  entry_point_names: &HashSet<String>,
) -> Option<StatsAssetsGroup> {
  let chunk_graph = &compilation.chunk_graph;
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

pub fn remove_assets(group: &mut StatsAssetsGroup, exclude: &HashSet<String>) {
  group.js.sync.retain(|asset| !exclude.contains(asset));
  group.js.r#async.retain(|asset| !exclude.contains(asset));
  group.css.sync.retain(|asset| !exclude.contains(asset));
  group.css.r#async.retain(|asset| !exclude.contains(asset));
  normalize_assets_group(group);
}

pub fn promote_primary_assets_to_sync(group: &mut StatsAssetsGroup) {
  if group.js.sync.is_empty() {
    group.js.sync.append(&mut group.js.r#async);
  }
  if group.css.sync.is_empty() {
    group.css.sync.append(&mut group.css.r#async);
  }
  normalize_assets_group(group);
}

pub fn collect_usage_files_for_module(
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
