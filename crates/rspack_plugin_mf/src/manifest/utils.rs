use std::path::Path;

use rspack_core::{Compilation, ModuleGraph, ModuleIdentifier};
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  data::{StatsAssetsGroup, StatsExpose, StatsRemote, StatsShared, StatsSharedRequirement},
  options::RemoteAliasTarget,
};
use crate::{ShareScope, SharedIdentity};

const HOT_UPDATE_SUFFIX: &str = ".hot-update";

/// Cloned because one expose participates in multiple import and asset lookup maps.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ExposeIdentity {
  pub(crate) path: String,
  pub(crate) layer: Option<String>,
}

impl ExposeIdentity {
  pub(crate) fn new(path: &str, layer: Option<&str>) -> Self {
    Self {
      path: path.to_string(),
      layer: layer.map(str::to_string),
    }
  }
}

pub fn ensure_configured_remotes(
  remote_list: &mut Vec<StatsRemote>,
  remote_alias_map: &HashMap<String, RemoteAliasTarget>,
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

fn compose_structural_shared_id(container: &str, identity: &SharedIdentity) -> String {
  compose_id_with_separator(container, &format!("shared:{}", identity.identifier_key()))
}

pub fn compose_shared_id(container: &str, identity: &SharedIdentity) -> String {
  if matches!(&identity.share_scope, ShareScope::Single(_)) && identity.layer.is_none() {
    compose_id_with_separator(container, &identity.share_key)
  } else {
    compose_structural_shared_id(container, identity)
  }
}

pub fn finalize_shared_ids(shared: &mut [StatsShared], container_name: &str) {
  let scope_collisions = shared
    .iter()
    .map(|entry| {
      if entry.layer.is_some() {
        return false;
      }
      let share_scope = entry
        .share_scope
        .clone()
        .unwrap_or_else(|| ShareScope::Single("default".to_string()));
      matches!(share_scope, ShareScope::Single(_))
        && shared.iter().any(|candidate| {
          if candidate.layer.is_some() || candidate.name != entry.name {
            return false;
          }
          let candidate_scope = candidate
            .share_scope
            .clone()
            .unwrap_or_else(|| ShareScope::Single("default".to_string()));
          matches!(candidate_scope, ShareScope::Single(_)) && candidate_scope != share_scope
        })
    })
    .collect::<Vec<_>>();

  for (entry, has_scope_collision) in shared.iter_mut().zip(scope_collisions) {
    let share_scope = entry
      .share_scope
      .clone()
      .unwrap_or_else(|| ShareScope::Single("default".to_string()));
    let identity = SharedIdentity::new(&share_scope, &entry.name, entry.layer.as_deref());
    entry.id = compose_shared_id(container_name, &identity);
    entry.identity_id = if has_scope_collision
      && entry.layer.is_none()
      && matches!(share_scope, ShareScope::Single(_))
    {
      Some(compose_structural_shared_id(container_name, &identity))
    } else {
      None
    };
  }
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
  shared_map: &'a mut HashMap<SharedIdentity, StatsShared>,
  identity: &SharedIdentity,
  container_name: &str,
) -> &'a mut StatsShared {
  shared_map
    .entry(identity.clone())
    .or_insert_with(|| StatsShared {
      id: compose_shared_id(container_name, identity),
      identity_id: None,
      name: identity.share_key.clone(),
      version: String::new(),
      requiredVersion: None,
      layer: identity.layer.clone(),
      share_scope: manifest_share_scope(identity),
      // default singleton to true
      singleton: Some(true),
      assets: super::data::StatsAssetsGroup::default(),
      usedIn: Vec::new(),
      usedExports: Vec::new(),
      providers: Vec::new(),
    })
}

pub(crate) fn manifest_share_scope(identity: &SharedIdentity) -> Option<ShareScope> {
  match &identity.share_scope {
    ShareScope::Single(scope) if scope == "default" => None,
    share_scope => Some(share_scope.clone()),
  }
}

pub fn record_shared_usage(
  shared_usage_links: &mut Vec<(SharedIdentity, ModuleIdentifier)>,
  identity: &SharedIdentity,
  module_identifier: &ModuleIdentifier,
  module_graph: &ModuleGraph,
) {
  // A direct expose resolves to the shared module itself, not an ordinary issuer.
  shared_usage_links.push((identity.clone(), *module_identifier));
  if let Some(issuer) = module_graph.get_issuer(module_identifier) {
    shared_usage_links.push((identity.clone(), issuer.identifier()));
  }
  for connection in module_graph.get_incoming_connections(module_identifier) {
    if let Some(issuer) = connection
      .original_module_identifier
      .or(connection.resolved_original_module_identifier)
    {
      shared_usage_links.push((identity.clone(), issuer));
    }
  }
}

pub fn collect_expose_requirements(
  shared_map: &mut HashMap<SharedIdentity, StatsShared>,
  exposes_map: &mut HashMap<ExposeIdentity, StatsExpose>,
  links: Vec<(SharedIdentity, ModuleIdentifier)>,
  expose_identities_by_module: &HashMap<ModuleIdentifier, Vec<ExposeIdentity>>,
  expose_module_paths: &HashMap<ModuleIdentifier, String>,
) {
  for (identity, module_id) in links {
    let identity_count = shared_map
      .keys()
      .filter(|candidate| candidate.share_key == identity.share_key)
      .count();
    let Some(shared) = shared_map.get_mut(&identity) else {
      continue;
    };
    let Some(expose_identities) = expose_identities_by_module.get(&module_id) else {
      continue;
    };
    let required_shared = StatsSharedRequirement {
      name: identity.share_key.clone(),
      layer: identity.layer.clone(),
      share_scope: manifest_share_scope(&identity),
    };
    let emit_structured_requirement = identity_count > 1
      || required_shared.layer.is_some()
      || required_shared.share_scope.is_some();
    for expose_identity in expose_identities {
      let Some(expose) = exposes_map.get_mut(expose_identity) else {
        continue;
      };
      if !expose.requires.contains(&shared.name) {
        expose.requires.push(shared.name.clone());
      }
      if emit_structured_requirement && !expose.required_shared.contains(&required_shared) {
        expose.required_shared.push(required_shared.clone());
      }
      let target = expose_module_paths
        .get(&module_id)
        .cloned()
        .unwrap_or_else(|| expose.path.clone());
      shared.usedIn.push(target);
    }
  }
}
