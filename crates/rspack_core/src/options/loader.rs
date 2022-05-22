use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter; // 0.17.1

#[derive(Debug, Clone, Copy, EnumIter)]
pub enum Loader {
  DataURI,
  Json,
  Text,
  Css,
  Less,
  Sass,
  Js,
  Jsx,
  Ts,
  Tsx,
  Null,
}

impl Loader {
  pub fn values() -> Vec<Loader> {
    Self::iter().into_iter().collect()
  }
}

// TODO: Loader should not have default value. It's meaningless.
impl Default for Loader {
  fn default() -> Self {
    Loader::Null
  }
}
pub type LoaderOptions = HashMap<String, Loader>;
