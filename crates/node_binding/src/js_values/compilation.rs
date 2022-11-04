use std::pin::Pin;

use napi::bindgen_prelude::*;
use napi::NapiRaw;

use rspack_core::rspack_sources::SourceExt;

use crate::{Asset, AssetInfo, CompatSource, ToWebpackSource, WebpackSource};

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
    self
      .inner
      .as_mut()
      .update_asset(&filename, |compilation_asset| {
        {
          let original_source = &mut compilation_asset.source;
          let new_source = match new_source_or_function {
            Either::A(new_source) => Into::<CompatSource>::into(new_source).boxed(),
            Either::B(new_source_fn) => {
              let webpack_source = unsafe {
                call_js_function_with_napi_objects!(
                  env,
                  new_source_fn,
                  original_source.to_webpack_source()
                )
              }?;

              let compat_source: CompatSource = unsafe {
                convert_raw_napi_value_to_napi_value!(env, WebpackSource, webpack_source.raw())
              }?
              .into();

              compat_source.boxed()
            }
          };
          *original_source = new_source;
        }

        {
          let original_info = &mut compilation_asset.info;
          let asset_info = match asset_info_update_or_function {
            Either::A(asset_info) => asset_info.into(),
            Either::B(asset_info_fn) => {
              let asset_info = unsafe {
                call_js_function_with_napi_objects!(
                  env,
                  asset_info_fn,
                  Into::<AssetInfo>::into(original_info.clone())
                )
              }?;

              unsafe { convert_raw_napi_value_to_napi_value!(env, AssetInfo, asset_info.raw()) }?
            }
          };

          *original_info = asset_info.into();
        }

        Ok(())
      })
      .map_err(|err| err.into())
  }

  #[napi(ts_return_type = "Readonly<Asset>[]")]
  pub fn get_assets(&self) -> Result<Vec<Asset>> {
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
  pub fn emit_asset(
    &mut self,
    filename: String,
    source: WebpackSource,
    asset_info: AssetInfo,
  ) -> Result<()> {
    let compat_source: CompatSource = source.into();

    // Safety: It is safe as modify for the asset will never move Compilation.
    unsafe {
      self.inner.as_mut().get_unchecked_mut().emit_asset(
        filename,
        rspack_core::CompilationAsset {
          source: compat_source.boxed(),
          info: asset_info.into(),
        },
      )
    };

    Ok(())
  }
}

impl RspackCompilation {
  pub fn from_compilation(c: Pin<&'static mut rspack_core::Compilation>) -> Self {
    Self { inner: c }
  }
}
