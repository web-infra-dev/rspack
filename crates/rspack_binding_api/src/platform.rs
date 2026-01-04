use napi::{Either, bindgen_prelude::Null};
use napi_derive::napi;
use rspack_core::CompilerPlatform;

#[derive(Debug)]
#[napi(object, object_to_js = false)]
pub struct RawCompilerPlatform {
  pub web: Option<Either<bool, Null>>,
  pub browser: Option<Either<bool, Null>>,
  pub webworker: Option<Either<bool, Null>>,
  pub node: Option<Either<bool, Null>>,
  pub nwjs: Option<Either<bool, Null>>,
  pub electron: Option<Either<bool, Null>>,
}

impl From<RawCompilerPlatform> for CompilerPlatform {
  fn from(value: RawCompilerPlatform) -> Self {
    Self {
      web: from_raw_platform(value.web),
      browser: from_raw_platform(value.browser),
      webworker: from_raw_platform(value.webworker),
      node: from_raw_platform(value.node),
      nwjs: from_raw_platform(value.nwjs),
      electron: from_raw_platform(value.electron),
    }
  }
}

fn from_raw_platform(v: Option<Either<bool, Null>>) -> Option<bool> {
  match v {
    Some(Either::A(v)) => Some(v),
    _ => None,
  }
}
