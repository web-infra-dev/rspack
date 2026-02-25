use std::{hash::Hash, sync::Arc};

use rspack_collections::IdentifierMap;
use rspack_core::{
  BoxDependency, ChunkUkey, CodeGenerationExportsFinalNames, Compilation,
  CompilationOptimizeChunkModules, CompilationParams, CompilerCompilation, CompilerFinishMake,
  ConcatenatedModule, ConcatenatedModuleExportsDefinitions, DependencyId, ExportsType,
  LibraryOptions, ModuleGraph, ModuleIdentifier, Plugin, PrefetchExportsInfoMode,
  RuntimeCodeTemplate, RuntimeSpec, RuntimeVariable, UsedNameItem,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_identifier,
};
use rspack_error::{Result, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  ConcatConfiguration, JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin,
  ModuleConcatenationPlugin, RenderSource,
  dependency::{ESMExportImportedSpecifierDependency, ESMImportSideEffectDependency},
};
use rustc_hash::FxHashSet as HashSet;

use super::modern_module::ModernModuleReexportStarExternalDependency;
use crate::{
  modern_module::ModernModuleReexportStarExternalDependencyTemplate,
  utils::{COMMON_LIBRARY_NAME_MESSAGE, get_options_for_chunk},
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
    if let Some(m) = mg.get_module_by_dependency_id(&dep.id)
      && let Some(m) = m.as_external_module()
      && (m.get_external_type() == "module" || m.get_external_type() == "module-import")
    {
      // Star reexport will meet the condition.
      return dep.name.is_none() && dep.other_star_exports.is_some();
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
    let module_graph = compilation.get_module_graph();

    let module_ids: Vec<_> = module_graph
      .module_graph_modules()
      .map(|(id, _)| *id)
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

        reasons
          .iter()
          .any(|r| r.contains("Module is an entry point"))
      })
      .collect::<HashSet<_>>();

    for module_id in unconcatenated_module_ids {
      let chunk_runtime = compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_module_runtimes_iter(
          *module_id,
          &compilation.build_chunk_graph_artifact.chunk_by_ukey,
        )
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

  fn preserve_reexports_star(&self, compilation: &mut Compilation) -> Result<()> {
    let mg = compilation.get_module_graph();
    let mut deps_to_replace: Vec<BoxDependency> = Vec::new();
    let mut external_connections = HashSet::default();

    // Reexport star from external module.
    // Only preserve star reexports for module graph entry, nested reexports are not supported.
    for dep_id in &compilation.build_module_graph_artifact.entry_dependencies {
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
        let export_dep = mg.dependency_by_id(dep_id);
        if let Some(reexport_dep) = export_dep
          .as_any()
          .downcast_ref::<ESMExportImportedSpecifierDependency>()
          && self.reexport_star_from_external_module(reexport_dep, mg)
        {
          let reexport_connection = mg.connection_by_dependency_id(&reexport_dep.id);
          if let Some(reexport_connection) = reexport_connection {
            let import_module_id = reexport_connection.module_identifier();
            let Some(import_module) = mg.module_by_identifier(import_module_id) else {
              continue;
            };

            if let Some(external_module) = import_module.as_external_module() {
              if let Some(connections) =
                module_id_to_connections.get(reexport_connection.module_identifier())
              {
                let reexport_star_count = connections
                  .iter()
                  .filter(|c| {
                    let dep = mg.dependency_by_id(c);
                    if let Some(dep) = dep
                      .as_any()
                      .downcast_ref::<ESMExportImportedSpecifierDependency>()
                    {
                      return self.reexport_star_from_external_module(dep, mg);
                    }

                    false
                  })
                  .count();

                let side_effect_count = connections
                  .iter()
                  .filter(|c| {
                    let dep = mg.dependency_by_id(c);
                    dep
                      .as_any()
                      .downcast_ref::<ESMImportSideEffectDependency>()
                      .is_some()
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

    let mg = compilation
      .build_module_graph_artifact
      .get_module_graph_mut();
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
}

#[plugin_hook(JavascriptModulesRenderStartup for ModernModuleLibraryPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module_id: &ModuleIdentifier,
  render_source: &mut RenderSource,
  runtime_template: &RuntimeCodeTemplate<'_>,
) -> Result<()> {
  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(chunk_ukey);
  let codegen = compilation
    .code_generation_results
    .get(module_id, Some(chunk.runtime()));

  // export local as exported
  let mut exports: Vec<(String, Option<String>)> = vec![];
  let mut exports_with_property_access = vec![];
  let mut exports_with_inlined = vec![];

  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };

  let mut source = ConcatSource::default();
  let module_graph = compilation.get_module_graph();
  source.add(render_source.source.clone());
  let module = module_graph
    .module_by_identifier(module_id)
    .expect("should have module");
  let exports_type = module.get_exports_type(
    module_graph,
    &compilation.module_graph_cache_artifact,
    &compilation.exports_info_artifact,
    module.build_meta().strict_esm_module,
  );
  if let Some(exports_final_names) = codegen
    .data
    .get::<CodeGenerationExportsFinalNames>()
    .map(|d: &CodeGenerationExportsFinalNames| d.inner())
  {
    let exports_info = compilation
      .exports_info_artifact
      .get_prefetched_exports_info(module_id, PrefetchExportsInfoMode::Default);
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
          exports.push((info_name.to_string(), None));
        } else {
          exports.push((final_name.clone(), Some(info_name.to_string())));
        }
      }
    }

    let exports_name = runtime_template.render_runtime_variable(&RuntimeVariable::Exports);

    for (final_name, info_name) in exports_with_property_access.iter() {
      let var_name = format!("{exports_name}{}", to_identifier(info_name));

      source.add(RawStringSource::from(format!(
        "var {var_name} = {final_name};\n"
      )));

      exports.push((var_name, Some(info_name.to_string())));
    }

    for (inlined, info_name) in exports_with_inlined.iter() {
      let var_name = format!("{exports_name}{}", to_identifier(info_name));

      source.add(RawStringSource::from(format!(
        "var {var_name} = {};\n",
        inlined.render("")
      )));

      exports.push((var_name, Some(info_name.to_string())));
    }
  }

  if !exports.is_empty() && !compilation.options.output.iife {
    let exports_string = match exports_type {
      ExportsType::DefaultOnly => render_as_default_only_export(&exports),
      ExportsType::DefaultWithNamed | ExportsType::Dynamic => {
        render_as_default_with_named_exports(&exports)
      }
      ExportsType::Namespace => render_as_named_exports(&exports),
    };
    source.add(RawStringSource::from(exports_string));
  }

  render_source.source = source.boxed();
  Ok(())
}

pub fn render_as_default_only_export(exports: &[(String, Option<String>)]) -> String {
  render_as_default_export_impl(exports)
}

pub fn render_as_named_exports(exports: &[(String, Option<String>)]) -> String {
  render_as_named_exports_impl(exports, false)
}

pub fn render_as_default_with_named_exports(exports: &[(String, Option<String>)]) -> String {
  format!(
    "{}\n{}",
    render_as_named_exports_impl(exports, true),
    render_as_default_only_export(exports),
  )
}

fn render_as_named_exports_impl(
  exports: &[(String, Option<String>)],
  ignore_default: bool,
) -> String {
  format!(
    "export {{ {} }};\n",
    exports
      .iter()
      .filter(|(_, exported)| {
        if ignore_default {
          !matches!(exported.as_deref(), Some("default"))
        } else {
          true
        }
      })
      .map(|(local, exported)| {
        if let Some(exported) = exported {
          format!("{local} as {exported}")
        } else {
          local.clone()
        }
      })
      .collect::<Vec<_>>()
      .join(", ")
  )
}

pub fn render_as_default_export_impl(exports: &[(String, Option<String>)]) -> String {
  if let Some((local, _)) = exports
    .iter()
    .find(|(_, exported)| matches!(exported.as_deref(), Some("default")))
  {
    return format!("export {{ {local} as default }};\n",);
  }

  format!(
    "var __rspack_exports_default = {{ {} }};\nexport default __rspack_exports_default;\n",
    exports
      .iter()
      .map(|(local, exported)| {
        if let Some(exported) = exported {
          format!("{exported}: {local}")
        } else {
          local.clone()
        }
      })
      .collect::<Vec<_>>()
      .join(", ")
  )
}

#[plugin_hook(CompilerFinishMake for ModernModuleLibraryPlugin)]
async fn finish_make(&self, compilation: &mut Compilation) -> Result<()> {
  self.preserve_reexports_star(compilation)?;

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
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));

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

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx
      .concatenated_module_hooks
      .exports_definitions
      .tap(exports_definitions::new(self));

    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));
    ctx.compiler_hooks.finish_make.tap(finish_make::new(self));

    Ok(())
  }
}
