use std::path::Path;

use rspack_core::{Compilation, ModuleGraph, ModuleIdentifier};
use rspack_util::fx_hash::FxHashMap as HashMap;

use super::data::{StatsExpose, StatsShared};

const HOT_UPDATE_SUFFIX: &str = ".hot-update";

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
      singleton: None,
      assets: super::data::StatsAssetsGroup::default(),
      usedIn: Vec::new(),
    })
}

pub fn record_shared_usage(
  shared_usage_links: &mut Vec<(String, String)>,
  pkg: &str,
  module_identifier: &ModuleIdentifier,
  module_graph: &ModuleGraph,
  compilation: &Compilation,
) {
  if let Some(issuer_module) = module_graph.get_issuer(module_identifier) {
    let issuer_name = issuer_module
      .readable_identifier(&compilation.options.context)
      .to_string();
    if !issuer_name.is_empty() {
      let key = strip_ext(&issuer_name);
      shared_usage_links.push((pkg.to_string(), key));
    }
  }
  if let Some(mgm) = module_graph.module_graph_module_by_identifier(module_identifier) {
    for dep_id in mgm.incoming_connections() {
      let Some(connection) = module_graph.connection_by_dependency_id(dep_id) else {
        continue;
      };
      let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) else {
        continue;
      };
      let maybe_request = dependency
        .as_module_dependency()
        .map(|dep| dep.user_request().to_string())
        .or_else(|| {
          dependency
            .as_context_dependency()
            .map(|dep| dep.request().to_string())
        });
      if let Some(request) = maybe_request {
        let key = strip_ext(&request);
        shared_usage_links.push((pkg.to_string(), key));
      }
    }
  }
}

pub fn parse_provide_shared_identifier(identifier: &str) -> Option<(String, String)> {
  let (before_request, _) = identifier.split_once(" = ")?;
  let token = before_request.split_whitespace().last()?;
  let (name, version) = token.split_once('@')?;
  Some((name.to_string(), version.to_string()))
}

pub fn parse_consume_shared_identifier(identifier: &str) -> Option<(String, Option<String>)> {
  let (_, rest) = identifier.split_once(") ")?;
  let token = rest.split_whitespace().next()?;
  let (name, version) = token.split_once('@')?;
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
