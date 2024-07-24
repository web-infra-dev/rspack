use std::hash::Hash;

use rspack_core::{
  get_entry_runtime, property_access,
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  ApplyContext, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationFinishModules, CompilationParams, CompilerCompilation, CompilerOptions, EntryData,
  LibraryExport, LibraryOptions, LibraryType, ModuleIdentifier, Plugin, PluginContext,
  RuntimeGlobals, UsageState,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesRenderStartup, JsPlugin, RenderSource,
};

use crate::utils::get_options_for_chunk;

#[derive(Debug)]
struct ExportPropertyLibraryPluginParsed<'a> {
  export: Option<&'a LibraryExport>,
}

#[plugin]
#[derive(Debug)]
pub struct ExportPropertyLibraryPlugin {
  library_type: LibraryType,
  ns_object_used: bool,
  runtime_exports_used: bool,
}

impl ExportPropertyLibraryPlugin {
  pub fn new(library_type: LibraryType, ns_object_used: bool, runtime_exports_used: bool) -> Self {
    Self::new_inner(library_type, ns_object_used, runtime_exports_used)
  }
}

impl ExportPropertyLibraryPlugin {
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

#[plugin_hook(CompilerCompilation for ExportPropertyLibraryPlugin)]
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

#[plugin_hook(JavascriptModulesRenderStartup for ExportPropertyLibraryPlugin)]
fn render_startup(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _module: &ModuleIdentifier,
  render_source: &mut RenderSource,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey) else {
    return Ok(());
  };
  if let Some(export) = options.export {
    let mut s = ConcatSource::default();
    s.add(render_source.source.clone());
    s.add(RawSource::from(format!(
      "__webpack_exports__ = __webpack_exports__{};",
      property_access(export, 0)
    )));
    render_source.source = s.boxed();
    return Ok(());
  }
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for ExportPropertyLibraryPlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  let Some(options) = self.get_options_for_chunk(compilation, chunk_ukey) else {
    return Ok(());
  };
  if let Some(export) = &options.export {
    export.hash(hasher);
  }
  Ok(())
}

#[plugin_hook(CompilationFinishModules for ExportPropertyLibraryPlugin)]
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
      if self.ns_object_used {
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
    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for ExportPropertyLibraryPlugin)]
fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if self
    .get_options_for_chunk(compilation, chunk_ukey)
    .is_none()
  {
    return Ok(());
  }

  if self.runtime_exports_used {
    runtime_requirements.insert(RuntimeGlobals::EXPORTS);
  }
  Ok(())
}
