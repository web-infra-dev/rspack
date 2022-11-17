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

impl ToString for ExternalType {
  fn to_string(&self) -> String {
    match self {
      ExternalType::NodeCommonjs => "node-commonjs".to_owned(),
      ExternalType::Window => "window".to_owned(),
      // TODO: didn't know where this field comes from, should be aligned to webpack in the future
      ExternalType::Auto => "auto".to_owned(),
    }
  }
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
