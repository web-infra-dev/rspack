use std::{fmt::Debug, path::Path};

use crate::{BundleContext, Chunk, Loader, NormalizedBundleOptions};
use async_trait::async_trait;
use rspack_swc::swc_ecma_ast as ast;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum ImportKind {
  Require,
  Import,
  DynamicImport,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolveArgs {
  pub id: String,
  pub importer: Option<String>,
  pub kind: ImportKind,
}

#[derive(Debug, Clone, Default)]
pub struct OnResolveResult {
  pub uri: String,
  pub external: bool,
  pub source: Option<LoadedSource>,
}

#[derive(Debug, Clone)]
pub struct LoadArgs {
  pub id: String,
}

pub type PluginLoadHookOutput = Option<LoadedSource>;
pub type PluginResolveHookOutput = Option<OnResolveResult>;
pub type PluginTransformAstHookOutput = ast::Module;
pub type PluginTransformHookOutput = String;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  fn name(&self) -> &'static str;

  #[inline]
  async fn build_start(&self, _ctx: &BundleContext) {}
  #[inline]
  async fn build_end(&self, _ctx: &BundleContext) {}

  #[inline]
  async fn resolve(&self, _ctx: &BundleContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
    None
  }

  #[inline]
  async fn load(&self, _ctx: &BundleContext, _args: &LoadArgs) -> PluginLoadHookOutput {
    None
  }

  #[inline]
  fn transform(
    &self,
    _ctx: &BundleContext,
    _uri: &str,
    _loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    raw
  }

  #[inline]
  fn transform_ast(
    &self,
    _ctx: &BundleContext,
    _path: &Path,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
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
  pub content: String,
  pub loader: Option<Loader>,
}

impl LoadedSource {
  pub fn new(content: String) -> Self {
    Self {
      content,
      ..Default::default()
    }
  }
  pub fn with_loader(content: String, loader: Loader) -> Self {
    Self {
      content,
      loader: Some(loader),
    }
  }
}
