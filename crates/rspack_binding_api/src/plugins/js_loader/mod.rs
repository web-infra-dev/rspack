mod cache;
mod channel;
mod context;
mod resolver;
mod scheduler;

pub use cache::{JsLoaderCache, JsLoaderCacheEntry};
pub use channel::JsLoaderChannel;
pub(crate) use channel::LoaderDispatcher;
pub use context::{JsLoaderContext, JsLoaderDependencies, JsLoaderItem};
use rspack_core::Plugin;
use rspack_error::Result;
use rspack_hook::plugin;

#[plugin]
#[derive(Debug, Default)]
pub(crate) struct JsLoaderRspackPlugin;

impl Plugin for JsLoaderRspackPlugin {
  fn name(&self) -> &'static str {
    "rspack.JsLoaderRspackPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .normal_module_factory_hooks
      .resolve_loader
      .tap(resolver::resolve_loader::new(self));
    Ok(())
  }
}
