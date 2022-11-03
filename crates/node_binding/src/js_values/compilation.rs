use std::collections::HashMap;
use std::pin::Pin;

use napi::bindgen_prelude::*;
use napi::{JsBuffer, JsUnknown, NapiRaw};

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
  #[napi(
    ts_args_type = r#"filename: string, newSourceOrFunction: WebpackSource | ((...args: any[]) => any), assetInfoUpdateOrFunction: AssetInfo | ((...args: any[]) => any)"#
  )]
  pub fn update_asset(
    &mut self,
    env: Env,
    filename: String,
    new_source_or_function: Either<WebpackSource, JsFunction>,
    asset_info_update_or_function: Either<AssetInfo, JsFunction>,
  ) -> Result<()> {
    dbg!("called update asset");
    self
      .inner
      .as_mut()
      .update_asset(
        &filename,
        |js_source| {
          let new_source = match new_source_or_function {
            Either::A(new_source) => Into::<CompatSource>::into(new_source).boxed(),
            Either::B(new_source_fn) => {
              let webpack_source = unsafe {
                call_js_function_with_napi_objects!(
                  env,
                  new_source_fn,
                  js_source.to_webpack_source()
                )
              }?;

              let compat_source: CompatSource = unsafe {
                convert_raw_napi_value_to_napi_value!(env, WebpackSource, webpack_source.raw())
              }?
              .into();

              compat_source.boxed()
            }
          };

          *js_source = new_source;

          Ok(())
        },
        |js_info| {
          let asset_info = match asset_info_update_or_function {
            Either::A(asset_info) => asset_info.into(),
            Either::B(asset_info_fn) => {
              let asset_info = unsafe {
                call_js_function_with_napi_objects!(
                  env,
                  asset_info_fn,
                  Into::<AssetInfo>::into(js_info.clone())
                )
              }?;

              unsafe { convert_raw_napi_value_to_napi_value!(env, AssetInfo, asset_info.raw()) }?
            }
          };

          *js_info = asset_info.into();

          Ok(())
        },
      )
      .map_err(|err| err.into())
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
