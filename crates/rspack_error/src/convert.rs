use miette::SourceOffset;

use crate::{
  Result,
  error::{Error, Label},
};

pub trait ToStringResultToRspackResultExt<T, E: ToString> {
  fn to_rspack_result(self) -> Result<T>;
  fn to_rspack_result_with_message(self, formatter: impl FnOnce(E) -> String) -> Result<T>;
}

impl<T, E: ToString> ToStringResultToRspackResultExt<T, E> for std::result::Result<T, E> {
  fn to_rspack_result(self) -> Result<T> {
    self.map_err(|e| crate::error!(e.to_string()))
  }
  fn to_rspack_result_with_message(self, formatter: impl FnOnce(E) -> String) -> Result<T> {
    self.map_err(|e| crate::error!(formatter(e)))
  }
}

pub trait SerdeResultToRspackResultExt<T> {
  fn to_rspack_result_with_detail(self, content: &str, msg: &str) -> Result<T>;
}

impl<T> SerdeResultToRspackResultExt<T> for std::result::Result<T, serde_json::Error> {
  fn to_rspack_result_with_detail(self, content: &str, msg: &str) -> Result<T> {
    self.map_err(|e| {
      let offset = SourceOffset::from_location(content, e.line(), e.column());
      let mut error = Error::error(msg.into());
      error.labels = Some(vec![Label {
        name: Some(e.to_string()),
        offset: offset.offset(),
        len: 0,
      }]);
      error.src = Some(content.to_string());
      error
    })
  }
}

pub trait AnyhowResultToRspackResultExt<T> {
  fn to_rspack_result_from_anyhow(self) -> Result<T>;
}

impl<T> AnyhowResultToRspackResultExt<T> for std::result::Result<T, anyhow::Error> {
  fn to_rspack_result_from_anyhow(self) -> Result<T> {
    self.map_err(|e| e.into())
  }
}
