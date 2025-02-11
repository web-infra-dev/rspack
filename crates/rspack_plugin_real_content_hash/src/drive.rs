use std::sync::Arc;

use rspack_core::{rspack_sources::Source, Compilation};
use rspack_hook::define_hook;

define_hook!(RealContentHashPluginUpdateHash: AsyncSeriesBail(compilation: &Compilation, assets: &[Arc<dyn Source>], old_hash: &str) -> String);

#[derive(Debug, Default)]
pub struct RealContentHashPluginHooks {
  pub update_hash: RealContentHashPluginUpdateHashHook,
}
