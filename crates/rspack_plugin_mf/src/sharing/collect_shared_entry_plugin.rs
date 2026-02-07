use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use regex::Regex;
use rspack_core::{
  Compilation, CompilationAsset, CompilationProcessAssets, DependenciesBlock, Plugin,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;
use serde::Serialize;

use super::consume_shared_plugin::ConsumeOptions;

const DEFAULT_FILENAME: &str = "collect-shared-entries.json";

#[derive(Debug, Serialize)]
struct CollectSharedEntryAssetItem<'a> {
  #[serde(rename = "shareScope")]
  share_scope: &'a str,
  requests: &'a [[String; 2]],
}

#[derive(Debug)]
pub struct CollectSharedEntryPluginOptions {
  pub consumes: Vec<(String, Arc<ConsumeOptions>)>,
  pub filename: Option<String>,
}

#[plugin]
#[derive(Debug)]
pub struct CollectSharedEntryPlugin {
  options: CollectSharedEntryPluginOptions,
}

impl CollectSharedEntryPlugin {
  pub fn new(options: CollectSharedEntryPluginOptions) -> Self {
    Self::new_inner(options)
  }

  /// Infer package version from a module request path
  /// Example: ../../../.eden-mono/temp/node_modules/.pnpm/react-dom@18.3.1_react@18.3.1/node_modules/react-dom/index.js
  /// It locates react-dom's package.json and reads the version field
  async fn infer_version(&self, request: &str) -> Option<String> {
    // 1) Try pnpm store path pattern: .pnpm/<pkg>@<version>_
    let pnpm_re = Regex::new(r"/\\.pnpm/[^/]*@([^/_]+)").ok();
    if let Some(re) = pnpm_re
      && let Some(caps) = re.captures(request)
      && let Some(m) = caps.get(1)
    {
      return Some(m.as_str().to_string());
    }

    // 2) Fallback: read version from the deepest node_modules/<pkg>/package.json
    let path = Path::new(request);
    let comps: Vec<String> = path
      .components()
      .map(|c| c.as_os_str().to_string_lossy().to_string())
      .collect();
    if let Some(idx) = comps.iter().rposition(|c| c == "node_modules") {
      let mut pkg_parts: Vec<&str> = Vec::new();
      if let Some(next) = comps.get(idx + 1) {
        if next.starts_with('@') {
          if let Some(next2) = comps.get(idx + 2) {
            pkg_parts.push(next.as_str());
            pkg_parts.push(next2.as_str());
          }
        } else {
          pkg_parts.push(next.as_str());
        }
      }
      if !pkg_parts.is_empty() {
        let mut package_json_path = PathBuf::new();
        for c in comps.iter().take(idx + 1) {
          package_json_path.push(c);
        }
        for p in &pkg_parts {
          package_json_path.push(p);
        }
        package_json_path.push("package.json");
        if package_json_path.exists()
          && let Ok(content) = std::fs::read_to_string(&package_json_path)
          && let Ok(json) = serde_json::from_str::<serde_json::Value>(&content)
          && let Some(version) = json.get("version").and_then(|v| v.as_str())
        {
          return Some(version.to_string());
        }
      }
    }

    None
  }
}

#[plugin_hook(CompilationProcessAssets for CollectSharedEntryPlugin)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Traverse ConsumeSharedModule in the graph and collect real resolved module paths from fallback
  let module_graph = compilation.get_module_graph();
  let mut ordered_requests: FxHashMap<String, Vec<[String; 2]>> = FxHashMap::default();
  let mut share_scopes: FxHashMap<String, String> = FxHashMap::default();

  for (_id, module) in module_graph.modules() {
    let module_type = module.module_type();
    if !matches!(module_type, rspack_core::ModuleType::ConsumeShared) {
      continue;
    }

    if let Some(consume) = module
      .as_any()
      .downcast_ref::<super::consume_shared_module::ConsumeSharedModule>()
    {
      let key = consume.share_key();
      let scope = consume.share_scope().join("|");
      if key.is_empty() {
        continue;
      }
      // Collect target modules from dependencies and async blocks
      let mut target_modules = Vec::new();
      for dep_id in consume.get_dependencies() {
        if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
          target_modules.push(*target_id);
        }
      }
      for block_id in consume.get_blocks() {
        if let Some(block) = module_graph.block_by_id(block_id) {
          for dep_id in block.get_dependencies() {
            if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
              target_modules.push(*target_id);
            }
          }
        }
      }

      // Add real module resource paths to the map and infer version
      let mut reqs = ordered_requests.remove(key).unwrap_or_default();
      for target_id in target_modules {
        if let Some(target) = module_graph.module_by_identifier(&target_id)
          && let Some(name) = target.name_for_condition()
        {
          let resource: String = name.into();
          let version = self
            .infer_version(&resource)
            .await
            .unwrap_or_else(String::new);
          let pair = [resource, version];
          if !reqs.iter().any(|p| p[0] == pair[0] && p[1] == pair[1]) {
            reqs.push(pair);
          }
        }
      }
      reqs.sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])));
      ordered_requests.insert(key.to_string(), reqs);
      if !scope.is_empty() {
        share_scopes.insert(key.to_string(), scope);
      }
    }
  }

  // Build asset content
  let mut shared: FxHashMap<&str, CollectSharedEntryAssetItem<'_>> = FxHashMap::default();
  for (share_key, requests) in ordered_requests.iter() {
    let scope = share_scopes.get(share_key).map_or("", |s| s.as_str());
    shared.insert(
      share_key.as_str(),
      CollectSharedEntryAssetItem {
        share_scope: scope,
        requests: requests.as_slice(),
      },
    );
  }

  let json = serde_json::to_string_pretty(&shared)
    .expect("CollectSharedEntryPlugin: failed to serialize share entries");

  // Get filename, or use default when absent
  let filename = self
    .options
    .filename
    .clone()
    .unwrap_or_else(|| DEFAULT_FILENAME.to_string());

  compilation.emit_asset(
    filename,
    CompilationAsset::new(
      Some(RawStringSource::from(json).boxed()),
      Default::default(),
    ),
  );
  Ok(())
}

impl Plugin for CollectSharedEntryPlugin {
  fn name(&self) -> &'static str {
    "rspack.CollectSharedEntryPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    Ok(())
  }
}
