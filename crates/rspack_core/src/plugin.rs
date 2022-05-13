use std::{fmt::Debug, path::Path};

use crate::{BundleContext, Chunk, NormalizedBundleOptions, ResolvedURI};
use async_trait::async_trait;
use rspack_swc::swc_ecma_ast as ast;

pub type PluginLoadHookOutput = Option<String>;
pub type PluginResolveHookOutput = Option<ResolvedURI>;
pub type PluginTransformHookOutput = ast::Module;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  fn name(&self) -> &'static str;

  #[inline]
  async fn build_start(&self, _ctx: &BundleContext) {}

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
    _path: &Path,
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
