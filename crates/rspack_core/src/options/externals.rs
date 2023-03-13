use std::collections::HashMap;

use rspack_regex::RspackRegex;

pub type Externals = Vec<ExternalItem>;

#[derive(Debug)]
pub enum ExternalItem {
  Object(HashMap<String, String>),
  String(String),
  RegExp(RspackRegex),
}

impl From<HashMap<String, String>> for ExternalItem {
  fn from(value: HashMap<String, String>) -> Self {
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
