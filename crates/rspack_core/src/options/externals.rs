use std::collections::HashMap;

pub type Externals = Vec<External>;

#[derive(Debug)]
pub enum External {
  Object(HashMap<String, String>),
  String(String),
}

impl From<HashMap<String, String>> for External {
  fn from(value: HashMap<String, String>) -> Self {
    Self::Object(value)
  }
}

impl From<String> for External {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}

pub type ExternalType = String;
