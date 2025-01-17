use napi::JsString;
use rspack_error::{miette::IntoDiagnostic, Result};

pub trait JsStringExt {
  fn into_string(self) -> String;
  fn try_into_string(self) -> Result<String>;
}

impl JsStringExt for JsString {
  fn into_string(self) -> String {
    self
      .into_utf8()
      .expect("Should into utf8")
      .as_str()
      .expect("Should as_str")
      .to_string()
  }
  fn try_into_string(self) -> Result<String> {
    Ok(
      self
        .into_utf8()
        .into_diagnostic()?
        .as_str()
        .into_diagnostic()?
        .to_string(),
    )
  }
}
