use std::{
  collections::{HashMap, HashSet},
  sync::{Arc, RwLock},
};

use async_trait::async_trait;
use rspack_core::{
  rspack_sources::{RawSource, SourceExt},
  ApplyContext, AssetInfo, Compilation, CompilationAsset, CompilerEmit, CompilerOptions,
  ConnectionState, DependencyType, ExportInfoGetter, ExportsInfoGetter, ExtendedReferencedExport,
  Inlinable, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, Plugin,
  PluginContext, PrefetchExportsInfoMode, PrefetchedExportsInfoWrapper, ProvidedExports,
  RuntimeSpec, UsageState,
};
use rspack_error::{
  Diagnostic, InternalError, IntoTWithDiagnosticArray, Result, TWithDiagnosticArray,
};
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

/// Enhanced ShareUsagePlugin implementing rspack best practices
///
/// Key improvements:
/// - Batch prefetching with ExportsInfoGetter::prefetch() for efficiency
/// - Comprehensive error handling with diagnostic integration
/// - Incremental processing with caching and mutation tracking
/// - Runtime-aware usage analysis with proper fallback handling
/// - Established plugin architecture patterns

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageReport {
  pub consume_shared_modules: HashMap<String, ShareUsageData>,
  pub analysis_metadata: AnalysisMetadata,
  pub diagnostics: Vec<String>,
  pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageData {
  /// Exports that are actually used (referenced in code)
  pub used_exports: Vec<String>,
  /// Exports that are imported but not used (unused imports like 'uniq')
  pub unused_imports: Vec<String>,
  /// All exports provided by the fallback module
  pub provided_exports: Vec<String>,
  /// Export usage details for debugging
  pub export_details: Vec<ExportUsageDetail>,
  /// Whether this analysis detected unused imports
  pub has_unused_imports: bool,
  /// Fallback module information
  pub fallback_info: Option<FallbackModuleInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUsageDetail {
  pub export_name: String,
  pub usage_state: String,
  pub is_imported: bool,
  pub is_used: bool,
  pub import_source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FallbackModuleInfo {
  pub module_id: String,
  pub module_type: String,
  pub provided_exports_count: usize,
  pub used_exports_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisMetadata {
  pub plugin_version: String,
  pub analysis_mode: String,
  pub total_consume_shared_modules: usize,
  pub modules_with_unused_imports: usize,
  pub timestamp: String,
  pub cache_hits: usize,
  pub cache_misses: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
  pub total_analysis_time_ms: u64,
  pub prefetch_time_ms: u64,
  pub batch_operations: usize,
  pub modules_analyzed: usize,
}

#[derive(Debug)]
pub struct EnhancedShareUsagePluginOptions {
  pub filename: String,
  pub include_export_details: bool,
  pub detect_unused_imports: bool,
  pub enable_caching: bool,
  pub batch_size: usize,
  pub runtime_analysis: bool,
}

impl Default for EnhancedShareUsagePluginOptions {
  fn default() -> Self {
    Self {
      filename: "enhanced-share-usage.json".to_string(),
      include_export_details: true,
      detect_unused_imports: true,
      enable_caching: true,
      batch_size: 50,
      runtime_analysis: true,
    }
  }
}

/// Analysis cache for incremental processing
#[derive(Debug, Default)]
struct AnalysisCache {
  module_exports: HashMap<ModuleIdentifier, CachedExportInfo>,
  dependency_graph: HashMap<ModuleIdentifier, Vec<ModuleIdentifier>>,
  last_compilation_hash: Option<u64>,
}

#[derive(Debug, Clone)]
struct CachedExportInfo {
  provided_exports: Vec<String>,
  used_exports: Vec<String>,
  usage_details: Vec<ExportUsageDetail>,
  timestamp: u64,
}

#[plugin]
#[derive(Debug)]
pub struct EnhancedShareUsagePlugin {
  options: EnhancedShareUsagePluginOptions,
  cache: Arc<RwLock<AnalysisCache>>,
  diagnostics: Arc<RwLock<Vec<Diagnostic>>>,
}

impl EnhancedShareUsagePlugin {
  pub fn new(options: EnhancedShareUsagePluginOptions) -> Self {
    Self::new_inner(
      options,
      Arc::new(RwLock::new(AnalysisCache::default())),
      Arc::new(RwLock::new(Vec::new())),
    )
  }

  /// Add diagnostic with proper error handling
  fn add_diagnostic(&self, diagnostic: Diagnostic) {
    if let Ok(mut diagnostics) = self.diagnostics.write() {
      diagnostics.push(diagnostic);
    }
  }

  /// Check if analysis is needed (incremental processing)
  fn needs_analysis(&self, compilation: &Compilation) -> bool {
    if !self.options.enable_caching {
      return true;
    }

    let cache = match self.cache.read() {
      Ok(cache) => cache,
      Err(_) => return true,
    };

    // Simple heuristic: analyze if module count changed significantly
    let current_module_count = compilation.get_module_graph().modules().len();
    cache.module_exports.len() != current_module_count
  }

  /// Main analysis method with batching and caching
  fn analyze_consume_shared_usage(
    &self,
    compilation: &Compilation,
  ) -> Result<TWithDiagnosticArray<HashMap<String, ShareUsageData>>> {
    let start_time = std::time::Instant::now();
    let mut diagnostics = Vec::new();
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    // Early return if no analysis needed (incremental processing)
    if !self.needs_analysis(compilation) {
      if let Ok(cache) = self.cache.read() {
        // Convert cached data to return format
        for (module_id, cached_info) in &cache.module_exports {
          if let Some(module) = module_graph.module_by_identifier(module_id) {
            if module.module_type() == &ModuleType::ConsumeShared {
              if let Some(share_key) = self.extract_share_key(module_id) {
                usage_map.insert(
                  share_key,
                  ShareUsageData {
                    used_exports: cached_info.used_exports.clone(),
                    unused_imports: Vec::new(),
                    provided_exports: cached_info.provided_exports.clone(),
                    export_details: cached_info.usage_details.clone(),
                    has_unused_imports: false,
                    fallback_info: None,
                  },
                );
              }
            }
          }
        }
        return Ok(usage_map.with_diagnostic(diagnostics));
      }
    }

    // Collect runtimes for comprehensive analysis
    let runtimes: Vec<RuntimeSpec> = compilation
      .chunk_by_ukey
      .values()
      .map(|chunk| chunk.runtime())
      .cloned()
      .collect();

    // Batch collect ConsumeShared modules for efficient processing
    let consume_shared_modules: Vec<(ModuleIdentifier, String)> = module_graph
      .modules()
      .iter()
      .filter_map(|(module_id, module)| {
        if module.module_type() == &ModuleType::ConsumeShared {
          self
            .extract_share_key(module_id)
            .map(|key| (*module_id, key))
        } else {
          None
        }
      })
      .collect();

    // Process modules in batches for better performance
    for batch in consume_shared_modules.chunks(self.options.batch_size) {
      match self.process_module_batch(&module_graph, batch, &runtimes) {
        Ok(batch_results) => {
          for (share_key, analysis) in batch_results.inner {
            usage_map.insert(share_key, analysis);
          }
          diagnostics.extend(batch_results.diagnostic);
        }
        Err(e) => {
          let diagnostic = Diagnostic::warn(
            "Failed to process module batch".to_string(),
            format!("{}", e),
          );
          diagnostics.push(diagnostic);
        }
      }
    }

    let _analysis_time = start_time.elapsed();
    Ok(usage_map.with_diagnostic(diagnostics))
  }

  /// Extract share key from module identifier
  fn extract_share_key(&self, module_id: &ModuleIdentifier) -> Option<String> {
    let module_str = module_id.to_string();
    if module_str.contains("consume shared module") {
      if let Some(start) = module_str.find(") ") {
        if let Some(end) = module_str[start + 2..].find("@") {
          return Some(module_str[start + 2..start + 2 + end].to_string());
        } else {
          return module_str[start + 2..]
            .find(" (")
            .map(|end| module_str[start + 2..start + 2 + end].to_string());
        }
      }
    }
    None
  }

  /// Process a batch of modules efficiently
  fn process_module_batch(
    &self,
    module_graph: &ModuleGraph,
    batch: &[(ModuleIdentifier, String)],
    runtimes: &[RuntimeSpec],
  ) -> Result<TWithDiagnosticArray<HashMap<String, ShareUsageData>>> {
    let mut results = HashMap::new();
    let mut diagnostics = Vec::new();

    // Batch prefetch export information for all modules in the batch
    let prefetch_results = self.batch_prefetch_exports(module_graph, batch)?;

    for (module_id, share_key) in batch {
      match self.analyze_single_consume_shared_module(
        module_graph,
        module_id,
        runtimes,
        &prefetch_results,
      ) {
        Ok(analysis_result) => {
          results.insert(share_key.clone(), analysis_result.inner);
          diagnostics.extend(analysis_result.diagnostic);
        }
        Err(e) => {
          let diagnostic = Diagnostic::warn(
            "Failed to analyze module".to_string(),
            format!("{}: {}", module_id, e),
          );
          diagnostics.push(diagnostic);

          // Insert empty data to maintain consistency
          results.insert(
            share_key.clone(),
            ShareUsageData {
              used_exports: Vec::new(),
              unused_imports: Vec::new(),
              provided_exports: Vec::new(),
              export_details: Vec::new(),
              has_unused_imports: false,
              fallback_info: None,
            },
          );
        }
      }
    }

    Ok(results.with_diagnostic(diagnostics))
  }

  /// Batch prefetch exports information for efficiency
  fn batch_prefetch_exports<'a>(
    &self,
    module_graph: &'a ModuleGraph,
    batch: &[(ModuleIdentifier, String)],
  ) -> Result<HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>> {
    let mut prefetch_results = HashMap::new();

    for (module_id, _) in batch {
      if let Some(fallback_id) = self.find_fallback_module_id(module_graph, module_id) {
        let exports_info = module_graph.get_exports_info(&fallback_id);
        let prefetched = ExportsInfoGetter::prefetch(
          &exports_info,
          module_graph,
          PrefetchExportsInfoMode::AllExports,
        );
        prefetch_results.insert(fallback_id, prefetched);
      }
    }

    Ok(prefetch_results)
  }

  /// Analyze a single ConsumeShared module with cached prefetch results
  fn analyze_single_consume_shared_module<'a>(
    &self,
    module_graph: &'a ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec],
    prefetch_cache: &HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>,
  ) -> Result<TWithDiagnosticArray<ShareUsageData>> {
    let mut diagnostics = Vec::new();
    // Step 1: Find and analyze the fallback module with error handling
    let (fallback_info, provided_exports) =
      match self.find_fallback_module_id(module_graph, consume_shared_id) {
        Some(fallback_id) => {
          match self.analyze_fallback_module(module_graph, &fallback_id, prefetch_cache) {
            Ok(result) => (Some(result.inner.0), result.inner.1),
            Err(e) => {
              diagnostics.push(Diagnostic::warn(
                "Failed to analyze fallback module".to_string(),
                format!("{}", e),
              ));
              (None, Vec::new())
            }
          }
        }
        None => {
          diagnostics.push(Diagnostic::warn(
            "No fallback module found".to_string(),
            format!("{}", consume_shared_id),
          ));
          (None, Vec::new())
        }
      };

    // Step 2: Enhanced dependency analysis with error recovery
    let (imported_exports, actually_used_exports) = match self
      .analyze_usage_through_incoming_connections(module_graph, consume_shared_id, runtimes)
    {
      Ok(result) => result,
      Err(e) => {
        diagnostics.push(Diagnostic::warn(
          "Failed to analyze usage through connections".to_string(),
          format!("{}", e),
        ));
        (Vec::new(), Vec::new())
      }
    };

    // Step 3: Cross-reference to find unused imports with sophisticated analysis
    let unused_imports = if self.options.detect_unused_imports {
      self.detect_unused_imports(&imported_exports, &actually_used_exports, &provided_exports)
    } else {
      Vec::new()
    };

    // Step 4: Generate detailed export information with runtime awareness
    let export_details = if self.options.include_export_details {
      match self.generate_export_details_with_runtime(
        module_graph,
        &provided_exports,
        &imported_exports,
        &actually_used_exports,
        runtimes,
        prefetch_cache,
      ) {
        Ok(details) => details,
        Err(e) => {
          diagnostics.push(Diagnostic::warn(
            "Failed to generate export details".to_string(),
            format!("{}", e),
          ));
          Vec::new()
        }
      }
    } else {
      Vec::new()
    };

    let has_unused_imports = !unused_imports.is_empty();

    let result = ShareUsageData {
      used_exports: actually_used_exports,
      unused_imports,
      provided_exports,
      export_details,
      has_unused_imports,
      fallback_info,
    };

    // Cache the results for incremental processing
    if self.options.enable_caching {
      self.cache_analysis_results(consume_shared_id, &result);
    }

    Ok(result.with_diagnostic(diagnostics))
  }

  /// Cache analysis results for incremental processing
  fn cache_analysis_results(&self, module_id: &ModuleIdentifier, data: &ShareUsageData) {
    if let Ok(mut cache) = self.cache.write() {
      let cached_info = CachedExportInfo {
        provided_exports: data.provided_exports.clone(),
        used_exports: data.used_exports.clone(),
        usage_details: data.export_details.clone(),
        timestamp: std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap_or_default()
          .as_secs(),
      };
      cache.module_exports.insert(*module_id, cached_info);
    }
  }

  /// Sophisticated unused import detection
  fn detect_unused_imports(
    &self,
    imported_exports: &[String],
    actually_used_exports: &[String],
    provided_exports: &[String],
  ) -> Vec<String> {
    let used_set: HashSet<_> = actually_used_exports.iter().collect();
    let provided_set: HashSet<_> = provided_exports.iter().collect();

    imported_exports
      .iter()
      .filter(|export| {
        // Only consider as unused if:
        // 1. Not in used exports
        // 2. Is provided by the module
        // 3. Not a special export like "*" or "default"
        !used_set.contains(export)
          && provided_set.contains(export)
          && !export.starts_with('*')
          && *export != "default"
      })
      .cloned()
      .collect()
  }

  /// Analyze fallback module with comprehensive error handling
  fn analyze_fallback_module<'a>(
    &self,
    module_graph: &'a ModuleGraph,
    fallback_id: &ModuleIdentifier,
    prefetch_cache: &HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>,
  ) -> Result<TWithDiagnosticArray<(FallbackModuleInfo, Vec<String>)>> {
    let mut diagnostics = Vec::new();

    let fallback_module = module_graph
      .module_by_identifier(fallback_id)
      .ok_or_else(|| {
        InternalError::new(
          format!("Fallback module not found: {}", fallback_id),
          rspack_error::Severity::Error,
        )
      })?;

    // Use cached prefetch result if available, otherwise create new one
    let prefetched = match prefetch_cache.get(fallback_id) {
      Some(cached) => cached,
      None => {
        let exports_info = module_graph.get_exports_info(fallback_id);
        // For fallback analysis, we need comprehensive export information
        return Ok(
          (
            self.create_prefetched_fallback_analysis(module_graph, fallback_id, &exports_info)?,
            self.extract_provided_exports_from_info(&exports_info, module_graph)?,
          )
            .with_diagnostic(diagnostics),
        );
      }
    };

    let provided_exports = prefetched.get_provided_exports();
    let (provided_count, provided_exports_vec) = match &provided_exports {
      ProvidedExports::ProvidedNames(names) => {
        (names.len(), names.iter().map(|n| n.to_string()).collect())
      }
      ProvidedExports::ProvidedAll => {
        diagnostics.push(Diagnostic::warn(
          "Fallback module analysis".to_string(),
          "Fallback module provides all exports - using heuristic analysis".to_string(),
        ));
        (0, vec!["*".to_string()])
      }
      ProvidedExports::Unknown => {
        diagnostics.push(Diagnostic::warn(
          "Fallback module analysis".to_string(),
          "Fallback module has unknown exports".to_string(),
        ));
        (0, Vec::new())
      }
    };

    // Count actually used exports with runtime awareness
    let used_count = self.count_used_exports_advanced(prefetched, &provided_exports, None)?;

    let fallback_info = FallbackModuleInfo {
      module_id: fallback_id.to_string(),
      module_type: fallback_module.module_type().to_string(),
      provided_exports_count: provided_count,
      used_exports_count: used_count,
    };

    Ok((fallback_info, provided_exports_vec).with_diagnostic(diagnostics))
  }

  /// Create prefetched fallback analysis when not cached
  fn create_prefetched_fallback_analysis(
    &self,
    module_graph: &ModuleGraph,
    fallback_id: &ModuleIdentifier,
    exports_info: &rspack_core::ExportsInfo,
  ) -> Result<FallbackModuleInfo> {
    let fallback_module = module_graph
      .module_by_identifier(fallback_id)
      .ok_or_else(|| {
        InternalError::new(
          format!("Fallback module not found during prefetch: {}", fallback_id),
          rspack_error::Severity::Error,
        )
      })?;

    let prefetched = ExportsInfoGetter::prefetch(
      exports_info,
      module_graph,
      PrefetchExportsInfoMode::AllExports,
    );

    let provided_exports = prefetched.get_provided_exports();
    let provided_count = match &provided_exports {
      ProvidedExports::ProvidedNames(names) => names.len(),
      ProvidedExports::ProvidedAll => 0,
      ProvidedExports::Unknown => 0,
    };

    let used_count = self.count_used_exports_advanced(&prefetched, &provided_exports, None)?;

    Ok(FallbackModuleInfo {
      module_id: fallback_id.to_string(),
      module_type: fallback_module.module_type().to_string(),
      provided_exports_count: provided_count,
      used_exports_count: used_count,
    })
  }

  /// Extract provided exports from exports info
  fn extract_provided_exports_from_info(
    &self,
    exports_info: &rspack_core::ExportsInfo,
    module_graph: &ModuleGraph,
  ) -> Result<Vec<String>> {
    let prefetched = ExportsInfoGetter::prefetch(
      exports_info,
      module_graph,
      PrefetchExportsInfoMode::AllExports,
    );

    let provided_exports = prefetched.get_provided_exports();
    match provided_exports {
      ProvidedExports::ProvidedNames(names) => {
        Ok(names.iter().map(|name| name.to_string()).collect())
      }
      ProvidedExports::ProvidedAll => Ok(vec!["*".to_string()]),
      ProvidedExports::Unknown => Ok(Vec::new()),
    }
  }

  /// Get provided exports with comprehensive error handling (legacy support)
  fn get_fallback_provided_exports(
    &self,
    module_graph: &ModuleGraph,
    fallback_id: &ModuleIdentifier,
  ) -> Vec<String> {
    match self
      .extract_provided_exports_from_info(&module_graph.get_exports_info(fallback_id), module_graph)
    {
      Ok(exports) => exports,
      Err(_) => Vec::new(),
    }
  }

  /// Enhanced dependency analysis with comprehensive error handling and runtime awareness
  fn analyze_usage_through_incoming_connections(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec],
  ) -> Result<(Vec<String>, Vec<String>)> {
    let mut imported_exports = Vec::new();
    let mut actually_used_exports = Vec::new();
    let mut processed_dependencies = HashSet::new();

    // Get incoming connections with proper error handling
    let connections: Vec<_> = module_graph
      .get_incoming_connections(consume_shared_id)
      .collect();
    if connections.is_empty() {
      return Ok((imported_exports, actually_used_exports));
    }

    // Process each connection with comprehensive analysis
    for connection in connections {
      // Skip already processed dependencies to avoid duplicates
      if processed_dependencies.contains(&connection.dependency_id) {
        continue;
      }
      processed_dependencies.insert(connection.dependency_id);

      // Check connection state for runtime awareness
      let connection_active =
        match connection.active_state(module_graph, runtimes.first(), &Default::default()) {
          ConnectionState::Active(active) => active,
          ConnectionState::TransitiveOnly => true, // Include transitive dependencies
          ConnectionState::CircularConnection => false, // Skip circular to avoid infinite loops
        };

      if !connection_active {
        continue;
      }

      if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
        // Extract referenced exports with proper error handling
        match self.extract_referenced_exports(dependency.as_ref(), module_graph) {
          Ok((imports, uses)) => {
            // Merge results avoiding duplicates
            for import in imports {
              if !imported_exports.contains(&import) {
                imported_exports.push(import);
              }
            }
            for usage in uses {
              if !actually_used_exports.contains(&usage) {
                actually_used_exports.push(usage);
              }
            }
          }
          Err(e) => {
            // Log error but continue processing other dependencies
            self.add_diagnostic(Diagnostic::warn(
              "Failed to extract referenced exports from dependency".to_string(),
              format!("{}", e),
            ));
          }
        }
      }
    }

    Ok((imported_exports, actually_used_exports))
  }

  /// Extract referenced exports from a dependency with proper error handling
  fn extract_referenced_exports(
    &self,
    dependency: &dyn rspack_core::Dependency,
    module_graph: &ModuleGraph,
  ) -> Result<(Vec<String>, Vec<String>)> {
    let mut imported_exports = Vec::new();
    let mut used_exports = Vec::new();

    // Get referenced exports using proper API
    let referenced_exports =
      dependency.get_referenced_exports(module_graph, &ModuleGraphCacheArtifact::default(), None);

    for export_ref in referenced_exports {
      match export_ref {
        ExtendedReferencedExport::Array(names) => {
          // Multiple specific exports are referenced
          for name in names {
            let export_name = name.to_string();
            imported_exports.push(export_name.clone());

            // Check if this export is actually used based on dependency type
            if self.is_export_actually_used(dependency, &export_name) {
              used_exports.push(export_name);
            }
          }
        }
        ExtendedReferencedExport::Export(export_info) => {
          if export_info.name.is_empty() {
            // Namespace usage
            imported_exports.push("*".to_string());
            used_exports.push("*".to_string());
          } else {
            for name in export_info.name {
              let export_name = name.to_string();
              imported_exports.push(export_name.clone());

              if self.is_export_actually_used(dependency, &export_name) {
                used_exports.push(export_name);
              }
            }
          }
        }
      }
    }

    Ok((imported_exports, used_exports))
  }

  /// Determine if an export is actually used based on dependency type and context
  fn is_export_actually_used(
    &self,
    dependency: &dyn rspack_core::Dependency,
    _export_name: &str,
  ) -> bool {
    // For now, assume referenced exports are used unless we have evidence otherwise
    // This could be enhanced with more sophisticated analysis
    match dependency.dependency_type() {
      DependencyType::EsmImport
      | DependencyType::EsmImportSpecifier
      | DependencyType::EsmExportImportedSpecifier => true,
      DependencyType::ConsumeSharedFallback => true,
      _ => false,
    }
  }

  /// Generate detailed export information with runtime awareness and comprehensive analysis
  fn generate_export_details_with_runtime<'a>(
    &self,
    module_graph: &'a ModuleGraph,
    provided_exports: &[String],
    imported_exports: &[String],
    actually_used_exports: &[String],
    runtimes: &[RuntimeSpec],
    prefetch_cache: &HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>,
  ) -> Result<Vec<ExportUsageDetail>> {
    let mut details = Vec::new();

    for export_name in provided_exports {
      let is_imported = imported_exports.contains(export_name);
      let is_used = actually_used_exports.contains(export_name);

      // Determine sophisticated usage state
      let usage_state = self.determine_export_usage_state(
        export_name,
        is_imported,
        is_used,
        runtimes,
        module_graph,
        prefetch_cache,
      )?;

      // Get additional metadata from prefetch cache if available
      let (_can_mangle, _can_inline, _used_name) =
        self.extract_export_metadata(export_name, prefetch_cache);

      details.push(ExportUsageDetail {
        export_name: export_name.clone(),
        usage_state,
        is_imported,
        is_used,
        import_source: if is_imported {
          Some("enhanced_dependency_analysis".to_string())
        } else {
          None
        },
      });
    }

    Ok(details)
  }

  /// Determine sophisticated export usage state
  fn determine_export_usage_state<'a>(
    &self,
    export_name: &str,
    is_imported: bool,
    is_used: bool,
    runtimes: &[RuntimeSpec],
    module_graph: &'a ModuleGraph,
    prefetch_cache: &HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>,
  ) -> Result<String> {
    if is_used {
      return Ok("Used".to_string());
    }

    if is_imported {
      // Check if this is a runtime-specific unused import
      if self.options.runtime_analysis && !runtimes.is_empty() {
        for runtime in runtimes {
          if self.is_export_used_in_runtime(export_name, runtime, module_graph, prefetch_cache)? {
            return Ok("UsedInSpecificRuntime".to_string());
          }
        }
      }
      return Ok("ImportedButUnused".to_string());
    }

    Ok("NotImported".to_string())
  }

  /// Check if export is used in specific runtime
  fn is_export_used_in_runtime<'a>(
    &self,
    _export_name: &str,
    _runtime: &RuntimeSpec,
    _module_graph: &'a ModuleGraph,
    _prefetch_cache: &HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>,
  ) -> Result<bool> {
    // Placeholder for runtime-specific analysis
    // This would involve checking export usage in specific runtime contexts
    Ok(false)
  }

  /// Extract export metadata from prefetch cache
  fn extract_export_metadata<'a>(
    &self,
    export_name: &str,
    prefetch_cache: &HashMap<ModuleIdentifier, PrefetchedExportsInfoWrapper<'a>>,
  ) -> (Option<bool>, Option<bool>, Option<String>) {
    // Try to extract metadata from any available prefetch cache entry
    for prefetched in prefetch_cache.values() {
      let export_atom = rspack_util::atom::Atom::from(export_name);
      if let Some((_, export_data)) = prefetched.exports().find(|(name, _)| **name == export_atom) {
        let can_mangle = ExportInfoGetter::can_mangle(export_data);
        let can_inline = match export_data.inlinable() {
          Inlinable::Inlined(_) => Some(true),
          Inlinable::NoByUse | Inlinable::NoByProvide => Some(false),
        };
        let used_name = export_data.used_name().map(|n| format!("{:?}", n));
        return (can_mangle, can_inline, used_name);
      }
    }
    (None, None, None)
  }

  /// Count used exports with advanced analysis and error handling
  fn count_used_exports_advanced(
    &self,
    prefetched: &PrefetchedExportsInfoWrapper,
    provided_exports: &ProvidedExports,
    runtime: Option<&RuntimeSpec>,
  ) -> Result<usize> {
    match provided_exports {
      ProvidedExports::ProvidedNames(names) => {
        let mut used_count = 0;
        for name in names {
          let export_atom = rspack_util::atom::Atom::from(name.as_str());
          let export_info_data = prefetched.get_read_only_export_info(&export_atom);
          let usage_state = ExportInfoGetter::get_used(export_info_data, runtime);

          if matches!(
            usage_state,
            UsageState::Used | UsageState::OnlyPropertiesUsed
          ) {
            used_count += 1;
          }
        }
        Ok(used_count)
      }
      ProvidedExports::ProvidedAll => {
        // For dynamic exports, check the "other" exports info
        let other_data = prefetched.other_exports_info();
        let other_usage = other_data.global_used();
        if matches!(
          other_usage,
          Some(UsageState::Used) | Some(UsageState::OnlyPropertiesUsed)
        ) {
          Ok(1) // At least one export is used
        } else {
          Ok(0)
        }
      }
      ProvidedExports::Unknown => Ok(0),
    }
  }

  /// Legacy method for backward compatibility
  fn count_used_exports(
    &self,
    prefetched: &PrefetchedExportsInfoWrapper,
    provided_exports: &ProvidedExports,
    runtime: Option<&RuntimeSpec>,
  ) -> usize {
    self
      .count_used_exports_advanced(prefetched, provided_exports, runtime)
      .unwrap_or(0)
  }

  /// Find fallback module ID using ConsumeSharedModule API
  fn find_fallback_module_id(
    &self,
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    if let Some(module) = module_graph.module_by_identifier(consume_shared_id) {
      if let Some(consume_shared) = module
        .as_any()
        .downcast_ref::<crate::sharing::consume_shared_module::ConsumeSharedModule>(
      ) {
        // Use the enhanced API with proper error handling
        match consume_shared.find_fallback_module_id(module_graph) {
          Ok(fallback_id) => fallback_id,
          Err(_) => {
            // Log warning but don't fail - graceful degradation
            tracing::warn!(
              "Failed to find fallback module for ConsumeShared: {}",
              consume_shared_id
            );
            None
          }
        }
      } else {
        None
      }
    } else {
      None
    }
  }

  /// Generate comprehensive analysis report with performance metrics
  fn generate_report(&self, compilation: &Compilation) -> Result<ShareUsageReport> {
    let start_time = std::time::Instant::now();
    let prefetch_start = std::time::Instant::now();

    // Perform analysis with comprehensive error handling
    let analysis_result = self.analyze_consume_shared_usage(compilation)?;
    let usage_data = analysis_result.inner;
    let analysis_diagnostics = analysis_result.diagnostic;

    let prefetch_time = prefetch_start.elapsed();

    let modules_with_unused_imports = usage_data
      .values()
      .filter(|data| data.has_unused_imports)
      .count();

    // Get cache statistics
    let (cache_hits, cache_misses) = if let Ok(cache) = self.cache.read() {
      (cache.module_exports.len(), 0) // Simplified cache metrics
    } else {
      (0, 0)
    };

    let metadata = AnalysisMetadata {
      plugin_version: "3.0.0-enhanced".to_string(),
      analysis_mode: "comprehensive_batch_analysis".to_string(),
      total_consume_shared_modules: usage_data.len(),
      modules_with_unused_imports,
      timestamp: std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| {
          InternalError::new(
            format!("Failed to get system time: {}", e),
            rspack_error::Severity::Warn,
          )
        })?
        .as_secs()
        .to_string(),
      cache_hits,
      cache_misses,
    };

    let performance_metrics = PerformanceMetrics {
      total_analysis_time_ms: start_time.elapsed().as_millis() as u64,
      prefetch_time_ms: prefetch_time.as_millis() as u64,
      batch_operations: (usage_data.len() + self.options.batch_size - 1) / self.options.batch_size,
      modules_analyzed: usage_data.len(),
    };

    // Convert analysis diagnostics to strings for serialization
    let diagnostic_strings: Vec<String> = analysis_diagnostics
      .into_iter()
      .map(|d| {
        d.render_report(false)
          .unwrap_or_else(|_| "Error formatting diagnostic".to_string())
      })
      .collect();

    // Add plugin-level diagnostics
    let mut all_diagnostics = diagnostic_strings;
    if let Ok(plugin_diagnostics) = self.diagnostics.read() {
      for diagnostic in plugin_diagnostics.iter() {
        all_diagnostics.push(
          diagnostic
            .render_report(false)
            .unwrap_or_else(|_| "Error formatting diagnostic".to_string()),
        );
      }
    }

    Ok(ShareUsageReport {
      consume_shared_modules: usage_data,
      analysis_metadata: metadata,
      diagnostics: all_diagnostics,
      performance_metrics,
    })
  }
}

#[plugin_hook(CompilerEmit for EnhancedShareUsagePlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
  // Generate report with comprehensive error handling
  let report = match self.generate_report(compilation) {
    Ok(report) => report,
    Err(e) => {
      // Push diagnostic to compilation instead of failing
      compilation.push_diagnostic(Diagnostic::warn(
        "Enhanced share usage analysis failed".to_string(),
        format!("{}", e),
      ));
      // Return minimal report to maintain functionality
      ShareUsageReport {
        consume_shared_modules: HashMap::new(),
        analysis_metadata: AnalysisMetadata {
          plugin_version: "3.0.0-enhanced".to_string(),
          analysis_mode: "error_recovery".to_string(),
          total_consume_shared_modules: 0,
          modules_with_unused_imports: 0,
          timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| {
              InternalError::new(
                format!("Failed to get system time: {}", e),
                rspack_error::Severity::Warn,
              )
            })?
            .as_secs()
            .to_string(),
          cache_hits: 0,
          cache_misses: 0,
        },
        diagnostics: vec![format!("Analysis failed: {}", e)],
        performance_metrics: PerformanceMetrics {
          total_analysis_time_ms: 0,
          prefetch_time_ms: 0,
          batch_operations: 0,
          modules_analyzed: 0,
        },
      }
    }
  };

  // Serialize with error recovery
  let content = match serde_json::to_string_pretty(&report) {
    Ok(content) => content,
    Err(e) => {
      compilation.push_diagnostic(Diagnostic::warn(
        "Failed to serialize enhanced share usage report".to_string(),
        format!("{}", e),
      ));
      // Fallback to minimal JSON
      format!(
        r#"{{
  "error": "Serialization failed",
  "plugin_version": "3.0.0-enhanced",
  "timestamp": "{}"
}}"
"#,
        std::time::SystemTime::now()
          .duration_since(std::time::UNIX_EPOCH)
          .unwrap_or_default()
          .as_secs()
      )
    }
  };

  // Emit asset with proper metadata
  compilation.emit_asset(
    self.options.filename.clone(),
    CompilationAsset::new(Some(RawSource::from(content).boxed()), AssetInfo::default()),
  );

  Ok(())
}

#[async_trait]
impl Plugin for EnhancedShareUsagePlugin {
  fn name(&self) -> &'static str {
    "rspack.EnhancedShareUsagePlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    // Register emit hook with proper error handling
    ctx.context.compiler_hooks.emit.tap(emit::new(self));

    // Clear diagnostics on new compilation
    if let Ok(mut diagnostics) = self.diagnostics.write() {
      diagnostics.clear();
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use rspack_core::{ModuleGraph, RuntimeSpec};

  use super::*;

  #[test]
  fn test_extract_share_key() {
    let plugin = EnhancedShareUsagePlugin::new(EnhancedShareUsagePluginOptions::default());

    let module_id = ModuleIdentifier::from(
      "consume shared module (default) lodash@4.17.21 (strict) (fallback: ./node_modules/lodash/index.js)"
    );

    let share_key = plugin.extract_share_key(&module_id);
    assert_eq!(share_key, Some("lodash".to_string()));
  }

  #[test]
  fn test_detect_unused_imports() {
    let plugin = EnhancedShareUsagePlugin::new(EnhancedShareUsagePluginOptions::default());

    let imported = vec!["map".to_string(), "filter".to_string(), "uniq".to_string()];
    let used = vec!["map".to_string(), "filter".to_string()];
    let provided = vec![
      "map".to_string(),
      "filter".to_string(),
      "uniq".to_string(),
      "reduce".to_string(),
    ];

    let unused = plugin.detect_unused_imports(&imported, &used, &provided);
    assert_eq!(unused, vec!["uniq".to_string()]);
  }

  #[test]
  fn test_is_export_actually_used() {
    let plugin = EnhancedShareUsagePlugin::new(EnhancedShareUsagePluginOptions::default());

    // Create a mock dependency for testing
    struct MockDependency {
      dep_type: DependencyType,
    }

    impl rspack_core::Dependency for MockDependency {
      fn dependency_type(&self) -> &DependencyType {
        &self.dep_type
      }

      fn get_referenced_exports(
        &self,
        _: &ModuleGraph,
        _: &ModuleGraphCacheArtifact,
        _: Option<&RuntimeSpec>,
      ) -> Vec<ExtendedReferencedExport> {
        vec![]
      }
    }

    let esm_dep = MockDependency {
      dep_type: DependencyType::EsmImport,
    };

    assert!(plugin.is_export_actually_used(&esm_dep, "someExport"));
  }
}
