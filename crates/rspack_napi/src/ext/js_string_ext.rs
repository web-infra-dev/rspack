use napi::JsString;

pub trait JsStringExt {
  fn into_string(self) -> String;
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
}
