use napi::JsString;

pub trait JsStringExt {
  fn into_string(self) -> String;
}

impl<'a> JsStringExt for JsString<'a> {
  fn into_string(self) -> String {
    self
      .into_utf8()
      .expect("Should into utf8")
      .into_owned()
      .expect("Should as_str")
  }
}
