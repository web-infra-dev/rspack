use std::{fmt::Debug, path::Path};

use crate::{BundleContext, Chunk, Loader, NormalizedBundleOptions, ResolvedURI};
use async_trait::async_trait;
use rspack_swc::swc_ecma_ast as ast;

pub type PluginLoadHookOutput = Option<LoadedSource>;
pub type PluginResolveHookOutput = Option<ResolvedURI>;
pub type PluginTransformHookOutput = ast::Module;
pub type PluginTransformRawHookOutput = String;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  fn name(&self) -> &'static str;

  #[inline]
  async fn build_start(&self, _ctx: &BundleContext) {}
  #[inline]
  async fn build_end(&self, _ctx: &BundleContext) {}

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
    _uri: &str,
    _loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformRawHookOutput {
    raw
  }

  #[inline]
  fn transform_ast(
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

#[derive(Debug, Default, Clone)]
pub struct LoadedSource {
  pub content: Option<String>,
  pub loader: Option<Loader>,
}

impl LoadedSource {
  pub fn new(content: String) -> Self {
    Self {
      content: Some(content),
      ..Default::default()
    }
  }
  pub fn with_loader(content: String, loader: Loader) -> Self {
    Self {
      content: Some(content),
      loader: Some(loader),
    }
  }
}
