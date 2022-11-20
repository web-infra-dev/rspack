use anyhow::anyhow;
use async_trait::async_trait;
use rspack_core::{
  runtime_globals, AdditionalChunkRuntimeRequirementsArgs, Plugin,
  PluginAdditionalChunkRuntimeRequirementsOutput, PluginContext,
};
use rspack_error::Result;

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
    runtime_requirements.insert(runtime_globals::ON_CHUNKS_LOADED.to_string());
    runtime_requirements.insert(runtime_globals::REQUIRE.to_string());
    // }
    runtime_requirements.insert(runtime_globals::CHUNK_CALLBACK.to_string());

    Ok(())
  }
}
