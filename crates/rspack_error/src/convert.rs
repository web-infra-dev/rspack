use miette::{miette, LabeledSpan, SourceOffset};

use crate::Result;

pub trait SerdeResultToRspackResultExt<T>: ToStringResultToRspackResultExt<T> {
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

pub trait ToStringResultToRspackResultExt<T> {
  fn to_rspack_result(self) -> Result<T>;
}

impl<T, E: ToString> ToStringResultToRspackResultExt<T> for std::result::Result<T, E> {
  fn to_rspack_result(self) -> Result<T> {
    self.map_err(|e| miette!(e.to_string()))
  }
}
