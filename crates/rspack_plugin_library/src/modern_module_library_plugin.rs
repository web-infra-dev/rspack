use std::hash::Hash;

use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  ApplyContext, ChunkUkey, CodeGenerationExportsFinalNames, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, ConcatenatedModuleExportsDefinitions, DependencyType,
  LibraryOptions, ModuleIdentifier, Plugin, PluginContext,
};
use rspack_error::{error_bail, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::HarmonyExportSpecifierDependency;
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin, RenderSource,
};

use crate::utils::{get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE};

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
}

#[plugin_hook(JavascriptModulesRenderStartup for ModernModuleLibraryPlugin)]
fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module_id: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
  let codegen = compilation
    .code_generation_results
    .get(module_id, Some(&chunk.runtime));

  let module_graph = compilation.get_module_graph();
  let module = module_graph
    .module_by_identifier(module_id)
    .expect("should have module");
  let mut exports = vec![];

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
    let exports_info = module_graph.get_exports_info(module_id);
    for id in exports_info.get_ordered_exports() {
      let info = id.get_export_info(&module_graph);
      let info_name = info.name.as_ref().expect("should have name");
      let used_name = info
        .get_used_name(info.name.as_ref(), Some(&chunk.runtime))
        .expect("name can't be empty");

      let final_name = exports_final_names.get(used_name.as_str());
      if let Some(final_name) = final_name {
        if info_name == final_name {
          exports.push(info_name.to_string());
        } else {
          exports.push(format!("{} as {}", final_name, info_name));
        }
      }
    }
  } else {
    let module_deps = module.get_dependencies();
    for dep in module_deps {
      let dep = module_graph
        .dependency_by_id(dep)
        .expect("should have dependency");

      if *dep.dependency_type() == DependencyType::EsmExportSpecifier {
        if let Some(dep) = dep.downcast_ref::<HarmonyExportSpecifierDependency>() {
          if dep.value == dep.name {
            exports.push(dep.value.to_string());
          } else {
            exports.push(format!("{} as {}", dep.value, dep.name));
          }
        }
      }
    }
  }

  if !exports.is_empty() {
    source.add(RawSource::from(format!(
      "export {{ {} }};\n",
      exports.join(", ")
    )));
  }

  render_source.source = source.boxed();
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

#[plugin_hook(CompilerCompilation for ModernModuleLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(ConcatenatedModuleExportsDefinitions for ModernModuleLibraryPlugin)]
fn exports_definitions(
  &self,
  _exports_definitions: &mut Vec<(String, String)>,
) -> Result<Option<bool>> {
  Ok(Some(true))
}

impl Plugin for ModernModuleLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
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

    Ok(())
  }
}
