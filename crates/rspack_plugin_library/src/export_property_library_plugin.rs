use std::hash::Hash;

use rspack_core::{
  property_access,
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, JsChunkHashArgs, LibraryExport, LibraryOptions, LibraryType, Plugin,
  PluginContext, PluginJsChunkHashHookOutput, PluginRenderStartupHookOutput, RenderStartupArgs,
};

use crate::utils::get_options_for_chunk;

#[derive(Debug)]
struct ExportPropertyLibraryPluginParsed<'a> {
  export: Option<&'a LibraryExport>,
}

#[derive(Debug, Default)]
pub struct ExportPropertyLibraryPlugin {
  library_type: LibraryType,
}

impl ExportPropertyLibraryPlugin {
  pub fn new(library_type: LibraryType) -> Self {
    Self { library_type }
  }

  fn parse_options<'a>(
    &self,
    library: &'a LibraryOptions,
  ) -> ExportPropertyLibraryPluginParsed<'a> {
    ExportPropertyLibraryPluginParsed {
      export: library.export.as_ref(),
    }
  }

  fn get_options_for_chunk<'a>(
    &self,
    compilation: &'a Compilation,
    chunk_ukey: &'a ChunkUkey,
  ) -> Option<ExportPropertyLibraryPluginParsed<'a>> {
    get_options_for_chunk(compilation, chunk_ukey)
      .filter(|library| library.library_type == self.library_type)
      .map(|library| self.parse_options(library))
  }
}

impl Plugin for ExportPropertyLibraryPlugin {
  fn name(&self) -> &'static str {
    "rspack.ExportPropertyLibraryPlugin"
  }

  fn render_startup(
    &self,
    _ctx: PluginContext,
    args: &RenderStartupArgs,
  ) -> PluginRenderStartupHookOutput {
    let Some(options) = self.get_options_for_chunk(args.compilation, args.chunk) else {
      return Ok(None);
    };
    if let Some(export) = options.export {
      let mut s = ConcatSource::default();
      s.add(args.source.clone());
      s.add(RawSource::from(format!(
        "__webpack_exports__ = __webpack_exports__{};",
        property_access(export, 0)
      )));
      return Ok(Some(s.boxed()));
    }
    Ok(Some(args.source.clone()))
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    let Some(options) = self.get_options_for_chunk(args.compilation, args.chunk_ukey) else {
      return Ok(());
    };
    if let Some(export) = &options.export {
      export.hash(&mut args.hasher);
    }
    Ok(())
  }
}
