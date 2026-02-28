use std::hash::Hash;

use rspack_core::{
  ChunkUkey, Compilation, CompilationParams, CompilerCompilation, ExportProvided, ExportsType,
  LibraryOptions, ModuleGraph, ModuleIdentifier, Plugin, PrefetchExportsInfoMode,
  RuntimeCodeTemplate, RuntimeVariable, UsedNameItem, property_access,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_identifier, to_module_export_name,
};
use rspack_error::{Result, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin, RenderSource,
};

use crate::{
  modern_module_library_plugin::{
    render_as_default_only_export, render_as_default_with_named_exports, render_as_named_exports,
  },
  utils::{COMMON_LIBRARY_NAME_MESSAGE, get_options_for_chunk},
};

const PLUGIN_NAME: &str = "rspack.ModuleLibraryPlugin";

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleLibraryPlugin;

impl ModuleLibraryPlugin {
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
      .filter(|library| library.library_type == "module")
      .map(|library| self.parse_options(library))
      .transpose()
  }
}

#[plugin_hook(CompilerCompilation for ModuleLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for ModuleLibraryPlugin)]
async fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  module: &ModuleIdentifier,
  render_source: &mut RenderSource,
  runtime_template: &RuntimeCodeTemplate<'_>,
) -> Result<()> {
  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let exports_name = runtime_template.render_runtime_variable(&RuntimeVariable::Exports);
  let mut source = ConcatSource::default();
  let is_async = ModuleGraph::is_async(&compilation.async_modules_artifact, module);
  let module_graph = compilation.get_module_graph();
  source.add(render_source.source.clone());
  // export { local as exported }
  let mut exports: Vec<(String, Option<String>)> = vec![];
  if is_async {
    source.add(RawStringSource::from(format!(
      "{exports_name} = await {exports_name};\n"
    )));
  }
  let exports_info = compilation
    .exports_info_artifact
    .get_prefetched_exports_info(module, PrefetchExportsInfoMode::Default);
  let boxed_module = module_graph
    .module_by_identifier(module)
    .expect("should have build meta");
  let exports_type = boxed_module.get_exports_type(
    module_graph,
    &compilation.module_graph_cache_artifact,
    &compilation.exports_info_artifact,
    boxed_module.build_info().strict,
  );
  for (_, export_info) in exports_info.exports() {
    if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
      continue;
    };

    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let info_name = export_info.name().expect("should have name");
    let used_name = export_info
      .get_used_name(Some(info_name), Some(chunk.runtime()))
      .expect("name can't be empty");
    let var_name = format!("{exports_name}{}", to_identifier(info_name));

    if info_name == "default"
      && matches!(
        exports_type,
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed | ExportsType::Dynamic
      )
    {
      source.add(RawStringSource::from(format!(
        "var {var_name} = {exports_name};\n",
      )));
    } else {
      source.add(RawStringSource::from(format!(
        "var {var_name} = {};\n",
        match used_name {
          UsedNameItem::Str(used_name) =>
            format!("{exports_name}{}", property_access(vec![used_name], 0)),
          UsedNameItem::Inlined(inlined) => inlined.render(""),
        }
      )));
    }

    exports.push((var_name, Some(to_module_export_name(info_name))))
  }
  if !exports.is_empty() {
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

#[plugin_hook(JavascriptModulesChunkHash for ModuleLibraryPlugin)]
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

impl Plugin for ModuleLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    Ok(())
  }
}
