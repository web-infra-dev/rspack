use std::{hash::Hash, sync::Arc};

use rspack_core::{
  get_entry_runtime, property_access,
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  ApplyContext, ChunkUkey, Compilation, CompilationParams, CompilerOptions, EntryData,
  LibraryExport, LibraryOptions, LibraryType, Plugin, PluginContext, UsageState,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook, AsyncSeries, AsyncSeries2};
use rspack_plugin_javascript::{
  JavascriptModulesPluginPlugin, JsChunkHashArgs, JsPlugin, PluginJsChunkHashHookOutput,
  PluginRenderJsStartupHookOutput, RenderJsStartupArgs,
};

use crate::utils::get_options_for_chunk;

#[derive(Debug)]
struct ExportPropertyLibraryPluginParsed<'a> {
  export: Option<&'a LibraryExport>,
}

#[plugin]
#[derive(Debug)]
pub struct ExportPropertyLibraryPlugin {
  js_plugin: Arc<ExportPropertyLibraryJavascriptModulesPluginPlugin>,
}

impl ExportPropertyLibraryPlugin {
  pub fn new(library_type: LibraryType, ns_object_used: bool) -> Self {
    Self::new_inner(Arc::new(
      ExportPropertyLibraryJavascriptModulesPluginPlugin {
        library_type,
        ns_object_used,
      },
    ))
  }
}

#[derive(Debug)]
struct ExportPropertyLibraryJavascriptModulesPluginPlugin {
  library_type: LibraryType,
  ns_object_used: bool,
}

impl ExportPropertyLibraryJavascriptModulesPluginPlugin {
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

impl JavascriptModulesPluginPlugin for ExportPropertyLibraryJavascriptModulesPluginPlugin {
  fn render_startup(&self, args: &RenderJsStartupArgs) -> PluginRenderJsStartupHookOutput {
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

  fn js_chunk_hash(&self, args: &mut JsChunkHashArgs) -> PluginJsChunkHashHookOutput {
    let Some(options) = self.get_options_for_chunk(args.compilation, args.chunk_ukey) else {
      return Ok(());
    };
    if let Some(export) = &options.export {
      export.hash(&mut args.hasher);
    }
    Ok(())
  }
}

#[plugin_hook(AsyncSeries2<Compilation, CompilationParams> for ExportPropertyLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut drive = JsPlugin::get_compilation_drives_mut(compilation);
  drive.add_plugin(self.js_plugin.clone());
  Ok(())
}

#[plugin_hook(AsyncSeries<Compilation> for ExportPropertyLibraryPlugin)]
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let mut runtime_info = Vec::with_capacity(compilation.entries.len());
  for (entry_name, entry) in compilation.entries.iter() {
    let EntryData {
      dependencies,
      options,
      ..
    } = entry;
    let runtime = get_entry_runtime(entry_name, options, &compilation.entries);
    let library_options = options
      .library
      .as_ref()
      .or_else(|| compilation.options.output.library.as_ref());
    let module_graph = compilation.get_module_graph();
    let module_of_last_dep = dependencies
      .last()
      .and_then(|dep| module_graph.get_module_by_dependency_id(dep));
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
      runtime_info.push((
        runtime,
        Some(export.clone()),
        module_of_last_dep.identifier(),
      ));
    } else {
      runtime_info.push((runtime, None, module_of_last_dep.identifier()));
    }
  }

  for (runtime, export, module_identifier) in runtime_info {
    let mut module_graph = compilation.get_module_graph_mut();
    if let Some(export) = export {
      let export_info_id =
        module_graph.get_export_info(module_identifier, &(export.as_str()).into());
      export_info_id.set_used(&mut module_graph, UsageState::Used, Some(&runtime));
      export_info_id
        .get_export_info_mut(&mut module_graph)
        .can_mangle_use = Some(false);
    } else {
      let exports_info_id = module_graph.get_exports_info(&module_identifier).id;
      if self.js_plugin.ns_object_used {
        exports_info_id.set_used_in_unknown_way(&mut module_graph, Some(&runtime));
      } else {
        exports_info_id.set_all_known_exports_used(&mut module_graph, Some(&runtime));
      }
    }
  }

  Ok(())
}

#[async_trait::async_trait]
impl Plugin for ExportPropertyLibraryPlugin {
  fn name(&self) -> &'static str {
    "rspack.ExportPropertyLibraryPlugin"
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}
