use std::hash::Hash;

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  to_identifier,
  tree_shaking::webpack_ext::ExportInfoExt,
  JsChunkHashArgs, Plugin, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderStartupHookOutput, RenderStartupArgs,
};

use crate::utils::property_access;

#[derive(Debug, Default)]
pub struct ModuleLibraryPlugin;

impl ModuleLibraryPlugin {}

impl Plugin for ModuleLibraryPlugin {
  fn name(&self) -> &'static str {
    "ModuleLibraryPlugin"
  }

  fn render_startup(
    &self,
    _ctx: PluginContext,
    args: &RenderStartupArgs,
  ) -> PluginRenderStartupHookOutput {
    if args
      .compilation
      .chunk_graph
      .get_number_of_entry_modules(args.chunk)
      == 0
    {
      return Ok(None);
    }
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
          property_access(&vec![info.name.to_string()])
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
    if args
      .compilation
      .chunk_graph
      .get_number_of_entry_modules(args.chunk_ukey)
      == 0
    {
      return Ok(());
    }
    self.name().hash(&mut args.hasher);
    args
      .compilation
      .options
      .output
      .library
      .hash(&mut args.hasher);
    Ok(())
  }
}
