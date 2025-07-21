use std::collections::{HashMap, HashSet};

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
pub struct ChunkCharacteristics {
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
  pub is_shared_chunk: bool,
  pub shared_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleModuleExports {
  pub used_exports: Vec<String>,
  pub unused_exports: Vec<String>,
  pub possibly_unused_exports: Vec<String>,
  pub entry_module_id: Option<String>,
  pub chunk_characteristics: Vec<ChunkCharacteristics>,
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
            let result = if let Some(fallback_id) =
              self.find_fallback_module_id(&module_graph, module_id)
            {
              let (used_exports, provided_exports) =
                self.analyze_module_usage(&module_graph, &fallback_id, module_id);
              let unused_exports = provided_exports
                .into_iter()
                .filter(|e| !used_exports.contains(e) && e != "*")
                .collect();
              let entry_module_id =
                ChunkGraph::get_module_id(&compilation.module_ids_artifact, fallback_id)
                  .map(|id| id.to_string());

              let chunk_characteristics = self.get_chunk_characteristics(compilation, &fallback_id);

              SimpleModuleExports {
                used_exports,
                unused_exports,
                possibly_unused_exports: Vec::new(),
                entry_module_id,
                chunk_characteristics,
              }
            } else {
              SimpleModuleExports {
                used_exports: Vec::new(),
                unused_exports: Vec::new(),
                possibly_unused_exports: Vec::new(),
                entry_module_id: None,
                chunk_characteristics: Vec::new(),
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

  fn get_chunk_characteristics(
    &self,
    compilation: &Compilation,
    module_id: &ModuleIdentifier,
  ) -> Vec<ChunkCharacteristics> {
    let mut characteristics = Vec::new();
    let chunk_graph = &compilation.chunk_graph;

    // Get all chunks that contain this module
    let chunks = chunk_graph.get_module_chunks(*module_id);

    for chunk_ukey in chunks {
      if let Some(chunk) = compilation.chunk_by_ukey.get(&chunk_ukey) {
        let chunk_groups: Vec<rspack_core::ChunkGroupUkey> =
          chunk.groups().iter().copied().collect();

        let (is_shared_chunk, shared_modules) = self.analyze_shared_chunk(compilation, &chunk_ukey);

        let chunk_characteristics = ChunkCharacteristics {
          is_runtime_chunk: chunk.has_runtime(&compilation.chunk_group_by_ukey),
          has_runtime: chunk.has_runtime(&compilation.chunk_group_by_ukey),
          is_entrypoint: chunk_groups.iter().any(|&group_ukey| {
            compilation
              .chunk_group_by_ukey
              .get(&group_ukey)
              .map_or(false, |group| group.kind.is_entrypoint())
          }),
          can_be_initial: chunk.can_be_initial(&compilation.chunk_group_by_ukey),
          is_only_initial: chunk.is_only_initial(&compilation.chunk_group_by_ukey),
          chunk_format: self.determine_chunk_format(compilation, &chunk_ukey),
          chunk_loading_type: self.get_chunk_loading_type(compilation, &chunk_groups),
          runtime_names: chunk.runtime().iter().map(|s| s.to_string()).collect(),
          entry_name: self.get_entry_name(compilation, &chunk_groups),
          has_async_chunks: chunk.has_async_chunks(&compilation.chunk_group_by_ukey),
          chunk_files: chunk.files().iter().map(|s| s.to_string()).collect(),
          is_shared_chunk,
          shared_modules,
        };

        characteristics.push(chunk_characteristics);
      }
    }

    characteristics
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
          if let Some(module_id) = module.identifier().as_str().split('/').last() {
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
        if let Some(group) = compilation.chunk_group_by_ukey.get(&group_ukey) {
          if let Some(entry_options) = group.kind.get_entry_options() {
            if let Some(chunk_loading) = &entry_options.chunk_loading {
              return Some(self.chunk_loading_to_format(chunk_loading));
            }
          }
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
      if let Some(group) = compilation.chunk_group_by_ukey.get(&group_ukey) {
        if let Some(entry_options) = group.kind.get_entry_options() {
          if let Some(chunk_loading) = &entry_options.chunk_loading {
            return Some(String::from(chunk_loading.clone()));
          }
        }
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
      if let Some(group) = compilation.chunk_group_by_ukey.get(&group_ukey) {
        if let Some(entry_options) = group.kind.get_entry_options() {
          return entry_options.name.clone();
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
