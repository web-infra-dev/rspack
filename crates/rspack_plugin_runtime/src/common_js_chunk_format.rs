use std::hash::Hash;

use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderChunkHookOutput, RenderChunkArgs, RenderStartupArgs, RuntimeGlobals,
};
use rspack_plugin_javascript::runtime::{render_chunk_runtime_modules, render_iife};

use crate::{
  generate_entry_startup, get_chunk_output_name, get_relative_path, get_runtime_chunk_output_name,
  update_hash_for_entry_startup,
};

#[derive(Debug)]
pub struct CommonJsChunkFormatPlugin;

#[async_trait]
impl Plugin for CommonJsChunkFormatPlugin {
  fn name(&self) -> &'static str {
    "rspack.CommonJsChunkFormatPlugin"
  }

  async fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk_ukey = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;
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

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    if args
      .chunk()
      .has_runtime(&args.compilation.chunk_group_by_ukey)
    {
      return Ok(());
    }

    self.name().hash(&mut args.hasher);

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

  async fn render_chunk(
    &self,
    _ctx: PluginContext,
    args: &RenderChunkArgs,
  ) -> PluginRenderChunkHookOutput {
    let chunk = args.chunk();
    let base_chunk_output_name = get_chunk_output_name(chunk, args.compilation);
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
      if let Some(s) = args
        .compilation
        .plugin_driver
        .render_startup(RenderStartupArgs {
          compilation: args.compilation,
          chunk: &chunk.ukey,
          module: *last_entry_module,
          source: start_up_source,
        })?
      {
        sources.add(s);
      }
      return Ok(Some(render_iife(sources.boxed())));
    }
    Ok(Some(sources.boxed()))
  }
}
