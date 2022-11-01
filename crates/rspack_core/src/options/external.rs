use anyhow::anyhow;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug)]
pub enum External {
  Object(HashMap<String, String>),
  String(String),
}

#[derive(Debug, Clone)]
pub enum ExternalType {
  NodeCommonjs,
  Window,
  Auto,
}

impl FromStr for ExternalType {
  type Err = anyhow::Error;

  fn from_str(external_type: &str) -> anyhow::Result<ExternalType> {
    match external_type {
      "window" => Ok(ExternalType::Window),
      "node-commonjs" => Ok(ExternalType::NodeCommonjs),
      "" => Ok(ExternalType::Auto),
      _ => Err(anyhow!("Unknown externals type {}", external_type)),
    }
  }
}
