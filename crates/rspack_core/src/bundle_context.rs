use std::{
  fmt::Debug,
  sync::{Arc, Mutex},
};

use swc::Compiler;

use crate::NormalizedBundleOptions;

pub struct BundleContext {
  pub assets: Mutex<Vec<Asset>>,
  pub compiler: Arc<Compiler>,
  pub options: Arc<NormalizedBundleOptions>,
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
  pub fn new(compiler: Arc<Compiler>, options: Arc<NormalizedBundleOptions>) -> Self {
    Self {
      assets: Default::default(),
      compiler,
      options,
      _noop: (),
    }
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
