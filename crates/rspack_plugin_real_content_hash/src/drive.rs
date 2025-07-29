use std::sync::Arc;

use rspack_core::{Compilation, rspack_sources::Source};
use rspack_hook::define_hook;

define_hook!(RealContentHashPluginUpdateHash: SeriesBail(compilation: &Compilation, assets: &[Arc<dyn Source>], old_hash: &str) -> String);

#[derive(Debug, Default)]
pub struct RealContentHashPluginHooks {
  pub update_hash: RealContentHashPluginUpdateHashHook,
}
