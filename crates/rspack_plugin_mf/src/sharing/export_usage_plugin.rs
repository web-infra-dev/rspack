use std::collections::HashMap;

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilerEmit, CompilerOptions,
  ModuleGraph, ModuleIdentifier, ModuleType, Plugin, PluginContext, RuntimeSpec,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

use super::{export_usage_analysis::analyze_module, export_usage_types::*};

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
    analyze_module(
      module_id,
      module_graph,
      runtimes,
      self.options.detailed_analysis,
    )
    .ok()
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

    // Calculate summary statistics
    let total_modules = modules.len();
    let consume_shared_modules = modules
      .values()
      .filter(|m| m.module_type == "ConsumeShared")
      .count();
    let provide_shared_modules = modules
      .values()
      .filter(|m| m.module_type == "ProvideShared")
      .count();
    let javascript_modules = modules
      .values()
      .filter(|m| m.module_type == "Javascript")
      .count();

    let modules_with_specific_usage = modules
      .values()
      .filter(|m| m.used_exports.is_some())
      .count();
    let modules_with_namespace_usage = modules
      .values()
      .filter(|m| m.uses_namespace.unwrap_or(false))
      .count();
    let modules_with_unknown_usage = modules
      .values()
      .filter(|m| m.used_exports.is_none() && !m.uses_namespace.unwrap_or(false))
      .count();

    let modules_with_provided_exports = modules
      .values()
      .filter(|m| !m.provided_exports.is_empty())
      .count();
    let modules_with_potentially_unused_exports = modules
      .values()
      .filter(|m| {
        m.potentially_unused_exports
          .as_ref()
          .map(|exports| !exports.is_empty())
          .unwrap_or(false)
      })
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
      analysis_version: "1.0.0".to_string(),
    };

    Ok(ModuleExportReport {
      timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .to_string(),
      modules,
      summary,
      metadata,
    })
  }

  /// Generate a simplified report with just module ID -> used/unused/possibly unused exports
  fn generate_simple_report(
    &self,
    compilation: &Compilation,
  ) -> Result<HashMap<String, SimpleModuleExports>> {
    let module_graph = compilation.get_module_graph();
    let mut simple_modules = HashMap::new();

    // Collect all runtimes for analysis
    let runtimes: Vec<RuntimeSpec> = compilation
      .chunk_by_ukey
      .values()
      .map(|chunk| chunk.runtime())
      .cloned()
      .collect();

    // Analyze all modules and extract simplified export information
    for (module_id, _module) in module_graph.modules() {
      if let Some(usage_info) =
        self.analyze_module(&module_graph, &module_id, compilation, &runtimes)
      {
        let used_exports = usage_info.used_exports.unwrap_or_default();
        let provided_exports = usage_info.provided_exports;
        let possibly_unused = usage_info.potentially_unused_exports.unwrap_or_default();

        // Calculate unused exports (those that are provided but not used)
        let mut unused_exports = Vec::new();
        for export in &provided_exports {
          if !used_exports.contains(export) && !possibly_unused.contains(export) && export != "*" {
            unused_exports.push(export.clone());
          }
        }

        simple_modules.insert(
          module_id.to_string(),
          SimpleModuleExports {
            used_exports,
            unused_exports,
            possibly_unused_exports: possibly_unused,
          },
        );
      }
    }

    Ok(simple_modules)
  }
}

#[plugin_hook(CompilerEmit for SharedExportUsagePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  // Generate the export usage report
  let report = self.generate_report(compilation)?;
  let json = serde_json::to_string_pretty(&report)
    .map_err(|e| rspack_error::Error::msg(format!("Failed to serialize report: {}", e)))?;

  // Create the asset
  compilation.emit_asset(
    self.options.filename.clone(),
    CompilationAsset::new(Some(RawSource::from(json).boxed()), AssetInfo::default()),
  );

  // Also generate a simplified report for easier consumption
  let simple_filename = format!("simple-{}", self.options.filename.replace(".json", ".json"));
  let simple_report = self.generate_simple_report(compilation)?;
  let simple_json = serde_json::to_string_pretty(&simple_report)
    .map_err(|e| rspack_error::Error::msg(format!("Failed to serialize simple report: {}", e)))?;

  compilation.emit_asset(
    simple_filename,
    CompilationAsset::new(
      Some(RawSource::from(simple_json).boxed()),
      AssetInfo::default(),
    ),
  );

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
