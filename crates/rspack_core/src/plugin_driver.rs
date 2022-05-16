use std::{path::Path, sync::Arc};

use futures::future::join_all;

use crate::{
  BundleContext, Chunk, NormalizedBundleOptions, Plugin, PluginTransformHookOutput, ResolvedURI,
};

#[derive(Debug)]
pub struct PluginDriver {
  pub plugins: Vec<Box<dyn Plugin>>,
  pub ctx: Arc<BundleContext>,
}

impl PluginDriver {
  pub async fn build_start(&self) {
    join_all(
      self
        .plugins
        .iter()
        .map(|plugin| plugin.build_start(&self.ctx)),
    )
    .await;
  }
  pub async fn build_end(&self) {
    join_all(
      self
        .plugins
        .iter()
        .map(|plugin| plugin.build_end(&self.ctx)),
    )
    .await;
  }

  pub async fn resolve_id(&self, importee: &str, importer: Option<&str>) -> Option<ResolvedURI> {
    for plugin in &self.plugins {
      let res = plugin.resolve(&self.ctx, importee, importer).await;
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub async fn load(&self, id: &str) -> Option<String> {
    for plugin in &self.plugins {
      let res = plugin.load(&self.ctx, id).await;
      if res.is_some() {
        return res;
      }
    }
    None
  }

  pub fn transform(
    &self,
    path: &Path,
    ast: PluginTransformHookOutput,
  ) -> PluginTransformHookOutput {
    self.plugins.iter().fold(ast, |transformed_ast, plugin| {
      plugin.transform(&self.ctx, path, transformed_ast)
    })
  }

  pub fn tap_generated_chunk(&self, chunk: &Chunk, bundle_options: &NormalizedBundleOptions) {
    self
      .plugins
      .iter()
      .for_each(|plugin| plugin.tap_generated_chunk(&self.ctx, chunk, bundle_options));
  }
}
