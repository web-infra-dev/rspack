use std::{path::Path, sync::Arc};

use futures::future::join_all;
use tracing::instrument;

use crate::{
  BundleContext, Chunk, LoadedSource, Loader, NormalizedBundleOptions, Plugin,
  PluginTransformHookOutput, PluginTransformRawHookOutput, ResolvedURI,
};

#[derive(Debug)]
pub struct PluginDriver {
  pub plugins: Vec<Box<dyn Plugin>>,
  pub ctx: Arc<BundleContext>,
}

impl PluginDriver {
  #[instrument(skip_all)]
  pub async fn build_start(&self) {
    join_all(
      self
        .plugins
        .iter()
        .map(|plugin| plugin.build_start(&self.ctx)),
    )
    .await;
  }
  #[instrument(skip_all)]
  pub async fn build_end(&self) {
    join_all(
      self
        .plugins
        .iter()
        .map(|plugin| plugin.build_end(&self.ctx)),
    )
    .await;
  }
  #[instrument(skip_all)]
  pub async fn resolve_id(&self, importee: &str, importer: Option<&str>) -> Option<ResolvedURI> {
    for plugin in &self.plugins {
      let res = plugin.resolve(&self.ctx, importee, importer).await;
      if res.is_some() {
        tracing::trace!("got load result of plugin {:?}", plugin.name());
        return res;
      }
    }
    None
  }
  #[instrument(skip_all)]
  pub async fn load(&self, id: &str) -> Option<LoadedSource> {
    for plugin in &self.plugins {
      let res = plugin.load(&self.ctx, id).await;
      if res.is_some() {
        return res;
      }
    }
    None
  }
  #[instrument(skip_all)]
  pub fn transform(
    &self,
    uri: &str,
    loader: &mut Loader,
    raw: String,
  ) -> PluginTransformRawHookOutput {
    self.plugins.iter().fold(raw, |transformed_raw, plugin| {
      plugin.transform(&self.ctx, uri, loader, transformed_raw)
    })
  }
  #[instrument(skip_all)]
  pub fn transform_ast(
    &self,
    path: &Path,
    ast: PluginTransformHookOutput,
  ) -> PluginTransformHookOutput {
    self.plugins.iter().fold(ast, |transformed_ast, plugin| {
      plugin.transform_ast(&self.ctx, path, transformed_ast)
    })
  }
  #[instrument(skip_all)]
  pub fn tap_generated_chunk(&self, chunk: &Chunk, bundle_options: &NormalizedBundleOptions) {
    self
      .plugins
      .iter()
      .for_each(|plugin| plugin.tap_generated_chunk(&self.ctx, chunk, bundle_options));
  }
}
