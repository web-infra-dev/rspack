use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilerEmit, CompilerOptions,
  ExportInfoGetter, ExportsInfoGetter, ExtendedReferencedExport, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, Plugin, PluginContext,
  PrefetchExportsInfoMode, ProvidedExports, RuntimeSpec, UsageState,
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};
use tracing::{debug, trace, warn};

/// Simple module export data for easy consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleModuleExports {
  /// Used exports
  pub used_exports: Vec<String>,
  /// Unused exports
  pub unused_exports: Vec<String>,
  /// Possibly unused exports
  pub possibly_unused_exports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageReport {
  pub consume_shared_modules: HashMap<String, SimpleModuleExports>,
  pub metadata: AnalysisMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
  pub total_modules: usize,
  pub modules_with_unused_exports: usize,
  pub analysis_timestamp: String,
  pub plugin_version: String,
}

#[derive(Debug)]
pub struct ShareUsagePluginOptions {
  pub filename: String,
  pub enable_detailed_analysis: bool,
  pub include_runtime_info: bool,
}

impl Default for ShareUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "share-usage.json".to_string(),
      enable_detailed_analysis: true,
      include_runtime_info: false,
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

  /// Analyze consume shared usage with proper error handling and bulk operations
  fn analyze_consume_shared_usage(
    &self,
    compilation: &Compilation,
  ) -> Result<HashMap<String, SimpleModuleExports>> {
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    // Collect runtimes for comprehensive analysis
    let runtimes: Vec<RuntimeSpec> = compilation
      .chunk_by_ukey
      .values()
      .map(|chunk| chunk.runtime())
      .cloned()
      .collect();

    debug!(
      "Starting ConsumeShared usage analysis for {} modules",
      module_graph.modules().len()
    );

    // Find all ConsumeShared modules
    for (module_id, module) in module_graph.modules() {
      if module.module_type() == &ModuleType::ConsumeShared {
        if let Some(share_key) = module.get_consume_shared_key() {
          trace!("Analyzing ConsumeShared module: {}", share_key);

          match self.analyze_single_consume_shared_module(&module_graph, &module_id, &runtimes) {
            Ok(analysis) => {
              usage_map.insert(share_key, analysis);
            }
            Err(e) => {
              warn!("Failed to analyze ConsumeShared module '{share_key}': {e}");
              // Continue with empty data rather than failing completely
              usage_map.insert(
                share_key,
                SimpleModuleExports {
                  used_exports: Vec::new(),
                  unused_exports: Vec::new(),
                  possibly_unused_exports: Vec::new(),
                },
              );
            }
          }
        }
      }
    }

    debug!(
      "Completed analysis for {} ConsumeShared modules",
      usage_map.len()
    );
    Ok(usage_map)
  }

  /// Analyze a single ConsumeShared module with comprehensive error handling
  fn analyze_single_consume_shared_module(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec],
  ) -> Result<SimpleModuleExports> {
    // Find the fallback module
    let fallback_id = self
      .find_fallback_module_id(module_graph, consume_shared_id)
      .ok_or_else(|| rspack_error::Error::msg("Fallback module not found"))?;

    // Analyze fallback module exports using proper bulk operations
    let provided_exports = self.get_provided_exports_bulk(module_graph, &fallback_id)?;

    // Analyze usage through incoming connections with proper dependency analysis
    let (used_exports, imported_exports) =
      self.analyze_dependency_usage(module_graph, consume_shared_id, &fallback_id, runtimes)?;

    // Calculate unused exports efficiently
    let unused_exports =
      self.calculate_unused_exports(&provided_exports, &used_exports, &imported_exports);

    // Identify potentially unused exports using heuristics
    let possibly_unused_exports = self.identify_possibly_unused_exports(
      module_graph,
      &fallback_id,
      &provided_exports,
      &used_exports,
    )?;

    Ok(SimpleModuleExports {
      used_exports,
      unused_exports,
      possibly_unused_exports,
    })
  }

  /// Get provided exports using efficient bulk operations
  fn get_provided_exports_bulk(
    &self,
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
  ) -> Result<Vec<String>> {
    let exports_info = module_graph.get_exports_info(module_id);
    let prefetched = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::AllExports,
    );

    let provided_exports = prefetched.get_provided_exports();
    let exports = match provided_exports {
      ProvidedExports::ProvidedNames(names) => names.iter().map(|n| n.to_string()).collect(),
      ProvidedExports::ProvidedAll => vec!["*".to_string()],
      ProvidedExports::Unknown => Vec::new(),
    };

    trace!(
      "Found {} provided exports for module {module_id}",
      exports.len()
    );
    Ok(exports)
  }

  /// Analyze dependency usage with comprehensive error handling
  fn analyze_dependency_usage(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    fallback_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec],
  ) -> Result<(Vec<String>, Vec<String>)> {
    let mut used_exports = Vec::new();
    let mut imported_exports = Vec::new();

    // Analyze fallback module usage patterns
    let fallback_used = self.analyze_fallback_usage(module_graph, fallback_id, runtimes.first())?;
    used_exports.extend(fallback_used);

    // Analyze incoming dependencies for imported exports
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        match self.extract_referenced_exports(dependency.as_ref(), module_graph) {
          Ok(refs) => imported_exports.extend(refs),
          Err(e) => {
            trace!("Failed to extract referenced exports: {e}");
            continue;
          }
        }
      }
    }

    // Deduplicate
    used_exports.sort();
    used_exports.dedup();
    imported_exports.sort();
    imported_exports.dedup();

    Ok((used_exports, imported_exports))
  }

  /// Analyze fallback module usage with proper error handling
  fn analyze_fallback_usage(
    &self,
    module_graph: &ModuleGraph,
    fallback_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<Vec<String>> {
    let exports_info = module_graph.get_exports_info(fallback_id);
    let prefetched = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::AllExports,
    );

    let provided_exports = prefetched.get_provided_exports();
    let mut used_exports = Vec::new();

    if let ProvidedExports::ProvidedNames(names) = provided_exports {
      for export_name in names {
        let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
        let export_info_data = prefetched.get_read_only_export_info(&export_atom);
        let usage_state = ExportInfoGetter::get_used(export_info_data, runtime);

        if matches!(
          usage_state,
          UsageState::Used | UsageState::OnlyPropertiesUsed
        ) {
          used_exports.push(export_name.to_string());
        }
      }
    }

    Ok(used_exports)
  }

  /// Extract referenced exports with proper error handling
  fn extract_referenced_exports(
    &self,
    dependency: &dyn rspack_core::Dependency,
    module_graph: &ModuleGraph,
  ) -> Result<Vec<String>> {
    let mut referenced_exports = Vec::new();

    let exports =
      dependency.get_referenced_exports(module_graph, &ModuleGraphCacheArtifact::default(), None);

    for export_ref in exports {
      match export_ref {
        ExtendedReferencedExport::Array(names) => {
          for name in names {
            referenced_exports.push(name.to_string());
          }
        }
        ExtendedReferencedExport::Export(export_info) => {
          if export_info.name.is_empty() {
            referenced_exports.push("*".to_string());
          } else {
            for name in export_info.name {
              referenced_exports.push(name.to_string());
            }
          }
        }
      }
    }

    Ok(referenced_exports)
  }

  /// Calculate unused exports efficiently
  fn calculate_unused_exports(
    &self,
    provided_exports: &[String],
    used_exports: &[String],
    imported_exports: &[String],
  ) -> Vec<String> {
    let mut unused = Vec::new();

    for export in provided_exports {
      if !used_exports.contains(export) && !export.starts_with('*') {
        unused.push(export.clone());
      }
    }

    // Also include imported but unused exports
    for export in imported_exports {
      if !used_exports.contains(export) && !unused.contains(export) && !export.starts_with('*') {
        unused.push(export.clone());
      }
    }

    unused.sort();
    unused.dedup();
    unused
  }

  /// Identify potentially unused exports using heuristics
  fn identify_possibly_unused_exports(
    &self,
    module_graph: &ModuleGraph,
    fallback_id: &ModuleIdentifier,
    provided_exports: &[String],
    used_exports: &[String],
  ) -> Result<Vec<String>> {
    let mut possibly_unused = Vec::new();

    if !self.options.enable_detailed_analysis {
      return Ok(possibly_unused);
    }

    let exports_info = module_graph.get_exports_info(fallback_id);
    let prefetched = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::AllExports,
    );

    for export_name in provided_exports {
      if used_exports.contains(export_name) || export_name.starts_with('*') {
        continue;
      }

      let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
      let export_info_data = prefetched.get_read_only_export_info(&export_atom);
      let usage_state = ExportInfoGetter::get_used(export_info_data, None);

      if matches!(usage_state, UsageState::NoInfo | UsageState::Unknown) {
        possibly_unused.push(export_name.clone());
      }
    }

    Ok(possibly_unused)
  }

  /// Find fallback module ID using ConsumeSharedModule API
  fn find_fallback_module_id(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    let module = module_graph.module_by_identifier(consume_shared_id)?;

    if let Some(consume_shared) = module
      .as_any()
      .downcast_ref::<crate::sharing::consume_shared_module::ConsumeSharedModule>(
    ) {
      // Use the enhanced API with proper error handling
      match consume_shared.find_fallback_module_id(module_graph) {
        Ok(fallback_id) => fallback_id,
        Err(_) => {
          // Log warning but don't fail - graceful degradation
          tracing::warn!("Failed to find fallback module for ConsumeShared: {consume_shared_id}");
          // Fallback to string parsing
          self.parse_fallback_from_identifier(module_graph, consume_shared_id)
        }
      }
    } else {
      // Fallback to string parsing for non-ConsumeShared modules
      self.parse_fallback_from_identifier(module_graph, consume_shared_id)
    }
  }

  /// Parse fallback module from ConsumeShared identifier string
  fn parse_fallback_from_identifier(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    let consume_shared_str = consume_shared_id.to_string();

    if let Some(fallback_start) = consume_shared_str.find("(fallback: ") {
      let fallback_path_start = fallback_start + "(fallback: ".len();
      if let Some(fallback_end) = consume_shared_str[fallback_path_start..].find(')') {
        let fallback_path =
          &consume_shared_str[fallback_path_start..fallback_path_start + fallback_end];

        // Find matching module
        for (module_id, _) in module_graph.modules() {
          let module_id_str = module_id.to_string();
          if module_id_str == fallback_path || module_id_str.ends_with(fallback_path) {
            return Some(module_id);
          }
        }
      }
    }

    None
  }

  /// Generate comprehensive report with metadata
  fn generate_report(&self, compilation: &Compilation) -> Result<ShareUsageReport> {
    let consume_shared_modules = self.analyze_consume_shared_usage(compilation)?;

    let modules_with_unused_exports = consume_shared_modules
      .values()
      .filter(|module| !module.unused_exports.is_empty())
      .count();

    let metadata = AnalysisMetadata {
      total_modules: consume_shared_modules.len(),
      modules_with_unused_exports,
      analysis_timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| rspack_error::Error::msg(format!("Time error: {e}")))?
        .as_secs()
        .to_string(),
      plugin_version: "1.0.0".to_string(),
    };

    Ok(ShareUsageReport {
      consume_shared_modules,
      metadata,
    })
  }
}

#[plugin_hook(CompilerEmit for ShareUsagePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  debug!("Starting ShareUsagePlugin emit phase");

  let report = self.generate_report(compilation).map_err(|e| {
    let diagnostic = Diagnostic::error(
      "ShareUsagePlugin".to_string(),
      format!("ShareUsagePlugin analysis failed: {e}"),
    );
    compilation.push_diagnostic(diagnostic);
    e
  })?;

  let content = serde_json::to_string_pretty(&report)
    .map_err(|e| rspack_error::Error::msg(format!("Failed to serialize report: {e}")))?;

  compilation.emit_asset(
    self.options.filename.clone(),
    CompilationAsset::new(Some(RawSource::from(content).boxed()), AssetInfo::default()),
  );

  debug!(
    "ShareUsagePlugin report generated: {}",
    self.options.filename
  );
  Ok(())
}

#[async_trait]
impl Plugin for ShareUsagePlugin {
  fn name(&self) -> &'static str {
    "rspack.ShareUsagePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx.context.compiler_hooks.emit.tap(emit::new(self));
    Ok(())
  }
}
