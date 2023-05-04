use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  Plugin, PluginContext, PluginRenderStartupHookOutput, RenderStartupArgs,
};

use crate::utils::property_access;

#[derive(Debug, Default)]
pub struct ExportPropertyLibraryPlugin {}

impl ExportPropertyLibraryPlugin {}

impl Plugin for ExportPropertyLibraryPlugin {
  fn name(&self) -> &'static str {
    "ExportPropertyLibraryPlugin"
  }

  fn render_startup(
    &self,
    _ctx: PluginContext,
    args: &RenderStartupArgs,
  ) -> PluginRenderStartupHookOutput {
    if let Some(export) = args
      .compilation
      .options
      .output
      .library
      .as_ref()
      .and_then(|lib| lib.export.as_ref())
    {
      let mut s = ConcatSource::default();
      s.add(args.source.clone());
      s.add(RawSource::from(format!(
        "__webpack_exports__ = __webpack_exports__{};",
        property_access(export)
      )));
      return Ok(Some(s.boxed()));
    }
    Ok(Some(args.source.clone()))
  }
}
