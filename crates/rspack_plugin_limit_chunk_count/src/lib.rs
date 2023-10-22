use rspack_core::{OptimizeChunksArgs, Plugin, PluginContext, PluginOptimizeChunksOutput};

#[derive(Debug, Clone, Default)]
pub struct LimitChunkCountPluginOptions {}

#[derive(Debug)]
pub struct LimitChunkCountPlugin {
  options: LimitChunkCountPluginOptions,
}

impl LimitChunkCountPlugin {
  pub fn new(options: LimitChunkCountPluginOptions) -> Self {
    Self { options }
  }
}

#[async_trait::async_trait]
impl Plugin for LimitChunkCountPlugin {
  fn name(&self) -> &'static str {
    "LimitChunkCountPlugin"
  }

  async fn optimize_chunks(
    &self,
    _ctx: PluginContext,
    _args: OptimizeChunksArgs<'_>,
  ) -> PluginOptimizeChunksOutput {
    Ok(())
  }
}
