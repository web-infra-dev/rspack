use std::hash::Hash;
use std::sync::Arc;

use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  property_access, to_identifier, ApplyContext, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, LibraryOptions, Plugin, PluginContext,
};
use rspack_error::{error_bail, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesPluginPlugin, JsChunkHashArgs, JsPlugin, PluginJsChunkHashHookOutput,
  PluginRenderJsStartupHookOutput, RenderJsStartupArgs,
};

use crate::utils::{get_options_for_chunk, COMMON_LIBRARY_NAME_MESSAGE};

const PLUGIN_NAME: &str = "rspack.ModuleLibraryPlugin";

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleLibraryPlugin {
  js_plugin: Arc<ModuleLibraryJavascriptModulesPluginPlugin>,
}

#[derive(Debug, Default)]
struct ModuleLibraryJavascriptModulesPluginPlugin;

impl ModuleLibraryJavascriptModulesPluginPlugin {
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

impl JavascriptModulesPluginPlugin for ModuleLibraryJavascriptModulesPluginPlugin {
  fn render_startup(&self, args: &RenderJsStartupArgs) -> PluginRenderJsStartupHookOutput {
    let Some(_) = self.get_options_for_chunk(args.compilation, args.chunk)? else {
      return Ok(None);
    };
    let mut source = ConcatSource::default();
    let module_graph = args.compilation.get_module_graph();
    source.add(args.source.clone());
    let mut exports = vec![];
    if args.compilation.options.is_new_tree_shaking() {
      let exports_info = module_graph.get_exports_info(&args.module);
      for id in exports_info.get_ordered_exports() {
        let info = id.get_export_info(&module_graph);
        let chunk = args.compilation.chunk_by_ukey.expect_get(args.chunk);
        let info_name = info.name.as_ref().expect("should have name");
        let used_name = info
          .get_used_name(info.name.as_ref(), Some(&chunk.runtime))
          .expect("name can't be empty");
        let var_name = format!("__webpack_exports__{}", to_identifier(info_name));
        source.add(RawSource::from(format!(
          "var {var_name} = __webpack_exports__{};\n",
          property_access(&vec![used_name], 0)
        )));
        exports.push(format!("{var_name} as {}", info_name));
      }
    } else if let Some(analyze_results) = args
      .compilation
      .optimize_analyze_result_map()
      .get(&args.module)
    {
      use rspack_core::tree_shaking::webpack_ext::ExportInfoExt;
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

  fn js_chunk_hash(&self, args: &mut JsChunkHashArgs) -> PluginJsChunkHashHookOutput {
    let Some(_) = self.get_options_for_chunk(args.compilation, args.chunk_ukey)? else {
      return Ok(());
    };
    PLUGIN_NAME.hash(&mut args.hasher);
    Ok(())
  }
}

#[plugin_hook(CompilerCompilation for ModuleLibraryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut drive = JsPlugin::get_compilation_drives_mut(compilation);
  drive.add_plugin(self.js_plugin.clone());
  Ok(())
}

impl Plugin for ModuleLibraryPlugin {
  fn name(&self) -> &'static str {
    PLUGIN_NAME
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
    Ok(())
  }
}
