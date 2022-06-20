use rspack_swc::{swc, swc_common};
use serde::{Deserialize, Serialize};
use std::{
  fmt::Debug,
  sync::{Arc, Mutex},
};
use swc::Compiler;
use swc_common::Mark;

use crate::{Helpers, NormalizedBundleOptions};

#[allow(clippy::manual_non_exhaustive)]
pub struct BundleContext {
  pub assets: Arc<Mutex<Vec<Asset>>>,
  pub compiler: Arc<Compiler>,
  pub helpers: Helpers,
  pub options: Arc<NormalizedBundleOptions>,
  pub top_level_mark: Mark,
  pub unresolved_mark: Mark,
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
    let helpers = compiler.run(|| {
      // use false as default, since `output.runtimeChunk` option is not supported.
      Helpers::new(false)
    });

    Self {
      assets: Default::default(),
      compiler,
      helpers,
      options,
      top_level_mark,
      unresolved_mark,
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AssetType {
  JavaScript,
  CSS,
  Asset,
}
#[derive(Debug, Clone)]
pub struct Asset {
  pub source: String,
  pub filename: String,
  pub asset_type: AssetType,
}
