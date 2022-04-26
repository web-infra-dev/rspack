use std::fmt::Debug;

use async_trait::async_trait;

use crate::{bundler::BundleContext, structs::ResolvedId};

pub type LoadHookOutput = Option<String>;
pub type ResolveHookOutput = Option<ResolvedId>;
pub type TransformHookOutput = swc_ecma_ast::Program;

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
  fn transform(&self, _ctx: &BundleContext, ast: swc_ecma_ast::Program) -> TransformHookOutput {
    ast
  }
}
