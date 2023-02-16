use std::sync::Arc;

use rspack_core::{
  ApplyContext, Plugin, PluginBeginIdleHookOutput, PluginContext, ResolverFactory,
};
use rspack_error::Result;

#[derive(Debug, Default)]
pub struct Resolve {
  resolver_factory: Option<Arc<ResolverFactory>>,
}

#[async_trait::async_trait]
impl Plugin for Resolve {
  fn name(&self) -> &'static str {
    "plugin-resolve"
  }

  fn apply(&mut self, ctx: PluginContext<&mut ApplyContext>) -> Result<()> {
    self.resolver_factory = Some(ctx.context.resolver_factory.clone());
    Ok(())
  }

  async fn begin_idle(&mut self) -> PluginBeginIdleHookOutput {
    if let Some(resolver) = self.resolver_factory.take() {
      // Clear the cache of the resolver. As we will clear it in the next build anyway,
      // so it's not necessary for us to wait for this to finish.
      // Writing this to avoid a big overhead for clearing on the next build.
      // This behavior might be changed in the future.
      std::thread::spawn(move || {
        resolver.clear_entries();
      });
    }
    Ok(())
  }
}
