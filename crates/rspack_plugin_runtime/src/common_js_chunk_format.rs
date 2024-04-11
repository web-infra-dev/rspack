use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  ApplyContext, ChunkUkey, Compilation, CompilationAdditionalChunkRuntimeRequirements,
  CompilationParams, CompilerCompilation, CompilerOptions, Plugin, PluginContext, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::runtime::{render_chunk_runtime_modules, render_iife};
use rspack_plugin_javascript::{
  JavascriptModulesPluginPlugin, JsChunkHashArgs, JsPlugin, PluginJsChunkHashHookOutput,
  PluginRenderJsChunkHookOutput, RenderJsChunkArgs, RenderJsStartupArgs,
};

use crate::{
  generate_entry_startup, get_chunk_output_name, get_relative_path, get_runtime_chunk_output_name,
  update_hash_for_entry_startup,
};

const PLUGIN_NAME: &str = "rspack.CommonJsChunkFormatPlugin";

#[derive(Debug, Default)]
struct CommonJsChunkFormatJavascriptModulesPluginPlugin;

#[async_trait]
impl JavascriptModulesPluginPlugin for CommonJsChunkFormatJavascriptModulesPluginPlugin {
  fn js_chunk_hash(&self, args: &mut JsChunkHashArgs) -> PluginJsChunkHashHookOutput {
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      return Ok(());
    }

    PLUGIN_NAME.hash(&mut args.hasher);

    update_hash_for_entry_startup(
      args.hasher,
      args.compilation,
      args
        .compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey),
      args.chunk_ukey,
    );

    Ok(())
  }

  async fn render_chunk(&self, args: &RenderJsChunkArgs) -> PluginRenderJsChunkHookOutput {
    let drive = JsPlugin::get_compilation_drives(args.compilation);
    let chunk = args.chunk();
    let base_chunk_output_name = get_chunk_output_name(chunk, args.compilation)?;
    let mut sources = ConcatSource::default();
    sources.add(RawSource::from(format!(
      "exports.ids = ['{}'];\n",
      &chunk.expect_id().to_string()
    )));
    sources.add(RawSource::from("exports.modules = "));
    sources.add(args.module_source.clone());
    sources.add(RawSource::from(";\n"));
    if args
      .compilation
      .chunk_graph
      .has_chunk_runtime_modules(args.chunk_ukey)
    {
      sources.add(RawSource::from("exports.runtime = "));
      sources.add(render_chunk_runtime_modules(
        args.compilation,
        args.chunk_ukey,
      )?);
      sources.add(RawSource::from(";\n"));
    }

    if chunk.has_entry_module(&args.compilation.chunk_graph) {
      let runtime_chunk_output_name = get_runtime_chunk_output_name(args)?;
      sources.add(RawSource::from(format!(
        "var {} = require('{}');\n",
        RuntimeGlobals::REQUIRE,
        get_relative_path(&base_chunk_output_name, &runtime_chunk_output_name)
      )));
      sources.add(RawSource::from(format!(
        "{}(exports)\n",
        RuntimeGlobals::EXTERNAL_INSTALL_CHUNK,
      )));

      let entries = args
        .compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey);
      let start_up_source =
        generate_entry_startup(args.compilation, args.chunk_ukey, entries, false);
      let last_entry_module = entries
        .keys()
        .last()
        .expect("should have last entry module");
      if let Some(s) = drive.render_startup(RenderJsStartupArgs {
        compilation: args.compilation,
        chunk: &chunk.ukey,
        module: *last_entry_module,
        source: start_up_source,
      })? {
        sources.add(s);
      }
      return Ok(Some(render_iife(sources.boxed())));
    }
    Ok(Some(sources.boxed()))
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct CommonJsChunkFormatPlugin {
  js_plugin: Arc<CommonJsChunkFormatJavascriptModulesPluginPlugin>,
}

#[plugin_hook(CompilerCompilation for CommonJsChunkFormatPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut drive = JsPlugin::get_compilation_drives_mut(compilation);
  drive.add_plugin(self.js_plugin.clone());
  Ok(())
}

#[plugin_hook(CompilationAdditionalChunkRuntimeRequirements for CommonJsChunkFormatPlugin)]
fn additional_chunk_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  let chunk = compilation.chunk_by_ukey.expect_get(chunk_ukey);

  if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
    return Ok(());
  }

  if compilation
    .chunk_graph
    .get_number_of_entry_modules(chunk_ukey)
    > 0
  {
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    runtime_requirements.insert(RuntimeGlobals::STARTUP_ENTRYPOINT);
    runtime_requirements.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
  }

  Ok(())
}

impl Plugin for CommonJsChunkFormatPlugin {
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
    ctx
      .context
      .compilation_hooks
      .additional_chunk_runtime_requirements
      .tap(additional_chunk_runtime_requirements::new(self));
    Ok(())
  }
}
