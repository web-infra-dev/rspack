use std::collections::HashMap;
use std::pin::Pin;

use napi::bindgen_prelude::*;
use napi::{JsBuffer, JsUnknown};

use rspack_core::rspack_sources::{BoxSource, RawSource, SourceExt};

use crate::{
  Asset, AssetContent, AssetInfo, AssetInfoRelated, CompatSource, ToWebpackSource,
  UpdateAssetOptions, WebpackSource,
};

#[napi]
pub struct RspackCompilation {
  inner: Pin<&'static mut rspack_core::Compilation>,
}

#[napi]
impl RspackCompilation {
  #[napi]
  pub fn update_asset(
    &self,
    env: Env,
    filename: String,
    new_source_or_function: Either<BoxSource, JsFunction>,
    asset_info_update_or_function: Either<AssetInfo, JsFunction>,
  ) -> Result<()> {
    self.inner.update_asset(
      &filename,
      |source| {
        let new_source = match new_source_or_function {
          Either::A(new_source) => new_source,
          Either::B(new_source_fn) => unsafe {
            <CompatSource as Into>::into(
              WebpackSource::from_napi_value(
                env.raw(),
                new_source_fn
                  .call(
                    None,
                    &[{
                      unsafe {
                        JsUnknown::from_napi_value(
                          env.raw(),
                          ToNapiValue::to_napi_value(
                            env.raw(),
                            source.to_webpack_source().unwrap(),
                          )
                          .unwrap(),
                        )
                        .unwrap()
                      }
                    }],
                  )
                  .unwrap()
                  .raw(),
              )
              .unwrap(),
            )
            .boxed()
          },
        };

        Ok(new_source)
      },
      |info| {
        let asset_info = match asset_info_update_or_function {
          Either::A(asset_info) => asset_info,
          Either::B(asset_info_fn) => unsafe {
            AssetInfo::from_napi_value(
              env.raw(),
              asset_info_fn
                .call(
                  None,
                  &[{
                    unsafe {
                      JsUnknown::from_napi_value(
                        env.raw(),
                        ToNapiValue::to_napi_value(env.raw(), info.clone()).unwrap(),
                      )
                      .unwrap()
                    }
                  }],
                )
                .unwrap()
                .raw(),
            )
            .unwrap()
          },
        };

        Ok(asset_info)
      },
    );

    OK(())
  }

  #[napi(ts_return_type = "Readonly<Asset>[]")]
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
