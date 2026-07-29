use std::sync::{Arc, RwLock};

use rspack_core::{
  AsyncDependenciesBlockIdentifier, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationDependencyReferencedExports,
  CompilationOptimizeDependencies, CompilationProcessAssets, DependenciesBlock, Dependency,
  DependencyId, DependencyType, ExportsInfoArtifact, ModuleGraph, ModuleIdentifier, Plugin,
  ReferencedExport, RuntimeGlobals, RuntimeModule, RuntimeModuleExt, RuntimeSpec,
  SideEffectsOptimizeArtifact,
  build_module_graph::BuildModuleGraphArtifact,
  module_declared_side_effect_free,
  rspack_sources::{RawStringSource, SourceExt, SourceValue},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{ESMImportSpecifierDependency, ImportDependency};
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};
use serde_json::Value;

use super::{
  RequestMatchKey, consume_shared_module::ConsumeSharedModule, find_exact_match,
  provide_shared_module::ProvideSharedModule,
  shared_used_exports_optimizer_runtime_module::SharedUsedExportsOptimizerRuntimeModule,
};
use crate::{ShareScope, SharedIdentity, container::container_entry_module::ContainerEntryModule};

fn shared_identity_from_output(
  share_key: &str,
  share_scope: Option<&ShareScope>,
  layer: Option<&str>,
) -> SharedIdentity {
  let default_scope = ShareScope::Single("default".to_string());
  SharedIdentity::new(share_scope.unwrap_or(&default_scope), share_key, layer)
}

fn share_scope_from_json(value: Option<&Value>) -> Option<Option<ShareScope>> {
  match value {
    None => Some(None),
    Some(Value::String(scope)) => Some(Some(ShareScope::Single(scope.clone()))),
    Some(Value::Array(scopes)) => scopes
      .iter()
      .map(|scope| scope.as_str().map(str::to_string))
      .collect::<Option<Vec<_>>>()
      .map(ShareScope::Multiple)
      .map(Some),
    Some(_) => None,
  }
}

#[inline(always)]
fn referenced_exports_for_output<'a>(
  shared_referenced_exports: &'a FxHashMap<SharedIdentity, FxHashSet<String>>,
  share_key: &str,
) -> Option<&'a FxHashSet<String>> {
  let mut matching = None;
  for (identity, exports) in shared_referenced_exports {
    if identity.share_key != share_key {
      continue;
    }
    if matching.replace(exports).is_some() {
      return None;
    }
  }
  matching
}

fn update_shared_exports(
  content: &str,
  shared_referenced_exports: &FxHashMap<SharedIdentity, FxHashSet<String>>,
  update_reference_exports: bool,
) -> Option<String> {
  let mut root = serde_json::from_str::<Value>(content).ok()?;
  for shared in root.get_mut("shared")?.as_array_mut()? {
    let (share_key, share_scope, layer) = {
      let shared = shared.as_object()?;
      let share_key = shared.get("name")?.as_str()?;
      let share_scope = share_scope_from_json(shared.get("shareScope"))?;
      let layer = shared.get("layer").and_then(Value::as_str);
      (share_key, share_scope, layer)
    };
    let exports_set = if share_scope.is_some() || layer.is_some() {
      let identity = shared_identity_from_output(share_key, share_scope.as_ref(), layer);
      shared_referenced_exports.get(&identity)
    } else {
      referenced_exports_for_output(shared_referenced_exports, share_key)
    };
    let Some(exports_set) = exports_set else {
      continue;
    };
    let mut exports = exports_set.iter().cloned().collect::<Vec<_>>();
    exports.sort_unstable();
    let exports = exports.into_iter().map(Value::String).collect::<Vec<_>>();
    let shared = shared.as_object_mut()?;
    shared.insert("usedExports".to_string(), Value::Array(exports.clone()));
    if update_reference_exports {
      shared.insert("referenceExports".to_string(), Value::Array(exports));
    }
  }
  serde_json::to_string_pretty(&root).ok()
}

#[derive(Debug, Clone)]
pub struct OptimizeSharedConfig {
  pub request: String,
  pub issuer_layer: Option<String>,
  pub share_key: String,
  pub share_scope: ShareScope,
  pub layer: Option<String>,
  pub tree_shaking: bool,
  pub used_exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SharedUsedExportsOptimizerPluginOptions {
  pub shared: Vec<OptimizeSharedConfig>,
  pub inject_tree_shaking_used_exports: bool,
  pub stats_file_name: Option<String>,
  pub manifest_file_name: Option<String>,
}

#[derive(Debug, Clone)]
struct SharedEntryData {
  used_exports: Vec<Atom>,
}

#[plugin]
#[derive(Debug, Clone)]
pub struct SharedUsedExportsOptimizerPlugin {
  shared_map: FxHashMap<SharedIdentity, SharedEntryData>,
  request_map: FxHashMap<RequestMatchKey, Vec<SharedIdentity>>,
  shared_referenced_exports: Arc<RwLock<FxHashMap<SharedIdentity, FxHashSet<String>>>>,
  inject_tree_shaking_used_exports: bool,
  stats_file_name: Option<String>,
  manifest_file_name: Option<String>,
}

impl SharedUsedExportsOptimizerPlugin {
  pub fn new(options: SharedUsedExportsOptimizerPluginOptions) -> Self {
    let mut shared_map = FxHashMap::default();
    let mut request_map: FxHashMap<RequestMatchKey, Vec<SharedIdentity>> = FxHashMap::default();
    let inject_tree_shaking_used_exports = options.inject_tree_shaking_used_exports;
    for config in options.shared.into_iter().filter(|c| c.tree_shaking) {
      let atoms = config
        .used_exports
        .into_iter()
        .map(Atom::from)
        .collect::<Vec<_>>();
      let identity = SharedIdentity::new(
        &config.share_scope,
        &config.share_key,
        config.layer.as_deref(),
      );
      let identities = request_map
        .entry(RequestMatchKey::new(
          &config.request,
          config.issuer_layer.as_deref(),
        ))
        .or_default();
      if !identities.contains(&identity) {
        identities.push(identity.clone());
      }
      shared_map.insert(
        identity,
        SharedEntryData {
          used_exports: atoms,
        },
      );
    }

    let shared_referenced_exports = Arc::new(RwLock::new(FxHashMap::<
      SharedIdentity,
      FxHashSet<String>,
    >::default()));

    Self::new_inner(
      shared_map,
      request_map,
      shared_referenced_exports,
      inject_tree_shaking_used_exports,
      options.stats_file_name,
      options.manifest_file_name,
    )
  }

  fn apply_custom_exports(&self) {
    let mut shared_referenced_exports = self
      .shared_referenced_exports
      .write()
      .expect("lock poisoned");
    for (share_key, shared_entry_data) in &self.shared_map {
      let export_set = shared_referenced_exports
        .entry(share_key.clone())
        .or_default();
      for used_export in &shared_entry_data.used_exports {
        export_set.insert(used_export.to_string());
      }
    }
  }
}

fn collect_processed_modules(
  module_graph: &ModuleGraph,
  module_blocks: &[AsyncDependenciesBlockIdentifier],
  module_deps: &[DependencyId],
  out: &mut Vec<ModuleIdentifier>,
) {
  for dep_id in module_deps {
    if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
      out.push(*target_id);
    }
  }

  for block_id in module_blocks {
    if let Some(block) = module_graph.block_by_id(block_id) {
      for dep_id in block.get_dependencies() {
        if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
          out.push(*target_id);
        }
      }
    }
  }
}

#[plugin_hook(
  CompilationOptimizeDependencies for SharedUsedExportsOptimizerPlugin,
  stage = 1
)]
async fn optimize_dependencies(
  &self,
  _compilation: &Compilation,
  _side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  let module_ids: Vec<_> = {
    let module_graph = build_module_graph_artifact.get_module_graph();
    module_graph.modules_keys().copied().collect()
  };
  self.apply_custom_exports();
  for module_id in module_ids {
    let module_graph = build_module_graph_artifact.get_module_graph();
    let share_info = {
      let module = module_graph.module_by_identifier(&module_id);
      module.and_then(|module| {
        let module_type = module.module_type();
        if !matches!(
          module_type,
          rspack_core::ModuleType::ConsumeShared
            | rspack_core::ModuleType::ProvideShared
            | rspack_core::ModuleType::ShareContainerShared
        ) {
          return None;
        }
        let mut modules_to_process = Vec::new();
        let shared_identity = match module_type {
          rspack_core::ModuleType::ConsumeShared => {
            let consume_shared_module = module.as_any().downcast_ref::<ConsumeSharedModule>()?;
            collect_processed_modules(
              module_graph,
              consume_shared_module.get_blocks(),
              consume_shared_module.get_dependencies(),
              &mut modules_to_process,
            );
            consume_shared_module.shared_identity()
          }
          rspack_core::ModuleType::ProvideShared => {
            let provide_shared_module = module.as_any().downcast_ref::<ProvideSharedModule>()?;
            collect_processed_modules(
              module_graph,
              provide_shared_module.get_blocks(),
              provide_shared_module.get_dependencies(),
              &mut modules_to_process,
            );
            provide_shared_module.shared_identity()
          }
          rspack_core::ModuleType::ShareContainerShared => {
            let share_container_entry_module =
              module.as_any().downcast_ref::<ContainerEntryModule>()?;
            collect_processed_modules(
              module_graph,
              share_container_entry_module.get_blocks(),
              share_container_entry_module.get_dependencies(),
              &mut modules_to_process,
            );
            share_container_entry_module.shared_identity()?
          }
          _ => return None,
        };
        Some((shared_identity, modules_to_process))
      })
    };

    let (shared_identity, modules_to_process) = match share_info {
      Some(result) => result,
      None => continue,
    };

    if shared_identity.share_key.is_empty() {
      continue;
    }

    let runtime_reference_exports = {
      self
        .shared_referenced_exports
        .read()
        .expect("lock poisoned")
        .get(&shared_identity)
        .cloned()
    };
    if !self.shared_map.contains_key(&shared_identity) {
      continue;
    }
    let Some(runtime_reference_exports) = runtime_reference_exports else {
      continue;
    };
    if runtime_reference_exports.is_empty() {
      continue;
    }
    let Some(real_shared_identifier) = modules_to_process.first().copied() else {
      continue;
    };
    let is_side_effect_free = module_graph
      .module_by_identifier(&real_shared_identifier)
      .and_then(|module| module_declared_side_effect_free(module.as_ref()))
      .unwrap_or(false);
    if !is_side_effect_free {
      if let Ok(mut shared_referenced_exports) = self.shared_referenced_exports.write()
        && let Some(set) = shared_referenced_exports.get_mut(&shared_identity)
      {
        set.clear();
      }
      continue;
    }

    exports_info_artifact.reset_all_exports_info_used();
    for module_id in &modules_to_process {
      let exports_info_data = exports_info_artifact.get_exports_info_data_mut(module_id);

      for export_name in &runtime_reference_exports {
        let export_atom = Atom::from(export_name.as_str());
        if let Some(export_info) = exports_info_data.named_exports_mut(&export_atom) {
          export_info.set_used(rspack_core::UsageState::Used, None);
        }
      }
    }

    let exports_info_data =
      exports_info_artifact.get_exports_info_data_mut(&real_shared_identifier);
    let exports_view = exports_info_data.exports();
    let can_update_module_used_stage = !exports_view.is_empty()
      && exports_view.iter().all(|(name, export_info)| {
        matches!(
          export_info.get_used(None),
          rspack_core::UsageState::Unknown | rspack_core::UsageState::Unused
        ) || runtime_reference_exports.contains(&name.to_string())
      });
    if !can_update_module_used_stage {
      continue;
    }
    for export_info in exports_info_data.exports_mut().values_mut() {
      export_info.set_used_conditionally(
        |used| *used == rspack_core::UsageState::Unknown,
        rspack_core::UsageState::Unused,
        None,
      );
      export_info.set_can_mangle_provide(Some(false));
      export_info.set_can_mangle_use(Some(false));
    }
  }

  Ok(None)
}

#[plugin_hook(CompilationProcessAssets for SharedUsedExportsOptimizerPlugin, stage = 1)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let shared_referenced_exports = self
    .shared_referenced_exports
    .read()
    .expect("lock poisoned");
  for (file_name, update_reference_exports) in [
    (&self.stats_file_name, false),
    (&self.manifest_file_name, true),
  ] {
    if let Some(file_name) = file_name
      && let Some(file) = compilation.assets().get(file_name)
      && let Some(source) = file.get_source()
      && let SourceValue::String(content) = source.source()
      && let Some(updated_content) = update_shared_exports(
        &content,
        &shared_referenced_exports,
        update_reference_exports,
      )
    {
      compilation.update_asset(file_name, |_, info| {
        Ok((RawStringSource::from(updated_content).boxed(), info))
      })?;
    }
  }

  Ok(())
}

#[plugin_hook(
  CompilationAdditionalTreeRuntimeRequirements for SharedUsedExportsOptimizerPlugin
)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  if self.shared_map.is_empty() {
    return Ok(());
  }

  runtime_requirements.insert(RuntimeGlobals::RUNTIME_ID);
  runtime_modules.push(
    SharedUsedExportsOptimizerRuntimeModule::new(
      &compilation.runtime_template,
      Arc::new(
        self
          .shared_referenced_exports
          .read()
          .expect("lock poisoned")
          .clone(),
      ),
    )
    .boxed(),
  );

  Ok(())
}

#[plugin_hook(CompilationDependencyReferencedExports for SharedUsedExportsOptimizerPlugin,tracing=false)]
fn dependency_referenced_exports(
  &self,
  compilation: &Compilation,
  dependency_id: &DependencyId,
  referenced_exports: &Option<Vec<ReferencedExport>>,
  _runtime: Option<&RuntimeSpec>,
  module_graph: Option<&ModuleGraph>,
) -> Result<()> {
  let module_graph = module_graph.unwrap_or_else(|| compilation.get_module_graph());
  if referenced_exports.is_none() {
    return Ok(());
  }
  let Some(exports) = referenced_exports else {
    return Ok(());
  };

  let dependency = module_graph.dependency_by_id(dependency_id);

  let Some(module_dependency) = dependency.as_module_dependency() else {
    return Ok(());
  };

  let request = module_dependency.request();
  let issuer_layer = module_graph
    .get_parent_module(dependency_id)
    .and_then(|identifier| module_graph.module_by_identifier(identifier))
    .and_then(|module| module.get_layer())
    .map(|layer| layer.as_str());
  let Some(shared_identities) = find_exact_match(&self.request_map, request, issuer_layer).cloned()
  else {
    return Ok(());
  };

  let shared_identities = shared_identities
    .into_iter()
    .filter(|identity| self.shared_map.contains_key(identity))
    .collect::<Vec<_>>();
  if shared_identities.is_empty() {
    return Ok(());
  }
  let mut final_exports = exports.clone();

  // If it's an import dependency and referenced exports indicate "exports object referenced",
  // clear any recorded shared referenced exports for this share key and stop here.
  let is_exports_object = matches!(final_exports.as_slice(), [export] if export.name.is_empty());
  if dependency
    .as_any()
    .downcast_ref::<ImportDependency>()
    .is_some()
    && is_exports_object
  {
    let mut shared_referenced_exports = self
      .shared_referenced_exports
      .write()
      .expect("lock poisoned");
    for shared_identity in &shared_identities {
      shared_referenced_exports.remove(shared_identity);
    }
    return Ok(());
  }
  if (final_exports.is_empty() || is_exports_object)
    && dependency.dependency_type() == &DependencyType::EsmImportSpecifier
    && let Some(esm_dep) = dependency
      .as_any()
      .downcast_ref::<ESMImportSpecifierDependency>()
  {
    let ids: &[Atom] = esm_dep.get_ids(module_graph);
    if ids.is_empty() {
      return Ok(());
    }
    if let Some(first) = ids.first()
      && *first == "default"
    {
      final_exports = esm_dep.get_referenced_exports_in_destructuring(Some(ids));
    } else {
      final_exports = esm_dep.get_referenced_exports(
        module_graph,
        &compilation.module_graph_cache_artifact,
        &compilation.exports_info_artifact,
        _runtime,
      );
    }
  }

  let mut shared_referenced_exports = self
    .shared_referenced_exports
    .write()
    .expect("lock poisoned");
  for shared_identity in shared_identities {
    let export_set = shared_referenced_exports
      .entry(shared_identity)
      .or_default();

    for referenced in &final_exports {
      if referenced.name.is_empty() {
        continue;
      }
      for atom in &referenced.name {
        export_set.insert(atom.to_string());
      }
    }
  }
  Ok(())
}

impl Plugin for SharedUsedExportsOptimizerPlugin {
  fn name(&self) -> &'static str {
    "rspack.sharing.SharedUsedExportsOptimizerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    if self.shared_map.is_empty() {
      return Ok(());
    }
    ctx
      .compilation_hooks
      .dependency_referenced_exports
      .tap(dependency_referenced_exports::new(self));
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    if self.inject_tree_shaking_used_exports {
      ctx
        .compilation_hooks
        .additional_tree_runtime_requirements
        .tap(additional_tree_runtime_requirements::new(self));
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use rustc_hash::{FxHashMap, FxHashSet};

  use super::{
    OptimizeSharedConfig, SharedUsedExportsOptimizerPlugin,
    SharedUsedExportsOptimizerPluginOptions, referenced_exports_for_output, update_shared_exports,
  };
  use crate::{ShareScope, SharedIdentity};

  #[test]
  fn updates_layered_manifest_exports_without_typed_deserialization() {
    let identity = SharedIdentity::new(
      &ShareScope::Single("scope".to_string()),
      "pkg",
      Some("server"),
    );
    let mut referenced_exports = FxHashMap::default();
    referenced_exports.insert(
      identity,
      FxHashSet::from_iter(["named".to_string(), "default".to_string()]),
    );
    let content = r#"{"shared":[{"name":"pkg","shareScope":"scope","layer":"server"}]}"#;

    let updated = update_shared_exports(content, &referenced_exports, true).expect("updated");
    let updated: serde_json::Value = serde_json::from_str(&updated).expect("valid json");
    assert_eq!(
      updated["shared"][0]["usedExports"],
      serde_json::json!(["default", "named"])
    );
    assert_eq!(
      updated["shared"][0]["referenceExports"],
      serde_json::json!(["default", "named"])
    );
  }
  #[test]
  fn optimizer_keeps_same_key_and_layer_separate_by_scope() {
    let plugin = SharedUsedExportsOptimizerPlugin::new(SharedUsedExportsOptimizerPluginOptions {
      shared: vec![
        OptimizeSharedConfig {
          request: "pkg".to_string(),
          issuer_layer: Some("issuer".to_string()),
          share_key: "pkg".to_string(),
          share_scope: ShareScope::Single("scope-a".to_string()),
          layer: Some("server".to_string()),
          tree_shaking: true,
          used_exports: vec!["a".to_string()],
        },
        OptimizeSharedConfig {
          request: "pkg".to_string(),
          issuer_layer: Some("issuer".to_string()),
          share_key: "pkg".to_string(),
          share_scope: ShareScope::Single("scope-b".to_string()),
          layer: Some("server".to_string()),
          tree_shaking: true,
          used_exports: vec!["b".to_string()],
        },
      ],
      inject_tree_shaking_used_exports: true,
      stats_file_name: None,
      manifest_file_name: None,
    });

    assert_eq!(plugin.shared_map.len(), 2);
    assert_eq!(plugin.request_map.len(), 1);
    assert_eq!(plugin.request_map.values().next().map(Vec::len), Some(2));
  }

  #[test]
  fn output_without_identity_metadata_uses_an_unambiguous_shared_entry() {
    let mut exports = FxHashMap::default();
    exports.insert(
      SharedIdentity::new(
        &ShareScope::Single("custom".to_string()),
        "react",
        Some("server"),
      ),
      FxHashSet::from_iter(["use".to_string()]),
    );

    assert!(referenced_exports_for_output(&exports, "react").is_some());
    exports.insert(
      SharedIdentity::new(
        &ShareScope::Single("other".to_string()),
        "react",
        Some("client"),
      ),
      FxHashSet::default(),
    );
    assert!(referenced_exports_for_output(&exports, "react").is_none());
  }
}
