pub trait NapiErrorIntoRspackError {
  fn into_rspack_error(self) -> rspack_error::Error;
}

pub trait NapiResultIntoRspackResult<T> {
  fn into_rspack_result(self) -> rspack_error::Result<T>;
}

impl NapiErrorIntoRspackError for napi::Error {
  fn into_rspack_error(self) -> rspack_error::Error {
    rspack_error::Error::Napi {
      status: format!("{}", self.status),
      reason: self.reason.clone(),
    }
  }
}

impl<T> NapiResultIntoRspackResult<T> for napi::Result<T> {
  fn into_rspack_result(self) -> rspack_error::Result<T> {
    self.map_err(|e| e.into_rspack_error())
  }
}
