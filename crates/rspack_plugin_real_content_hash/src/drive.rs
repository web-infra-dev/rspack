use std::sync::Arc;

use rspack_core::{Compilation, rspack_sources::Source};
use rspack_hook::define_hook;
#[cfg(allocative)]
use rspack_util::allocative;

define_hook!(RealContentHashPluginUpdateHash: SeriesBail(compilation: &Compilation, assets: &[Arc<dyn Source>], old_hash: &str) -> String);

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct RealContentHashPluginHooks {
  #[cfg_attr(allocative, allocative(skip))]
  pub update_hash: RealContentHashPluginUpdateHashHook,
}
