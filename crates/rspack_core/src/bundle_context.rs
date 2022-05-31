use rspack_swc::{swc, swc_common};
use std::{
  fmt::Debug,
  sync::{Arc, Mutex},
};
use swc::Compiler;
use swc_common::Mark;

use crate::{
  plugin_hook::{self},
  ImportKind, NormalizedBundleOptions, PluginDriver, ResolveArgs, ResolvedURI,
};

#[allow(clippy::manual_non_exhaustive)]
pub struct BundleContext {
  pub assets: Mutex<Vec<Asset>>,
  pub compiler: Arc<Compiler>,
  pub options: Arc<NormalizedBundleOptions>,
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
  plugin_driver: Mutex<Option<Arc<PluginDriver>>>,
  _noop: (),
}

impl Debug for BundleContext {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("BundleContext")
      .field("assets", &self.assets)
      .field("compiler", &"{..}")
      .finish()
  }
}

impl BundleContext {
  pub fn new(
    compiler: Arc<Compiler>,
    options: Arc<NormalizedBundleOptions>,
    top_level_mark: Mark,
    unresolved_mark: Mark,
  ) -> Self {
    Self {
      assets: Default::default(),
      compiler,
      options,
      top_level_mark,
      unresolved_mark,
      plugin_driver: Default::default(),
      _noop: (),
    }
  }

  pub async fn resolve(&self, importer: &str, importee: &str, kind: ImportKind) -> ResolvedURI {
    // We might need to supports skiping plugin
    let plugin_driver = self.plugin_driver.lock().unwrap().as_ref().unwrap().clone();
    plugin_hook::resolve_id(
      ResolveArgs {
        id: importee.to_string(),
        importer: Some(importer.to_string()),
        kind,
      },
      false,
      &plugin_driver,
    )
    .await
  }

  pub fn init(&self, plugin_driver: Arc<PluginDriver>) {
    self.plugin_driver.lock().unwrap().replace(plugin_driver);
  }

  #[inline]
  pub fn emit_asset(&self, asset: Asset) {
    self.emit_assets([asset])
  }

  pub fn emit_assets(&self, assets_to_be_emited: impl IntoIterator<Item = Asset>) {
    let mut assets = self.assets.lock().unwrap();
    assets_to_be_emited.into_iter().for_each(|asset| {
      assets.push(asset);
    });
  }
}

#[derive(Debug)]
pub struct Asset {
  pub source: String,
  pub filename: String,
}
