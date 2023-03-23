use rspack_regex::RspackRegex;
use rustc_hash::FxHashMap as HashMap;

pub type Externals = Vec<ExternalItem>;

#[derive(Debug)]
pub enum ExternalItemValue {
  String(String),
  Bool(bool),
  // TODO: string[] | Record<string, string|string[]>
}

pub type ExternalItemObject = HashMap<String, ExternalItemValue>;

#[derive(Debug)]
pub enum ExternalItem {
  Object(ExternalItemObject),
  String(String),
  RegExp(RspackRegex),
}

impl From<ExternalItemObject> for ExternalItem {
  fn from(value: ExternalItemObject) -> Self {
    Self::Object(value)
  }
}

impl From<String> for ExternalItem {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}

impl From<RspackRegex> for ExternalItem {
  fn from(value: RspackRegex) -> Self {
    Self::RegExp(value)
  }
}

pub type ExternalType = String;
