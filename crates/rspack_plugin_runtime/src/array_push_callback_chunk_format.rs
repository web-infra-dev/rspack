use std::hash::Hash;

use anyhow::anyhow;
use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkKind, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderChunkHookOutput, RenderChunkArgs, RenderStartupArgs, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_plugin_javascript::runtime::{render_chunk_runtime_modules, render_runtime_modules};

use super::{generate_entry_startup, update_hash_for_entry_startup};
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
    runtime_requirements.insert(RuntimeGlobals::ON_CHUNKS_LOADED);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    // }
    runtime_requirements.insert(RuntimeGlobals::CHUNK_CALLBACK);

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
    let output = &args.compilation.options.output;
    output.global_object.hash(&mut args.hasher);
    output.chunk_loading_global.hash(&mut args.hasher);

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
    let runtime_modules = args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(args.chunk_ukey);
    let global_object: &String = &args.compilation.options.output.global_object;
    let mut source = ConcatSource::default();

    if matches!(chunk.kind, ChunkKind::HotUpdate) {
      source.add(RawSource::Source(format!(
        "{}['hotUpdate']('{}', ",
        global_object,
        chunk.expect_id()
      )));
      source.add(args.module_source.clone());
      if !runtime_modules.is_empty() {
        source.add(RawSource::Source(",".to_string()));
        source.add(render_chunk_runtime_modules(
          args.compilation,
          args.chunk_ukey,
        )?);
      }
      source.add(RawSource::Source(");".to_string()));
    } else {
      let chunk_loading_global = &args.compilation.options.output.chunk_loading_global;

      source.add(RawSource::from(format!(
        r#"({}['{}'] = {}['{}'] || []).push([["{}"], "#,
        global_object,
        chunk_loading_global,
        global_object,
        chunk_loading_global,
        chunk.expect_id(),
      )));
      source.add(args.module_source.clone());
      let has_entry = chunk.has_entry_module(&args.compilation.chunk_graph);
      if has_entry || !runtime_modules.is_empty() {
        source.add(RawSource::from(","));
        source.add(RawSource::from(format!(
          "function({}) {{\n",
          RuntimeGlobals::REQUIRE
        )));
        if !runtime_modules.is_empty() {
          source.add(render_runtime_modules(args.compilation, args.chunk_ukey)?);
        }
        if has_entry {
          let entries = args
            .compilation
            .chunk_graph
            .get_chunk_entry_modules_with_chunk_group_iterable(args.chunk_ukey);
          let start_up_source =
            generate_entry_startup(args.compilation, args.chunk_ukey, entries, true);
          let last_entry_module = entries
            .keys()
            .last()
            .expect("should have last entry module");
          if let Some(s) =
            args
              .compilation
              .plugin_driver
              .read()
              .await
              .render_startup(RenderStartupArgs {
                compilation: args.compilation,
                chunk: &chunk.ukey,
                module: *last_entry_module,
                source: start_up_source,
              })?
          {
            source.add(s);
          }
          let runtime_requirements = args
            .compilation
            .chunk_graph
            .get_tree_runtime_requirements(args.chunk_ukey);
          if runtime_requirements.contains(RuntimeGlobals::RETURN_EXPORTS_FROM_RUNTIME) {
            source.add(RawSource::from("return __webpack_exports__;\n"));
          }
        }
        source.add(RawSource::from("\n}\n"));
      }
      source.add(RawSource::from("]);"));
    }

    Ok(Some(source.boxed()))
  }
}
