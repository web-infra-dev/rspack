use anyhow::Result;
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

#[derive(Debug, Clone)]
pub struct ResolveArgs {
  pub id: String,
  pub importer: Option<String>,
  pub kind: ImportKind,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct OnResolveResult {
  pub uri: String,
  pub external: bool,
}

#[derive(Debug, Clone)]
pub struct LoadArgs {
  pub id: String,
  pub kind: ImportKind,
}

pub type PluginBuildStartHookOutput = Result<()>;
pub type PluginBuildEndHookOutput = Result<()>;
pub type PluginLoadHookOutput = Result<Option<LoadedSource>>;
pub type PluginResolveHookOutput = Result<Option<OnResolveResult>>;
pub type PluginTransformAstHookOutput = Result<ast::Module>;
pub type PluginTransformHookOutput = Result<String>;
pub type PluginTapGeneratedChunkHookOutput = Result<()>;

#[async_trait]
pub trait Plugin: Sync + Send + Debug {
  fn name(&self) -> &'static str;

  #[inline]
  async fn build_start(&self, _ctx: &BundleContext) -> PluginBuildStartHookOutput {
    Ok(())
  }
  #[inline]
  async fn build_end(&self, _ctx: &BundleContext) -> PluginBuildEndHookOutput {
    Ok(())
  }

  #[inline]
  async fn resolve(&self, _ctx: &BundleContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
    Ok(None)
  }

  #[inline]
  async fn load(&self, _ctx: &BundleContext, _args: &LoadArgs) -> PluginLoadHookOutput {
    Ok(None)
  }

  #[inline]
  fn transform(
    &self,
    _ctx: &BundleContext,
    _uri: &str,
    _loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    Ok(raw)
  }

  #[inline]
  fn transform_ast(
    &self,
    _ctx: &BundleContext,
    _path: &Path,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    Ok(ast)
  }

  #[inline]
  fn tap_generated_chunk(
    &self,
    _ctx: &BundleContext,
    _chunk: &Chunk,
    _bundle_options: &NormalizedBundleOptions,
  ) -> PluginTapGeneratedChunkHookOutput {
    Ok(())
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
