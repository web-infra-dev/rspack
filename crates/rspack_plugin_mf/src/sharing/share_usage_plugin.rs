use std::collections::{HashMap, HashSet};

use rspack_core::{
  AssetInfo, ChunkGraph, Compilation, CompilationAfterProcessAssets, CompilationAsset,
  DependenciesBlock, DependencyType, ExtendedReferencedExport, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, ModuleType, Plugin,
  rspack_sources::{RawStringSource, SourceExt},
};
use rspack_error::{Error, Result};
use rspack_hook::{plugin, plugin_hook};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkCharacteristics {
  pub entry_module_id: Option<String>,
  pub is_runtime_chunk: bool,
  pub has_runtime: bool,
  pub is_entrypoint: bool,
  pub can_be_initial: bool,
  pub is_only_initial: bool,
  pub chunk_format: Option<String>,
  pub chunk_loading_type: Option<String>,
  pub runtime_names: Vec<String>,
  pub entry_name: Option<String>,
  pub has_async_chunks: bool,
  pub chunk_files: Vec<String>,
  pub shared_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleExportUsage {
  #[serde(flatten)]
  pub exports: HashMap<String, bool>,
  pub chunk_characteristics: ChunkCharacteristics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareUsageReport {
  #[serde(rename = "treeShake")]
  pub tree_shake: HashMap<String, ModuleExportUsage>,
}

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

  /// Analyzes CommonJS module by examining dependencies and connections
  fn analyze_commonjs_exports(
    &self,
    module: &dyn rspack_core::Module,
    module_graph: &ModuleGraph,
  ) -> HashSet<String> {
    let mut exports = HashSet::new();

    // 1. Analyze module dependencies for CommonJS export patterns
    for dep_id in module.get_dependencies() {
      if let Some(dependency) = module_graph.dependency_by_id(dep_id) {
        // Check dependency type to identify CommonJS exports
        match dependency.dependency_type() {
          DependencyType::CjsExports => {
            // This is a CommonJS export dependency
            // Try to downcast to CommonJsExportsDependency to get the names field
            // Since we can't directly access the CommonJsExportsDependency type from plugin_mf,
            // we'll try to extract info through the get_referenced_exports method below

            // Also check referenced exports from the dependency
            let referenced = dependency.get_referenced_exports(
              module_graph,
              &ModuleGraphCacheArtifact::default(),
              None,
            );

            for export_ref in referenced {
              let names = match export_ref {
                ExtendedReferencedExport::Array(names) => names,
                ExtendedReferencedExport::Export(export_info) => export_info.name,
              };

              for name in names {
                let name_str = name.to_string();
                if !name_str.is_empty() && name_str != "*" && name_str != "__esModule" {
                  exports.insert(name_str);
                }
              }
            }
          }
          DependencyType::CjsExportRequire => {
            // module.exports = require('...') pattern
            // This typically means re-exporting another module
            exports.insert("__reexport__".to_string());
          }
          _ => {}
        }
      }
    }

    // 2. Analyze presentational dependencies if available
    if let Some(presentational_deps) = module.get_presentational_dependencies() {
      for _dep in presentational_deps {
        // Presentational dependencies don't have as_module_dependency method
        // They are typically used for runtime requirements and code generation
        // We can try to extract information from their type or other methods
      }
    }

    // 3. Check module blocks for additional export patterns
    for block_id in module.get_blocks() {
      if let Some(block) = module_graph.block_by_id(block_id) {
        for dep_id in block.get_dependencies() {
          if let Some(dependency) = module_graph.dependency_by_id(dep_id) {
            // Similar analysis as above but for async blocks
            let referenced = dependency.get_referenced_exports(
              module_graph,
              &ModuleGraphCacheArtifact::default(),
              None,
            );

            for export_ref in referenced {
              let names = match export_ref {
                ExtendedReferencedExport::Array(names) => names,
                ExtendedReferencedExport::Export(export_info) => export_info.name,
              };

              for name in names {
                let name_str = name.to_string();
                if !name_str.is_empty() && name_str != "*" && name_str != "__esModule" {
                  exports.insert(name_str);
                }
              }
            }
          }
        }
      }
    }

    exports
  }

  fn analyze_consume_shared_usage(
    &self,
    compilation: &Compilation,
  ) -> HashMap<String, ModuleExportUsage> {
    let mut usage_map = HashMap::new();
    let module_graph = compilation.get_module_graph();

    for module_id in module_graph.modules().keys() {
      if let Some(module) = module_graph.module_by_identifier(module_id)
        && module.module_type() == &ModuleType::ConsumeShared
        && let Some(share_key) = module.get_consume_shared_key()
      {
        let result =
          if let Some(fallback_id) = self.find_fallback_module_id(&module_graph, module_id) {
            let (used_exports, provided_exports) =
              self.analyze_module_usage(&module_graph, &fallback_id, module_id);

            let entry_module_id =
              ChunkGraph::get_module_id(&compilation.module_ids_artifact, fallback_id)
                .map(|id| id.to_string());

            let (usage, chunk_characteristics) = self.get_single_chunk_characteristics(
              compilation,
              &fallback_id,
              entry_module_id,
              used_exports,
              provided_exports,
            );

            ModuleExportUsage {
              exports: usage,
              chunk_characteristics,
            }
          } else {
            ModuleExportUsage {
              exports: HashMap::new(),
              chunk_characteristics: ChunkCharacteristics {
                entry_module_id: None,
                is_runtime_chunk: false,
                has_runtime: false,
                is_entrypoint: false,
                can_be_initial: false,
                is_only_initial: false,
                chunk_format: None,
                chunk_loading_type: None,
                runtime_names: Vec::new(),
                entry_name: None,
                has_async_chunks: false,
                chunk_files: Vec::new(),
                shared_modules: Vec::new(),
              },
            }
          };

        usage_map.insert(share_key, result);
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
    use rspack_core::{
      BuildMetaExportsType, ExportsInfoGetter, PrefetchExportsInfoMode, ProvidedExports, UsageState,
    };

    let mut used_exports = Vec::new();
    let mut provided_exports = Vec::new();
    let mut all_imported_exports = HashSet::new();

    // Check if this is a CommonJS module and analyze its exports
    let (is_commonjs, cjs_exports) =
      if let Some(module) = module_graph.module_by_identifier(fallback_id) {
        let build_meta = module.build_meta();
        let is_cjs = matches!(
          build_meta.exports_type,
          BuildMetaExportsType::Dynamic | BuildMetaExportsType::Unset
        );

        // Try to extract CommonJS exports from the module source
        let mut detected_exports = HashSet::new();
        if is_cjs {
          // Use our CommonJS export analyzer
          detected_exports = self.analyze_commonjs_exports(module.as_ref(), module_graph);
        }

        (is_cjs, detected_exports)
      } else {
        (false, HashSet::new())
      };

    // Get exports info from the module graph - this is the most comprehensive source
    let exports_info = module_graph.get_exports_info(fallback_id);
    let prefetched = ExportsInfoGetter::prefetch(
      &exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    // ALSO check the ConsumeShared module's exports info - this is where usage is actually tracked!
    let consume_shared_exports_info = module_graph.get_exports_info(consume_shared_id);
    let consume_shared_prefetched = ExportsInfoGetter::prefetch(
      &consume_shared_exports_info,
      module_graph,
      PrefetchExportsInfoMode::Default,
    );

    // Check BOTH fallback and ConsumeShared modules for provided exports and usage
    // Sometimes the exports are tracked in the ConsumeShared module instead of fallback

    // Try fallback first
    match prefetched.get_provided_exports() {
      ProvidedExports::ProvidedNames(names) => {
        provided_exports = names.iter().map(|n| n.to_string()).collect();

        for export_name in names {
          let export_atom = rspack_util::atom::Atom::from(export_name.as_str());

          // Check usage in BOTH the fallback AND ConsumeShared modules
          let fallback_export_info = prefetched.get_read_only_export_info(&export_atom);
          let fallback_usage = fallback_export_info.get_used(None);

          let consume_shared_export_info =
            consume_shared_prefetched.get_read_only_export_info(&export_atom);
          let consume_shared_usage = consume_shared_export_info.get_used(None);

          // Export is used if EITHER module shows usage
          if (matches!(
            fallback_usage,
            UsageState::Used | UsageState::OnlyPropertiesUsed
          ) || matches!(
            consume_shared_usage,
            UsageState::Used | UsageState::OnlyPropertiesUsed
          )) && export_name != "*"
          {
            used_exports.push(export_name.to_string());
          }
        }
      }
      ProvidedExports::ProvidedAll | ProvidedExports::Unknown => {
        // For modules with unknown/dynamic exports, try alternative approaches

        // Try to get exports from the ExportsInfo data structure directly
        if let Some(exports_data) = module_graph.try_get_exports_info_by_id(&exports_info) {
          // Iterate through all registered exports
          for (export_name, _export_info_id) in exports_data.exports().iter() {
            let export_name_str = export_name.to_string();
            if !export_name_str.is_empty() && export_name_str != "__esModule" {
              // Add to provided exports if not already there
              if !provided_exports.contains(&export_name_str) {
                provided_exports.push(export_name_str.clone());
              }

              // Check if it's used - we'll check this through the prefetched info
              // since we already have it
              let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
              let export_info_data = prefetched.get_read_only_export_info(&export_atom);
              let usage = export_info_data.get_used(None);
              if matches!(usage, UsageState::Used | UsageState::OnlyPropertiesUsed)
                && !used_exports.contains(&export_name_str)
              {
                used_exports.push(export_name_str);
              }
            }
          }
        }

        // If still no exports found and it's CommonJS, use detected exports or mark as dynamic
        if provided_exports.is_empty() && is_commonjs {
          if !cjs_exports.is_empty() {
            // Use the exports we detected from dependencies
            provided_exports = cjs_exports.into_iter().collect();
          } else {
            // Fall back to dynamic marker
            provided_exports = vec!["*".to_string(), "__commonjs_module__".to_string()];
          }
        }
      }
    }

    // First, check incoming connections to the ConsumeShared module
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
            if !used_exports.contains(&name) && !name.is_empty() {
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

    // ALSO check if ConsumeShared module has provided exports that fallback doesn't
    // This can happen when export info is propagated differently
    if let ProvidedExports::ProvidedNames(names) = consume_shared_prefetched.get_provided_exports()
    {
      for export_name in names {
        let export_name_str = export_name.to_string();
        if !provided_exports.contains(&export_name_str) && export_name_str != "*" {
          provided_exports.push(export_name_str.clone());
        }

        // Check if this export is used
        let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
        let consume_shared_export_info =
          consume_shared_prefetched.get_read_only_export_info(&export_atom);
        let consume_shared_usage = consume_shared_export_info.get_used(None);

        if matches!(
          consume_shared_usage,
          UsageState::Used | UsageState::OnlyPropertiesUsed
        ) && !used_exports.contains(&export_name_str)
          && export_name_str != "*"
        {
          used_exports.push(export_name_str);
        }
      }
    }

    // Also check incoming connections to the fallback module to catch imports
    // This is needed for both CommonJS and ESM modules
    for connection in module_graph.get_incoming_connections(fallback_id) {
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
            if !name.is_empty() && name != "*" {
              // For CommonJS with dynamic exports, add to provided_exports if not already there
              if is_commonjs
                && !provided_exports.contains(&name)
                && provided_exports.contains(&"*".to_string())
              {
                provided_exports.push(name.clone());
              }
              // Mark as used
              if !used_exports.contains(&name) {
                used_exports.push(name);
              }
            }
          }
        }
      }
    }

    // IMPORTANT: Also check if this fallback module is used by OTHER modules (including shared modules)
    // This handles the case where @reduxjs/toolkit (shared) imports from redux (shared)
    // We need to check ALL incoming connections, not just from ConsumeShared
    for connection in module_graph.get_incoming_connections(fallback_id) {
      if connection.original_module_identifier.is_some() {
        // Check if this connection actually references any exports
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
              if !name.is_empty() && name != "*" {
                // For CommonJS, add to provided exports if we discovered it through usage
                if is_commonjs && !provided_exports.contains(&name) {
                  // Only add if we don't have the wildcard marker
                  if !provided_exports.contains(&"*".to_string()) {
                    provided_exports.push(name.clone());
                  }
                }

                // Mark as used
                if !used_exports.contains(&name) {
                  used_exports.push(name);
                }
              }
            }
          }

          // For CommonJS requires, also check the dependency type
          if matches!(
            dependency.dependency_type(),
            DependencyType::CjsRequire | DependencyType::CjsFullRequire
          ) {
            // This is a CommonJS require - the referenced exports tell us what's destructured
            // Already handled above through get_referenced_exports
          }
        }
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

    // Also try to extract from dependency type and context
    match dependency.dependency_type() {
      DependencyType::CjsRequire | DependencyType::CjsFullRequire => {
        // For CommonJS requires, we should track what's being destructured
        if let Some(module_dep) = dependency.as_module_dependency() {
          // The request tells us what module is being imported
          let _request = module_dep.request();
          // But we need the actual usage pattern - this would be in the parent context
        }
      }
      DependencyType::CommonJSRequireContext | DependencyType::RequireContext => {
        // Dynamic requires
        imports.insert("*".to_string());
      }
      _ => {}
    }

    if !found_exports && imports.is_empty() {
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

  fn get_single_chunk_characteristics(
    &self,
    compilation: &Compilation,
    module_id: &ModuleIdentifier,
    entry_module_id: Option<String>,
    used_exports: Vec<String>,
    provided_exports: Vec<String>,
  ) -> (HashMap<String, bool>, ChunkCharacteristics) {
    let chunk_graph = &compilation.chunk_graph;
    let chunks = chunk_graph.get_module_chunks(*module_id);

    // Create usage map - all provided exports with their usage status
    let mut usage = HashMap::new();

    // Check if this is a module with dynamic/unknown exports (CommonJS)
    let is_commonjs_module = provided_exports.contains(&"__commonjs_module__".to_string());
    let has_dynamic_exports = provided_exports.contains(&"*".to_string());

    if is_commonjs_module {
      // For CommonJS modules, we can at least track what's being imported from them
      // even if we can't determine all available exports statically

      // Add any detected used exports
      for export in &used_exports {
        usage.insert(export.clone(), true);
      }

      // If no specific exports were detected, mark as having dynamic usage
      if usage.is_empty() {
        usage.insert("__dynamic_commonjs__".to_string(), true);
      }
    } else if has_dynamic_exports && provided_exports.len() == 1 {
      // For modules with only dynamic exports (not explicitly CommonJS)
      usage.insert("__dynamic__".to_string(), true);
    } else {
      // For modules with known exports, track each one
      for export in &provided_exports {
        if export != "*" && export != "__commonjs_module__" {
          usage.insert(export.clone(), used_exports.contains(export));
        }
      }
    }

    // If we have chunks, use the first one for characteristics
    if let Some(&chunk_ukey) = chunks.iter().next()
      && let Some(chunk) = compilation.chunk_by_ukey.get(&chunk_ukey)
    {
      let chunk_groups: Vec<rspack_core::ChunkGroupUkey> = chunk.groups().iter().copied().collect();

      let (_, shared_modules) = self.analyze_shared_chunk(compilation, &chunk_ukey);

      return (
        usage,
        ChunkCharacteristics {
          entry_module_id,
          is_runtime_chunk: chunk.has_runtime(&compilation.chunk_group_by_ukey),
          has_runtime: chunk.has_runtime(&compilation.chunk_group_by_ukey),
          is_entrypoint: chunk_groups.iter().any(|&group_ukey| {
            compilation
              .chunk_group_by_ukey
              .get(&group_ukey)
              .is_some_and(|group| group.kind.is_entrypoint())
          }),
          can_be_initial: chunk.can_be_initial(&compilation.chunk_group_by_ukey),
          is_only_initial: chunk.is_only_initial(&compilation.chunk_group_by_ukey),
          chunk_format: self.determine_chunk_format(compilation, &chunk_ukey),
          chunk_loading_type: self.get_chunk_loading_type(compilation, &chunk_groups),
          runtime_names: chunk.runtime().iter().map(|s| s.to_string()).collect(),
          entry_name: self.get_entry_name(compilation, &chunk_groups),
          has_async_chunks: chunk.has_async_chunks(&compilation.chunk_group_by_ukey),
          chunk_files: chunk.files().iter().map(|s| s.to_string()).collect(),
          shared_modules,
        },
      );
    }

    // Fallback if no chunks found
    (
      usage,
      ChunkCharacteristics {
        entry_module_id,
        is_runtime_chunk: false,
        has_runtime: false,
        is_entrypoint: false,
        can_be_initial: false,
        is_only_initial: false,
        chunk_format: None,
        chunk_loading_type: None,
        runtime_names: Vec::new(),
        entry_name: None,
        has_async_chunks: false,
        chunk_files: Vec::new(),
        shared_modules: Vec::new(),
      },
    )
  }

  fn analyze_shared_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &rspack_core::ChunkUkey,
  ) -> (bool, Vec<String>) {
    let chunk_graph = &compilation.chunk_graph;
    let module_graph = compilation.get_module_graph();
    let modules = chunk_graph.get_chunk_modules(chunk_ukey, &module_graph);
    let mut shared_modules = Vec::new();
    let mut is_shared = false;

    for module in modules {
      // Check if this is a shared module and collect share keys
      match module.module_type() {
        rspack_core::ModuleType::ProvideShared => {
          // For ProvideShared modules, we need to extract the share key differently
          // This is a placeholder - we'd need to check the actual API for ProvideShared modules
          if let Some(module_id) = module.identifier().as_str().split('/').next_back() {
            shared_modules.push(module_id.to_string());
            is_shared = true;
          }
        }
        rspack_core::ModuleType::ConsumeShared => {
          if let Some(share_key) = module.get_consume_shared_key() {
            shared_modules.push(share_key);
            is_shared = true;
          }
        }
        _ => {}
      }
    }

    (is_shared, shared_modules)
  }

  fn determine_chunk_format(
    &self,
    compilation: &Compilation,
    chunk_ukey: &rspack_core::ChunkUkey,
  ) -> Option<String> {
    // Get the chunk format from the output configuration
    if let Some(chunk) = compilation.chunk_by_ukey.get(chunk_ukey) {
      // Check chunk groups to find entry options with chunk loading configuration
      for &group_ukey in chunk.groups().iter() {
        if let Some(group) = compilation.chunk_group_by_ukey.get(&group_ukey)
          && let Some(entry_options) = group.kind.get_entry_options()
          && let Some(chunk_loading) = &entry_options.chunk_loading
        {
          return Some(self.chunk_loading_to_format(chunk_loading));
        }
      }

      // Check if this is an ESM output based on module type
      if compilation.options.output.module {
        return Some("module".to_string());
      }

      // Check the global chunk loading configuration
      let chunk_loading = &compilation.options.output.chunk_loading;
      Some(self.chunk_loading_to_format(chunk_loading))
    } else {
      None
    }
  }

  fn chunk_loading_to_format(&self, chunk_loading: &rspack_core::ChunkLoading) -> String {
    match chunk_loading {
      rspack_core::ChunkLoading::Enable(chunk_loading_type) => match chunk_loading_type {
        rspack_core::ChunkLoadingType::Jsonp => "jsonp".to_string(),
        rspack_core::ChunkLoadingType::ImportScripts => "import-scripts".to_string(),
        rspack_core::ChunkLoadingType::Require => "require".to_string(),
        rspack_core::ChunkLoadingType::AsyncNode => "async-node".to_string(),
        rspack_core::ChunkLoadingType::Import => "import".to_string(),
        rspack_core::ChunkLoadingType::Custom(custom) => custom.clone(),
      },
      rspack_core::ChunkLoading::Disable => {
        // When chunk loading is disabled, chunks are not loaded dynamically
        // This typically means the code is bundled directly without chunk loading mechanism
        "false".to_string()
      }
    }
  }

  fn get_chunk_loading_type(
    &self,
    compilation: &Compilation,
    chunk_groups: &[rspack_core::ChunkGroupUkey],
  ) -> Option<String> {
    // Extract chunk loading type from entry options
    for &group_ukey in chunk_groups {
      if let Some(group) = compilation.chunk_group_by_ukey.get(&group_ukey)
        && let Some(entry_options) = group.kind.get_entry_options()
        && let Some(chunk_loading) = &entry_options.chunk_loading
      {
        return Some(String::from(chunk_loading.clone()));
      }
    }
    None
  }

  fn get_entry_name(
    &self,
    compilation: &Compilation,
    chunk_groups: &[rspack_core::ChunkGroupUkey],
  ) -> Option<String> {
    for &group_ukey in chunk_groups {
      if let Some(group) = compilation.chunk_group_by_ukey.get(&group_ukey)
        && let Some(entry_options) = group.kind.get_entry_options()
      {
        return entry_options.name.clone();
      }
    }
    None
  }
}

#[plugin_hook(CompilationAfterProcessAssets for ShareUsagePlugin)]
async fn after_process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let usage_data = self.analyze_consume_shared_usage(compilation);

  let report = ShareUsageReport {
    tree_shake: usage_data,
  };

  let content = serde_json::to_string_pretty(&report)
    .map_err(|e| Error::error(format!("Failed to serialize share usage report: {e}")))?;

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
      CompilationAsset::new(
        Some(RawStringSource::from(content).boxed()),
        AssetInfo::default(),
      ),
    );
  } else {
    compilation.assets_mut().insert(
      filename.clone(),
      CompilationAsset::new(
        Some(RawStringSource::from(content).boxed()),
        AssetInfo::default(),
      ),
    );
  }

  Ok(())
}

impl Plugin for ShareUsagePlugin {
  fn name(&self) -> &'static str {
    "ShareUsagePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .after_process_assets
      .tap(after_process_assets::new(self));
    Ok(())
  }
}
