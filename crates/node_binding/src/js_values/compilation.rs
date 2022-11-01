use std::collections::HashMap;
use std::pin::Pin;

use napi::bindgen_prelude::*;
use napi::JsBuffer;

use rspack_core::rspack_sources::{RawSource, SourceExt};

use crate::{
  Asset, AssetContent, AssetInfo, AssetInfoRelated, ToWebpackSource, UpdateAssetOptions,
};

#[napi]
pub struct RspackCompilation {
  inner: Pin<&'static mut rspack_core::Compilation>,
}

#[napi]
impl RspackCompilation {
  #[napi]
  pub fn get_assets(&self, env: Env) -> Result<Vec<Asset>> {
    let mut assets = Vec::<Asset>::with_capacity(self.inner.assets.len());

    for (filename, asset) in self.inner.assets() {
      assets.push(Asset {
        name: filename.clone(),
        source: asset.source.to_webpack_source()?,
        info: asset.info.clone().into(),
      });
    }

    Ok(assets)
  }

  #[napi]
  pub fn emit_asset(&mut self, options: UpdateAssetOptions) -> Result<()> {
    // Safety: It is safe as modify for the asset will never move Compilation.
    let assets = unsafe { &mut self.inner.as_mut().get_unchecked_mut().assets };

    let asset = assets.get_mut(&options.filename).unwrap();
    asset.set_source(match options.asset {
      AssetContent {
        buffer: Some(buffer),
        source: None,
      } => Ok(RawSource::Buffer(buffer.into()).boxed()),
      AssetContent {
        buffer: None,
        source: Some(source),
      } => Ok(RawSource::Source(source).boxed()),
      _ => Err(Error::from_reason(
        "AssetContent can only be string or buffer",
      )),
    }?);

    Ok(())
  }
}

impl RspackCompilation {
  pub fn from_compilation(c: Pin<&'static mut rspack_core::Compilation>) -> Self {
    Self { inner: c }
  }
}
