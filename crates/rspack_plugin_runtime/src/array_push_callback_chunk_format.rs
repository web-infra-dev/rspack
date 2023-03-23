use anyhow::anyhow;
use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, ChunkKind, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginRenderChunkHookOutput,
  RenderChunkArgs, RenderStartupArgs,
};
use rspack_error::Result;
use rspack_plugin_javascript::runtime::{
  generate_chunk_entry_code, render_chunk_modules, render_chunk_runtime_modules,
  render_runtime_modules,
};

#[derive(Debug)]
pub struct ArrayPushCallbackChunkFormatPlugin {}

#[async_trait]
impl Plugin for ArrayPushCallbackChunkFormatPlugin {
  fn name(&self) -> &'static str {
    "ArrayPushCallbackChunkFormatPlugin"
  }

  fn apply(
    &mut self,
    _ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
  ) -> Result<()> {
    Ok(())
  }

  fn additional_chunk_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &mut args.compilation;
    let chunk_ukey = args.chunk;
    let runtime_requirements = &mut args.runtime_requirements;
    let chunk = compilation
      .chunk_by_ukey
      .get(chunk_ukey)
      .ok_or_else(|| anyhow!("chunk not found"))?;

    if chunk.has_runtime(&compilation.chunk_group_by_ukey) {
      return Ok(());
    }

    // TODO: check if chunk is entrypoint
    // if compilation
    //   .chunk_graph
    //   .get_number_of_entry_modules(chunk_ukey)
    //   > 0
    // {
    runtime_requirements.insert(runtime_globals::ON_CHUNKS_LOADED);
    runtime_requirements.insert(runtime_globals::REQUIRE);
    // }
    runtime_requirements.insert(runtime_globals::CHUNK_CALLBACK);

    Ok(())
  }

  async fn render_chunk(
    &self,
    _ctx: PluginContext,
    args: &RenderChunkArgs,
  ) -> PluginRenderChunkHookOutput {
    let chunk = args.chunk();
    let runtime_modules = args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(args.chunk_ukey);
    let mut source = ConcatSource::default();

    if matches!(chunk.kind, ChunkKind::HotUpdate) {
      source.add(RawSource::Source(format!(
        "self['hotUpdate']('{}', ",
        chunk.expect_id()
      )));
      source.add(render_chunk_modules(args.compilation, args.chunk_ukey)?);
      if !runtime_modules.is_empty() {
        source.add(RawSource::Source(",".to_string()));
        source.add(render_chunk_runtime_modules(
          args.compilation,
          args.chunk_ukey,
        )?);
      }
      source.add(RawSource::Source(");".to_string()));
    } else {
      source.add(RawSource::from(format!(
        r#"(self['webpackChunkwebpack'] = self['webpackChunkwebpack'] || []).push([["{}"], "#,
        chunk.expect_id(),
      )));
      source.add(render_chunk_modules(args.compilation, args.chunk_ukey)?);
      let has_entry = chunk.has_entry_module(&args.compilation.chunk_graph);
      if has_entry || !runtime_modules.is_empty() {
        source.add(RawSource::from(","));
        source.add(RawSource::from(format!(
          "function({}) {{\n",
          runtime_globals::REQUIRE
        )));
        if !runtime_modules.is_empty() {
          source.add(render_runtime_modules(args.compilation, args.chunk_ukey)?);
        }
        if has_entry {
          source.add(generate_chunk_entry_code(args.compilation, args.chunk_ukey));
          let runtime_requirements = args
            .compilation
            .chunk_graph
            .get_tree_runtime_requirements(args.chunk_ukey);
          if runtime_requirements.contains(runtime_globals::RETURN_EXPORTS_FROM_RUNTIME) {
            source.add(RawSource::from("return __webpack_exports__;\n"));
          }
          if let Some(s) =
            args
              .compilation
              .plugin_driver
              .read()
              .await
              .render_startup(RenderStartupArgs {
                compilation: args.compilation,
                chunk: &chunk.ukey,
              })?
          {
            source.add(s);
          }
        }
        source.add(RawSource::from("\n}\n"));
      }
      source.add(RawSource::from("]);"));
    }

    Ok(Some(source.boxed()))
  }
}
