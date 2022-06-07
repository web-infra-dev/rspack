use anyhow::Result;
use std::fmt::Debug;

use crate::{Chunk, Loader, NormalizedBundleOptions};
use async_trait::async_trait;
use rspack_swc::swc_ecma_ast as ast;

mod context;
pub use context::*;

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
  fn transform_include(&self, _: &str) -> bool {
    true
  }
  #[inline]
  fn need_build_start(&self) -> bool {
    true
  }

  #[inline]
  fn need_build_end(&self) -> bool {
    true
  }

  #[inline]
  fn need_resolve(&self) -> bool {
    true
  }

  #[inline]
  fn need_load(&self) -> bool {
    true
  }

  #[inline]
  fn need_transform(&self) -> bool {
    true
  }

  #[inline]
  fn need_transform_ast(&self) -> bool {
    true
  }

  #[inline]
  fn need_tap_generated_chunk(&self) -> bool {
    true
  }
  #[inline]
  async fn build_start(&self, _ctx: &PluginContext) -> PluginBuildStartHookOutput {
    Ok(())
  }
  #[inline]
  async fn build_end(&self, _ctx: &PluginContext) -> PluginBuildEndHookOutput {
    Ok(())
  }

  #[inline]
  async fn resolve(&self, _ctx: &PluginContext, _args: &ResolveArgs) -> PluginResolveHookOutput {
    Ok(None)
  }

  #[inline]
  async fn load(&self, _ctx: &PluginContext, _args: &LoadArgs) -> PluginLoadHookOutput {
    Ok(None)
  }

  #[inline]
  fn transform(
    &self,
    _ctx: &PluginContext,
    _uri: &str,
    _loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    Ok(raw)
  }

  #[inline]
  fn transform_ast(
    &self,
    _ctx: &PluginContext,
    _uri: &str,
    ast: ast::Module,
  ) -> PluginTransformAstHookOutput {
    Ok(ast)
  }

  #[inline]
  fn tap_generated_chunk(
    &self,
    _ctx: &PluginContext,
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
