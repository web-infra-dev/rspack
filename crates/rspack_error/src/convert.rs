use std::fmt::Display;

use miette::SourceOffset;

use crate::{
  Result,
  error::{Error, Label},
};

pub fn error_from_display(e: &dyn Display) -> Error {
  Error::error(e.to_string())
}

pub fn error_from_string(message: String) -> Error {
  Error::error(message)
}

pub fn serde_error_with_detail(e: &serde_json::Error, content: &str, msg: &str) -> Error {
  let offset = SourceOffset::from_location(content, e.line(), e.column());
  let mut error = Error::error(msg.into());
  error.labels = Some(vec![Label {
    name: Some(e.to_string()),
    offset: offset.offset(),
    len: 0,
  }]);
  error.src = Some(content.to_string());
  error
}

pub trait ToStringResultToRspackResultExt<T, E: Display> {
  fn to_rspack_result(self) -> Result<T>;
  fn to_rspack_result_with_message(self, formatter: impl FnOnce(E) -> String) -> Result<T>;
  fn to_rspack_result_with_message_ref(
    self,
    formatter: &dyn Fn(&dyn Display) -> String,
  ) -> Result<T>;
}

impl<T, E: Display> ToStringResultToRspackResultExt<T, E> for std::result::Result<T, E> {
  fn to_rspack_result(self) -> Result<T> {
    self.map_err(|e| error_from_display(&e))
  }

  fn to_rspack_result_with_message(self, formatter: impl FnOnce(E) -> String) -> Result<T> {
    self.map_err(|e| error_from_string(formatter(e)))
  }

  fn to_rspack_result_with_message_ref(
    self,
    formatter: &dyn Fn(&dyn Display) -> String,
  ) -> Result<T> {
    self.map_err(|e| error_from_string(formatter(&e)))
  }
}

pub trait SerdeResultToRspackResultExt<T> {
  fn to_rspack_result_with_detail(self, content: &str, msg: &str) -> Result<T>;
}

impl<T> SerdeResultToRspackResultExt<T> for std::result::Result<T, serde_json::Error> {
  fn to_rspack_result_with_detail(self, content: &str, msg: &str) -> Result<T> {
    self.map_err(|e| serde_error_with_detail(&e, content, msg))
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
