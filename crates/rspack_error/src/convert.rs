use miette::{LabeledSpan, SourceOffset, miette};

use crate::{AnyhowError, Result};

pub trait ToStringResultToRspackResultExt<T, E: ToString> {
  fn to_rspack_result(self) -> Result<T>;
  fn to_rspack_result_with_message(self, formatter: impl FnOnce(E) -> String) -> Result<T>;
}

impl<T, E: ToString> ToStringResultToRspackResultExt<T, E> for std::result::Result<T, E> {
  fn to_rspack_result(self) -> Result<T> {
    self.map_err(|e| miette!(e.to_string()))
  }
  fn to_rspack_result_with_message(self, formatter: impl FnOnce(E) -> String) -> Result<T> {
    self.map_err(|e| miette!(formatter(e)))
  }
}

pub trait SerdeResultToRspackResultExt<T> {
  fn to_rspack_result_with_detail(self, content: &str, msg: &str) -> Result<T>;
}

impl<T> SerdeResultToRspackResultExt<T> for std::result::Result<T, serde_json::Error> {
  fn to_rspack_result_with_detail(self, content: &str, msg: &str) -> Result<T> {
    self.map_err(|e| {
      let offset = SourceOffset::from_location(content, e.line(), e.column());
      let span = LabeledSpan::at_offset(offset.offset(), e.to_string());
      miette!(labels = vec![span], "{msg}").with_source_code(content.to_string())
    })
  }
}

pub trait AnyhowResultToRspackResultExt<T> {
  fn to_rspack_result_from_anyhow(self) -> Result<T>;
}

impl<T> AnyhowResultToRspackResultExt<T> for std::result::Result<T, anyhow::Error> {
  fn to_rspack_result_from_anyhow(self) -> Result<T> {
    self.map_err(|e| AnyhowError::from(e).into())
  }
}
