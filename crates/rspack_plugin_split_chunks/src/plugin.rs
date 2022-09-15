use rspack_core::Plugin;

#[derive(Debug)]
pub struct SplitChunksPlugin {}

impl Plugin for SplitChunksPlugin {
  fn name(&self) -> &'static str {
    "split_chunks"
  }

  fn optimize_chunks(
    &self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    let compilation = args.compilation;
    Ok(())
  }
}
