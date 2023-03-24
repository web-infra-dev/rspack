use anyhow::anyhow;
use async_trait::async_trait;
use rspack_core::rspack_sources::{ConcatSource, RawSource, SourceExt};
use rspack_core::{
  AdditionalChunkRuntimeRequirementsArgs, FilenameRenderOptions, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext, PluginRenderChunkHookOutput,
  RenderChunkArgs, RenderStartupArgs, RuntimeGlobals,
};
use rspack_error::Result;
use rspack_plugin_javascript::runtime::{
  generate_chunk_entry_code, render_chunk_modules, render_chunk_runtime_modules,
};
#[derive(Debug)]
pub struct CommonJsChunkFormatPlugin {}

#[async_trait]
impl Plugin for CommonJsChunkFormatPlugin {
  fn name(&self) -> &'static str {
    "CommonJsChunkFormatPlugin"
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
      runtime_requirements.insert(RuntimeGlobals::EXTERNAL_INSTALL_CHUNK);
    }

    Ok(())
  }

  async fn render_chunk(
    &self,
    _ctx: PluginContext,
    args: &RenderChunkArgs,
  ) -> PluginRenderChunkHookOutput {
    let chunk = args.chunk();
    let mut sources = ConcatSource::default();
    sources.add(RawSource::from(format!(
      "exports.ids = ['{}'];\n",
      &chunk.expect_id().to_string()
    )));
    sources.add(RawSource::from("exports.modules = "));
    sources.add(render_chunk_modules(args.compilation, args.chunk_ukey)?);
    sources.add(RawSource::from(";\n"));
    if !args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_in_order(args.chunk_ukey)
      .is_empty()
    {
      sources.add(RawSource::from("exports.runtime = "));
      sources.add(render_chunk_runtime_modules(
        args.compilation,
        args.chunk_ukey,
      )?);
      sources.add(RawSource::from(";\n"));
    }

    if chunk.has_entry_module(&args.compilation.chunk_graph) {
      let entry_point = {
        let entry_points = args
          .compilation
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group(&chunk.ukey);

        let entry_point_ukey = entry_points
          .iter()
          .next()
          .ok_or_else(|| anyhow!("should has entry point ukey"))?;

        args
          .compilation
          .chunk_group_by_ukey
          .get(entry_point_ukey)
          .ok_or_else(|| anyhow!("should has entry point"))?
      };

      let runtime_chunk_filename = {
        let runtime_chunk = args
          .compilation
          .chunk_by_ukey
          .get(&entry_point.get_runtime_chunk())
          .ok_or_else(|| anyhow!("should has runtime chunk"))?;

        let hash = Some(runtime_chunk.get_render_hash());
        args
          .compilation
          .options
          .output
          .chunk_filename
          .render(FilenameRenderOptions {
            name: runtime_chunk.name_for_filename_template(),
            extension: Some(".js".to_string()),
            id: runtime_chunk.id.clone(),
            contenthash: hash.clone(),
            chunkhash: hash.clone(),
            hash,
            ..Default::default()
          })
      };

      sources.add(RawSource::from(format!(
        "\nvar {} = require('./{}')",
        RuntimeGlobals::REQUIRE,
        runtime_chunk_filename
      )));
      sources.add(RawSource::from(format!(
        "\n{}(exports)\n",
        RuntimeGlobals::EXTERNAL_INSTALL_CHUNK,
      )));
      sources.add(generate_chunk_entry_code(args.compilation, args.chunk_ukey));
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
        sources.add(s);
      }
    }
    Ok(Some(sources.boxed()))
  }
}
