use std::{hash::Hash, sync::Arc};

use rspack_collections::IdentifierMap;
use rspack_core::{
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_identifier, ApplyContext, BoxDependency, ChunkUkey, CodeGenerationExportsFinalNames,
  Compilation, CompilationOptimizeChunkModules, CompilationParams, CompilerCompilation,
  CompilerFinishMake, CompilerOptions, ConcatenatedModule, ConcatenatedModuleExportsDefinitions,
  DependenciesBlock, Dependency, DependencyId, LibraryOptions, ModuleGraph, ModuleIdentifier,
  Plugin, PluginContext, PrefetchExportsInfoMode, RuntimeSpec, UsedNameItem,
};
use rspack_error::{error_bail, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  dependency::{
    ESMExportImportedSpecifierDependency, ESMImportSideEffectDependency, ImportDependency,
  },
  ConcatConfiguration, JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin,
  ModuleConcatenationPlugin, RenderSource,
};
use rustc_hash::FxHashSet as HashSet;

use super::modern_module::ModernModuleReexportStarExternalDependency;
use crate::{
  modern_module::{
    ModernModuleImportDependency, ModernModuleImportDependencyTemplate,
    ModernModuleReexportStarExternalDependencyTemplate,
  },
  utils::{get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE},
};

const PLUGIN_NAME: &str = "rspack.ModernModuleLibraryPlugin";

#[plugin]
#[derive(Debug, Default)]
pub struct ModernModuleLibraryPlugin;

impl ModernModuleLibraryPlugin {
  fn parse_options(&self, library: &LibraryOptions) -> Result<()> {
    if library.name.is_some() {
      error_bail!("Library name must be unset. {COMMON_LIBRARY_NAME_MESSAGE}")
    }
    Ok(())
  }

  pub fn reexport_star_from_external_module(
    &self,
    dep: &ESMExportImportedSpecifierDependency,
    mg: &ModuleGraph,
  ) -> bool {
    if let Some(m) = mg.get_module_by_dependency_id(&dep.id) {
      if let Some(m) = m.as_external_module() {
        if m.get_external_type() == "module" || m.get_external_type() == "module-import" {
          // Star reexport will meet the condition.
          return dep.name.is_none() && dep.other_star_exports.is_some();
        }
      }
    }

    false
  }

  fn get_options_for_chunk(
    &self,
    compilation: &Compilation,
    chunk_ukey: &ChunkUkey,
  ) -> Result<Option<()>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == "modern-module")
      .map(|library| self.parse_options(library))
      .transpose()
  }

  // The `optimize_chunk_modules_impl` here will be invoked after the one in `ModuleConcatenationPlugin`.
  // Force trigger concatenation for single modules what bails from `ModuleConcatenationPlugin.is_empty`,
  // to keep all chunks can benefit from runtime optimization.
  async fn optimize_chunk_modules_impl(&self, compilation: &mut Compilation) -> Result<()> {
    let module_graph: rspack_core::ModuleGraph = compilation.get_module_graph();

    let module_ids: Vec<_> = module_graph
      .module_graph_modules()
      .keys()
      .copied()
      .collect();

    let mut concatenated_module_ids = HashSet::default();

    for module_id in &module_ids {
      let module = module_graph
        .module_by_identifier(module_id)
        .expect("we have mgm we know for sure we have module");

      if let Some(module) = module.as_ref().downcast_ref::<ConcatenatedModule>() {
        concatenated_module_ids.insert(*module_id);
        for inner_module in module.get_modules() {
          concatenated_module_ids.insert(inner_module.id);
        }
      }
    }

    let unconcatenated_module_ids = module_ids
      .iter()
      .filter(|id| !concatenated_module_ids.contains(id))
      .filter(|id| {
        let Some(mgm) = module_graph.module_graph_module_by_identifier(id) else {
          return false;
        };
        let reasons = &mgm.optimization_bailout;

        let is_concatenation_entry_candidate = reasons
          .iter()
          .any(|r| r.contains("Module is an entry point"));

        is_concatenation_entry_candidate
      })
      .collect::<HashSet<_>>();

    for module_id in unconcatenated_module_ids.into_iter() {
      let chunk_runtime = compilation
        .chunk_graph
        .get_module_runtimes_iter(*module_id, &compilation.chunk_by_ukey)
        .fold(RuntimeSpec::default(), |mut acc, r| {
          acc.extend(r);
          acc
        });

      let current_configuration: ConcatConfiguration =
        ConcatConfiguration::new(*module_id, Some(chunk_runtime.clone()));

      let mut used_modules = HashSet::default();

      ModuleConcatenationPlugin::process_concatenated_configuration(
        compilation,
        current_configuration,
        &mut used_modules,
      )
      .await?;
    }

    Ok(())
  }
}

#[plugin_hook(JavascriptModulesRenderStartup for ModernModuleLibraryPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module_id: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let codegen = compilation
    .code_generation_results
    .get(module_id, Some(chunk.runtime()));

  let mut exports = vec![];
  let mut exports_with_property_access = vec![];
  let mut exports_with_inlined = vec![];

  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };

  let mut source = ConcatSource::default();
  let module_graph = compilation.get_module_graph();
  source.add(render_source.source.clone());

  if let Some(exports_final_names) = codegen
    .data
    .get::<CodeGenerationExportsFinalNames>()
    .map(|d: &CodeGenerationExportsFinalNames| d.inner())
  {
    let exports_info =
      module_graph.get_prefetched_exports_info(module_id, PrefetchExportsInfoMode::Default);
    for (_, export_info) in exports_info.exports() {
      let info_name = export_info.name().expect("should have name");
      let used_name = export_info
        .get_used_name(Some(info_name), Some(chunk.runtime()))
        .expect("name can't be empty");

      let used_name = match used_name {
        UsedNameItem::Inlined(inlined) => {
          exports_with_inlined.push((inlined, info_name));
          continue;
        }
        UsedNameItem::Str(used_name) => used_name,
      };

      let final_name = exports_final_names.get(used_name.as_str());

      let contains_char =
        |string: &str, chars: &str| -> bool { string.chars().any(|c| chars.contains(c)) };

      if let Some(final_name) = final_name {
        // Currently, there's not way to determine if a final_name contains a property access.
        if contains_char(final_name, "[]().") {
          exports_with_property_access.push((final_name, info_name));
        } else if info_name == final_name {
          exports.push(info_name.to_string());
        } else {
          exports.push(format!("{final_name} as {info_name}"));
        }
      }
    }

    for (final_name, info_name) in exports_with_property_access.iter() {
      let var_name = format!("__webpack_exports__{}", to_identifier(info_name));

      source.add(RawStringSource::from(format!(
        "var {var_name} = {final_name};\n"
      )));

      exports.push(format!("{var_name} as {info_name}"));
    }

    for (inlined, info_name) in exports_with_inlined.iter() {
      let var_name = format!("__webpack_exports__{}", to_identifier(info_name));

      source.add(RawStringSource::from(format!(
        "var {var_name} = {};\n",
        inlined.render()
      )));

      exports.push(format!("{var_name} as {info_name}"));
    }
  }

  if !exports.is_empty() {
    source.add(RawStringSource::from(format!(
      "export {{ {} }};\n",
      exports.join(", ")
    )));
  }

  render_source.source = source.boxed();
  Ok(())
}

#[plugin_hook(CompilerFinishMake for ModernModuleLibraryPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  let mg = compilation.get_module_graph();
  let mut deps_to_replace: Vec<BoxDependency> = Vec::new();
  let mut external_connections = HashSet::default();

  // Remove `import()` runtime.
  for module in mg.modules().values() {
    for block_id in module.get_blocks() {
      let Some(block) = mg.block_by_id(block_id) else {
        continue;
      };
      for block_dep_id in block.get_dependencies() {
        let block_dep = mg.dependency_by_id(block_dep_id);
        if let Some(block_dep) = block_dep {
          if let Some(import_dependency) = block_dep.as_any().downcast_ref::<ImportDependency>() {
            let import_dep_connection = mg.connection_by_dependency_id(block_dep_id);
            if let Some(import_dep_connection) = import_dep_connection {
              // Try find the connection with a import dependency pointing to an external module.
              // If found, remove the connection and add a new import dependency to performs the external module ID replacement.
              let import_module_id = import_dep_connection.module_identifier();
              let Some(import_module) = mg.module_by_identifier(import_module_id) else {
                continue;
              };

              if let Some(external_module) = import_module.as_external_module() {
                let new_dep = ModernModuleImportDependency::new(
                  *block_dep.id(),
                  import_dependency.request.as_str().into(),
                  external_module.request.clone(),
                  external_module.external_type.clone(),
                  import_dependency.range.clone(),
                  import_dependency.get_attributes().cloned(),
                );

                deps_to_replace.push(Box::new(new_dep));
              }
            }
          }
        }
      }
    }
  }

  // Reexport star from external module.
  // Only preserve star reexports for module graph entry, nested reexports are not supported.
  for dep_id in &compilation.make_artifact.entry_dependencies {
    let Some(module) = mg.get_module_by_dependency_id(dep_id) else {
      continue;
    };

    let mut module_id_to_connections: IdentifierMap<Vec<DependencyId>> = IdentifierMap::default();
    mg.get_outgoing_connections(&module.identifier())
      .for_each(|connection| {
        module_id_to_connections
          .entry(*connection.module_identifier())
          .or_default()
          .push(connection.dependency_id);
      });

    for dep_id in module.get_dependencies() {
      if let Some(export_dep) = mg.dependency_by_id(dep_id) {
        if let Some(reexport_dep) = export_dep
          .as_any()
          .downcast_ref::<ESMExportImportedSpecifierDependency>()
        {
          if self.reexport_star_from_external_module(reexport_dep, &mg) {
            let reexport_connection = mg.connection_by_dependency_id(&reexport_dep.id);
            if let Some(reexport_connection) = reexport_connection {
              let import_module_id = reexport_connection.module_identifier();
              let Some(import_module) = mg.module_by_identifier(import_module_id) else {
                continue;
              };

              if let Some(external_module) = import_module.as_external_module() {
                if reexport_dep.request == external_module.user_request() {
                  if let Some(connections) =
                    module_id_to_connections.get(reexport_connection.module_identifier())
                  {
                    let reexport_star_count = connections
                      .iter()
                      .filter(|c| {
                        if let Some(dep) = mg.dependency_by_id(c) {
                          if let Some(dep) = dep
                            .as_any()
                            .downcast_ref::<ESMExportImportedSpecifierDependency>()
                          {
                            return self.reexport_star_from_external_module(dep, &mg);
                          }
                        }

                        false
                      })
                      .count();

                    let side_effect_count = connections
                      .iter()
                      .filter(|c| {
                        if let Some(dep) = mg.dependency_by_id(c) {
                          if dep
                            .as_any()
                            .downcast_ref::<ESMImportSideEffectDependency>()
                            .is_some()
                          {
                            return true;
                          }
                        }

                        false
                      })
                      .count();

                    // Every ESMExportImportedSpecifierDependency comes along with an ESMImportSideEffectDependency.
                    // So if there are an equal number of ESMExportImportedSpecifierDependency (export star) and ESMImportSideEffectDependency,
                    // we can consider that it only contains reexport star, and safely remove it.
                    if side_effect_count == reexport_star_count
                      && side_effect_count + reexport_star_count == connections.len()
                    {
                      for c in connections.iter() {
                        external_connections.insert(*c);
                      }
                    }
                  }

                  let new_dep = ModernModuleReexportStarExternalDependency::new(
                    *dep_id,
                    reexport_dep.request.as_str().into(),
                    external_module.request.clone(),
                    external_module.external_type.clone(),
                  );

                  deps_to_replace.push(Box::new(new_dep));
                }
              }
            }
          }
        }
      }
    }
  }

  let mut mg = compilation.get_module_graph_mut();
  for dep in deps_to_replace {
    let dep_id = dep.id();
    external_connections.remove(dep_id);
    // remove connection
    mg.revoke_dependency(dep_id, false);
    // overwrite dependency
    mg.add_dependency(dep);
  }

  for connection in &external_connections {
    mg.revoke_dependency(connection, true);
  }

  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for ModernModuleLibraryPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  PLUGIN_NAME.hash(hasher);
  Ok(())
}

#[plugin_hook(CompilationOptimizeChunkModules for ModernModuleLibraryPlugin)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  self.optimize_chunk_modules_impl(compilation).await?;
  Ok(None)
}

#[plugin_hook(CompilerCompilation for ModernModuleLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));

  compilation.set_dependency_template(
    ModernModuleImportDependencyTemplate::template_type(),
    Arc::new(ModernModuleImportDependencyTemplate::default()),
  );
  compilation.set_dependency_template(
    ModernModuleReexportStarExternalDependencyTemplate::template_type(),
    Arc::new(ModernModuleReexportStarExternalDependencyTemplate::default()),
  );
  Ok(())
}

#[plugin_hook(ConcatenatedModuleExportsDefinitions for ModernModuleLibraryPlugin)]
async fn exports_definitions(
  &self,
  _exports_definitions: &mut Vec<(String, String)>,
  is_entry_module: bool,
) -> Result<Option<bool>> {
  // Only the inlined module could skip render definitions as it's in the module scope.
  match is_entry_module {
    true => Ok(Some(true)),
    false => Ok(Some(false)),
  }
}

impl Plugin for ModernModuleLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));

    ctx
      .context
      .concatenated_module_hooks
      .exports_definitions
      .tap(exports_definitions::new(self));

    ctx
      .context
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));
    ctx
      .context
      .compiler_hooks
      .finish_make
      .tap(finish_make::new(self));

    Ok(())
  }
}
