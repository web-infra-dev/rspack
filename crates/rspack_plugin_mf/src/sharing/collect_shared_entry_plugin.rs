use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use regex::Regex;
use rspack_core::{
  Compilation, CompilationAsset, CompilerFinishMake, DependenciesBlock, NormalModule, Plugin,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer, ser::SerializeMap};

use super::{
  consume_shared_plugin::ConsumeOptions, provide_shared_dependency::ProvideSharedDependency,
};
use crate::{ShareScope, SharedIdentity};

const DEFAULT_FILENAME: &str = "collect-shared-entries.json";

#[derive(Debug, Serialize)]
struct CollectSharedEntryVariant {
  #[serde(rename = "shareScope")]
  share_scope: ShareScope,
  #[serde(skip_serializing_if = "Option::is_none")]
  layer: Option<String>,
  requests: Vec<[String; 2]>,
  #[serde(rename = "requestOrigins", skip_serializing_if = "Vec::is_empty")]
  request_origins: Vec<[String; 3]>,
}

#[derive(Debug, Serialize)]
struct CollectSharedEntryAssetItem<'a> {
  #[serde(rename = "shareScope")]
  share_scope: &'a ShareScope,
  requests: &'a [[String; 2]],
  #[serde(rename = "requestOrigins", skip_serializing_if = "Option::is_none")]
  request_origins: Option<&'a [[String; 3]]>,
  #[serde(skip_serializing_if = "Option::is_none")]
  variants: Option<&'a [CollectSharedEntryVariant]>,
}

struct CollectSharedEntries<'a>(&'a [(&'a str, CollectSharedEntryAssetItem<'a>)]);

impl Serialize for CollectSharedEntries<'_> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    let mut map = serializer.serialize_map(Some(self.0.len()))?;
    for (share_key, entry) in self.0 {
      map.serialize_entry(share_key, entry)?;
    }
    map.end()
  }
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
  let mut ordered_requests: FxHashMap<SharedIdentity, CollectSharedEntryVariant> =
    FxHashMap::default();

  // Provider versions are authoritative for the same resolved import. Collect
  // them before consumes, whose fallback versions are inferred from packages.
  let mut modules = module_graph
    .modules()
    .filter(|(_, module)| {
      matches!(
        module.module_type(),
        rspack_core::ModuleType::ConsumeShared | rspack_core::ModuleType::ProvideShared
      )
    })
    .collect::<Vec<_>>();
  modules.sort_unstable_by_key(|(_, module)| {
    matches!(module.module_type(), rspack_core::ModuleType::ConsumeShared)
  });
  for (id, module) in modules {
    let module_type = module.module_type();
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
          consume.get_dependency_ids(),
          consume.get_blocks(),
          None,
          Vec::new(),
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
          provide.get_dependency_ids(),
          provide.get_blocks(),
          Some(provide.manifest_version().to_string()),
          module_graph
            .get_incoming_connections(id)
            .filter_map(|connection| {
              module_graph
                .dependency_by_id(&connection.dependency_id)
                .downcast_ref::<ProvideSharedDependency>()
                .map(|dependency| dependency.original_request.clone())
            })
            .collect(),
        )
      }
      _ => continue,
    };

    let (identity, dependencies, blocks, provided_version, mut original_requests) = share_info;
    if identity.share_key.is_empty() || identity.share_scope.is_empty() {
      continue;
    }

    // Collect target modules from dependencies and async blocks.
    let mut dependency_ids: Vec<_> = dependencies.copied().collect();
    for block_id in blocks {
      if let Some(block) = module_graph.block_by_id(block_id) {
        dependency_ids.extend(block.get_dependency_ids().copied());
      }
    }
    let mut target_modules = Vec::new();
    for dep_id in dependency_ids {
      if let Some(target_id) = module_graph.module_identifier_by_dependency_id(&dep_id) {
        target_modules.push(*target_id);
        if matches!(module_type, rspack_core::ModuleType::ConsumeShared)
          && let Some(dependency) = module_graph
            .dependency_by_id(&dep_id)
            .as_module_dependency()
        {
          original_requests.push(dependency.request().to_string());
        }
      }
    }

    let entry =
      ordered_requests
        .entry(identity.clone())
        .or_insert_with(|| CollectSharedEntryVariant {
          share_scope: identity.share_scope.clone(),
          layer: identity.layer.clone(),
          requests: Vec::new(),
          request_origins: Vec::new(),
        });
    for target_id in target_modules {
      if let Some(target) = module_graph.module_by_identifier(&target_id)
        && let Some(name) = target.name_for_condition()
      {
        let resource = target.as_any().downcast_ref::<NormalModule>().map_or_else(
          || name.into(),
          |module| module.resource_resolved_data().resource().to_string(),
        );
        let original_requests = original_requests
          .iter()
          .filter(|original_request| {
            provided_version.is_some()
              || !entry
                .request_origins
                .iter()
                .any(|[request, _, import]| request == &resource && import == *original_request)
          })
          .collect::<Vec<_>>();
        if provided_version.is_none() && original_requests.is_empty() {
          continue;
        }
        let version = match &provided_version {
          Some(version) => version.clone(),
          None => self
            .infer_version(&resource)
            .await
            .unwrap_or_else(String::new),
        };
        let pair = [resource, version];
        if !entry.requests.contains(&pair) {
          entry.requests.push(pair.clone());
        }
        for original_request in original_requests {
          let origin = [pair[0].clone(), pair[1].clone(), original_request.clone()];
          if !entry.request_origins.contains(&origin) {
            entry.request_origins.push(origin);
          }
        }
      }
    }
    entry.requests.sort_unstable();
    entry.request_origins.sort_unstable();
  }

  // Build asset content
  let mut shared_variants: FxHashMap<String, Vec<CollectSharedEntryVariant>> = FxHashMap::default();
  for (identity, variant) in ordered_requests {
    shared_variants
      .entry(identity.share_key)
      .or_default()
      .push(variant);
  }
  let mut shared = shared_variants
    .iter_mut()
    .filter_map(|(share_key, variants)| {
      variants.sort_unstable_by(|a, b| {
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
      let variants =
        (variants.len() != 1 || variants[0].layer.is_some()).then_some(variants.as_slice());
      Some((
        share_key.as_str(),
        CollectSharedEntryAssetItem {
          share_scope: &preferred.share_scope,
          requests: &preferred.requests,
          request_origins: (!preferred.request_origins.is_empty())
            .then_some(preferred.request_origins.as_slice()),
          variants,
        },
      ))
    })
    .collect::<Vec<_>>();
  shared.sort_unstable_by(|a, b| a.0.cmp(b.0));

  let json = serde_json::to_string_pretty(&CollectSharedEntries(&shared))
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
