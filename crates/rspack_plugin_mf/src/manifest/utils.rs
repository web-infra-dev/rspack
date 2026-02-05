use std::path::Path;

use rspack_core::{Compilation, ModuleGraph, ModuleIdentifier};
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  data::{StatsAssetsGroup, StatsExpose, StatsRemote, StatsShared},
  options::RemoteAliasTarget,
};

const HOT_UPDATE_SUFFIX: &str = ".hot-update";

pub fn ensure_configured_remotes(
  remote_list: &mut Vec<StatsRemote>,
  remote_alias_map: &std::collections::HashMap<String, RemoteAliasTarget>,
  container_name: &str,
) {
  for (alias, target) in remote_alias_map {
    if !remote_list.iter().any(|r| r.alias == *alias) {
      let remote_container_name = if target.name.is_empty() {
        alias.clone()
      } else {
        target.name.clone()
      };
      remote_list.push(StatsRemote {
        alias: alias.clone(),
        consumingFederationContainerName: container_name.to_string(),
        federationContainerName: remote_container_name,
        moduleName: ".".to_string(),
        entry: target.entry.clone(),
        usedIn: vec!["UNKNOWN".to_string()],
      });
    }
  }
}

pub fn collect_entry_files(compilation: &Compilation, container_name: &str) -> HashSet<String> {
  let mut entry_files = HashSet::default();
  for (name, entrypoint_ukey) in &compilation.build_chunk_graph_artifact.entrypoints {
    if name == container_name {
      continue;
    }
    let entrypoint = compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(entrypoint_ukey);
    for chunk_ukey in &entrypoint.chunks {
      if let Some(chunk) = compilation
        .build_chunk_graph_artifact
        .chunk_by_ukey
        .get(chunk_ukey)
      {
        for file in chunk.files() {
          entry_files.insert(file.clone());
        }
        for async_chunk_ukey in
          chunk.get_all_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
        {
          if let Some(async_chunk) = compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .get(&async_chunk_ukey)
          {
            let mut should_filter = false;
            if let Some(chunk_name) = async_chunk.name()
              && chunk_name.contains(name)
            {
              should_filter = true;
            }
            if !should_filter {
              for file in async_chunk.files() {
                if file.contains(name) {
                  should_filter = true;
                  break;
                }
              }
            }
            if should_filter {
              for file in async_chunk.files() {
                entry_files.insert(file.clone());
              }
            }
          }
        }
      }
    }
    let runtime_chunk_ukey =
      entrypoint.get_runtime_chunk(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
    if let Some(chunk) = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get(&runtime_chunk_ukey)
    {
      for file in chunk.files() {
        entry_files.insert(file.clone());
      }
    }
  }
  entry_files
}

pub fn filter_assets(
  assets: &mut StatsAssetsGroup,
  entry_files: &HashSet<String>,
  shared_asset_files: &HashSet<String>,
  remove_shared: bool,
) {
  let filter_fn =
    |asset: &String| !entry_files.contains(asset) || shared_asset_files.contains(asset);

  assets.js.sync.retain(filter_fn);
  assets.js.r#async.retain(filter_fn);
  assets.css.sync.retain(filter_fn);
  assets.css.r#async.retain(filter_fn);

  if remove_shared {
    let filter_shared = |asset: &String| !shared_asset_files.contains(asset);
    assets.js.sync.retain(filter_shared);
    assets.js.r#async.retain(filter_shared);
    assets.css.sync.retain(filter_shared);
    assets.css.r#async.retain(filter_shared);

    // Remove async assets that are already in sync
    let sync_js: HashSet<_> = assets.js.sync.iter().cloned().collect();
    assets.js.r#async.retain(|asset| !sync_js.contains(asset));

    let sync_css: HashSet<_> = assets.css.sync.iter().cloned().collect();
    assets.css.r#async.retain(|asset| !sync_css.contains(asset));
  }
}

pub fn compose_id_with_separator(container: &str, name: &str) -> String {
  format!("{container}:{name}")
}

pub fn is_hot_file(file: &str) -> bool {
  file.contains(HOT_UPDATE_SUFFIX)
}

pub fn strip_ext(path: &str) -> String {
  match Path::new(path).extension() {
    Some(_) => path
      .trim_end_matches(
        Path::new(path)
          .extension()
          .and_then(|e| e.to_str())
          .map(|e| format!(".{e}"))
          .unwrap_or_default()
          .as_str(),
      )
      .to_string(),
    None => path.to_string(),
  }
}

pub fn ensure_shared_entry<'a>(
  shared_map: &'a mut HashMap<String, StatsShared>,
  container_name: &str,
  pkg: &str,
) -> &'a mut StatsShared {
  shared_map
    .entry(pkg.to_string())
    .or_insert_with(|| StatsShared {
      id: compose_id_with_separator(container_name, pkg),
      name: pkg.to_string(),
      version: String::new(),
      requiredVersion: None,
      // default singleton to true
      singleton: Some(true),
      assets: super::data::StatsAssetsGroup::default(),
      usedIn: Vec::new(),
      usedExports: Vec::new(),
    })
}

pub fn record_shared_usage(
  shared_usage_links: &mut Vec<(String, String)>,
  pkg: &str,
  module_identifier: &ModuleIdentifier,
  module_graph: &ModuleGraph,
  compilation: &Compilation,
) {
  fn strip_aggregate_suffix(s: &str) -> String {
    if let Some((before, _)) = s.split_once(" + ") {
      before.to_string()
    } else {
      s.to_string()
    }
  }
  if let Some(issuer_module) = module_graph.get_issuer(module_identifier) {
    let issuer_name = issuer_module
      .readable_identifier(&compilation.options.context)
      .to_string();
    if !issuer_name.is_empty() {
      let key = strip_ext(&strip_aggregate_suffix(&issuer_name));
      shared_usage_links.push((pkg.to_string(), key));
    }
  }
  if let Some(mgm) = module_graph.module_graph_module_by_identifier(module_identifier) {
    for dep_id in mgm.incoming_connections() {
      let Some(connection) = module_graph.connection_by_dependency_id(dep_id) else {
        continue;
      };
      let dependency = module_graph.dependency_by_id(&connection.dependency_id);
      let maybe_request = dependency
        .as_module_dependency()
        .map(|dep| dep.user_request().to_string())
        .or_else(|| {
          dependency
            .as_context_dependency()
            .map(|dep| dep.request().to_string())
        });
      if let Some(request) = maybe_request {
        let key = strip_ext(&strip_aggregate_suffix(&request));
        shared_usage_links.push((pkg.to_string(), key));
      }
    }
  }
}

pub fn parse_provide_shared_identifier(identifier: &str) -> Option<(String, String)> {
  let (before_request, _) = identifier.split_once(" = ")?;
  let token = before_request.split_whitespace().last()?;
  // For scoped packages like @scope/pkg@1.0.0, split at the LAST '@'
  let (name, version) = token.rsplit_once('@')?;
  Some((name.to_string(), version.to_string()))
}

pub fn parse_consume_shared_identifier(identifier: &str) -> Option<(String, Option<String>)> {
  let (_, rest) = identifier.split_once(") ")?;
  let token = rest.split_whitespace().next()?;
  // For scoped packages like @scope/pkg@1.0.0, split at the LAST '@'
  let (name, version) = token.rsplit_once('@')?;
  let version = version.trim();
  let required = if version.is_empty() || version == "*" {
    None
  } else {
    Some(version.to_string())
  };
  Some((name.to_string(), required))
}

pub fn collect_expose_requirements(
  shared_map: &mut HashMap<String, StatsShared>,
  exposes_map: &mut HashMap<String, StatsExpose>,
  links: Vec<(String, String)>,
  expose_module_paths: &HashMap<String, String>,
) {
  #[cfg(debug_assertions)]
  for (pkg, expose_key) in links {
    if let Some(expose) = exposes_map.get_mut(&expose_key) {
      if !expose.requires.contains(&pkg) {
        expose.requires.push(pkg.clone());
      }
      if let Some(shared) = shared_map.get_mut(&pkg) {
        let target = expose_module_paths
          .get(&expose_key)
          .cloned()
          .unwrap_or_else(|| expose.path.clone());
        shared.usedIn.push(target);
      }
    }
  }
}
