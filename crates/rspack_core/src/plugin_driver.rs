use std::{path::Path, sync::Arc};

use anyhow::Result;
use futures::future::try_join_all;
use nodejs_resolver::Resolver;
use rspack_swc::swc_ecma_ast as ast;
use tracing::instrument;

use crate::{
  BundleContext, Chunk, LoadArgs, LoadedSource, Loader, NormalizedBundleOptions, OnResolveResult,
  Plugin, PluginBuildEndHookOutput, PluginBuildStartHookOutput, PluginTransformAstHookOutput,
  PluginTransformHookOutput, ResolveArgs,
};

#[derive(Debug)]
pub struct PluginDriver {
  pub plugins: Vec<Box<dyn Plugin>>,
  pub ctx: Arc<BundleContext>,
  pub resolver: Arc<Resolver>,
}

impl PluginDriver {
  #[instrument(skip_all)]
  pub async fn build_start(&self) -> PluginBuildStartHookOutput {
    try_join_all(
      self
        .plugins
        .iter()
        .map(|plugin| plugin.build_start(&self.ctx)),
    )
    .await?;
    Ok(())
  }
  #[instrument(skip_all)]
  pub async fn build_end(&self) -> PluginBuildEndHookOutput {
    try_join_all(
      self
        .plugins
        .iter()
        .map(|plugin| plugin.build_end(&self.ctx)),
    )
    .await?;
    Ok(())
  }
  #[instrument(skip_all)]
  pub async fn resolve_id(&self, args: &ResolveArgs) -> Result<Option<OnResolveResult>> {
    for plugin in &self.plugins {
      let res = plugin.resolve(&self.ctx, args).await?;
      if res.is_some() {
        tracing::trace!("got load result of plugin {:?}", plugin.name());
        return Ok(res);
      }
    }
    Ok(None)
  }
  #[instrument(skip_all)]
  pub async fn load(&self, args: &LoadArgs) -> Result<Option<LoadedSource>> {
    for plugin in &self.plugins {
      let res = plugin.load(&self.ctx, args).await?;
      if res.is_some() {
        return Ok(res);
      }
    }
    Ok(None)
  }
  #[instrument(skip_all)]
  pub fn transform(
    &self,
    uri: &str,
    loader: &mut Option<Loader>,
    raw: String,
  ) -> PluginTransformHookOutput {
    self
      .plugins
      .iter()
      .fold(Ok(raw), |transformed_raw, plugin| {
        plugin.transform(&self.ctx, uri, loader, transformed_raw?)
      })
  }
  #[instrument(skip_all)]
  pub fn transform_ast(&self, path: &Path, ast: ast::Module) -> PluginTransformAstHookOutput {
    self
      .plugins
      .iter()
      .fold(Ok(ast), |transformed_ast, plugin| {
        plugin.transform_ast(&self.ctx, path, transformed_ast?)
      })
  }
  #[instrument(skip_all)]
  pub fn tap_generated_chunk(
    &self,
    chunk: &Chunk,
    bundle_options: &NormalizedBundleOptions,
  ) -> Result<()> {
    self.plugins.iter().try_for_each(|plugin| -> Result<()> {
      plugin.tap_generated_chunk(&self.ctx, chunk, bundle_options)
    })
  }
}
