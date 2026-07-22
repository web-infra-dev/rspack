use std::{
  collections::BTreeMap,
  path::{Path, PathBuf},
  sync::Arc,
};

use regex::Regex;
use rspack_core::{
  Compilation, CompilationAsset, CompilerFinishMake, DependenciesBlock, Plugin,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;
use serde::Serialize;

use super::consume_shared_plugin::ConsumeOptions;
use crate::{ShareScope, SharedIdentity};

const DEFAULT_FILENAME: &str = "collect-shared-entries.json";

#[derive(Debug, Serialize)]
struct CollectSharedEntryVariant {
  #[serde(rename = "shareScope")]
  share_scope: ShareScope,
  #[serde(skip_serializing_if = "Option::is_none")]
  layer: Option<String>,
  requests: Vec<[String; 2]>,
}

#[derive(Debug, Serialize)]
struct CollectSharedEntryAssetItem {
  #[serde(rename = "shareScope")]
  share_scope: ShareScope,
  requests: Vec<[String; 2]>,
  #[serde(skip_serializing_if = "Option::is_none")]
  variants: Option<Vec<CollectSharedEntryVariant>>,
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

#[plugin_hook(CompilerFinishMake for CollectSharedEntryPlugin, stage = 100)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  // Traverse ConsumeSharedModule in the graph and collect real resolved module paths from fallback
  let module_graph = compilation.get_module_graph();
  let mut ordered_requests: FxHashMap<SharedIdentity, Vec<[String; 2]>> = FxHashMap::default();

  for (_id, module) in module_graph.modules() {
    let module_type = module.module_type();
    if !matches!(
      module_type,
      rspack_core::ModuleType::ConsumeShared | rspack_core::ModuleType::ProvideShared
    ) {
      continue;
    }

    let share_info = match module_type {
      rspack_core::ModuleType::ConsumeShared => {
        let Some(consume) = module
          .as_any()
          .downcast_ref::<super::consume_shared_module::ConsumeSharedModule>()
        else {
          continue;
        };
        (
          consume.shared_identity(),
          consume.get_dependencies(),
          consume.get_blocks(),
          None,
        )
      }
      rspack_core::ModuleType::ProvideShared => {
        let Some(provide) = module
          .as_any()
          .downcast_ref::<super::provide_shared_module::ProvideSharedModule>()
        else {
          continue;
        };
        (
          provide.shared_identity(),
          provide.get_dependencies(),
          provide.get_blocks(),
          provide.version().map(str::to_string),
        )
      }
      _ => continue,
    };

    let (identity, dependencies, blocks, provided_version) = share_info;
    if identity.share_key.is_empty() || identity.share_scope.is_empty() {
      continue;
    }

    // Collect target modules from dependencies and async blocks
    let mut target_modules = Vec::new();
    for dep_id in dependencies {
      if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
        target_modules.push(*target_id);
      }
    }
    for block_id in blocks {
      if let Some(block) = module_graph.block_by_id(block_id) {
        for dep_id in block.get_dependencies() {
          if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
            target_modules.push(*target_id);
          }
        }
      }
    }

    // Add real module resource paths to the map and infer version
    let mut reqs = ordered_requests.remove(&identity).unwrap_or_default();
    for target_id in target_modules {
      if let Some(target) = module_graph.module_by_identifier(&target_id)
        && let Some(name) = target.name_for_condition()
      {
        let resource: String = name.into();
        let version = match &provided_version {
          Some(version) => version.clone(),
          None => self
            .infer_version(&resource)
            .await
            .unwrap_or_else(String::new),
        };
        let pair = [resource, version];
        if !reqs.iter().any(|p| p[0] == pair[0] && p[1] == pair[1]) {
          reqs.push(pair);
        }
      }
    }
    reqs.sort_by(|a, b| a[0].cmp(&b[0]).then(a[1].cmp(&b[1])));
    ordered_requests.insert(identity, reqs);
  }

  // Build asset content
  let mut shared_variants: BTreeMap<String, Vec<CollectSharedEntryVariant>> = BTreeMap::new();
  for (identity, requests) in ordered_requests {
    shared_variants
      .entry(identity.share_key)
      .or_default()
      .push(CollectSharedEntryVariant {
        share_scope: identity.share_scope,
        layer: identity.layer,
        requests,
      });
  }
  let shared = shared_variants
    .into_iter()
    .filter_map(|(share_key, mut variants)| {
      variants.sort_by(|a, b| {
        a.layer.cmp(&b.layer).then_with(|| {
          a.share_scope
            .identifier_key()
            .cmp(&b.share_scope.identifier_key())
        })
      });
      let preferred = variants
        .iter()
        .find(|entry| entry.layer.is_none())
        .or_else(|| variants.first())?;
      let share_scope = preferred.share_scope.clone();
      let requests = preferred.requests.clone();
      let variants = (variants.len() != 1 || variants[0].layer.is_some()).then_some(variants);
      Some((
        share_key,
        CollectSharedEntryAssetItem {
          share_scope,
          requests,
          variants,
        },
      ))
    })
    .collect::<BTreeMap<_, _>>();

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
    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));
    Ok(())
  }
}
