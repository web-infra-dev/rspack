use anyhow::anyhow;
use std::collections::HashMap;

#[derive(Debug)]
pub enum External {
  Object(HashMap<String, String>),
  String(String),
}

#[derive(Debug)]
pub enum ExternalType {
  NodeCommonjs,
  Window,
  Auto,
}

impl ExternalType {
  pub fn new(external_type: String) -> anyhow::Result<ExternalType> {
    match external_type.as_str() {
      "window" => Ok(ExternalType::Window),
      "node-commonjs" => Ok(ExternalType::NodeCommonjs),
      "" => Ok(ExternalType::Auto),
      _ => Err(anyhow!("Unknown externals type {}", external_type)),
    }
  }
}
