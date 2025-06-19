use std::collections::HashMap;

use rspack_core::{
  ConnectionState, DependencyType, ExportInfoGetter, ExportProvided, ExportsInfoGetter,
  ExtendedReferencedExport, Inlinable, ModuleGraph, ModuleIdentifier, ModuleType,
  PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper, ProvidedExports, RuntimeSpec, UsageState,
};
use rspack_error::Result;

use super::export_usage_types::{
  ConsumeSharedUsageInfo, DependencyDetail, ExportUsageDetail, ModuleExportUsage, RuntimeUsageInfo,
};

/// Analyzes a single module's export usage
pub fn analyze_module(
  module_id: &ModuleIdentifier,
  module_graph: &ModuleGraph,
  runtimes: &[RuntimeSpec],
  detailed_analysis: bool,
) -> Result<ModuleExportUsage> {
  let module = module_graph
    .module_by_identifier(module_id)
    .ok_or_else(|| rspack_error::Error::msg("Module not found"))?;

  // Get exports info for this module
  let exports_info = module_graph.get_exports_info(module_id);

  // Use prefetched mode for efficient access
  let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    if detailed_analysis {
      PrefetchExportsInfoMode::AllExports
    } else {
      PrefetchExportsInfoMode::Default
    },
  );

  let provided_exports = prefetched.get_provided_exports();

  // Extract export names from provided exports
  let provided_exports_vec: Vec<String> = match provided_exports {
    ProvidedExports::ProvidedNames(names) => names.iter().map(|n| n.to_string()).collect(),
    ProvidedExports::ProvidedAll => vec!["*".to_string()],
    ProvidedExports::Unknown => vec![],
  };

  // Get used exports information if available
  let used_exports_result = get_used_exports(module_graph, module_id, &provided_exports_vec);

  // Analyze dependencies
  let dependencies = if detailed_analysis {
    analyze_module_dependencies(module_graph, module_id).unwrap_or_default()
  } else {
    vec![]
  };

  // Get detailed export usage information
  let export_usage_details = if detailed_analysis {
    get_detailed_export_usage(module_graph, module_id, &provided_exports_vec, runtimes)?
  } else {
    vec![]
  };

  // Get runtime-specific usage information
  let runtime_usage = if detailed_analysis && !runtimes.is_empty() {
    Some(get_runtime_usage_info(
      module_graph,
      module_id,
      &provided_exports_vec,
      runtimes,
    )?)
  } else {
    None
  };

  // Handle ConsumeShared modules specially
  let (share_key, fallback_module) = if module.module_type() == &ModuleType::ConsumeShared {
    extract_consume_shared_info(module_id, module_graph)
  } else {
    (None, None)
  };

  // Get potentially unused exports using simplified heuristics
  let potentially_unused_exports = get_simplified_export_usage(&provided_exports_vec);

  Ok(ModuleExportUsage {
    share_key,
    module_identifier: module_id.to_string(),
    provided_exports: provided_exports_vec,
    used_exports: used_exports_result.used_exports,
    uses_namespace: used_exports_result.uses_namespace,
    fallback_module,
    module_type: module.module_type().to_string(),
    has_side_effects: Some({
      // Check factory meta first (from package.json sideEffects field)
      if let Some(side_effect_free) = module.factory_meta().and_then(|m| m.side_effect_free) {
        !side_effect_free
      } else if let Some(side_effect_free) = module.build_meta().side_effect_free {
        !side_effect_free
      } else {
        true // Default to having side effects if unknown
      }
    }),
    potentially_unused_exports,
    dependencies,
    export_usage_details,
    runtime_usage,
  })
}

/// Result of used exports analysis
pub struct UsedExportsResult {
  pub used_exports: Option<Vec<String>>,
  pub uses_namespace: Option<bool>,
}

/// Gets used exports information for a module
pub fn get_used_exports(
  module_graph: &ModuleGraph,
  module_id: &ModuleIdentifier,
  provided_exports: &[String],
) -> UsedExportsResult {
  let exports_info = module_graph.get_exports_info(module_id);

  // Use prefetched exports info to avoid mutable borrow
  let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
  );

  let mut used_exports = Vec::new();
  let mut uses_namespace = false;

  for export_name in provided_exports {
    let export_atom = rspack_util::atom::Atom::from(export_name.as_str());

    // Use prefetched exports info to get usage state without mutable borrow
    let export_info_data = prefetched.get_read_only_export_info(&export_atom);
    let usage_state = ExportInfoGetter::get_used(export_info_data, None);

    match usage_state {
      UsageState::Used | UsageState::OnlyPropertiesUsed => {
        if export_name == "*" {
          uses_namespace = true;
        } else {
          used_exports.push(export_name.clone());
        }
      }
      _ => {}
    }
  }

  UsedExportsResult {
    used_exports: if used_exports.is_empty() {
      None
    } else {
      Some(used_exports)
    },
    uses_namespace: if uses_namespace { Some(true) } else { None },
  }
}

/// Analyzes dependencies for a module
pub fn analyze_module_dependencies(
  module_graph: &ModuleGraph,
  module_id: &ModuleIdentifier,
) -> Result<Vec<DependencyDetail>> {
  let module = module_graph
    .module_by_identifier(module_id)
    .ok_or_else(|| rspack_error::Error::msg("Module not found"))?;

  let mut dependencies = Vec::new();

  for dep_id in module.get_dependencies() {
    if let Some(dep) = module_graph.dependency_by_id(dep_id) {
      let target_module = module_graph
        .module_identifier_by_dependency_id(dep_id)
        .map(|id| id.to_string());

      let connection_state =
        if let Some(connection) = module_graph.connection_by_dependency_id(dep_id) {
          match connection.active_state(module_graph, None, &Default::default()) {
            ConnectionState::Active(true) => "active".to_string(),
            ConnectionState::Active(false) => "inactive".to_string(),
            ConnectionState::TransitiveOnly => "transitive".to_string(),
            ConnectionState::CircularConnection => "circular".to_string(),
          }
        } else {
          "unknown".to_string()
        };

      let is_module_federation = matches!(
        dep.dependency_type(),
        DependencyType::ConsumeSharedFallback | DependencyType::ProvideModuleForShared
      );

      dependencies.push(DependencyDetail {
        dependency_type: format!("{:?}", dep.dependency_type()),
        target_module,
        request: dep
          .as_module_dependency()
          .map(|md| md.request().to_string()),
        connection_state,
        is_module_federation,
      });
    }
  }

  Ok(dependencies)
}

/// Gets detailed export usage information for a module
pub fn get_detailed_export_usage(
  module_graph: &ModuleGraph,
  module_id: &ModuleIdentifier,
  provided_exports: &[String],
  runtimes: &[RuntimeSpec],
) -> Result<Vec<ExportUsageDetail>> {
  let exports_info = module_graph.get_exports_info(module_id);

  // Use prefetched exports info to avoid mutable borrow
  let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
  );

  let mut export_usage_details = Vec::new();

  for export_name in provided_exports {
    let export_atom = rspack_util::atom::Atom::from(export_name.as_str());

    // Use prefetched exports info to get export data without mutable borrow
    let export_info_data = prefetched.get_read_only_export_info(&export_atom);

    // Get usage state using the runtime info
    let runtime_spec = runtimes.first();
    let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);

    // Get usage information from export_info_data
    let can_mangle = export_info_data.can_mangle_use();
    let used_name = export_info_data
      .used_name()
      .map(|name| format!("{:?}", name));
    let is_provided = export_info_data.provided().is_some();

    // Get inlining information
    let can_inline = Some(match export_info_data.inlinable() {
      rspack_core::Inlinable::Inlined(_) => true,
      _ => false,
    });

    export_usage_details.push(ExportUsageDetail {
      export_name: export_name.clone(),
      usage_state: format!("{:?}", usage_state),
      can_mangle,
      can_inline,
      is_provided: Some(is_provided),
      used_name,
    });
  }

  Ok(export_usage_details)
}

/// Gets runtime-specific usage information
pub fn get_runtime_usage_info(
  module_graph: &ModuleGraph,
  module_id: &ModuleIdentifier,
  provided_exports: &[String],
  runtimes: &[RuntimeSpec],
) -> Result<HashMap<String, RuntimeUsageInfo>> {
  let exports_info = module_graph.get_exports_info(module_id);

  // Use prefetched exports info to avoid mutable borrow
  let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports,
  );

  let mut runtime_usage = HashMap::new();

  for runtime in runtimes {
    let runtime_str = runtime
      .iter()
      .map(|s| s.to_string())
      .collect::<Vec<_>>()
      .join(",");

    let mut used_exports = Vec::new();
    let mut uses_namespace = false;
    let mut export_usage_states = HashMap::new();

    for export_name in provided_exports {
      let export_atom = rspack_util::atom::Atom::from(export_name.as_str());

      // Use prefetched exports info to get export data without mutable borrow
      let export_info_data = prefetched.get_read_only_export_info(&export_atom);
      let usage_state = ExportInfoGetter::get_used(export_info_data, Some(runtime));

      export_usage_states.insert(export_name.clone(), format!("{:?}", usage_state));

      match usage_state {
        UsageState::Used | UsageState::OnlyPropertiesUsed => {
          if export_name == "*" {
            uses_namespace = true;
          } else {
            used_exports.push(export_name.clone());
          }
        }
        _ => {}
      }
    }

    runtime_usage.insert(
      runtime_str,
      RuntimeUsageInfo {
        used_exports: if used_exports.is_empty() {
          None
        } else {
          Some(used_exports)
        },
        uses_namespace: if uses_namespace { Some(true) } else { None },
        export_usage_states,
      },
    );
  }

  Ok(runtime_usage)
}

/// Extracts ConsumeShared module information
pub fn extract_consume_shared_info(
  module_id: &ModuleIdentifier,
  _module_graph: &ModuleGraph,
) -> (Option<String>, Option<String>) {
  let module_str = module_id.to_string();

  // Extract share key from ConsumeShared module identifier
  let share_key = if module_str.contains("consume shared module") {
    // Parse the module identifier to extract the share key
    // Format: "consume shared module (default) package-name@version (strict) (fallback: ...)"
    if let Some(start) = module_str.find(") ") {
      if let Some(end) = module_str[start + 2..].find("@") {
        Some(module_str[start + 2..start + 2 + end].to_string())
      } else if let Some(end) = module_str[start + 2..].find(" (") {
        Some(module_str[start + 2..start + 2 + end].to_string())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };

  // Extract fallback module path
  let fallback_module = if module_str.contains("(fallback: ") {
    if let Some(start) = module_str.find("(fallback: ") {
      if let Some(end) = module_str[start + 11..].find(")") {
        Some(module_str[start + 11..start + 11 + end].to_string())
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  };

  (share_key, fallback_module)
}

/// Gets simplified export usage analysis for potentially unused exports
pub fn get_simplified_export_usage(provided_exports: &[String]) -> Option<Vec<String>> {
  // Simple heuristic: mark exports as potentially unused if we have many exports
  // This is a placeholder for more sophisticated analysis
  if provided_exports.len() > 10 {
    // Assume some exports might be unused in large modules
    Some(provided_exports.iter().take(5).cloned().collect())
  } else {
    None
  }
}

/// Analyzes ConsumeShared modules with enhanced federation-aware usage tracking
#[allow(dead_code)]
pub fn analyze_consume_shared_module(
  module_graph: &ModuleGraph,
  module_id: &ModuleIdentifier,
  runtimes: &[RuntimeSpec],
  share_key: Option<String>,
  detailed_analysis: bool,
  include_runtime_info: bool,
) -> Option<ModuleExportUsage> {
  let module = module_graph.module_by_identifier(module_id)?;

  // Get the fallback module for detailed analysis
  let fallback_module_id = find_fallback_module(module_graph, module_id);

  // Analyze usage from consumers of this ConsumeShared module
  let consumer_usage =
    analyze_consume_shared_usage_from_consumers(module_graph, module_id, runtimes);

  // If we have a fallback module, get its export information and use that as the ConsumeShared provided exports
  let (provided_exports_vec, fallback_export_details) =
    if let Some(ref fallback_id_str) = fallback_module_id {
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
        let (fallback_provided, fallback_details) =
          get_fallback_module_exports(module_graph, &fallback_id, runtimes, detailed_analysis);

        // The ConsumeShared module should provide the same exports as its fallback
        (fallback_provided, fallback_details)
      } else {
        (vec!["*".to_string()], Vec::new())
      }
    } else {
      // For shared modules without fallback, get exports from the shared module itself
      let exports_info = module_graph.get_exports_info(module_id);
      let prefetch_mode = determine_optimal_prefetch_mode(module.as_ref(), &exports_info);
      let prefetched_exports =
        ExportsInfoGetter::prefetch(&exports_info, module_graph, prefetch_mode);

      // Get provided exports using the prefetched exports info
      let provided_exports = prefetched_exports.get_provided_exports();
      let provided_exports_vec = match provided_exports {
        ProvidedExports::Unknown => vec!["*unknown*".to_string()],
        ProvidedExports::ProvidedAll => vec!["*".to_string()],
        ProvidedExports::ProvidedNames(exports) => exports.iter().map(|e| e.to_string()).collect(),
      };

      // Get export details
      let export_details = if detailed_analysis {
        get_detailed_export_usage_from_prefetched(
          &prefetched_exports,
          &provided_exports_vec,
          module_graph,
        )?
      } else {
        get_simplified_export_usage(&provided_exports_vec)
          .unwrap_or_default()
          .into_iter()
          .map(|exp| ExportUsageDetail {
            export_name: exp,
            usage_state: "NotAnalyzed".to_string(),
            can_mangle: None,
            can_inline: None,
            is_provided: None,
            used_name: None,
          })
          .collect()
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
    merge_consume_shared_usage_data(
      &consumer_usage,
      &corrected_provided_exports,
      &fallback_export_details,
    );

  // Get detailed dependency information
  let dependencies = analyze_module_dependencies(module_graph, module_id).unwrap_or_default();

  // Check for side effects
  let has_side_effects = match module.factory_meta() {
    Some(meta) => Some(!meta.side_effect_free.unwrap_or_default()),
    None => None,
  };

  // Calculate potentially unused exports based on the merged analysis
  let potentially_unused_exports = calculate_unused_exports(
    &corrected_provided_exports,
    &merged_used_exports,
    &merged_uses_namespace,
    &merged_export_details,
  );

  // Get runtime-specific usage information if requested
  let runtime_usage = if include_runtime_info {
    Some(get_consume_shared_runtime_usage(
      module_graph,
      module_id,
      runtimes,
      &consumer_usage,
    ))
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

/// Helper functions that need to be implemented
#[allow(dead_code)]
pub fn find_fallback_module(
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

#[allow(dead_code)]
pub fn analyze_consume_shared_usage_from_consumers(
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
          }
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
          }
        }
      }

      // Fallback: also use general extraction method
      extract_import_usage_from_dependency(
        dependency.as_ref(),
        &mut used_exports,
        &mut uses_namespace,
        &mut import_types,
      );
    }
  }

  // Also check for usage through ESM import dependencies for additional analysis
  let (esm_used_exports, esm_uses_namespace) =
    analyze_esm_import_usage_static(module_graph, consume_shared_id);

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
    used_exports: if used_exports.is_empty() {
      None
    } else {
      Some(used_exports)
    },
    uses_namespace: Some(uses_namespace),
    import_types,
  }
}

#[allow(dead_code)]
pub fn get_fallback_module_exports(
  module_graph: &ModuleGraph,
  fallback_module_id: &ModuleIdentifier,
  _runtimes: &[RuntimeSpec],
  detailed_analysis: bool,
) -> (Vec<String>, Vec<ExportUsageDetail>) {
  if let Some(_fallback_module) = module_graph.module_by_identifier(fallback_module_id) {
    // Get exports info for the fallback module with optimized prefetch mode
    let exports_info = module_graph.get_exports_info(fallback_module_id);
    let prefetch_mode = determine_optimal_prefetch_mode(_fallback_module.as_ref(), &exports_info);
    let prefetched_exports =
      ExportsInfoGetter::prefetch(&exports_info, module_graph, prefetch_mode);

    // Get provided exports
    let provided_exports = prefetched_exports.get_provided_exports();
    let provided_exports_vec = match provided_exports {
      ProvidedExports::Unknown => vec!["*unknown*".to_string()],
      ProvidedExports::ProvidedAll => vec!["*".to_string()],
      ProvidedExports::ProvidedNames(exports) => exports.iter().map(|e| e.to_string()).collect(),
    };

    // Get detailed export usage information from the fallback module
    let export_details = if detailed_analysis {
      get_detailed_export_usage_from_prefetched(
        &prefetched_exports,
        &provided_exports_vec,
        module_graph,
      )
      .unwrap_or_else(|| {
        get_simplified_export_usage(&provided_exports_vec)
          .unwrap_or_default()
          .into_iter()
          .map(|exp| ExportUsageDetail {
            export_name: exp,
            usage_state: "NotAnalyzed".to_string(),
            can_mangle: None,
            can_inline: None,
            is_provided: None,
            used_name: None,
          })
          .collect()
      })
    } else {
      get_simplified_export_usage(&provided_exports_vec)
        .unwrap_or_default()
        .into_iter()
        .map(|exp| ExportUsageDetail {
          export_name: exp,
          usage_state: "NotAnalyzed".to_string(),
          can_mangle: None,
          can_inline: None,
          is_provided: None,
          used_name: None,
        })
        .collect()
    };

    (provided_exports_vec, export_details)
  } else {
    (vec!["*".to_string()], Vec::new())
  }
}

#[allow(dead_code)]
pub fn determine_optimal_prefetch_mode<'a>(
  module: &'a dyn rspack_core::Module,
  _exports_info: &'a rspack_core::ExportsInfo,
) -> PrefetchExportsInfoMode<'a> {
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
    }
    // For other module types, use targeted analysis
    ModuleType::ConsumeShared | ModuleType::ProvideShared => {
      // Shared modules need full analysis for federation optimization
      PrefetchExportsInfoMode::AllExports
    }
    // For CSS, Asset, and other modules, minimal analysis is sufficient
    _ => PrefetchExportsInfoMode::Default,
  }
}

#[allow(dead_code)]
pub fn get_detailed_export_usage_from_prefetched(
  prefetched_exports: &PrefetchedExportsInfoWrapper,
  provided_exports: &[String],
  _module_graph: &ModuleGraph,
) -> Option<Vec<ExportUsageDetail>> {
  let mut export_usage = Vec::new();

  // Analyze each provided export using the prefetched exports data
  for export_name in provided_exports {
    // Skip special markers
    if export_name.starts_with('*') || export_name.contains('?') {
      continue;
    }

    let export_atom = rspack_util::atom::Atom::from(export_name.as_str());

    // Get detailed export information from the prefetched data
    if let Some(export_info_data) = prefetched_exports
      .exports()
      .find(|(name, _)| **name == export_atom)
      .map(|(_, data)| data)
    {
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
        Inlinable::NoByUse | Inlinable::NoByProvide => Some(false),
      };

      // Check provision status
      let is_provided = export_info_data.provided().map(|p| match p {
        ExportProvided::Provided => true,
        ExportProvided::Unknown => false,
        ExportProvided::NotProvided => false,
      });

      // Get used name (considering mangling)
      let used_name = export_info_data.used_name().map(|n| format!("{:?}", n));

      export_usage.push(ExportUsageDetail {
        export_name: export_name.clone(),
        usage_state: usage_state.to_string(),
        can_mangle: can_mangle,
        can_inline,
        is_provided,
        used_name,
      });
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
        Inlinable::NoByUse | Inlinable::NoByProvide => Some(false),
      },
      is_provided: other_data.provided().map(|p| match p {
        ExportProvided::Provided => true,
        ExportProvided::Unknown => false,
        ExportProvided::NotProvided => false,
      }),
      used_name: other_data.used_name().map(|n| n.as_str().to_string()),
    });
  }

  Some(export_usage)
}

#[allow(dead_code)]
pub fn merge_consume_shared_usage_data(
  consumer_usage: &ConsumeSharedUsageInfo,
  provided_exports: &[String],
  fallback_export_details: &[ExportUsageDetail],
) -> (Option<Vec<String>>, Option<bool>, Vec<ExportUsageDetail>) {
  let mut merged_export_details = Vec::new();

  // Create export details based on consumer usage and fallback information
  for export_name in provided_exports {
    let is_used_by_consumer = consumer_usage
      .used_exports
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
      fallback_detail
        .map(|d| d.usage_state.as_str())
        .unwrap_or("Unused")
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

#[allow(dead_code)]
pub fn calculate_unused_exports(
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

#[allow(dead_code)]
pub fn get_consume_shared_runtime_usage(
  _module_graph: &ModuleGraph,
  _consume_shared_id: &ModuleIdentifier,
  runtimes: &[RuntimeSpec],
  consumer_usage: &ConsumeSharedUsageInfo,
) -> HashMap<String, RuntimeUsageInfo> {
  let mut runtime_info = HashMap::new();

  for runtime in runtimes {
    let runtime_key = format_runtime_key(runtime);

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

/// Formats runtime key for consistent runtime identification
#[allow(dead_code)]
pub fn format_runtime_key(runtime: &RuntimeSpec) -> String {
  // Create a deterministic, readable runtime key
  if runtime.is_empty() {
    "default".to_string()
  } else {
    let mut runtime_names: Vec<String> = runtime.iter().map(|s| s.to_string()).collect();
    runtime_names.sort();
    runtime_names.join("+")
  }
}

/// Extracts usage information from individual dependencies
#[allow(dead_code)]
pub fn extract_import_usage_from_dependency(
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
    }
    DependencyType::EsmImportSpecifier => {
      // Named imports - we'll need to infer from connection context
      // For now, mark as namespace usage to be safe
      *uses_namespace = true;
      import_types.insert("*".to_string(), "named_import".to_string());
    }
    DependencyType::EsmExportImportedSpecifier => {
      // Re-exports - mark as namespace usage
      *uses_namespace = true;
      import_types.insert("*".to_string(), "reexport".to_string());
    }
    _ => {
      // For other import types, assume namespace usage
      *uses_namespace = true;
    }
  }
}

/// Analyzes ESM import usage patterns using static analysis (without compilation context)
#[allow(dead_code)]
pub fn analyze_esm_import_usage_static(
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
        }
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
              }
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
              }
            }
          }

          // If we couldn't extract specific exports, mark as namespace
          if !found_specific_exports {
            uses_namespace = true;
          }
        }
        DependencyType::EsmExportImportedSpecifier => {
          // Re-export case - mark as namespace usage
          uses_namespace = true;
        }
        _ => {
          // For other dependency types, mark as namespace usage for safety
          uses_namespace = true;
        }
      }
    }
  }

  (used_exports, uses_namespace)
}
