use std::path::Path;

use rspack_core::{Compilation, ModuleGraph, ModuleIdentifier};
use rspack_util::fx_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::{
  data::{StatsAssetsGroup, StatsExpose, StatsRemote, StatsShared, StatsSharedRequirement},
  options::RemoteAliasTarget,
};
use crate::{ShareScope, SharedIdentity};

const HOT_UPDATE_SUFFIX: &str = ".hot-update";

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
    })
}

pub(crate) fn manifest_share_scope(identity: &SharedIdentity) -> Option<ShareScope> {
  match &identity.share_scope {
    ShareScope::Single(scope) if scope == "default" => None,
    share_scope => Some(share_scope.clone()),
  }
}

pub fn record_shared_usage(
  shared_usage_links: &mut Vec<(SharedIdentity, String, Option<String>)>,
  identity: &SharedIdentity,
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
  let issuer_layer = module_graph
    .get_issuer(module_identifier)
    .and_then(|module| module.get_layer().cloned());
  if let Some(issuer_module) = module_graph.get_issuer(module_identifier) {
    let issuer_name = issuer_module
      .readable_identifier(&compilation.options.context)
      .to_string();
    if !issuer_name.is_empty() {
      let key = strip_ext(&strip_aggregate_suffix(&issuer_name));
      shared_usage_links.push((identity.clone(), key, issuer_layer.clone()));
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
        let connection_issuer_layer = connection
          .original_module_identifier
          .or(connection.resolved_original_module_identifier)
          .and_then(|identifier| module_graph.module_by_identifier(&identifier))
          .and_then(|module| module.get_layer().cloned())
          .or_else(|| issuer_layer.clone());
        shared_usage_links.push((identity.clone(), key, connection_issuer_layer));
      }
    }
  }
}

pub fn collect_expose_requirements(
  shared_map: &mut HashMap<SharedIdentity, StatsShared>,
  exposes_map: &mut HashMap<ExposeIdentity, StatsExpose>,
  links: Vec<(SharedIdentity, String, Option<String>)>,
  expose_identities_by_import: &HashMap<String, Vec<ExposeIdentity>>,
  expose_module_paths: &HashMap<ExposeIdentity, String>,
) {
  for (identity, expose_import, issuer_layer) in links {
    let identity_count = shared_map
      .keys()
      .filter(|candidate| candidate.share_key == identity.share_key)
      .count();
    let Some(shared) = shared_map.get_mut(&identity) else {
      continue;
    };
    let Some(expose_identities) = expose_identities_by_import.get(&expose_import) else {
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
      if let Some(issuer_layer) = issuer_layer.as_deref()
        && expose_identity
          .layer
          .as_deref()
          .is_some_and(|layer| layer != issuer_layer)
      {
        continue;
      }
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
        .get(expose_identity)
        .cloned()
        .unwrap_or_else(|| expose.path.clone());
      shared.usedIn.push(target);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{
    ExposeIdentity, HashMap, collect_expose_requirements, compose_shared_id, finalize_shared_ids,
  };
  use crate::{
    ShareScope, SharedIdentity,
    manifest::data::{StatsAssetsGroup, StatsExpose, StatsShared},
  };

  fn stats_shared(name: &str, share_scope: Option<ShareScope>) -> StatsShared {
    StatsShared {
      id: String::new(),
      identity_id: None,
      name: name.to_string(),
      version: String::new(),
      requiredVersion: None,
      layer: None,
      share_scope,
      singleton: Some(true),
      assets: StatsAssetsGroup::default(),
      usedIn: Vec::new(),
      usedExports: Vec::new(),
    }
  }

  #[test]
  fn shared_ids_preserve_legacy_scalar_and_distinguish_new_identities() {
    let default = SharedIdentity::new(&ShareScope::Single("default".to_string()), "react", None);
    let scoped = SharedIdentity::new(&ShareScope::Single("server".to_string()), "react", None);
    let layered = SharedIdentity::new(
      &ShareScope::Single("default".to_string()),
      "react",
      Some("server"),
    );

    assert_eq!(compose_shared_id("app", &default), "app:react");
    assert_eq!(compose_shared_id("app", &scoped), "app:react");
    assert_ne!(
      compose_shared_id("app", &scoped),
      compose_shared_id("app", &layered)
    );
  }

  #[test]
  fn scalar_scope_collisions_keep_legacy_ids_and_add_structural_identity_ids() {
    let mut shared = vec![
      stats_shared("react", Some(ShareScope::Single("client".to_string()))),
      stats_shared("react", Some(ShareScope::Single("server".to_string()))),
      stats_shared("vue", Some(ShareScope::Single("server".to_string()))),
    ];

    finalize_shared_ids(&mut shared, "app");

    assert_eq!(shared[0].id, "app:react");
    assert_eq!(shared[1].id, "app:react");
    assert_ne!(shared[0].identity_id, shared[1].identity_id);
    assert!(
      shared[0]
        .identity_id
        .as_deref()
        .is_some_and(|id| id.starts_with("app:shared:"))
    );
    assert!(
      shared[1]
        .identity_id
        .as_deref()
        .is_some_and(|id| id.starts_with("app:shared:"))
    );
    assert_eq!(shared[2].id, "app:vue");
    assert_eq!(shared[2].identity_id, None);
  }

  #[test]
  fn expose_requirements_follow_the_consuming_issuer_layer() {
    let shared_identity = SharedIdentity::new(
      &ShareScope::Single("default".to_string()),
      "react",
      Some("server"),
    );
    let mut shared_map = HashMap::default();
    shared_map.insert(
      shared_identity.clone(),
      stats_shared("react", Some(ShareScope::Single("default".to_string()))),
    );
    let client_expose = ExposeIdentity::new("./entry", Some("client"));
    let server_expose = ExposeIdentity::new("./entry", Some("server"));
    let expose = |layer: &str| StatsExpose {
      path: "./entry".to_string(),
      file: String::new(),
      id: String::new(),
      name: "entry".to_string(),
      layer: Some(layer.to_string()),
      requires: Vec::new(),
      required_shared: Vec::new(),
      assets: StatsAssetsGroup::default(),
    };
    let mut exposes_map = HashMap::from_iter([
      (client_expose.clone(), expose("client")),
      (server_expose.clone(), expose("server")),
    ]);
    let expose_identities_by_import = HashMap::from_iter([(
      "entry".to_string(),
      vec![client_expose.clone(), server_expose.clone()],
    )]);

    collect_expose_requirements(
      &mut shared_map,
      &mut exposes_map,
      vec![(
        shared_identity,
        "entry".to_string(),
        Some("client".to_string()),
      )],
      &expose_identities_by_import,
      &HashMap::default(),
    );

    assert_eq!(exposes_map[&client_expose].requires, ["react"]);
    assert!(exposes_map[&server_expose].requires.is_empty());
  }

  #[test]
  fn unlayered_expose_accepts_a_rule_layered_issuer() {
    let shared_identity =
      SharedIdentity::new(&ShareScope::Single("default".to_string()), "react", None);
    let mut shared_map = HashMap::default();
    shared_map.insert(
      shared_identity.clone(),
      stats_shared("react", Some(ShareScope::Single("default".to_string()))),
    );
    let expose_identity = ExposeIdentity::new("./fallback", None);
    let layered_expose_identity = ExposeIdentity::new("./server", Some("server"));
    let expose = |path: &str, layer: Option<&str>| StatsExpose {
      path: path.to_string(),
      file: String::new(),
      id: String::new(),
      name: path.trim_start_matches("./").to_string(),
      layer: layer.map(str::to_string),
      requires: Vec::new(),
      required_shared: Vec::new(),
      assets: StatsAssetsGroup::default(),
    };
    let mut exposes_map = HashMap::from_iter([
      (expose_identity.clone(), expose("./fallback", None)),
      (
        layered_expose_identity.clone(),
        expose("./server", Some("server")),
      ),
    ]);
    let expose_identities_by_import = HashMap::from_iter([(
      "entry".to_string(),
      vec![expose_identity.clone(), layered_expose_identity.clone()],
    )]);

    collect_expose_requirements(
      &mut shared_map,
      &mut exposes_map,
      vec![(
        shared_identity,
        "entry".to_string(),
        Some("server".to_string()),
      )],
      &expose_identities_by_import,
      &HashMap::default(),
    );

    assert_eq!(exposes_map[&expose_identity].requires, ["react"]);
    assert_eq!(exposes_map[&layered_expose_identity].requires, ["react"]);
    assert_eq!(
      shared_map[&SharedIdentity::new(&ShareScope::Single("default".to_string()), "react", None,)]
        .usedIn,
      ["./fallback", "./server"]
    );
  }
}
