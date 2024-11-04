use std::hash::Hash;

use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  property_access, to_identifier, ApplyContext, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, LibraryOptions, ModuleGraph, ModuleIdentifier, Plugin,
  PluginContext,
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
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks.render_startup.tap(render_startup::new(self));
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderStartup for ModuleLibraryPlugin)]
fn render_startup(
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
    source.add(RawSource::from(
      "__webpack_exports__ = await __webpack_exports__;\n",
    ));
  }
  let exports_info = module_graph.get_exports_info(module);
  for export_info in exports_info.ordered_exports(&module_graph) {
    let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);
    let info_name = export_info.name(&module_graph).expect("should have name");
    let used_name = export_info
      .get_used_name(&module_graph, Some(info_name), Some(&chunk.runtime))
      .expect("name can't be empty");
    let var_name = format!("__webpack_exports__{}", to_identifier(info_name));
    source.add(RawSource::from(format!(
      "var {var_name} = __webpack_exports__{};\n",
      property_access(vec![used_name], 0)
    )));
    exports.push(format!("{var_name} as {info_name}"));
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
