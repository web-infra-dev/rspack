use std::{fmt::Debug, sync::Arc};

use futures::future::BoxFuture;
use napi::{
  bindgen_prelude::{FromNapiValue, TypeName},
  Either,
};
use rspack_core::{Filename, FilenameFn, LocalFilenameFn, PathData, PublicPath};
use rspack_napi::threadsafe_function::ThreadsafeFunction;

use crate::{AssetInfo, JsPathData};

/// A js filename value. Either a string or a function
#[derive(Debug)]
pub struct JsFilename {
  pub filename: Either<String, ThreadsafeFunction<(JsPathData, Option<AssetInfo>), String>>,
}

impl FromNapiValue for JsFilename {
  unsafe fn from_napi_value(
    env: napi::sys::napi_env,
    napi_val: napi::sys::napi_value,
  ) -> napi::Result<Self> {
    Ok(Self {
      filename: Either::from_napi_value(env, napi_val)?,
    })
  }
}

impl TypeName for JsFilename {
  fn type_name() -> &'static str {
    "JsFilename"
  }

  fn value_type() -> napi::ValueType {
    napi::ValueType::Unknown
  }
}

impl From<JsFilename> for Filename {
  fn from(value: JsFilename) -> Self {
    match value.filename {
      Either::A(template) => Filename::from(template),
      Either::B(f) => Filename::from(Arc::new(ThreadSafeFilenameFn(Arc::new(
        move |path_data, asset_info| {
          let f = f.clone();
          Box::pin(async move { f.call_with_sync((path_data, asset_info)).await })
        },
      ))) as Arc<dyn FilenameFn>),
    }
  }
}

impl From<JsFilename> for PublicPath {
  fn from(value: JsFilename) -> Self {
    match value.filename {
      Either::A(template) => template.into(),
      Either::B(f) => PublicPath::Filename(Filename::from(Arc::new(ThreadSafeFilenameFn(Arc::new(
        move |path_data, asset_info| {
          let f = f.clone();
          Box::pin(async move { f.call_with_sync((path_data, asset_info)).await })
        },
      ))) as Arc<dyn FilenameFn>)),
    }
  }
}

pub type FilenameTsfn = Arc<
  dyn Fn(JsPathData, Option<AssetInfo>) -> BoxFuture<'static, rspack_error::Result<String>>
    + Sync
    + Send,
>;

/// Wrapper of a thread-safe filename js function. Implements `FilenameFn`
struct ThreadSafeFilenameFn(FilenameTsfn);

impl Debug for ThreadSafeFilenameFn {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ThreadSafeFilenameFn").finish()
  }
}

#[async_trait::async_trait]
impl LocalFilenameFn for ThreadSafeFilenameFn {
  async fn call(
    &self,
    path_data: &PathData,
    asset_info: Option<&rspack_core::AssetInfo>,
  ) -> rspack_error::Result<String> {
    (self.0)(
      JsPathData::from_path_data(*path_data),
      asset_info.cloned().map(AssetInfo::from),
    )
    .await
  }
}
impl FilenameFn for ThreadSafeFilenameFn {}
