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
pub struct ModuleLibraryPlugin {}

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
    let mut source = ConcatSource::default();
    let mut exports = vec![];
    if let Some(result) = args.compilation.tree_shaking_result.get(&args.module) {
      for info in result.ordered_exports() {
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
