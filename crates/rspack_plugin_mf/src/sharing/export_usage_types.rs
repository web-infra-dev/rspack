use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Information about export usage for any module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExportUsage {
  /// The module's shareKey (for shared modules) or identifier
  pub share_key: Option<String>,
  /// The module identifier
  pub module_identifier: String,
  /// All exports provided by this module
  pub provided_exports: Vec<String>,
  /// Exports that are actually used (if available)
  pub used_exports: Option<Vec<String>>,
  /// Whether all exports are used as a namespace
  pub uses_namespace: Option<bool>,
  /// The fallback module path (if any, for ConsumeShared modules)
  pub fallback_module: Option<String>,
  /// Module type
  pub module_type: String,
  /// Whether this module has side effects
  pub has_side_effects: Option<bool>,
  /// Potential unused exports (if we can determine them)
  pub potentially_unused_exports: Option<Vec<String>>,
  /// Import dependencies of this module (what it imports) with details
  pub dependencies: Vec<DependencyDetail>,
  /// Detailed export usage information (export name -> usage state)
  pub export_usage_details: Vec<ExportUsageDetail>,
  /// Runtime-specific usage information
  pub runtime_usage: Option<HashMap<String, RuntimeUsageInfo>>,
}

/// Detailed dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyDetail {
  /// Type of dependency
  pub dependency_type: String,
  /// Target module (if available)
  pub target_module: Option<String>,
  /// What's being imported/required
  pub request: Option<String>,
  /// Connection state (active, transitive, etc.)
  pub connection_state: String,
  /// If it's a module federation related dependency
  pub is_module_federation: bool,
}

/// Detailed export usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUsageDetail {
  /// Export name
  pub export_name: String,
  /// Usage state (Used, Unused, OnlyPropertiesUsed, etc.)
  pub usage_state: String,
  /// Whether it can be mangled
  pub can_mangle: Option<bool>,
  /// Whether it can be inlined
  pub can_inline: Option<bool>,
  /// Whether this export is provided
  pub is_provided: Option<bool>,
  /// Used name (if different from export name due to mangling)
  pub used_name: Option<String>,
}

/// Runtime-specific usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeUsageInfo {
  /// Used exports in this runtime
  pub used_exports: Option<Vec<String>>,
  /// Whether namespace is used in this runtime
  pub uses_namespace: Option<bool>,
  /// Usage state for specific exports
  pub export_usage_states: HashMap<String, String>,
}

/// Usage information extracted from ConsumeShared module consumers
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConsumeSharedUsageInfo {
  /// Exports that consumers are importing
  pub used_exports: Option<Vec<String>>,
  /// Whether consumers are using namespace imports
  pub uses_namespace: Option<bool>,
  /// Map of export name to import type (default_import, named_import, reexport)
  pub import_types: std::collections::HashMap<String, String>,
}

/// Information about re-export usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReexportUsageDetail {
  /// The module doing the re-export
  pub reexporting_module: String,
  /// Original export name
  pub original_export: String,
  /// Re-exported name (if different)
  pub reexported_name: Option<String>,
  /// Whether the re-export is used
  pub is_used: bool,
}

/// Complete export usage report for all modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExportReport {
  /// Timestamp when this report was generated
  pub timestamp: String,
  /// Map of module identifier to export usage information
  pub modules: HashMap<String, ModuleExportUsage>,
  /// Summary statistics
  pub summary: ExportUsageSummary,
  /// Analysis metadata
  pub metadata: AnalysisMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUsageSummary {
  /// Total number of modules analyzed
  pub total_modules: usize,
  /// Number of ConsumeShared modules
  pub consume_shared_modules: usize,
  /// Number of ProvideShared modules
  pub provide_shared_modules: usize,
  /// Number of regular JavaScript modules
  pub javascript_modules: usize,
  /// Number of modules with specific used exports
  pub modules_with_specific_usage: usize,
  /// Number of modules using namespace imports
  pub modules_with_namespace_usage: usize,
  /// Number of modules with unknown usage
  pub modules_with_unknown_usage: usize,
  /// Number of modules with provided exports
  pub modules_with_provided_exports: usize,
  /// Number of modules with potentially unused exports
  pub modules_with_potentially_unused_exports: usize,
  /// Total number of dependencies analyzed
  pub total_dependencies: usize,
  /// Number of module federation dependencies
  pub module_federation_dependencies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
  /// Number of runtimes analyzed
  pub runtimes_analyzed: usize,
  /// Whether detailed usage analysis was performed
  pub detailed_analysis_enabled: bool,
  /// Analysis version
  pub analysis_version: String,
}

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

#[derive(Debug)]
pub struct SharedExportUsagePluginOptions {
  /// Output filename for the export usage report (default: "module-export-usage.json")
  #[allow(dead_code)]
  pub filename: String,
  /// Whether to include detailed runtime information
  #[allow(dead_code)]
  pub include_runtime_info: bool,
  /// Whether to include all modules or just shared modules (default: true for all modules)
  #[allow(dead_code)]
  pub include_all_modules: bool,
  /// Whether to perform detailed usage analysis (like flag dependency usage plugin)
  #[allow(dead_code)]
  pub detailed_analysis: bool,
}

impl Default for SharedExportUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "module-export-usage.json".to_string(),
      include_runtime_info: true,
      include_all_modules: true,
      detailed_analysis: true,
    }
  }
}
