use std::hash::Hash;

use rspack_core::{
  ChunkUkey, Compilation, CompilationParams, CompilerCompilation, ExportProvided, ExportsType,
  LibraryOptions, ModuleGraph, ModuleIdentifier, Plugin, PrefetchExportsInfoMode, UsedNameItem,
  property_access,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_identifier, to_module_export_name,
};
use rspack_error::{Result, error_bail};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin, RenderSource,
};

use crate::utils::{COMMON_LIBRARY_NAME_MESSAGE, get_options_for_chunk};

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
) -> Result<()> {
  let Some(_) = self.get_options_for_chunk(compilation, chunk_ukey)? else {
    return Ok(());
  };
  let mut source = ConcatSource::default();
  let is_async = ModuleGraph::is_async(compilation, module);
  let module_graph = compilation.get_module_graph();
  source.add(render_source.source.clone());
  let mut exports = vec![];
  if is_async {
    source.add(RawStringSource::from(
      "__webpack_exports__ = await __webpack_exports__;\n",
    ));
  }
  let exports_info =
    module_graph.get_prefetched_exports_info(module, PrefetchExportsInfoMode::Default);
  let boxed_module = module_graph
    .module_by_identifier(module)
    .expect("should have build meta");
  let exports_type = boxed_module.get_exports_type(
    &module_graph,
    &compilation.module_graph_cache_artifact,
    boxed_module.build_info().strict,
  );
  for (_, export_info) in exports_info.exports() {
    if matches!(export_info.provided(), Some(ExportProvided::NotProvided)) {
      continue;
    };

    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let info_name = export_info.name().expect("should have name");
    let used_name = export_info
      .get_used_name(Some(info_name), Some(chunk.runtime()))
      .expect("name can't be empty");
    let var_name = format!("__webpack_exports__{}", to_identifier(info_name));

    if info_name == "default"
      && matches!(
        exports_type,
        ExportsType::DefaultOnly | ExportsType::DefaultWithNamed | ExportsType::Dynamic
      )
    {
      source.add(RawStringSource::from(format!(
        "var {var_name} = __webpack_exports__;\n",
      )));
    } else {
      source.add(RawStringSource::from(format!(
        "var {var_name} = {};\n",
        match used_name {
          UsedNameItem::Str(used_name) =>
            format!("__webpack_exports__{}", property_access(vec![used_name], 0)),
          UsedNameItem::Inlined(inlined) => inlined.render().into_owned(),
        }
      )));
    }
    exports.push(format!(
      "{var_name} as {}",
      to_module_export_name(info_name)
    ));
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
