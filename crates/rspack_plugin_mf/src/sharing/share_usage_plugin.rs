use std::{
  collections::{HashMap, HashSet},
  path::PathBuf,
};

use async_trait::async_trait;
use rspack_core::{
  ApplyContext, AssetInfo, Compilation, CompilationAfterProcessAssets, CompilationAsset,
  CompilationOptimizeDependencies, DependenciesBlock, DependencyType, ExtendedReferencedExport,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, Plugin,
  rspack_sources::{RawSource, SourceExt},
};
use rspack_error::{Error, Result};
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageReport {
  #[serde(rename = "treeShake")]
  pub tree_shake: HashMap<String, HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MergeStrategy {
  Union,
  Intersection,
  Override,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUsageModuleData {
  #[serde(rename = "preservedExports")]
  pub preserved_exports: PreservedExports,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub source: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub priority: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub conditions: Option<PreservationConditions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PreservedExports {
  All(String), // "*"
  List(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservationConditions {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub remotes: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub environments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUsageSettings {
  #[serde(
    rename = "defaultPreservation",
    skip_serializing_if = "Option::is_none"
  )]
  pub default_preservation: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalUsageData {
  pub version: String,
  pub modules: HashMap<String, ExternalUsageModuleData>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub settings: Option<ExternalUsageSettings>,
}

#[derive(Debug, Clone)]
pub struct ExternalUsageConfig {
  pub sources: Vec<PathBuf>,
  pub inline: Option<ShareUsageReport>,
}

#[derive(Debug)]
pub struct ShareUsagePluginOptions {
  pub filename: String,
  pub external_usage: Option<ExternalUsageConfig>,
}

impl Default for ShareUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "share-usage.json".to_string(),
      external_usage: None,
    }
  }
}

#[plugin]
#[derive(Debug)]
pub struct ShareUsagePlugin {
  options: ShareUsagePluginOptions,
}

impl ShareUsagePlugin {
  pub fn new(options: ShareUsagePluginOptions) -> Self {
    Self::new_inner(options)
  }

  fn load_external_usage(
    &self,
    _compilation: &Compilation,
  ) -> Result<HashMap<String, HashMap<String, bool>>> {
    let mut merged_usage = HashMap::new();

    // Load external usage from configuration options
    if let Some(external_config) = &self.options.external_usage {
      // Load from inline data first
      if let Some(inline_data) = &external_config.inline {
        merged_usage.extend(inline_data.tree_shake.clone());
      }

      // Load from external files specified in config
      for source in &external_config.sources {
        if source.exists() {
          let content = std::fs::read_to_string(source)
            .map_err(|e| Error::msg(format!("Failed to read external usage file: {e}")))?;

          if let Ok(external_report) = serde_json::from_str::<ShareUsageReport>(&content) {
            // Merge the tree_shake data from the external share-usage.json
            for (share_key, external_exports) in external_report.tree_shake {
              match merged_usage.entry(share_key) {
                std::collections::hash_map::Entry::Occupied(mut entry) => {
                  let existing = entry.get_mut();
                  for (export_name, should_preserve) in external_exports {
                    // True always wins - preserve if ANY source needs it
                    if should_preserve {
                      existing.insert(export_name, true);
                    }
                  }
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                  entry.insert(external_exports);
                }
              }
            }
          }
        }
      }
    }

    Ok(merged_usage)
  }

  fn merge_external_usage(
    &self,
    target: &mut HashMap<String, ExternalUsageModuleData>,
    source: HashMap<String, ExternalUsageModuleData>,
    strategy: &MergeStrategy,
  ) {
    // Merges external usage data from multiple sources
    // Note: When converted to final usage map, true values always win over false
    for (key, source_data) in source {
      match target.entry(key) {
        std::collections::hash_map::Entry::Occupied(mut entry) => {
          let existing = entry.get_mut();
          match strategy {
            MergeStrategy::Union => {
              // Merge preserved exports
              let merged_exports = self.merge_preserved_exports(
                &existing.preserved_exports,
                &source_data.preserved_exports,
                true, // union
              );
              existing.preserved_exports = merged_exports;

              // Use higher priority
              if let Some(source_priority) = source_data.priority {
                if existing.priority.map_or(true, |p| source_priority > p) {
                  existing.priority = Some(source_priority);
                  existing.source = source_data.source;
                }
              }
            }
            MergeStrategy::Intersection => {
              // Keep only common exports
              let merged_exports = self.merge_preserved_exports(
                &existing.preserved_exports,
                &source_data.preserved_exports,
                false, // intersection
              );
              existing.preserved_exports = merged_exports;
            }
            MergeStrategy::Override => {
              // Replace with source data if priority is higher
              if source_data.priority.unwrap_or(0) >= existing.priority.unwrap_or(0) {
                *existing = source_data;
              }
            }
          }
        }
        std::collections::hash_map::Entry::Vacant(entry) => {
          entry.insert(source_data);
        }
      }
    }
  }

  fn merge_preserved_exports(
    &self,
    a: &PreservedExports,
    b: &PreservedExports,
    union: bool,
  ) -> PreservedExports {
    match (a, b) {
      (PreservedExports::All(_), _) | (_, PreservedExports::All(_)) => {
        PreservedExports::All("*".to_string())
      }
      (PreservedExports::List(list_a), PreservedExports::List(list_b)) => {
        let set_a: HashSet<_> = list_a.iter().cloned().collect();
        let set_b: HashSet<_> = list_b.iter().cloned().collect();

        let result = if union {
          set_a.union(&set_b).cloned().collect()
        } else {
          set_a.intersection(&set_b).cloned().collect()
        };

        PreservedExports::List(result)
      }
    }
  }

  fn analyze_consume_shared_usage(
    &self,
    compilation: &Compilation,
  ) -> HashMap<String, HashMap<String, bool>> {
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    // First, try to find ConsumeShared modules (for consumer apps)
    for module_id in module_graph.modules().keys() {
      if let Some(module) = module_graph.module_by_identifier(module_id)
        && module.module_type() == &ModuleType::ConsumeShared
        && let Some(share_key) = module.get_consume_shared_key()
      {
        let exports_usage =
          if let Some(fallback_id) = self.find_fallback_module_id(&module_graph, module_id) {
            let (used_exports, provided_exports) =
              self.analyze_module_usage(&module_graph, &fallback_id, module_id);

            // Build usage map from exports
            let mut usage = HashMap::new();
            for export in provided_exports {
              usage.insert(export.clone(), used_exports.contains(&export));
            }
            usage
          } else {
            HashMap::new()
          };

        usage_map.insert(share_key.to_string(), exports_usage);
      }
    }

    // If no ConsumeShared modules found, look for shared modules being exposed
    if usage_map.is_empty() {
      usage_map = self.analyze_shared_module_usage(compilation);
    }

    usage_map
  }

  fn analyze_shared_module_usage(
    &self,
    compilation: &Compilation,
  ) -> HashMap<String, HashMap<String, bool>> {
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    // Look through all modules to find ones that are being shared
    for module_id in module_graph.modules().keys() {
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        // Check if this module is being shared by looking at its usage
        let module_identifier = module.identifier().to_string();

        // For now, we'll assume the share key is "module" based on the config
        // In a real implementation, we would need to map from the shared config
        if module_identifier.contains("module.js") {
          let usage = self.analyze_exports_usage(&module_graph, module_id);
          usage_map.insert("module".to_string(), usage);
          break;
        }
      }
    }

    usage_map
  }

  fn analyze_exports_usage(
    &self,
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
  ) -> HashMap<String, bool> {
    use rspack_core::{ExportsInfoGetter, PrefetchExportsInfoMode, ProvidedExports, UsageState};

    let mut usage_map = HashMap::new();

    let exports_info = module_graph.get_exports_info(module_id);
    let prefetched = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    match prefetched.get_provided_exports() {
      ProvidedExports::ProvidedNames(names) => {
        for export_name in names {
          let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
          let export_info_data = prefetched.get_read_only_export_info(&export_atom);
          let usage_state = export_info_data.get_used(None);

          // Mark as used if the usage state indicates it's being used
          let is_used = matches!(
            usage_state,
            UsageState::Used | UsageState::OnlyPropertiesUsed
          );
          usage_map.insert(export_name.to_string(), is_used);
        }
      }
      ProvidedExports::ProvidedAll => {
        // If all exports are provided but we don't know specifics,
        // we can't determine individual usage
        usage_map.insert("*".to_string(), false);
      }
      ProvidedExports::Unknown => {
        // Cannot determine exports statically
      }
    }

    usage_map
  }

  fn analyze_module_usage(
    &self,
    module_graph: &ModuleGraph,
    fallback_id: &ModuleIdentifier,
    consume_shared_id: &ModuleIdentifier,
  ) -> (Vec<String>, Vec<String>) {
    use rspack_core::{ExportsInfoGetter, PrefetchExportsInfoMode, ProvidedExports, UsageState};

    let mut used_exports = Vec::new();
    let mut provided_exports = Vec::new();
    let mut all_imported_exports = HashSet::new();

    let exports_info = module_graph.get_exports_info(fallback_id);
    let prefetched = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    match prefetched.get_provided_exports() {
      ProvidedExports::ProvidedNames(names) => {
        provided_exports = names.iter().map(|n| n.to_string()).collect();

        for export_name in names {
          let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
          let export_info_data = prefetched.get_read_only_export_info(&export_atom);
          let usage = export_info_data.get_used(None);

          if matches!(usage, UsageState::Used | UsageState::OnlyPropertiesUsed)
            && export_name != "*"
          {
            used_exports.push(export_name.to_string());
          }
        }
      }
      ProvidedExports::ProvidedAll => provided_exports = vec!["*".to_string()],
      ProvidedExports::Unknown => {}
    }

    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        let referenced_exports = dependency.get_referenced_exports(
          module_graph,
          &ModuleGraphCacheArtifact::default(),
          None,
        );

        for export_ref in referenced_exports {
          let names = match export_ref {
            ExtendedReferencedExport::Array(names) => {
              names.into_iter().map(|n| n.to_string()).collect::<Vec<_>>()
            }
            ExtendedReferencedExport::Export(export_info) => export_info
              .name
              .into_iter()
              .map(|n| n.to_string())
              .collect::<Vec<_>>(),
          };

          for name in names {
            if !used_exports.contains(&name) {
              used_exports.push(name);
            }
          }
        }

        self.extract_dependency_imports(
          dependency.as_ref(),
          module_graph,
          &mut all_imported_exports,
        );
      }
    }

    for imported in &all_imported_exports {
      if provided_exports.contains(imported) && !used_exports.contains(imported) {
        used_exports.push(imported.clone());
      }
    }

    (used_exports, provided_exports)
  }

  fn extract_dependency_imports(
    &self,
    dependency: &dyn rspack_core::Dependency,
    module_graph: &ModuleGraph,
    imports: &mut HashSet<String>,
  ) {
    let referenced_exports =
      dependency.get_referenced_exports(module_graph, &ModuleGraphCacheArtifact::default(), None);

    let mut found_exports = false;
    for export_ref in referenced_exports {
      found_exports = true;
      let names = match export_ref {
        ExtendedReferencedExport::Array(names) => names,
        ExtendedReferencedExport::Export(export_info) => export_info.name,
      };

      for name in names {
        let name_str = name.to_string();
        if !name_str.is_empty() {
          imports.insert(name_str);
        }
      }
    }

    if !found_exports {
      imports.insert("default".to_string());
    }
  }

  fn find_fallback_module_id(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    if let Some(module) = module_graph.module_by_identifier(consume_shared_id) {
      for dep_id in module.get_dependencies() {
        if let Some(dep) = module_graph.dependency_by_id(dep_id)
          && matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback)
          && let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id)
        {
          return Some(*fallback_id);
        }
      }

      for block_id in module.get_blocks() {
        if let Some(block) = module_graph.block_by_id(block_id) {
          for dep_id in block.get_dependencies() {
            if let Some(dep) = module_graph.dependency_by_id(dep_id)
              && matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback)
              && let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id)
            {
              return Some(*fallback_id);
            }
          }
        }
      }
    }

    None
  }
}

#[plugin_hook(CompilationOptimizeDependencies for ShareUsagePlugin)]
async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  // Step 1: Analyze what THIS application uses from shared modules
  let mut usage_data = self.analyze_consume_shared_usage(compilation);

  // Step 2: Load external usage data - exports that OTHER apps need (don't tree-shake these!)
  let external_usage = self.load_external_usage(compilation)?;

  // Step 3: Merge both sources - the output will contain:
  //   - Exports this app uses (marked as true)
  //   - Exports other apps need (also marked as true, even if unused locally)
  //   - Everything else (marked as false, safe to tree-shake)
  for (share_key, external_exports) in external_usage {
    // Merge with existing usage data - true always wins
    match usage_data.entry(share_key) {
      std::collections::hash_map::Entry::Occupied(mut entry) => {
        let existing = entry.get_mut();
        for (export_name, should_preserve) in external_exports {
          // Only set to true, never overwrite true with false
          if should_preserve {
            existing.insert(export_name, true);
          }
        }
      }
      std::collections::hash_map::Entry::Vacant(entry) => {
        entry.insert(external_exports);
      }
    }
  }

  // Write to context directory so FlagDependencyUsagePlugin can read it
  let context_path = compilation.options.context.as_path();
  let usage_file_path = context_path.join("share-usage.json");

  let report = ShareUsageReport {
    tree_shake: usage_data,
  };

  let content = serde_json::to_string_pretty(&report)
    .map_err(|e| Error::msg(format!("Failed to serialize share usage report: {e}")))?;

  // Write to filesystem for FlagDependencyUsagePlugin to read
  std::fs::write(&usage_file_path, content)
    .map_err(|e| Error::msg(format!("Failed to write share usage file: {e}")))?;

  Ok(None)
}

#[plugin_hook(CompilationAfterProcessAssets for ShareUsagePlugin)]
async fn after_process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  // Generate ONLY the local usage data and emit as asset
  // This represents what THIS application uses from shared modules
  let usage_data = self.analyze_consume_shared_usage(compilation);

  let report = ShareUsageReport {
    tree_shake: usage_data,
  };

  let content = serde_json::to_string_pretty(&report)
    .map_err(|e| Error::msg(format!("Failed to serialize share usage report: {e}")))?;

  let filename = &self.options.filename;

  // Always emit the share-usage.json as a build asset
  compilation.assets_mut().insert(
    filename.clone(),
    CompilationAsset::new(Some(RawSource::from(content).boxed()), AssetInfo::default()),
  );

  Ok(())
}

#[async_trait]
impl Plugin for ShareUsagePlugin {
  fn name(&self) -> &'static str {
    "ShareUsagePlugin"
  }

  fn apply(&self, ctx: &mut ApplyContext) -> Result<()> {
    // Hook into optimize_dependencies to provide data to FlagDependencyUsagePlugin
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));

    // Still generate the report file for debugging/external tools
    ctx
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));
    Ok(())
  }
}
