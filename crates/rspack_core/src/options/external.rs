use std::{collections::HashMap, fmt::Display, str::FromStr};

use anyhow::anyhow;

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

impl Display for ExternalType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ExternalType::NodeCommonjs => write!(f, "node-commonjs"),
      ExternalType::Window => write!(f, "window"),
      // TODO: didn't know where this field comes from, should be aligned to webpack in the future
      ExternalType::Auto => write!(f, "auto"),
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
