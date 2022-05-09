use std::{fmt::Debug, path::Path, sync::Arc};

use crate::{
  bundle::Bundle, BundleContext, BundleOptions, Chunk, NormalizedBundleOptions, ResolvedId,
};
use async_trait::async_trait;

pub type PluginLoadHookOutput = Option<String>;
pub type PluginResolveHookOutput = Option<ResolvedId>;
pub type PluginTransformHookOutput = ast::Module;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  fn name(&self) -> &'static str;

  // #[inline]
  // async fn before_build(&self, _ctx: &BundleContext, bundle: &mut Bundle) {}

  #[inline]
  async fn resolve(
    &self,
    _ctx: &BundleContext,
    _importee: &str,
    _importer: Option<&str>,
  ) -> PluginResolveHookOutput {
    None
  }

  #[inline]
  async fn load(&self, _ctx: &BundleContext, _id: &str) -> PluginLoadHookOutput {
    None
  }

  #[inline]
  fn transform(
    &self,
    _ctx: &BundleContext,
    path: &Path,
    ast: ast::Module,
  ) -> PluginTransformHookOutput {
    ast
  }

  #[inline]
  fn tap_generated_chunk(
    &self,
    _ctx: &BundleContext,
    _chunk: &Chunk,
    _bundle_options: &NormalizedBundleOptions,
  ) {
  }
}
