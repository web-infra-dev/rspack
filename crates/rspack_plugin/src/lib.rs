#![feature(future_poll_fn)]

mod combined_plugin;

use std::fmt::Debug;

use async_trait::async_trait;
use rspack_shared::{BundleContext, BundleOptions, Chunk, ResolvedId};

pub type LoadHookOutput = Option<String>;
pub type ResolveHookOutput = Option<ResolvedId>;
pub type TransformHookOutput = ast::Program;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  #[inline]
  async fn prepare(&self, _ctx: &BundleContext) {}

  #[inline]
  async fn resolve(
    &self,
    _ctx: &BundleContext,
    _importee: &str,
    _importer: Option<&str>,
  ) -> ResolveHookOutput {
    None
  }

  #[inline]
  async fn load(&self, _ctx: &BundleContext, _id: &str) -> LoadHookOutput {
    None
  }

  #[inline]
  fn transform(&self, _ctx: &BundleContext, ast: ast::Program) -> TransformHookOutput {
    ast
  }

  #[inline]
  fn tap_generated_chunk(
    &self,
    _ctx: &BundleContext,
    _chunk: &Chunk,
    _bundle_options: &BundleOptions,
  ) {
  }
}
