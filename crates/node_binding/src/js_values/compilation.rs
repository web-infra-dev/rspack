use std::collections::HashMap;

use napi::bindgen_prelude::*;
use napi::JsBuffer;

use rspack_core::rspack_sources::{RawSource, SourceExt};

use crate::{AssetContent, UpdateAssetOptions};

#[napi]
pub struct RspackCompilation {
  inner: &'static mut rspack_core::Compilation,
}

#[napi]
impl RspackCompilation {
  #[napi]
  pub fn get_assets(&self, env: Env) -> Result<HashMap<String, JsBuffer>> {
    Ok(
      self
        .inner
        .assets()
        .iter()
        .map(|record| {
          let buf = env
            .create_buffer_with_data(record.1.source.buffer().to_vec())
            .unwrap();
          (record.0.to_owned(), buf.into_raw())
        })
        .collect(),
    )
  }

  #[napi]
  pub fn emit_asset(&mut self, options: UpdateAssetOptions) -> Result<()> {
    let asset = self.inner.assets.get_mut(&options.filename).unwrap();
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
  pub fn from_compilation(c: &'static mut rspack_core::Compilation) -> Self {
    Self { inner: c }
  }
}
