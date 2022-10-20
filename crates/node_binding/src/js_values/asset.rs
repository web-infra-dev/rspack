use std::fmt::Debug;

use napi::bindgen_prelude::*;

#[napi(object)]
pub struct AssetContent {
  pub buffer: Option<Buffer>,
  pub source: Option<String>,
}
impl Debug for AssetContent {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AssetContent")
      .field("buffer", &"buffer")
      .field("source", &self.source)
      .finish()
  }
}

#[derive(Debug)]
#[napi(object)]
pub struct UpdateAssetOptions {
  pub asset: AssetContent,
  pub filename: String,
}
