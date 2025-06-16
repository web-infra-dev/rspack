use std::hash::Hash;

use rspack_core::{
  property_access,
  rspack_sources::{ConcatSource, RawStringSource, SourceExt},
  to_identifier, ApplyContext, ChunkUkey, Compilation, CompilationParams, CompilerCompilation,
  CompilerOptions, ExportInfoGetter, ExportProvided, ExportsType, LibraryOptions, ModuleGraph,
  ModuleIdentifier, Plugin, PluginContext, UsedNameItem,
};
use rspack_error::{error_bail, Result};
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin, RenderSource,
};

use crate::utils::{get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE};

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
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
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
  let exports_info = module_graph.get_exports_info(module);
  let boxed_module = module_graph
    .module_by_identifier(module)
    .expect("should have build meta");
  let exports_type = boxed_module.get_exports_type(&module_graph, boxed_module.build_info().strict);
  for export_info in exports_info.ordered_exports(&module_graph) {
    let export_info_data = export_info.as_data(&module_graph);
    if matches!(
      ExportInfoGetter::provided(export_info_data),
      Some(ExportProvided::NotProvided)
    ) {
      continue;
    };

    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let info_name = ExportInfoGetter::name(export_info_data).expect("should have name");
    let used_name =
      ExportInfoGetter::get_used_name(export_info_data, Some(info_name), Some(chunk.runtime()))
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
    exports.push(format!("{var_name} as {info_name}"));
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

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    Ok(())
  }
}
