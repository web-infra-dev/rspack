use std::hash::Hash;

use rspack_core::{
  property_access,
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  to_identifier,
  tree_shaking::webpack_ext::ExportInfoExt,
  ChunkUkey, Compilation, JsChunkHashArgs, LibraryOptions, Plugin, PluginContext,
  PluginJsChunkHashHookOutput, PluginRenderStartupHookOutput, RenderStartupArgs,
};
use rspack_error::{internal_error_bail, Result};

use crate::utils::{get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE};

#[derive(Debug, Default)]
pub struct ModuleLibraryPlugin;

impl ModuleLibraryPlugin {
  fn parse_options(&self, library: &LibraryOptions) -> Result<()> {
    if library.name.is_some() {
      internal_error_bail!("Library name must be unset. {COMMON_LIBRARY_NAME_MESSAGE}")
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

impl Plugin for ModuleLibraryPlugin {
  fn name(&self) -> &'static str {
    "rspack.ModuleLibraryPlugin"
  }

  fn render_startup(
    &self,
    _ctx: PluginContext,
    args: &RenderStartupArgs,
  ) -> PluginRenderStartupHookOutput {
    let Some(_) = self.get_options_for_chunk(args.compilation, args.chunk)? else {
      return Ok(None);
    };
    let mut source = ConcatSource::default();
    source.add(args.source.clone());
    let mut exports = vec![];
    if let Some(analyze_results) = args
      .compilation
      .optimize_analyze_result_map
      .get(&args.module)
    {
      for info in analyze_results.ordered_exports() {
        let name = to_identifier(info.name.as_ref());
        let var_name = format!("__webpack_exports__{}", name);
        source.add(RawSource::from(format!(
          "var {var_name} = __webpack_exports__{};\n",
          property_access(&vec![&info.name], 0)
        )));
        exports.push(format!("{var_name} as {}", info.name));
      }
    }
    if !exports.is_empty() {
      source.add(RawSource::from(format!(
        "export {{ {} }};\n",
        exports.join(", ")
      )));
    }
    Ok(Some(source.boxed()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    let Some(_) = self.get_options_for_chunk(args.compilation, args.chunk_ukey)? else {
      return Ok(());
    };
    self.name().hash(&mut args.hasher);
    Ok(())
  }
}
