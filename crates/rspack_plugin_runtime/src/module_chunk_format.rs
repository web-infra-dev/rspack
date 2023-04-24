use std::hash::Hash;

use anyhow::anyhow;
use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, ChunkKind, JsChunkHashArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRenderChunkHookOutput, RenderChunkArgs, RenderStartupArgs, RuntimeGlobals,
};
use rspack_error::{internal_error, Result};
use rspack_plugin_javascript::runtime::render_chunk_runtime_modules;

use super::update_hash_for_entry_startup;
use crate::get_runtime_chunk_path;
#[derive(Debug)]
pub struct ModuleChunkFormatPlugin {}

#[async_trait]
impl Plugin for ModuleChunkFormatPlugin {
  fn name(&self) -> &'static str {
    "ModuleChunkFormatPlugin"
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

    if matches!(chunk.kind, ChunkKind::HotUpdate) {
      return Err(internal_error!(
        "HMR is not implemented for module chunk format yet"
      ));
    }

    let mut sources = ConcatSource::default();
    sources.add(RawSource::from(format!(
      "export const ids = ['{}'];\n",
      &chunk.expect_id().to_string()
    )));
    sources.add(RawSource::from("export const modules = "));
    sources.add(args.module_source.clone());
    sources.add(RawSource::from(";\n"));

    if !args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(args.chunk_ukey)
      .is_empty()
    {
      sources.add(RawSource::from("export const runtime = "));
      sources.add(render_chunk_runtime_modules(
        args.compilation,
        args.chunk_ukey,
      )?);
      sources.add(RawSource::from(";\n"));
    }

    let has_entry = chunk.has_entry_module(&args.compilation.chunk_graph);
    if has_entry {
      sources.add(RawSource::from(format!(
        "import __webpack_require__ from '{}';\n",
        get_runtime_chunk_path(args)?
      )));

      //   let mut startup_source = ConcatSource::default();

      //   startup_source.add(RawSource::from(
      //     r#"
      //   var __webpack_exec__ = function(moduleId) {
      //     __webpack_require__(__webpack_require__.s = moduleId);
      //   }
      //   "#,
      //   ));
      let last_entry_module = args
        .compilation
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(&chunk.ukey)
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
          })?
      {
        sources.add(s);
      }
    }

    Ok(Some(sources.boxed()))
  }
}
