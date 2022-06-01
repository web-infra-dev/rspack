use std::{path::Path, sync::Arc};

use anyhow::Result;
use futures::future::try_join_all;
use nodejs_resolver::Resolver;
use rspack_swc::swc_ecma_ast as ast;
use tracing::instrument;

use crate::{
  BundleContext, Chunk, LoadArgs, LoadedSource, Loader, NormalizedBundleOptions, OnResolveResult,
  Plugin, PluginBuildEndHookOutput, PluginBuildStartHookOutput, PluginTapGeneratedChunkHookOutput,
  PluginTransformAstHookOutput, PluginTransformHookOutput, ResolveArgs,
};

#[derive(Debug)]
pub struct PluginDriver {
  pub plugins: Vec<Box<dyn Plugin>>,
  pub ctx: Arc<BundleContext>,
  pub resolver: Arc<Resolver>,
}

impl PluginDriver {
  /// TODO: Maybe we need to add some tracing here? It should not take too much time to init.
  pub fn new(plugins: Vec<Box<dyn Plugin>>, ctx: Arc<BundleContext>) -> Self {
    let mut build_start_hints: Vec<usize> = Vec::with_capacity(plugins.len());
    let mut build_end_hints = Vec::with_capacity(plugins.len());
    let mut resolve_id_hints = Vec::with_capacity(plugins.len());
    let mut load_hints = Vec::with_capacity(plugins.len());
    let mut transform_hints = Vec::with_capacity(plugins.len());
    let mut transform_ast_hints = Vec::with_capacity(plugins.len());
    let mut tap_generated_chunk_hints = Vec::with_capacity(plugins.len());

    plugins.iter().enumerate().for_each(|(i, p)| {
      if p.need_build_start() {
        build_start_hints.push(i);
      }
      if p.need_build_end() {
        build_end_hints.push(i);
      }
      if p.need_load() {
        load_hints.push(i);
      }
      if p.need_resolve() {
        resolve_id_hints.push(i);
      }
      if p.need_transform() {
        transform_hints.push(i);
      }
      if p.need_transform_ast() {
        transform_ast_hints.push(i);
      }
      if p.need_tap_generated_chunk() {
        tap_generated_chunk_hints.push(i);
      }
    });
    Self {
      plugins,
      ctx,
      build_start_hints,
      build_end_hints,
      resolve_id_hints,
      load_hints,
      transform_hints,
      transform_ast_hints,
      tap_generated_chunk_hints,
    }
  }

  #[instrument(skip_all)]
  pub async fn build_start(&self) -> PluginBuildStartHookOutput {
    try_join_all(
      self
        .build_start_hints
        .iter()
        .map(|i| self.plugins[*i].build_start(&self.ctx)),
    )
    .await?;
    Ok(())
  }
  #[instrument(skip_all)]
  pub async fn build_end(&self) -> PluginBuildEndHookOutput {
    try_join_all(
      self
        .build_end_hints
        .iter()
        .map(|i| self.plugins[*i].build_end(&self.ctx)),
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
      .tap_generated_chunk_hints
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
  ) -> PluginTapGeneratedChunkHookOutput {
    self.plugins.iter().try_for_each(|plugin| -> Result<()> {
      plugin.tap_generated_chunk(&self.ctx, chunk, bundle_options)
    })
  }
}
