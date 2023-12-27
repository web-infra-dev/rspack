use std::hash::Hash;

use rspack_core::{
  property_access,
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  ChunkUkey, Compilation, EntryData, JsChunkHashArgs, LibraryExport, LibraryOptions, LibraryType,
  Plugin, PluginContext, PluginJsChunkHashHookOutput, PluginRenderStartupHookOutput,
  RenderStartupArgs, UsageState,
};
use rspack_error::Result;

use crate::utils::get_options_for_chunk;

#[derive(Debug)]
struct ExportPropertyLibraryPluginParsed<'a> {
  export: Option<&'a LibraryExport>,
}

#[derive(Debug, Default)]
pub struct ExportPropertyLibraryPlugin {
  library_type: LibraryType,
  ns_object_used: bool,
}

impl ExportPropertyLibraryPlugin {
  pub fn new(library_type: LibraryType, ns_object_used: bool) -> Self {
    Self {
      library_type,
      ns_object_used,
    }
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

#[async_trait::async_trait]
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

  async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    for (entry_name, entry) in compilation.entries.iter() {
      let EntryData {
        dependencies,
        options,
        ..
      } = entry;
      let runtime = compilation.get_entry_runtime(entry_name, Some(options));
      let library_options = options
        .library
        .as_ref()
        .or_else(|| compilation.options.output.library.as_ref());
      let module_of_last_dep = dependencies
        .last()
        .and_then(|dep| compilation.module_graph.get_module(dep));
      let Some(module_of_last_dep) = module_of_last_dep else {
        continue;
      };
      let Some(library_options) = library_options else {
        continue;
      };
      if let Some(export) = library_options
        .export
        .as_ref()
        .and_then(|item| item.first())
      {
        let export_info_id = compilation
          .module_graph
          .get_export_info(module_of_last_dep.identifier(), &(export.as_str()).into());
        export_info_id.set_used(
          &mut compilation.module_graph,
          UsageState::Used,
          Some(&runtime),
        );
        export_info_id
          .get_export_info_mut(&mut compilation.module_graph)
          .can_mangle_use = Some(false);
      } else {
        let exports_info_id = compilation
          .module_graph
          .get_exports_info(&module_of_last_dep.identifier())
          .id;
        if self.ns_object_used {
          exports_info_id.set_used_in_unknown_way(&mut compilation.module_graph, Some(&runtime));
        } else {
          exports_info_id.set_all_known_exports_used(&mut compilation.module_graph, Some(&runtime));
        }
      }
    }
    Ok(())
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
