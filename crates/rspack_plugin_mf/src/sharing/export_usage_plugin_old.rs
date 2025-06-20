use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilerEmit, CompilerOptions,
  ConnectionState, DependencyType, ExportInfoGetter, ExportProvided, ExportsInfoGetter, 
  ExtendedReferencedExport, Inlinable, ModuleGraph, ModuleIdentifier, ModuleType, 
  PrefetchedExportsInfoWrapper, Plugin, PluginContext, PrefetchExportsInfoMode, 
  ProvidedExports, RuntimeSpec, UsageState, UsedExports,
};
use serde::{Deserialize, Serialize};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::export_usage_types::*;
use super::export_usage_analysis::{
  analyze_module, analyze_module_dependencies, extract_import_usage_from_dependency,
  get_detailed_export_usage, merge_consume_shared_usage_data, calculate_unused_exports,
  determine_optimal_prefetch_mode, format_runtime_key, get_runtime_usage_info,
  extract_consume_shared_info, get_simplified_export_usage
};

#[derive(Debug)]
pub struct SharedExportUsagePluginOptions {
  /// Output filename for the export usage report (default: "module-export-usage.json")
  pub filename: String,
  /// Whether to include detailed runtime information
  pub include_runtime_info: bool,
  /// Whether to include all modules or just shared modules (default: true for all modules)
  pub include_all_modules: bool,
  /// Whether to perform detailed usage analysis (like flag dependency usage plugin)
  pub detailed_analysis: bool,
}

impl Default for SharedExportUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "module-export-usage.json".to_string(),
      include_runtime_info: false,
      include_all_modules: true,
      detailed_analysis: true,
    }
  }
}

#[plugin]
#[derive(Debug)]
pub struct SharedExportUsagePlugin {
  options: SharedExportUsagePluginOptions,
}

impl SharedExportUsagePlugin {
  pub fn new(options: SharedExportUsagePluginOptions) -> Self {
    Self::new_inner(options)
  }
}

impl SharedExportUsagePlugin {
  /// Analyzes any module to extract export usage information
  fn analyze_module(
    &self,
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
    _compilation: &Compilation,
    runtimes: &[RuntimeSpec],
  ) -> Option<ModuleExportUsage> {
    let module = module_graph.module_by_identifier(module_id)?;

    // Skip if we only want shared modules and this isn't one
    if !self.options.include_all_modules {
      match module.module_type() {
        ModuleType::ConsumeShared | ModuleType::ProvideShared => {}
        _ => return None,
      }
    }

    // Use the separated analyze_module function
    analyze_module(module_id, module_graph, runtimes, self.options.detailed_analysis).ok()
  }
  
  /// Generates the complete export usage report
  fn generate_report(&self, compilation: &Compilation) -> Result<ModuleExportReport> {
    let module_graph = compilation.get_module_graph();
    let mut modules = HashMap::new();

    // Collect all runtimes for analysis
    let runtimes: Vec<RuntimeSpec> = compilation
      .chunk_by_ukey
      .values()
      .map(|chunk| chunk.runtime())
      .cloned()
      .collect();
    
    // If we have a fallback module, get its export information and use that as the ConsumeShared provided exports
    let (provided_exports_vec, fallback_export_details) = if let Some(ref fallback_id_str) = fallback_module_id {
      // Try to find the fallback module by iterating through modules
      let mut found_fallback_id = None;
      for (module_id, _) in module_graph.modules() {
        if module_id.to_string() == *fallback_id_str {
          found_fallback_id = Some(module_id);
          break;
        }
      }
      
      if let Some(fallback_id) = found_fallback_id {
        // Get the fallback module's provided exports - this is what the ConsumeShared module should provide
        let (fallback_provided, fallback_details) = self.get_fallback_module_exports(module_graph, &fallback_id, runtimes);
        
        // The ConsumeShared module should provide the same exports as its fallback
        (fallback_provided, fallback_details)
      } else {
        (vec!["*".to_string()], Vec::new())
      }
    } else {
      // For shared modules without fallback, get exports from the shared module itself
      let exports_info = module_graph.get_exports_info(module_id);
      let prefetch_mode = self.determine_optimal_prefetch_mode(module.as_ref(), &exports_info);
      let prefetched_exports = ExportsInfoGetter::prefetch(
        &exports_info,
        module_graph,
        prefetch_mode,
      );

      // Get provided exports using the prefetched exports info
      let provided_exports = prefetched_exports.get_provided_exports();
      let provided_exports_vec = match provided_exports {
        ProvidedExports::Unknown => vec!["*unknown*".to_string()],
        ProvidedExports::ProvidedAll => vec!["*".to_string()],
        ProvidedExports::ProvidedNames(exports) => exports.iter().map(|e| e.to_string()).collect(),
      };

      // Get export details
      let export_details = if self.options.detailed_analysis {
        self.get_detailed_export_usage(&prefetched_exports, &provided_exports_vec, module_graph)
      } else {
        self.get_simplified_export_usage(&provided_exports_vec)
      };

      (provided_exports_vec, export_details)
    };

    // For ConsumeShared modules, the provided exports should be based on what's actually used
    // If we detected specific used exports, those become the "provided" exports for reporting purposes
    let corrected_provided_exports = if let Some(ref used_exports) = consumer_usage.used_exports {
      if !used_exports.is_empty() {
        // Use the detected exports as the provided exports for accurate reporting
        let corrected = used_exports.clone();
        // Add any additional exports from fallback that might be relevant
        for fallback_export in &provided_exports_vec {
          if !fallback_export.starts_with('*') && !corrected.contains(fallback_export) {
            // Only add if it's not a wildcard and we haven't already included it
            // This is conservative - we only include what we know is used
          }
        }
        corrected
      } else {
        provided_exports_vec.clone()
      }
    } else {
      provided_exports_vec.clone()
    };

    // Merge consumer usage with fallback export information
    let (merged_used_exports, merged_uses_namespace, merged_export_details) = 
      self.merge_consume_shared_usage_data(
        &consumer_usage,
        &corrected_provided_exports,
        &fallback_export_details,
      );

    // Get detailed dependency information
    let dependencies = self.analyze_dependencies(module_graph, module_id, compilation);

    // Check for side effects
    let has_side_effects = match module.factory_meta() {
      Some(meta) => Some(!meta.side_effect_free.unwrap_or_default()),
      None => None,
    };

    // Calculate potentially unused exports based on the merged analysis
    let potentially_unused_exports = self.calculate_unused_exports(
      &corrected_provided_exports,
      &merged_used_exports,
      &merged_uses_namespace,
      &merged_export_details,
    );

    // Get runtime-specific usage information if requested
    let runtime_usage = if self.options.include_runtime_info {
      Some(self.get_consume_shared_runtime_usage(module_graph, module_id, runtimes, &consumer_usage))
    } else {
      None
    };

    Some(ModuleExportUsage {
      share_key,
      module_identifier: module_id.to_string(),
      provided_exports: corrected_provided_exports,
      used_exports: merged_used_exports,
      uses_namespace: merged_uses_namespace,
      fallback_module: fallback_module_id,
      module_type: module.module_type().to_string(),
      has_side_effects,
      potentially_unused_exports,
      dependencies,
      export_usage_details: merged_export_details,
      runtime_usage,
    })
  }

  /// Analyzes usage patterns from modules that consume this ConsumeShared module
  fn analyze_consume_shared_usage_from_consumers(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    _runtimes: &[RuntimeSpec],
  ) -> ConsumeSharedUsageInfo {
    let mut used_exports = Vec::new();
    let mut uses_namespace = false;
    let mut import_types = std::collections::HashMap::new();

    // Use incoming connections for more accurate dependency analysis
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Use get_referenced_exports to extract specific export names
        let referenced_exports = dependency.get_referenced_exports(
          module_graph,
          &rspack_core::ModuleGraphCacheArtifact::default(),
          None,
        );
        
        // Process referenced exports to extract used export names
        for export_ref in referenced_exports {
          match export_ref {
            ExtendedReferencedExport::Array(names) => {
              // Multiple specific exports are referenced
              for name in names {
                let export_name = name.to_string();
                if !used_exports.contains(&export_name) {
                  used_exports.push(export_name.clone());
                  import_types.insert(export_name, "named_import".to_string());
                }
              }
            },
            ExtendedReferencedExport::Export(export_info) => {
              // Single export or namespace reference
              if export_info.name.is_empty() {
                // No specific name indicates namespace usage
                uses_namespace = true;
                import_types.insert("*".to_string(), "namespace_import".to_string());
              } else {
                for name in export_info.name {
                  let export_name = name.to_string();
                  if !used_exports.contains(&export_name) {
                    used_exports.push(export_name.clone());
                    import_types.insert(export_name, "named_import".to_string());
                  }
                }
              }
            },
          }
        }
        
        // Fallback: also use general extraction method
        self.extract_import_usage_from_dependency(
          dependency.as_ref(),
          &mut used_exports,
          &mut uses_namespace,
          &mut import_types,
        );
      }
    }

    // Also check for usage through ESM import dependencies for additional analysis
    let (esm_used_exports, esm_uses_namespace) = self.analyze_esm_import_usage_static(
      module_graph, 
      consume_shared_id
    );
    
    // Merge ESM analysis results
    for export in esm_used_exports {
      if !used_exports.contains(&export) {
        used_exports.push(export);
      }
    }
    if esm_uses_namespace {
      uses_namespace = true;
    }

    ConsumeSharedUsageInfo {
      used_exports: if used_exports.is_empty() { None } else { Some(used_exports) },
      uses_namespace: Some(uses_namespace),
      import_types,
    }
  }

  /// Extracts usage information from individual dependencies
  fn extract_import_usage_from_dependency(
    &self,
    dependency: &dyn rspack_core::Dependency,
    used_exports: &mut Vec<String>,
    uses_namespace: &mut bool,
    import_types: &mut std::collections::HashMap<String, String>,
  ) {
    use rspack_core::DependencyType;
    
    match dependency.dependency_type() {
      DependencyType::EsmImport => {
        // Default import (import React from "react")
        if !used_exports.contains(&"default".to_string()) {
          used_exports.push("default".to_string());
          import_types.insert("default".to_string(), "default_import".to_string());
        }
      },
      DependencyType::EsmImportSpecifier => {
        // Named imports - we'll need to infer from connection context
        // For now, mark as namespace usage to be safe
        *uses_namespace = true;
        import_types.insert("*".to_string(), "named_import".to_string());
      },
      DependencyType::EsmExportImportedSpecifier => {
        // Re-exports - mark as namespace usage
        *uses_namespace = true;
        import_types.insert("*".to_string(), "reexport".to_string());
      },
      _ => {
        // For other import types, assume namespace usage
        *uses_namespace = true;
      }
    }
  }

  /// Analyzes ESM import usage patterns using static analysis (without compilation context)
  fn analyze_esm_import_usage_static(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> (Vec<String>, bool) {
    let mut used_exports = Vec::new();
    let mut uses_namespace = false;

    // Check incoming connections to this ConsumeShared module
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        
        // Analyze based on dependency type for static analysis
        match dependency.dependency_type() {
          DependencyType::EsmImport => {
            // Default import (import React from "react")
            if !used_exports.contains(&"default".to_string()) {
              used_exports.push("default".to_string());
            }
          },
          DependencyType::EsmImportSpecifier => {
            // Named import - try get_referenced_exports for specific names
            let referenced_exports = dependency.get_referenced_exports(
              module_graph,
              &rspack_core::ModuleGraphCacheArtifact::default(),
              None,
            );
            
            let mut found_specific_exports = false;
            for export_ref in referenced_exports {
              match export_ref {
                ExtendedReferencedExport::Array(names) => {
                  for name in names {
                    let export_name = name.to_string();
                    if !used_exports.contains(&export_name) {
                      used_exports.push(export_name);
                      found_specific_exports = true;
                    }
                  }
                },
                ExtendedReferencedExport::Export(export_info) => {
                  if !export_info.name.is_empty() {
                    for name in export_info.name {
                      let export_name = name.to_string();
                      if !used_exports.contains(&export_name) {
                        used_exports.push(export_name);
                        found_specific_exports = true;
                      }
                    }
                  }
                },
              }
            }
            
            // If we couldn't extract specific exports, mark as namespace
            if !found_specific_exports {
              uses_namespace = true;
            }
          },
          DependencyType::EsmExportImportedSpecifier => {
            // Re-export case - mark as namespace usage
            uses_namespace = true;
          },
          _ => {
            // For other dependency types, mark as namespace usage for safety
            uses_namespace = true;
          }
        }
      }
    }

    (used_exports, uses_namespace)
  }

  /// Analyzes ESM import usage patterns with full compilation context (for future use)
  fn analyze_esm_import_usage_with_cache(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &rspack_core::ModuleGraphCacheArtifact,
    consume_shared_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> (Vec<String>, bool) {
    let mut used_exports = Vec::new();
    let mut uses_namespace = false;

    // Check incoming connections to this ConsumeShared module
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        
        // Get specific export usage from the dependency using get_referenced_exports
        let referenced_exports = dependency.get_referenced_exports(
          module_graph,
          module_graph_cache,
          runtime,
        );
        
        // Process referenced exports to extract used export names
        for export_ref in referenced_exports {
          match export_ref {
            ExtendedReferencedExport::Array(names) => {
              // Multiple specific exports are referenced
              for name in names {
                let export_name = name.to_string();
                if !used_exports.contains(&export_name) {
                  used_exports.push(export_name);
                }
              }
            },
            ExtendedReferencedExport::Export(export_info) => {
              // Single export or namespace reference
              if export_info.name.is_empty() {
                // No specific name indicates namespace usage
                uses_namespace = true;
              } else {
                for name in export_info.name {
                  let export_name = name.to_string();
                  if !used_exports.contains(&export_name) {
                    used_exports.push(export_name);
                  }
                }
              }
            },
          }
        }
      }
    }

    (used_exports, uses_namespace)
  }

  /// Gets export information from the fallback module
  fn get_fallback_module_exports(
    &self,
    module_graph: &ModuleGraph,
    fallback_module_id: &ModuleIdentifier,
    _runtimes: &[RuntimeSpec],
  ) -> (Vec<String>, Vec<ExportUsageDetail>) {
    if let Some(_fallback_module) = module_graph.module_by_identifier(fallback_module_id) {
      // Get exports info for the fallback module with optimized prefetch mode
      let exports_info = module_graph.get_exports_info(fallback_module_id);
      let prefetch_mode = self.determine_optimal_prefetch_mode(_fallback_module.as_ref(), &exports_info);
      let prefetched_exports = ExportsInfoGetter::prefetch(
        &exports_info,
        module_graph,
        prefetch_mode,
      );

      // Get provided exports
      let provided_exports = prefetched_exports.get_provided_exports();
      let provided_exports_vec = match provided_exports {
        ProvidedExports::Unknown => vec!["*unknown*".to_string()],
        ProvidedExports::ProvidedAll => vec!["*".to_string()],
        ProvidedExports::ProvidedNames(exports) => exports.iter().map(|e| e.to_string()).collect(),
      };

      // Get detailed export usage information from the fallback module
      let export_details = if self.options.detailed_analysis {
        self.get_detailed_export_usage(&prefetched_exports, &provided_exports_vec, module_graph)
      } else {
        self.get_simplified_export_usage(&provided_exports_vec)
      };

      (provided_exports_vec, export_details)
    } else {
      (vec!["*".to_string()], Vec::new())
    }
  }

  /// Merges usage data from consumers with fallback module export information
  fn merge_consume_shared_usage_data(
    &self,
    consumer_usage: &ConsumeSharedUsageInfo,
    provided_exports: &[String],
    fallback_export_details: &[ExportUsageDetail],
  ) -> (Option<Vec<String>>, Option<bool>, Vec<ExportUsageDetail>) {
    let mut merged_export_details = Vec::new();
    
    // Create export details based on consumer usage and fallback information
    for export_name in provided_exports {
      let is_used_by_consumer = consumer_usage.used_exports
        .as_ref()
        .map(|exports| exports.contains(export_name))
        .unwrap_or(false);
      
      let fallback_detail = fallback_export_details
        .iter()
        .find(|detail| detail.export_name == *export_name);
      
      let usage_state = if is_used_by_consumer {
        "Used"
      } else if consumer_usage.uses_namespace.unwrap_or(false) {
        "OnlyPropertiesUsed"
      } else {
        fallback_detail.map(|d| d.usage_state.as_str()).unwrap_or("Unused")
      };
      
      let _import_type = consumer_usage.import_types.get(export_name);
      
      merged_export_details.push(ExportUsageDetail {
        export_name: export_name.clone(),
        usage_state: usage_state.to_string(),
        can_mangle: fallback_detail.and_then(|d| d.can_mangle),
        can_inline: fallback_detail.and_then(|d| d.can_inline),
        is_provided: fallback_detail.and_then(|d| d.is_provided).or(Some(true)),
        used_name: fallback_detail.and_then(|d| d.used_name.clone()),
      });
    }
    
    (
      consumer_usage.used_exports.clone(),
      consumer_usage.uses_namespace,
      merged_export_details,
    )
  }

  /// Gets runtime-specific usage information for ConsumeShared modules
  fn get_consume_shared_runtime_usage(
    &self,
    _module_graph: &ModuleGraph,
    _consume_shared_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec],
    consumer_usage: &ConsumeSharedUsageInfo,
  ) -> HashMap<String, RuntimeUsageInfo> {
    let mut runtime_info = HashMap::new();
    
    for runtime in runtimes {
      let runtime_key = self.format_runtime_key(runtime);
      
      let mut export_usage_states = HashMap::new();
      if let Some(ref used_exports) = consumer_usage.used_exports {
        for export_name in used_exports {
          export_usage_states.insert(export_name.clone(), "Used".to_string());
        }
      }
      
      runtime_info.insert(
        runtime_key,
        RuntimeUsageInfo {
          used_exports: consumer_usage.used_exports.clone(),
          uses_namespace: consumer_usage.uses_namespace,
          export_usage_states,
        },
      );
    }
    
    runtime_info
  }

  /// Determines the optimal prefetch mode based on module characteristics and analysis requirements
  fn determine_optimal_prefetch_mode(
    &self,
    module: &dyn rspack_core::Module,
    _exports_info: &rspack_core::ExportsInfo,
  ) -> PrefetchExportsInfoMode {
    // If detailed analysis is disabled, use minimal prefetch
    if !self.options.detailed_analysis {
      return PrefetchExportsInfoMode::Default;
    }

    // For large modules (many exports), use selective prefetch to optimize performance
    // Estimate export count - skip for now as exports() method not available
    let export_count = 50; // Conservative estimate
    if export_count > 100 {
      return PrefetchExportsInfoMode::Default;
    }

    // For JavaScript modules, use full analysis for better tree-shaking insights
    match module.module_type() {
      ModuleType::JsAuto | ModuleType::JsDynamic | ModuleType::JsEsm => {
        PrefetchExportsInfoMode::AllExports
      },
      // For other module types, use targeted analysis
      ModuleType::ConsumeShared | ModuleType::ProvideShared => {
        // Shared modules need full analysis for federation optimization
        PrefetchExportsInfoMode::AllExports
      },
      // For CSS, Asset, and other modules, minimal analysis is sufficient
      _ => PrefetchExportsInfoMode::Default,
    }
  }

  /// Gets detailed export usage information using prefetched exports
  fn get_detailed_export_usage(
    &self,
    prefetched_exports: &PrefetchedExportsInfoWrapper,
    provided_exports: &[String],
    module_graph: &ModuleGraph,
  ) -> Vec<ExportUsageDetail> {
    let mut export_usage = Vec::new();

    // Analyze each provided export using the prefetched exports data
    for export_name in provided_exports {
      // Skip special markers
      if export_name.starts_with('*') || export_name.contains('?') {
        continue;
      }

      let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
      
      // Get detailed export information from the prefetched data  
      if let Some(export_info_data) = prefetched_exports.exports().find(|(name, _)| **name == export_atom).map(|(_, data)| data) {

        // Extract comprehensive usage information
        let usage_state = match export_info_data.global_used() {
          Some(UsageState::Used) => "Used",
          Some(UsageState::OnlyPropertiesUsed) => "OnlyPropertiesUsed", 
          Some(UsageState::Unused) => "Unused",
          Some(UsageState::NoInfo) => "NoInfo",
          Some(UsageState::Unknown) => "Unknown",
          None => "NotAnalyzed",
        };

        // Check mangling capabilities 
        let can_mangle = ExportInfoGetter::can_mangle(export_info_data);
        
        // Check inlining capabilities
        let can_inline = match export_info_data.inlinable() {
          Inlinable::Inlined(_) => Some(true),
          Inlinable::NoByUse => Some(false),
          Inlinable::NoByProvide => Some(false),
        };

        // Check provision status
        let is_provided = export_info_data.provided().map(|p| match p {
          ExportProvided::Provided => true,
          ExportProvided::Unknown => false,
          ExportProvided::NotProvided => false,
        });

        // Get used name (considering mangling)
        let used_name = export_info_data.used_name().map(|n| n.to_string());

        export_usage.push(ExportUsageDetail {
          export_name: export_name.clone(),
          usage_state: usage_state.to_string(),
          can_mangle,
          can_inline,
          is_provided,
          used_name,
        });

        // Handle nested exports if they exist
        if let Some(nested_exports_info) = export_info_data.exports_info() {
          let nested_details = self.analyze_nested_exports(
            prefetched_exports,
            &nested_exports_info,
            module_graph,
            &format!("{}.{}.", export_name, "")
          );
          export_usage.extend(nested_details);
        }
      } else {
        // Export not found in detailed analysis - use fallback
        export_usage.push(ExportUsageDetail {
          export_name: export_name.clone(),
          usage_state: "NotTracked".to_string(),
          can_mangle: None,
          can_inline: None,
          is_provided: None,
          used_name: None,
        });
      }
    }

    // Also analyze other exports (catch-all for dynamic exports)
    let other_data = prefetched_exports.other_exports_info();
    let other_usage = match other_data.global_used() {
      Some(UsageState::Used) => "Used",
      Some(UsageState::OnlyPropertiesUsed) => "OnlyPropertiesUsed",
      Some(UsageState::Unused) => "Unused", 
      Some(UsageState::NoInfo) => "NoInfo",
      Some(UsageState::Unknown) => "Unknown",
      None => "NotAnalyzed",
    };

    if !matches!(other_usage, "NotAnalyzed" | "Unused") {
      export_usage.push(ExportUsageDetail {
        export_name: "*".to_string(),
        usage_state: other_usage.to_string(),
        can_mangle: other_data.can_mangle_use(),
        can_inline: match other_data.inlinable() {
          Inlinable::Inlined(_) => Some(true),
          Inlinable::NoByUse => Some(false),
          Inlinable::NoByProvide => Some(false),
        },
        is_provided: other_data.provided().map(|p| match p {
          ExportProvided::Provided => true,
          ExportProvided::Unknown => false, 
          ExportProvided::NotProvided => false,
        }),
        used_name: other_data.used_name().map(|n| n.to_string()),
      });
    }

    export_usage
  }

  /// Analyzes nested exports recursively
  fn analyze_nested_exports(
    &self,
    prefetched_exports: &rspack_core::PrefetchedExportsInfoWrapper,
    nested_exports_info: &rspack_core::ExportsInfo,
    _module_graph: &ModuleGraph,
    prefix: &str,
  ) -> Vec<ExportUsageDetail> {
    let mut nested_usage = Vec::new();

    // Get nested exports data by redirecting the prefetched wrapper
    let nested_wrapper = prefetched_exports.redirect(*nested_exports_info, true);
    
    for (nested_name, nested_export_data) in nested_wrapper.exports() {
      let full_name = format!("{}{}", prefix, nested_name);

      let usage_state = match nested_export_data.global_used() {
        Some(UsageState::Used) => "Used",
        Some(UsageState::OnlyPropertiesUsed) => "OnlyPropertiesUsed",
        Some(UsageState::Unused) => "Unused",
        Some(UsageState::NoInfo) => "NoInfo", 
        Some(UsageState::Unknown) => "Unknown",
        None => "NotAnalyzed",
      };

      nested_usage.push(ExportUsageDetail {
        export_name: full_name.clone(),
        usage_state: usage_state.to_string(),
        can_mangle: ExportInfoGetter::can_mangle(nested_export_data),
        can_inline: match nested_export_data.inlinable() {
          Inlinable::Inlined(_) => Some(true),
          Inlinable::NoByUse => Some(false),
          Inlinable::NoByProvide => Some(false),
        },
        is_provided: nested_export_data.provided().map(|p| match p {
          ExportProvided::Provided => true,
          ExportProvided::Unknown => false,
          ExportProvided::NotProvided => false,
        }),
        used_name: nested_export_data.used_name().map(|n| n.to_string()),
      });

      // Recurse deeper if there are more nested exports
      if let Some(deeper_exports_info) = nested_export_data.exports_info() {
        let deeper_details = self.analyze_nested_exports(
          prefetched_exports,
          &deeper_exports_info,
          _module_graph,
          &format!("{}.", full_name)
        );
        nested_usage.extend(deeper_details);
      }
    }

    nested_usage
  }

  /// Gets comprehensive runtime-specific usage information
  fn get_runtime_usage_info(
    &self,
    prefetched_exports: &rspack_core::PrefetchedExportsInfoWrapper,
    runtimes: &[RuntimeSpec],
  ) -> HashMap<String, RuntimeUsageInfo> {
    let mut runtime_info = HashMap::new();

    for runtime in runtimes {
      let mut used_exports = Vec::new();
      let mut uses_namespace = false;
      let mut export_usage_states = HashMap::new();

      // Get runtime-specific used exports
      let used_exports_info = prefetched_exports.get_used_exports(Some(runtime));
      match used_exports_info {
        UsedExports::UsedNames(names) => {
          used_exports = names.iter().map(|n| n.to_string()).collect();
        }
        UsedExports::UsedNamespace(ns_used) => {
          uses_namespace = ns_used;
        }
        UsedExports::Unknown => {
          // When usage is unknown, analyze individual exports
          self.analyze_individual_export_usage_for_runtime(
            prefetched_exports,
            runtime,
            &mut used_exports,
            &mut export_usage_states,
          );
        }
      }

      // Get detailed usage states for each export
      for (export_name, export_info) in prefetched_exports.exports() {
        let usage_state = ExportInfoGetter::get_used(export_info, Some(runtime));
        let state_str = match usage_state {
          UsageState::Used => {
            if !used_exports.contains(&export_name.to_string()) {
              used_exports.push(export_name.to_string());
            }
            "Used"
          }
          UsageState::OnlyPropertiesUsed => {
            if !used_exports.contains(&export_name.to_string()) {
              used_exports.push(export_name.to_string());
            }
            "OnlyPropertiesUsed"
          }
          UsageState::Unused => "Unused",
          UsageState::NoInfo => "NoInfo",
          UsageState::Unknown => "Unknown",
        };
        export_usage_states.insert(export_name.to_string(), state_str.to_string());
      }

      // Check namespace usage from other exports
      let other_data = prefetched_exports.other_exports_info();
      let other_usage = ExportInfoGetter::get_used(other_data, Some(runtime));
      match other_usage {
        UsageState::Used | UsageState::OnlyPropertiesUsed => {
          uses_namespace = true;
        }
        _ => {}
      }
      
      if !matches!(other_usage, UsageState::Unused | UsageState::NoInfo) {
        export_usage_states.insert("*".to_string(), format!("{:?}", other_usage));
      }

      // Check side effects only usage
      let side_effects_data = prefetched_exports.side_effects_only_info();
      let side_effects_usage = ExportInfoGetter::get_used(side_effects_data, Some(runtime));
      if !matches!(side_effects_usage, UsageState::Unused | UsageState::NoInfo) {
        export_usage_states.insert("__sideEffects__".to_string(), format!("{:?}", side_effects_usage));
      }

      let runtime_key = self.format_runtime_key(runtime);
      runtime_info.insert(
        runtime_key,
        RuntimeUsageInfo {
          used_exports: if used_exports.is_empty() {
            None
          } else {
            Some(used_exports)
          },
          uses_namespace: Some(uses_namespace),
          export_usage_states,
        },
      );
    }

    runtime_info
  }

  /// Analyzes individual export usage when overall usage is unknown
  fn analyze_individual_export_usage_for_runtime(
    &self,
    prefetched_exports: &rspack_core::PrefetchedExportsInfoWrapper,
    runtime: &RuntimeSpec,
    used_exports: &mut Vec<String>,
    export_usage_states: &mut HashMap<String, String>,
  ) {
    // Get relevant exports for this runtime (excludes unused and not provided)
    let relevant_exports = prefetched_exports.get_relevant_exports(Some(runtime));
    
    for export_info_data in relevant_exports {
      if let Some(export_name) = export_info_data.name() {
        let usage_state = ExportInfoGetter::get_used(export_info_data, Some(runtime));
        
        match usage_state {
          UsageState::Used | UsageState::OnlyPropertiesUsed => {
            used_exports.push(export_name.to_string());
          }
          _ => {}
        }
        
        export_usage_states.insert(
          export_name.to_string(),
          format!("{:?}", usage_state),
        );
      }
    }
  }

  /// Formats runtime key for consistent identification
  fn format_runtime_key(&self, runtime: &RuntimeSpec) -> String {
    // Create a deterministic, readable runtime key
    if runtime.is_empty() {
      "default".to_string()
    } else {
      let mut runtime_names: Vec<String> = runtime.iter().map(|s| s.to_string()).collect();
      runtime_names.sort();
      runtime_names.join("+")
    }
  }

  /// Calculates unused exports based on detailed usage information
  fn calculate_unused_exports(
    &self,
    provided_exports: &[String],
    used_exports: &Option<Vec<String>>,
    uses_namespace: &Option<bool>,
    export_usage_details: &[ExportUsageDetail],
  ) -> Option<Vec<String>> {
    // If namespace is used, all exports are potentially used
    if uses_namespace == &Some(true) {
      return None;
    }

    // Use detailed export usage information to find unused exports
    let unused_from_details: Vec<String> = export_usage_details
      .iter()
      .filter_map(|detail| {
        if detail.usage_state == "Unused" {
          Some(detail.export_name.clone())
        } else {
          None
        }
      })
      .collect();

    if !unused_from_details.is_empty() {
      return Some(unused_from_details);
    }

    // Fallback: if we have specific used exports, calculate unused ones
    if let Some(used) = used_exports {
      if !used.is_empty() && !provided_exports.is_empty() {
        let unused: Vec<String> = provided_exports
          .iter()
          .filter(|export| {
            !export.starts_with('*') && !export.contains('?') && !used.contains(export)
          })
          .cloned()
          .collect();

        if !unused.is_empty() {
          return Some(unused);
        }
      }
    }

    None
  }

  /// Analyzes dependencies with detailed information
  fn analyze_dependencies(
    &self,
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
    compilation: &Compilation,
  ) -> Vec<DependencyDetail> {
    let module = match module_graph.module_by_identifier(module_id) {
      Some(m) => m,
      None => return Vec::new(),
    };

    let mut dependencies = Vec::new();
    let module_graph_cache = &compilation.module_graph_cache_artifact;

    for dep_id in module.get_dependencies() {
      let dependency = match module_graph.dependency_by_id(dep_id) {
        Some(dep) => dep,
        None => continue,
      };

      let connection = module_graph.connection_by_dependency_id(dep_id);
      let (target_module, connection_state, request) = if let Some(connection) = connection {
        let state = connection.active_state(module_graph, None, module_graph_cache);
        let state_str = match state {
          ConnectionState::Active(true) => "Active",
          ConnectionState::Active(false) => "Inactive",
          ConnectionState::TransitiveOnly => "TransitiveOnly",
          ConnectionState::CircularConnection => "Circular",
        };

        let target = Some(connection.module_identifier().to_string());
        let req = if let Some(md) = dependency.as_module_dependency() {
          Some(md.request().to_string())
        } else {
          None
        };

        (target, state_str.to_string(), req)
      } else {
        (None, "NoConnection".to_string(), None)
      };

      let dependency_type = dependency.dependency_type();
      let is_module_federation = matches!(
        dependency_type,
        DependencyType::ConsumeSharedFallback
          | DependencyType::ProvideModuleForShared
          | DependencyType::ProvideSharedModule
      );

      dependencies.push(DependencyDetail {
        dependency_type: format!("{}", dependency_type),
        target_module,
        request,
        connection_state,
        is_module_federation,
      });
    }

    dependencies
  }

  /// Finds the fallback module for a ConsumeShared module
  fn find_fallback_module(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<String> {
    let module = module_graph.module_by_identifier(consume_shared_id)?;

    for dep_id in module.get_dependencies() {
      if let Some(_dep) = module_graph.dependency_by_id(dep_id) {
        if let Some(module_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
          if let Some(fallback_module) = module_graph.module_by_identifier(module_id) {
            if matches!(
              fallback_module.module_type(),
              ModuleType::JsAuto | ModuleType::JsDynamic | ModuleType::JsEsm
            ) {
              return Some(fallback_module.identifier().to_string());
            }
          }
        }
      }
    }

    None
  }

  /// Gets simplified export usage information (fallback)
  fn get_simplified_export_usage(&self, provided_exports: &[String]) -> Vec<ExportUsageDetail> {
    provided_exports
      .iter()
      .filter(|export_name| !export_name.starts_with('*'))
      .map(|export_name| ExportUsageDetail {
        export_name: export_name.clone(),
        usage_state: "NotAnalyzed".to_string(),
        can_mangle: None,
        can_inline: None,
        is_provided: None,
        used_name: None,
      })
      .collect()
  }

  /// Detects and analyzes reexported modules and their usage patterns
  fn analyze_reexport_patterns(
    &self,
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
    compilation: &Compilation,
  ) -> Vec<ReexportUsageDetail> {
    let mut reexport_details = Vec::new();
    
    if let Some(module) = module_graph.module_by_identifier(module_id) {
      // Find dependencies that are re-exports
      for dep_id in module.get_dependencies() {
        if let Some(dependency) = module_graph.dependency_by_id(dep_id) {
          // Check if this is a re-export dependency
          if matches!(
            dependency.dependency_type(),
            DependencyType::EsmExportImportedSpecifier
          ) {
            // Get the target module of the re-export
            if let Some(target_module_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
              // Analyze what's being re-exported
              let referenced_exports = dependency.get_referenced_exports(
                module_graph,
                &compilation.module_graph_cache_artifact,
                None,
              );
              
              let mut reexported_names = Vec::new();
              for export_ref in referenced_exports {
                match export_ref {
                  ExtendedReferencedExport::Array(names) => {
                    reexported_names.extend(names.iter().map(|n| n.to_string()));
                  },
                  ExtendedReferencedExport::Export(export_info) => {
                    if let Some(name) = export_info.name.first() {
                      reexported_names.push(name.to_string());
                    }
                  },
                }
              }
              
              reexport_details.push(ReexportUsageDetail {
                source_module: module_id.to_string(),
                target_module: target_module_id.to_string(),
                reexported_names,
                reexport_type: format!("{:?}", dependency.dependency_type()),
              });
            }
          }
        }
      }
    }
    
    reexport_details
  }

  /// Generates the complete export usage report
  fn generate_report(&self, compilation: &Compilation) -> Result<ModuleExportReport> {
    let module_graph = compilation.get_module_graph();
    let mut modules = HashMap::new();

    // Collect all runtimes for analysis
    let runtimes: Vec<RuntimeSpec> = compilation
      .chunk_by_ukey
      .values()
      .map(|chunk| chunk.runtime())
      .cloned()
      .collect();

    let mut total_dependencies = 0;
    let mut module_federation_dependencies = 0;

    // Analyze all modules based on configuration
    for (module_id, _module) in module_graph.modules() {
      if let Some(usage_info) =
        self.analyze_module(&module_graph, &module_id, compilation, &runtimes)
      {
        total_dependencies += usage_info.dependencies.len();
        module_federation_dependencies += usage_info
          .dependencies
          .iter()
          .filter(|dep| dep.is_module_federation)
          .count();

        modules.insert(module_id.to_string(), usage_info);
      }
    }

    // Generate summary statistics
    let total_modules = modules.len();
    let consume_shared_modules = modules
      .values()
      .filter(|m| m.module_type == "consume-shared-module")
      .count();
    let provide_shared_modules = modules
      .values()
      .filter(|m| m.module_type == "provide-shared-module" || m.module_type == "provide-module")
      .count();
    let javascript_modules = modules
      .values()
      .filter(|m| m.module_type.contains("javascript"))
      .count();
    let modules_with_specific_usage = modules
      .values()
      .filter(|m| m.used_exports.is_some())
      .count();
    let modules_with_namespace_usage = modules
      .values()
      .filter(|m| m.uses_namespace == Some(true))
      .count();
    let modules_with_unknown_usage = modules
      .values()
      .filter(|m| m.used_exports.is_none() && m.uses_namespace.is_none())
      .count();
    let modules_with_provided_exports = modules
      .values()
      .filter(|m| !m.provided_exports.is_empty())
      .count();
    let modules_with_potentially_unused_exports = modules
      .values()
      .filter(|m| m.potentially_unused_exports.is_some())
      .count();

    let summary = ExportUsageSummary {
      total_modules,
      consume_shared_modules,
      provide_shared_modules,
      javascript_modules,
      modules_with_specific_usage,
      modules_with_namespace_usage,
      modules_with_unknown_usage,
      modules_with_provided_exports,
      modules_with_potentially_unused_exports,
      total_dependencies,
      module_federation_dependencies,
    };

    let metadata = AnalysisMetadata {
      runtimes_analyzed: runtimes.len(),
      detailed_analysis_enabled: self.options.detailed_analysis,
      analysis_version: "2.0.0".to_string(),
    };

    Ok(ModuleExportReport {
      timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string(),
      modules,
      summary,
      metadata,
    })
  }

  /// Generate a simplified report with just module ID -> used/unused/possibly unused exports
  fn generate_simple_report(&self, compilation: &Compilation) -> Result<HashMap<String, SimpleModuleExports>> {
    let module_graph = compilation.get_module_graph();
    let mut simple_modules = HashMap::new();

    for (module_id, module) in module_graph.modules() {
      // Get exports info for this module
      let exports_info = module_graph.get_exports_info(module_id);

      // Use prefetched mode for efficient access
      let prefetched = ExportsInfoGetter::prefetch(
        &exports_info,
        &module_graph,
        PrefetchExportsInfoMode::AllExports,
      );

      let provided_exports = prefetched.get_provided_exports();

      // Extract export names from provided exports
      let all_export_names: Vec<String> = match provided_exports {
        ProvidedExports::ProvidedNames(names) => names.iter().map(|n| n.to_string()).collect(),
        ProvidedExports::ProvidedAll => vec!["*".to_string()],
        ProvidedExports::Unknown => vec![],
      };

      if all_export_names.is_empty() {
        continue; // Skip modules with no exports
      }

      // Determine used exports
      let mut used_exports = Vec::new();
      let mut unused_exports = Vec::new();
      let mut possibly_unused_exports = Vec::new();

      for export_name in &all_export_names {
        if export_name == "*" {
          // Handle namespace exports differently
          let export_info = exports_info.get_export_info(&module_graph, export_name);
          let usage_state = ExportInfoGetter::get_used(&export_info.as_data(&module_graph), None);

          match usage_state {
            UsageState::Used => used_exports.push(export_name.clone()),
            UsageState::OnlyPropertiesUsed => used_exports.push(export_name.clone()),
            UsageState::Unused => unused_exports.push(export_name.clone()),
            UsageState::NoInfo => possibly_unused_exports.push(export_name.clone()),
            UsageState::Unknown => possibly_unused_exports.push(export_name.clone()),
          }
        } else {
          let export_info = exports_info.get_export_info(&module_graph, export_name);
          let usage_state = ExportInfoGetter::get_used(&export_info.as_data(&module_graph), None);

          match usage_state {
            UsageState::Used => used_exports.push(export_name.clone()),
            UsageState::OnlyPropertiesUsed => used_exports.push(export_name.clone()),
            UsageState::Unused => unused_exports.push(export_name.clone()),
            UsageState::NoInfo => possibly_unused_exports.push(export_name.clone()),
            UsageState::Unknown => possibly_unused_exports.push(export_name.clone()),
          }
        }
      }

      simple_modules.insert(
        module_id.to_string(),
        SimpleModuleExports {
          used_exports,
          unused_exports,
          possibly_unused_exports,
        },
      );
    }

    Ok(simple_modules)
  }
}

#[plugin_hook(CompilerEmit for SharedExportUsagePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  // Generate the export usage report
  let report = self.generate_report(compilation)?;

  // Serialize the report to JSON
  let json_content = serde_json::to_string_pretty(&report).map_err(|e| {
    rspack_error::Error::msg(format!("Failed to serialize export usage report: {}", e))
  })?;

  // Create the asset
  let source = RawSource::from(json_content).boxed();
  let asset = CompilationAsset::new(Some(source), AssetInfo::default());

  // Emit the asset
  compilation.emit_asset(self.options.filename.clone(), asset);

  Ok(())
}

#[async_trait]
impl Plugin for SharedExportUsagePlugin {
  fn name(&self) -> &'static str {
    "rspack.SharedExportUsagePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx.context.compiler_hooks.emit.tap(emit::new(self));
    Ok(())
  }
}
