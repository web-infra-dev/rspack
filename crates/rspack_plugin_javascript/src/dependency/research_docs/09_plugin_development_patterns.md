# Plugin Development Patterns in Rspack

## Overview

This document provides comprehensive patterns and best practices for developing rspack plugins, specifically focused on export analysis, module graph manipulation, and compilation hooks. The insights are derived from real-world plugin implementations including ShareUsagePlugin, ConsumeSharedPlugin, and various export analysis tools.

## Plugin Structure and Organization

### Basic Plugin Architecture

#### Plugin Declaration and Hooks
```rust
use rspack_core::{Plugin, PluginContext, CompilerEmit, CompilerOptions};

#[plugin]
#[derive(Debug)]
pub struct ExportAnalysisPlugin {
    options: ExportAnalysisOptions,
}

#[plugin_hook(CompilerEmit for ExportAnalysisPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
    // Plugin implementation
    Ok(())
}
```

**Key Patterns:**
- `#[plugin]` macro for plugin registration
- `#[plugin_hook(HookName for PluginName)]` for hook implementation
- Async functions with `Result<()>` return type for error handling
- Plugin structs should derive `Debug` for diagnostics

#### Plugin Options and Configuration
```rust
#[derive(Debug, Clone)]
pub struct ExportAnalysisOptions {
    pub output_path: Option<String>,
    pub detailed_analysis: bool,
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

impl Default for ExportAnalysisOptions {
    fn default() -> Self {
        Self {
            output_path: None,
            detailed_analysis: false,
            include_patterns: vec!["**/*.js".to_string(), "**/*.ts".to_string()],
            exclude_patterns: vec!["node_modules/**".to_string()],
        }
    }
}
```

## Compilation Hooks and Their Usage

### Available Compilation Hooks

#### CompilerEmit Hook
**Timing**: After optimization, before asset emission
**Use Case**: Asset generation, analysis reports, final processing

```rust
#[plugin_hook(CompilerEmit for MyPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
    // Generate analysis reports
    let report = self.generate_analysis_report(compilation)?;
    
    // Create assets
    let source = RawSource::from(serde_json::to_string_pretty(&report)?);
    let asset = CompilationAsset::new(
        Some(source), 
        AssetInfo::default().with_development(true)
    );
    
    compilation.emit_asset("analysis-report.json".to_string(), asset);
    Ok(())
}
```

#### CompilationFinishModules Hook
**Timing**: After module building, before optimization
**Use Case**: Module metadata manipulation, export information copying

```rust
#[plugin_hook(CompilationFinishModules for MetadataCopyPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph = compilation.get_module_graph();
    
    // Collect modules that need processing
    let target_modules: Vec<ModuleIdentifier> = module_graph
        .modules()
        .keys()
        .filter(|&id| self.should_process_module(&module_graph, id))
        .copied()
        .collect();
    
    // Process each module individually to avoid borrow checker issues
    for module_id in target_modules {
        self.process_module(compilation, &module_id)?;
    }
    
    Ok(())
}
```

#### CompilationOptimizeDependencies Hook
**Timing**: During optimization phase
**Use Case**: Dependency analysis, usage tracking

```rust
#[plugin_hook(CompilationOptimizeDependencies for DependencyOptimizer)]
async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<bool> {
    let mut module_graph = compilation.get_module_graph_mut();
    let mut changes_made = false;
    
    // Optimize dependencies based on usage analysis
    for (module_id, _) in module_graph.modules() {
        if self.optimize_module_dependencies(&mut module_graph, module_id)? {
            changes_made = true;
        }
    }
    
    Ok(changes_made) // Return true if further optimization needed
}
```

### Hook Timing and Dependencies

```
Compilation Lifecycle:
┌─────────────────────────────────────────────────────────────┐
│  Module Building Phase                                      │
│  ├── Module parsing and dependency creation                 │
│  ├── Dependency resolution                                  │
│  └── CompilationFinishModules ← Good for metadata copying   │
│                                                             │
│  Optimization Phase                                         │
│  ├── CompilationOptimizeDependencies ← Usage analysis      │
│  ├── FlagDependencyExportsPlugin                          │
│  ├── FlagDependencyUsagePlugin                            │
│  └── Various optimization passes                           │
│                                                             │
│  Asset Generation Phase                                     │
│  ├── Code generation                                       │
│  ├── CompilerEmit ← Asset creation, report generation      │
│  └── Asset emission                                        │
└─────────────────────────────────────────────────────────────┘
```

## Module Graph Manipulation Patterns

### Safe Module Graph Access

#### Avoiding Borrow Checker Issues
```rust
// ❌ Problematic - multiple mutable borrows
let mut module_graph = compilation.get_module_graph_mut();
let module = module_graph.module_by_identifier_mut(&id);
let other_module = module_graph.module_by_identifier_mut(&other_id); // Error!

// ✅ Correct - separate scopes
{
    let module_graph = compilation.get_module_graph(); // immutable
    let module_ids = collect_target_modules(&module_graph);
}
{
    let mut module_graph = compilation.get_module_graph_mut(); // mutable
    for module_id in module_ids {
        process_module(&mut module_graph, &module_id);
    }
}

// ✅ Alternative - helper methods with contained borrows
fn process_modules_individually(
    compilation: &mut Compilation,
    module_ids: &[ModuleIdentifier]
) -> Result<()> {
    for &module_id in module_ids {
        {
            let mut module_graph = compilation.get_module_graph_mut();
            // Process one module at a time
            process_single_module(&mut module_graph, &module_id)?;
        } // Borrow ends here
    }
    Ok(())
}
```

#### Module Iteration Patterns
```rust
// Collect first, then process
let module_ids: Vec<ModuleIdentifier> = {
    let module_graph = compilation.get_module_graph();
    module_graph
        .modules()
        .keys()
        .filter(|&id| should_process(module_graph, id))
        .copied()
        .collect()
};

// Process with mutable access
for module_id in module_ids {
    let mut module_graph = compilation.get_module_graph_mut();
    process_module(&mut module_graph, &module_id)?;
}
```

### Module Type Detection and Filtering

```rust
// Module type checking patterns
fn should_process_module(module_graph: &ModuleGraph, module_id: &ModuleIdentifier) -> bool {
    if let Some(module) = module_graph.module_by_identifier(module_id) {
        match module.module_type() {
            ModuleType::Js => true,
            ModuleType::ConsumeShared => true, // Special handling
            ModuleType::Asset => false,
            _ => false,
        }
    } else {
        false
    }
}

// Layer-based filtering
fn filter_by_layer(module_graph: &ModuleGraph, target_layer: Option<&str>) -> Vec<ModuleIdentifier> {
    module_graph
        .modules()
        .iter()
        .filter_map(|(id, module)| {
            match (module.get_layer(), target_layer) {
                (Some(layer), Some(target)) if layer == target => Some(*id),
                (None, None) => Some(*id),
                _ => None,
            }
        })
        .collect()
}
```

## Export Analysis API Usage

### Correct API Usage Patterns

#### ExportsInfoGetter vs ExportInfoGetter
```rust
use rspack_core::{ExportsInfoGetter, ExportInfoGetter, PrefetchExportsInfoMode};

// Use ExportsInfoGetter for bulk operations and prefetching
let exports_info = module_graph.get_exports_info(module_id);
let prefetched = ExportsInfoGetter::prefetch(
    &exports_info,
    module_graph,
    PrefetchExportsInfoMode::AllExports, // Efficient bulk analysis
);

// Extract provided exports
let provided_exports = prefetched.get_provided_exports();

// Use ExportInfoGetter for individual export analysis
for export_name in export_names {
    let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
    let export_info_data = prefetched.get_read_only_export_info(&export_atom);
    let usage_state = ExportInfoGetter::get_used(export_info_data, runtime_spec);
    
    // Process usage state
    match usage_state {
        UsageState::Used => { /* Export is actively used */ },
        UsageState::OnlyPropertiesUsed => { /* Only properties accessed */ },
        UsageState::Unused => { /* Export can be tree-shaken */ },
        _ => { /* Other usage patterns */ }
    }
}
```

#### PrefetchExportsInfoMode Usage
```rust
// Different prefetch modes for different use cases
let prefetch_mode = match analysis_type {
    AnalysisType::Comprehensive => PrefetchExportsInfoMode::AllExports,
    AnalysisType::Specific(names) => PrefetchExportsInfoMode::NamedExports(names),
    AnalysisType::Nested(path) => PrefetchExportsInfoMode::NamedNestedExports(path),
    AnalysisType::Basic => PrefetchExportsInfoMode::Default,
};

let prefetched = ExportsInfoGetter::prefetch(&exports_info, module_graph, prefetch_mode);
```

### ProvidedExports Handling

```rust
use rspack_core::ProvidedExports;

fn extract_provided_exports(provided: &ProvidedExports) -> Vec<String> {
    match provided {
        ProvidedExports::ProvidedNames(names) => {
            // Specific named exports
            names.iter().map(|name| name.as_str().to_string()).collect()
        },
        ProvidedExports::ProvidedAll => {
            // Module provides all possible exports dynamically
            vec!["*".to_string()]
        },
        ProvidedExports::Unknown => {
            // Cannot determine exports statically
            vec![] // Empty indicates unknown exports
        }
    }
}

// Usage in analysis
fn analyze_module_exports(
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec]
) -> Result<ModuleExportInfo> {
    let exports_info = module_graph.get_exports_info(module_id);
    let prefetched = ExportsInfoGetter::prefetch(
        &exports_info,
        module_graph,
        PrefetchExportsInfoMode::AllExports,
    );
    
    let provided_exports = prefetched.get_provided_exports();
    let provided_export_names = extract_provided_exports(&provided_exports);
    
    let mut export_details = Vec::new();
    for export_name in &provided_export_names {
        if export_name == "*" {
            // Handle dynamic exports
            continue;
        }
        
        let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
        let export_info_data = prefetched.get_read_only_export_info(&export_atom);
        
        for runtime in runtimes {
            let usage_state = ExportInfoGetter::get_used(export_info_data, Some(runtime));
            export_details.push(ExportDetail {
                name: export_name.clone(),
                usage_state,
                runtime: runtime.clone(),
            });
        }
    }
    
    Ok(ModuleExportInfo {
        module_id: module_id.to_string(),
        provided_exports: provided_export_names,
        export_details,
    })
}
```

## ConsumeShared Module Analysis Patterns

### Understanding ConsumeShared Proxy Behavior

```rust
// ConsumeShared modules require special analysis patterns with enhanced dependency analysis
fn analyze_consume_shared_module(
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec]
) -> Result<ConsumeSharedAnalysis> {
    // 1. ConsumeShared modules act as proxies - their direct usage is often empty
    let direct_exports = get_direct_module_exports(module_graph, module_id, runtimes);
    
    // 2. Find the fallback module for real export information
    let fallback_module_id = find_fallback_module(module_graph, module_id);
    let fallback_exports = if let Some(fallback_id) = fallback_module_id {
        get_module_exports(module_graph, &fallback_id, runtimes, true)?
    } else {
        vec![]
    };
    
    // 3. Enhanced analysis using incoming connections and get_referenced_exports()
    let consumer_usage = analyze_consume_shared_usage_from_consumers(
        module_graph,
        module_id,
        runtimes
    )?;
    
    // 4. Merge all analysis sources
    Ok(ConsumeSharedAnalysis {
        direct_exports,
        fallback_exports,
        consumer_usage,
        proxy_behavior_detected: direct_exports.is_empty() && !fallback_exports.is_empty(),
    })
}

// Enhanced analysis using get_referenced_exports() for precise export extraction
fn analyze_consume_shared_usage_from_consumers(
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier,
    runtimes: &[RuntimeSpec]
) -> Result<Vec<ConsumerUsage>> {
    let mut usage_patterns = Vec::new();
    
    // Get incoming connections to the ConsumeShared module
    for connection in module_graph.get_incoming_connections(consume_shared_id) {
        if let Some(dependency) = module_graph.dependency_by_id(&connection.dependency_id) {
            // Use get_referenced_exports() to extract specific export names being used
            let referenced_exports = dependency.get_referenced_exports(
                module_graph,
                &rspack_core::ModuleGraphCacheArtifact::default(),
                None,
            );
            
            // Process ExtendedReferencedExport patterns for comprehensive export extraction
            for export_ref in referenced_exports {
                match export_ref {
                    ExtendedReferencedExport::Array(names) => {
                        // Multiple specific exports are referenced
                        for name in names {
                            usage_patterns.push(ConsumerUsage {
                                consumer_module: connection.origin_module_identifier.unwrap(),
                                export_name: name.to_string(),
                                usage_type: UsageType::SpecificExport,
                            });
                        }
                    },
                    ExtendedReferencedExport::Export(export_info) => {
                        // Single export or namespace reference
                        if export_info.name.is_empty() {
                            // No specific name indicates namespace usage
                            usage_patterns.push(ConsumerUsage {
                                consumer_module: connection.origin_module_identifier.unwrap(),
                                export_name: "*".to_string(),
                                usage_type: UsageType::NamespaceUsage,
                            });
                        } else {
                            // Specific named exports
                            for name in export_info.name {
                                usage_patterns.push(ConsumerUsage {
                                    consumer_module: connection.origin_module_identifier.unwrap(),
                                    export_name: name.to_string(),
                                    usage_type: UsageType::NamedExport,
                                });
                            }
                        }
                    },
                }
            }
        }
    }
    
    Ok(usage_patterns)
}
```

### Finding Related Modules

```rust
// Find fallback module for ConsumeShared modules
fn find_fallback_module(
    module_graph: &ModuleGraph,
    consume_shared_id: &ModuleIdentifier
) -> Option<ModuleIdentifier> {
    if let Some(module) = module_graph.module_by_identifier(consume_shared_id) {
        // Check direct dependencies
        for dep_id in module.get_dependencies() {
            if let Some(dep) = module_graph.dependency_by_id(dep_id) {
                if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
                    return module_graph.module_identifier_by_dependency_id(dep_id).copied();
                }
            }
        }
        
        // Also check async dependencies (for lazy loading)
        for block_id in module.get_blocks() {
            if let Some(block) = module_graph.block_by_id(block_id) {
                for dep_id in block.get_dependencies() {
                    if let Some(dep) = module_graph.dependency_by_id(dep_id) {
                        if matches!(dep.dependency_type(), DependencyType::ConsumeSharedFallback) {
                            return module_graph.module_identifier_by_dependency_id(dep_id).copied();
                        }
                    }
                }
            }
        }
    }
    None
}

// Extract share key from ConsumeShared modules
fn extract_share_key(
    module_graph: &ModuleGraph,
    module_id: &ModuleIdentifier
) -> Option<String> {
    if let Some(module) = module_graph.module_by_identifier(module_id) {
        if module.module_type() == &ModuleType::ConsumeShared {
            // Access ConsumeShared-specific methods
            return module.get_consume_shared_key();
        }
    }
    None
}

// Cross-reference used exports with provided exports for accurate filtering
fn cross_reference_usage_with_provided(
    used_exports: Vec<String>,
    provided_exports: &[String]
) -> Vec<String> {
    used_exports
        .into_iter()
        .filter(|export| {
            // Include if it's a namespace usage or if it's actually provided
            export == "*" || provided_exports.contains(export) || provided_exports.contains(&"*".to_string())
        })
        .collect()
}
```

## Asset Generation Patterns

### Creating JSON Reports

```rust
use rspack_core::{CompilationAsset, RawSource, AssetInfo};
use serde_json;

fn generate_analysis_report(
    &self,
    compilation: &Compilation
) -> Result<AnalysisReport> {
    let module_graph = compilation.get_module_graph();
    let mut modules = HashMap::new();
    
    // Collect runtimes for comprehensive analysis
    let runtimes: Vec<RuntimeSpec> = compilation
        .chunk_by_ukey
        .values()
        .map(|chunk| chunk.runtime())
        .cloned()
        .collect();
    
    // Analyze each module
    for (module_id, _module) in module_graph.modules() {
        if let Some(usage_info) = self.analyze_module(
            &module_graph,
            &module_id,
            &runtimes
        )? {
            modules.insert(module_id.to_string(), usage_info);
        }
    }
    
    Ok(AnalysisReport {
        modules,
        summary: self.generate_summary(&modules),
        metadata: ReportMetadata {
            timestamp: current_timestamp(),
            rspack_version: env!("CARGO_PKG_VERSION").to_string(),
            total_modules: modules.len(),
            runtimes: runtimes.iter().map(|r| r.to_string()).collect(),
        },
    })
}

// Asset creation with proper metadata
#[plugin_hook(CompilerEmit for AnalysisPlugin)]
async fn emit(&self, compilation: &mut Compilation) -> Result<()> {
    let report = self.generate_analysis_report(compilation)?;
    
    // Serialize to JSON with pretty printing
    let json_content = serde_json::to_string_pretty(&report)
        .map_err(|e| rspack_error::Error::from(format!("JSON serialization failed: {}", e)))?;
    
    // Create asset with appropriate metadata
    let source = RawSource::from(json_content);
    let asset_info = AssetInfo::default()
        .with_development(true) // Mark as development asset
        .with_generated(true);  // Mark as generated content
    
    let asset = CompilationAsset::new(Some(source), asset_info);
    
    // Determine output filename
    let filename = self.options.output_path
        .clone()
        .unwrap_or_else(|| "export-analysis.json".to_string());
    
    compilation.emit_asset(filename, asset);
    Ok(())
}
```

### Custom Asset Types

```rust
// Create multiple related assets
fn emit_comprehensive_analysis(&self, compilation: &mut Compilation) -> Result<()> {
    let analysis = self.generate_analysis_report(compilation)?;
    
    // Main JSON report
    let json_source = RawSource::from(serde_json::to_string_pretty(&analysis)?);
    compilation.emit_asset(
        "analysis/exports.json".to_string(),
        CompilationAsset::new(Some(json_source), AssetInfo::default())
    );
    
    // Summary CSV
    let csv_content = self.generate_csv_summary(&analysis)?;
    let csv_source = RawSource::from(csv_content);
    compilation.emit_asset(
        "analysis/exports-summary.csv".to_string(),
        CompilationAsset::new(Some(csv_source), AssetInfo::default())
    );
    
    // Debug logs if enabled
    if self.options.include_debug_info {
        let debug_content = self.generate_debug_logs(&analysis)?;
        let debug_source = RawSource::from(debug_content);
        compilation.emit_asset(
            "analysis/debug.txt".to_string(),
            CompilationAsset::new(Some(debug_source), AssetInfo::default())
        );
    }
    
    Ok(())
}
```

## Error Handling and Diagnostics

### Proper Error Patterns

```rust
use rspack_error::{Diagnostic, DiagnosticKind};

// Convert various error types to rspack errors
fn handle_analysis_error(&self, error: AnalysisError, context: &str) -> rspack_error::Error {
    match error {
        AnalysisError::ModuleNotFound(id) => {
            rspack_error::Error::from(format!(
                "Module not found during {}: {}",
                context, id
            ))
        },
        AnalysisError::SerializationFailed(err) => {
            rspack_error::Error::from(format!(
                "Failed to serialize analysis results in {}: {}",
                context, err
            ))
        },
        AnalysisError::InvalidExportInfo(details) => {
            rspack_error::Error::from(format!(
                "Invalid export information in {}: {}",
                context, details
            ))
        },
    }
}

// Add diagnostics to compilation
fn add_warning(&self, compilation: &mut Compilation, message: String) {
    compilation.push_diagnostic(Diagnostic::warn(
        "ExportAnalysisPlugin".into(),
        message,
    ));
}

fn add_error(&self, compilation: &mut Compilation, message: String) {
    compilation.push_diagnostic(Diagnostic::error(
        "ExportAnalysisPlugin".into(), 
        message,
    ));
}

// Safe module processing with error handling
fn process_module_safely(
    &self,
    compilation: &mut Compilation,
    module_id: &ModuleIdentifier
) -> Result<()> {
    match self.analyze_module_exports(compilation, module_id) {
        Ok(analysis) => {
            self.store_analysis(module_id, analysis);
            Ok(())
        },
        Err(e) => {
            let warning_msg = format!(
                "Failed to analyze exports for module {}: {}. Skipping.",
                module_id, e
            );
            self.add_warning(compilation, warning_msg);
            Ok(()) // Continue processing other modules
        }
    }
}
```

### Comprehensive Error Recovery

```rust
// Robust module iteration with error recovery
fn process_all_modules(&self, compilation: &mut Compilation) -> Result<ProcessingStats> {
    let module_graph = compilation.get_module_graph();
    let module_ids: Vec<ModuleIdentifier> = module_graph.modules().keys().copied().collect();
    
    let mut stats = ProcessingStats::default();
    
    for module_id in module_ids {
        match self.process_module_safely(compilation, &module_id) {
            Ok(()) => {
                stats.successful += 1;
            },
            Err(e) => {
                stats.failed += 1;
                stats.errors.push(format!("Module {}: {}", module_id, e));
                
                // Log but continue processing
                tracing::warn!(
                    "Export analysis failed for module {}: {}",
                    module_id, e
                );
            }
        }
    }
    
    // Add summary diagnostic
    if stats.failed > 0 {
        let summary = format!(
            "Export analysis completed with {} successes and {} failures",
            stats.successful, stats.failed
        );
        self.add_warning(compilation, summary);
    }
    
    Ok(stats)
}
```

## Performance Optimization Patterns

### Efficient Module Graph Traversal

```rust
// Use sets for efficient lookups
use rustc_hash::{FxHashSet, FxHashMap};

fn optimize_module_processing(&self, compilation: &Compilation) -> Result<ProcessingPlan> {
    let module_graph = compilation.get_module_graph();
    
    // Categorize modules for efficient processing
    let mut js_modules = FxHashSet::default();
    let mut consume_shared_modules = FxHashSet::default();
    let mut other_modules = FxHashSet::default();
    
    for (module_id, module) in module_graph.modules() {
        match module.module_type() {
            ModuleType::Js => { js_modules.insert(*module_id); },
            ModuleType::ConsumeShared => { consume_shared_modules.insert(*module_id); },
            _ => { other_modules.insert(*module_id); }
        }
    }
    
    Ok(ProcessingPlan {
        js_modules: js_modules.into_iter().collect(),
        consume_shared_modules: consume_shared_modules.into_iter().collect(),
        other_modules: other_modules.into_iter().collect(),
    })
}

// Batch processing for related modules
fn process_module_batch(
    &self,
    compilation: &mut Compilation,
    batch: &[ModuleIdentifier]
) -> Result<()> {
    // Process all modules in a batch with shared setup
    let shared_context = self.create_analysis_context(compilation)?;
    
    for &module_id in batch {
        self.process_module_with_context(compilation, &module_id, &shared_context)?;
    }
    
    Ok(())
}
```

### Caching and Memoization

```rust
use std::collections::HashMap;

// Cache analysis results to avoid recomputation
#[derive(Debug)]
pub struct CachedAnalysisPlugin {
    options: AnalysisOptions,
    cache: std::sync::Mutex<HashMap<String, CachedResult>>,
}

impl CachedAnalysisPlugin {
    fn get_cache_key(&self, module_id: &ModuleIdentifier, runtime: &RuntimeSpec) -> String {
        format!("{}:{}", module_id, runtime)
    }
    
    fn get_cached_analysis(
        &self,
        module_id: &ModuleIdentifier,
        runtime: &RuntimeSpec
    ) -> Option<ModuleExportInfo> {
        let cache = self.cache.lock().ok()?;
        let key = self.get_cache_key(module_id, runtime);
        cache.get(&key).and_then(|cached| {
            if cached.is_valid() {
                Some(cached.result.clone())
            } else {
                None
            }
        })
    }
    
    fn cache_analysis(
        &self,
        module_id: &ModuleIdentifier,
        runtime: &RuntimeSpec,
        result: ModuleExportInfo
    ) {
        if let Ok(mut cache) = self.cache.lock() {
            let key = self.get_cache_key(module_id, runtime);
            cache.insert(key, CachedResult {
                result,
                timestamp: std::time::Instant::now(),
            });
        }
    }
}
```

## Integration Testing Patterns

### Plugin Testing Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rspack_testing::{test_fixture, TestCompilation};
    
    #[test]
    fn test_export_analysis_basic() {
        let plugin = ExportAnalysisPlugin::new(ExportAnalysisOptions::default());
        
        test_fixture("export-analysis-basic", |fixture| {
            let mut compilation = TestCompilation::new(fixture);
            compilation.add_plugin(Box::new(plugin));
            
            let result = compilation.build()?;
            
            // Verify analysis results
            assert_eq!(result.assets.len(), 1);
            assert!(result.assets.contains_key("export-analysis.json"));
            
            // Parse and validate report
            let report_content = result.get_asset_content("export-analysis.json")?;
            let report: AnalysisReport = serde_json::from_str(&report_content)?;
            
            assert!(!report.modules.is_empty());
            assert!(report.summary.total_modules > 0);
        });
    }
    
    #[test]
    fn test_consume_shared_analysis() {
        let options = ExportAnalysisOptions {
            detailed_analysis: true,
            ..Default::default()
        };
        let plugin = ExportAnalysisPlugin::new(options);
        
        test_fixture("module-federation-consume-shared", |fixture| {
            let mut compilation = TestCompilation::new(fixture);
            compilation.add_plugin(Box::new(plugin));
            
            let result = compilation.build()?;
            let report_content = result.get_asset_content("export-analysis.json")?;
            let report: AnalysisReport = serde_json::from_str(&report_content)?;
            
            // Find ConsumeShared modules in results
            let consume_shared_modules: Vec<_> = report.modules
                .values()
                .filter(|m| m.module_type == "ConsumeShared")
                .collect();
            
            assert!(!consume_shared_modules.is_empty());
            
            // Verify proxy behavior detection
            for module in consume_shared_modules {
                if module.has_fallback {
                    assert!(
                        module.proxy_behavior_detected,
                        "ConsumeShared with fallback should show proxy behavior"
                    );
                }
            }
        });
    }
}

// Integration test helper
fn create_test_plugin_with_options(options: ExportAnalysisOptions) -> Box<dyn Plugin> {
    Box::new(ExportAnalysisPlugin::new(options))
}

// Mock module graph for unit testing
fn create_mock_module_graph() -> MockModuleGraph {
    let mut graph = MockModuleGraph::new();
    
    // Add test modules
    graph.add_module("./src/index.js", ModuleType::Js);
    graph.add_module("./src/utils.js", ModuleType::Js);
    graph.add_module("shared-library", ModuleType::ConsumeShared);
    
    // Add dependencies
    graph.add_dependency("./src/index.js", "./src/utils.js", DependencyType::EsmImport);
    graph.add_dependency("./src/index.js", "shared-library", DependencyType::ConsumeSharedImport);
    
    graph
}
```

## Best Practices Summary

### Plugin Development Guidelines

1. **Hook Selection**: Choose appropriate hooks based on timing requirements
   - `CompilationFinishModules` for metadata manipulation
   - `CompilerEmit` for asset generation
   - `CompilationOptimizeDependencies` for analysis requiring optimization data

2. **Borrow Checker Management**: Use separate scopes and helper methods to avoid conflicts

3. **Error Handling**: Always use `Result<()>` and provide meaningful error messages

4. **Performance**: Use efficient data structures and caching where appropriate

5. **Testing**: Include comprehensive integration tests with real module scenarios

### Export Analysis Best Practices

1. **API Usage**: Use `ExportsInfoGetter::prefetch()` for bulk operations

2. **ConsumeShared Handling**: Understand proxy behavior and analyze through connections using `get_referenced_exports()`

3. **Runtime Awareness**: Always consider multiple runtimes in analysis

4. **Data Validation**: Validate export information before processing

5. **Reporting**: Provide comprehensive, structured output with metadata

6. **Advanced Dependency Analysis**: Use `module_graph.get_incoming_connections()` and `dependency.get_referenced_exports()` for precise usage extraction

7. **Pattern Matching**: Handle `ExtendedReferencedExport::Array` and `ExtendedReferencedExport::Export` patterns comprehensively

8. **Cross-referencing**: Compare extracted usage with provided exports for accurate filtering

### Common Pitfalls to Avoid

1. **Multiple Mutable Borrows**: Structure code to avoid simultaneous mutable module graph access

2. **Incorrect API Usage**: Don't confuse `ExportsInfoGetter` and `ExportInfoGetter` methods

3. **Missing Error Handling**: Always handle potential failures gracefully

4. **ConsumeShared Assumptions**: Don't expect direct export usage data from proxy modules

5. **Runtime Ignorance**: Consider all applicable runtimes for comprehensive analysis

6. **Incomplete Pattern Matching**: Ensure both `ExtendedReferencedExport` variants are handled properly

7. **Missing Cross-reference**: Don't forget to cross-reference extracted usage with provided exports

8. **Shallow Dependency Analysis**: Use incoming connections and `get_referenced_exports()` instead of relying on direct module analysis only

This document provides the foundation for developing robust, efficient plugins for rspack's export analysis and module federation systems.