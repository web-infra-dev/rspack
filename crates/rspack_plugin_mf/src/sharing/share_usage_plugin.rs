use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, ChunkGraph, Compilation, CompilationAfterProcessAssets,
  CompilationAsset, CompilerOptions, DependenciesBlock, DependencyType, ExtendedReferencedExport,
  ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, Plugin, PluginContext,
};
use rspack_error::{Error, Result};
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleModuleExports {
  pub used_exports: Vec<String>,
  pub unused_exports: Vec<String>,
  pub possibly_unused_exports: Vec<String>,
  pub entry_module_id: Option<String>,
}

pub type ShareUsageReport = HashMap<String, SimpleModuleExports>;

#[derive(Debug)]
pub struct ShareUsagePluginOptions {
  pub filename: String,
}

impl Default for ShareUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "share-usage.json".to_string(),
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

  fn analyze_consume_shared_usage(
    &self,
    compilation: &Compilation,
  ) -> HashMap<String, SimpleModuleExports> {
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    for module_id in module_graph.modules().keys() {
      if let Some(module) = module_graph.module_by_identifier(module_id) {
        if module.module_type() == &ModuleType::ConsumeShared {
          if let Some(share_key) = module.get_consume_shared_key() {
            let result =
              if let Some(fallback_id) = self.find_fallback_module_id(&module_graph, module_id) {
                let (used_exports, provided_exports) =
                  self.analyze_module_usage(&module_graph, &fallback_id, module_id);
                let unused_exports = provided_exports
                  .into_iter()
                  .filter(|e| !used_exports.contains(e) && e != "*")
                  .collect();
                let entry_module_id =
                  ChunkGraph::get_module_id(&compilation.module_ids_artifact, fallback_id)
                    .map(|id| id.to_string());

                SimpleModuleExports {
                  used_exports,
                  unused_exports,
                  possibly_unused_exports: Vec::new(),
                  entry_module_id,
                }
              } else {
                SimpleModuleExports {
                  used_exports: Vec::new(),
                  unused_exports: Vec::new(),
                  possibly_unused_exports: Vec::new(),
                  entry_module_id: None,
                }
              };

            usage_map.insert(share_key, result);
          }
        }
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
    let mut all_imported_exports = Vec::new();

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
    imports: &mut Vec<String>,
  ) {
    use rspack_core::DependencyType;

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
        if !imports.contains(&name_str) && !name_str.is_empty() {
          imports.push(name_str);
        }
      }
    }

    match dependency.dependency_type() {
      DependencyType::EsmImportSpecifier
      | DependencyType::EsmExportSpecifier
      | DependencyType::EsmExportImportedSpecifier => {
        let ids = dependency._get_ids(module_graph);
        for id in ids {
          let name = id.to_string();
          if !imports.contains(&name) && !name.is_empty() {
            imports.push(name);
          }
        }
      }
      DependencyType::CjsRequire => {
        if !found_exports {
          let name = "default".to_string();
          if !imports.contains(&name) {
            imports.push(name);
          }
        }
      }
      DependencyType::CjsFullRequire => {
        if !found_exports {
          let name = "default".to_string();
          if !imports.contains(&name) {
            imports.push(name);
          }
        }
      }
      DependencyType::DynamicImport => {
        if !found_exports {
          let name = "default".to_string();
          if !imports.contains(&name) {
            imports.push(name);
          }
        }
      }
      DependencyType::CjsExports => {}
      _ => {
        if !found_exports && dependency.as_module_dependency().is_some() {
          let name = "default".to_string();
          if !imports.contains(&name) {
            imports.push(name);
          }
        }
      }
    }
  }

  fn find_fallback_module_id(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    if let Some(module) = module_graph.module_by_identifier(consume_shared_id) {
      for dep_id in module.get_dependencies() {
        if let Some(dep) = module_graph.dependency_by_id(dep_id) {
          if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
            if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
              return Some(*fallback_id);
            }
          }
        }
      }

      for block_id in module.get_blocks() {
        if let Some(block) = module_graph.block_by_id(block_id) {
          for dep_id in block.get_dependencies() {
            if let Some(dep) = module_graph.dependency_by_id(dep_id) {
              if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
                if let Some(fallback_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
                  return Some(*fallback_id);
                }
              }
            }
          }
        }
      }
    }

    None
  }
}

#[plugin_hook(CompilationAfterProcessAssets for ShareUsagePlugin)]
async fn after_process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let usage_data = self.analyze_consume_shared_usage(compilation);

  let report: ShareUsageReport = usage_data;

  let content = serde_json::to_string_pretty(&report)
    .map_err(|e| Error::msg(format!("Failed to serialize share usage report: {e}")))?;

  let filename = &self.options.filename;

  if compilation.assets().contains_key(filename) {
    let mut counter = 1;
    let mut unique_filename = format!("{filename}.{counter}");
    while compilation.assets().contains_key(&unique_filename) {
      counter += 1;
      unique_filename = format!("{filename}.{counter}");
    }
    compilation.assets_mut().insert(
      unique_filename,
      CompilationAsset::new(Some(RawSource::from(content).boxed()), AssetInfo::default()),
    );
  } else {
    compilation.assets_mut().insert(
      filename.clone(),
      CompilationAsset::new(Some(RawSource::from(content).boxed()), AssetInfo::default()),
    );
  }

  Ok(())
}

#[async_trait]
impl Plugin for ShareUsagePlugin {
  fn name(&self) -> &'static str {
    "ShareUsagePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use serde_json;

  use super::*;

  #[test]
  fn test_simple_module_exports_serialization() {
    let exports = SimpleModuleExports {
      used_exports: vec!["map".to_string(), "filter".to_string()],
      unused_exports: vec!["uniq".to_string(), "debounce".to_string()],
      possibly_unused_exports: vec!["isEmpty".to_string()],
      entry_module_id: Some("123".to_string()),
    };

    let json = serde_json::to_string(&exports).expect("Failed to serialize SimpleModuleExports");
    let parsed: SimpleModuleExports =
      serde_json::from_str(&json).expect("Failed to deserialize SimpleModuleExports");

    assert_eq!(parsed.used_exports, vec!["map", "filter"]);
    assert_eq!(parsed.unused_exports, vec!["uniq", "debounce"]);
    assert_eq!(parsed.possibly_unused_exports, vec!["isEmpty"]);
    assert_eq!(parsed.entry_module_id, Some("123".to_string()));
  }

  #[test]
  fn test_share_usage_report_serialization() {
    let mut report = ShareUsageReport::new();

    report.insert(
      "lodash-es".to_string(),
      SimpleModuleExports {
        used_exports: vec!["map".to_string(), "filter".to_string()],
        unused_exports: vec!["uniq".to_string(), "debounce".to_string()],
        possibly_unused_exports: vec![],
        entry_module_id: Some("42".to_string()),
      },
    );

    report.insert(
      "react".to_string(),
      SimpleModuleExports {
        used_exports: vec!["createElement".to_string()],
        unused_exports: vec![],
        possibly_unused_exports: vec!["useState".to_string()],
        entry_module_id: Some("24".to_string()),
      },
    );

    let json = serde_json::to_string_pretty(&report).expect("Failed to serialize ShareUsageReport");
    let parsed: ShareUsageReport =
      serde_json::from_str(&json).expect("Failed to deserialize ShareUsageReport");

    assert_eq!(parsed.len(), 2);
    assert!(parsed.contains_key("lodash-es"));
    assert!(parsed.contains_key("react"));

    let lodash_data = &parsed["lodash-es"];
    assert_eq!(lodash_data.used_exports, vec!["map", "filter"]);
    assert_eq!(lodash_data.unused_exports, vec!["uniq", "debounce"]);
    assert_eq!(lodash_data.entry_module_id, Some("42".to_string()));

    let react_data = &parsed["react"];
    assert_eq!(react_data.used_exports, vec!["createElement"]);
    assert_eq!(react_data.possibly_unused_exports, vec!["useState"]);
    assert_eq!(react_data.entry_module_id, Some("24".to_string()));
  }

  #[test]
  fn test_share_usage_plugin_options() {
    let default_options = ShareUsagePluginOptions::default();
    assert_eq!(default_options.filename, "share-usage.json");

    let custom_options = ShareUsagePluginOptions {
      filename: "custom-usage.json".to_string(),
    };
    assert_eq!(custom_options.filename, "custom-usage.json");
  }

  #[test]
  fn test_plugin_creation() {
    let plugin = ShareUsagePlugin::new(ShareUsagePluginOptions::default());
    assert_eq!(plugin.name(), "ShareUsagePlugin");
    assert_eq!(plugin.options.filename, "share-usage.json");
  }

  #[test]
  fn test_json_structure_validation() {
    let mut report = ShareUsageReport::new();
    report.insert(
      "test-module".to_string(),
      SimpleModuleExports {
        used_exports: vec!["exportA".to_string()],
        unused_exports: vec!["exportB".to_string()],
        possibly_unused_exports: vec![],
        entry_module_id: Some("123".to_string()),
      },
    );

    let json = serde_json::to_string_pretty(&report).unwrap();

    // Verify it's valid JSON
    let _: serde_json::Value = serde_json::from_str(&json).unwrap();

    // Verify structure contains expected keys
    assert!(json.contains("\"used_exports\""));
    assert!(json.contains("\"unused_exports\""));
    assert!(json.contains("\"possibly_unused_exports\""));
    assert!(json.contains("\"entry_module_id\""));
  }

  #[test]
  fn test_module_id_scenarios() {
    // Test with module ID
    let exports_with_id = SimpleModuleExports {
      used_exports: vec!["map".to_string()],
      unused_exports: vec!["filter".to_string()],
      possibly_unused_exports: vec![],
      entry_module_id: Some("42".to_string()),
    };

    let json = serde_json::to_string(&exports_with_id).unwrap();
    let parsed: SimpleModuleExports = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.entry_module_id, Some("42".to_string()));

    // Test without module ID (fallback scenario)
    let exports_without_id = SimpleModuleExports {
      used_exports: vec![],
      unused_exports: vec![],
      possibly_unused_exports: vec![],
      entry_module_id: None,
    };

    let json = serde_json::to_string(&exports_without_id).unwrap();
    let parsed: SimpleModuleExports = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.entry_module_id, None);
  }

  #[test]
  fn test_comprehensive_data_structure() {
    let mut report = ShareUsageReport::new();

    // Add comprehensive test data
    report.insert(
      "lodash-es".to_string(),
      SimpleModuleExports {
        used_exports: vec![
          "map".to_string(),
          "filter".to_string(),
          "reduce".to_string(),
        ],
        unused_exports: vec![
          "uniq".to_string(),
          "debounce".to_string(),
          "throttle".to_string(),
        ],
        possibly_unused_exports: vec!["isEmpty".to_string(), "isEqual".to_string()],
        entry_module_id: Some("42".to_string()),
      },
    );

    report.insert(
      "react".to_string(),
      SimpleModuleExports {
        used_exports: vec!["createElement".to_string(), "Component".to_string()],
        unused_exports: vec!["Fragment".to_string()],
        possibly_unused_exports: vec!["useState".to_string(), "useEffect".to_string()],
        entry_module_id: Some("24".to_string()),
      },
    );

    let json = serde_json::to_string_pretty(&report).unwrap();
    let parsed: ShareUsageReport = serde_json::from_str(&json).unwrap();

    // Verify all data is preserved
    assert_eq!(parsed.len(), 2);

    let lodash_data = &parsed["lodash-es"];
    assert_eq!(lodash_data.used_exports.len(), 3);
    assert_eq!(lodash_data.unused_exports.len(), 3);
    assert_eq!(lodash_data.possibly_unused_exports.len(), 2);
    assert_eq!(lodash_data.entry_module_id, Some("42".to_string()));

    let react_data = &parsed["react"];
    assert_eq!(react_data.used_exports.len(), 2);
    assert_eq!(react_data.unused_exports.len(), 1);
    assert_eq!(react_data.possibly_unused_exports.len(), 2);
    assert_eq!(react_data.entry_module_id, Some("24".to_string()));
  }
}
