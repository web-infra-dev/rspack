use std::{str::FromStr, string::ParseError};

#[derive(Debug)]
pub enum TargetOptions {
  Web,
  Node(String),
}

#[derive(Debug)]
pub enum Target {
  Target(TargetOptions),
  // we are not going to support StringArray in the near feature
  // StringArray(Vec<String>),
  None,
}

impl FromStr for Target {
  type Err = ParseError;

  fn from_str(s: &str) -> std::result::Result<Target, Self::Err> {
    if s.eq("web") {
      Ok(Target::Target(TargetOptions::Web))
    } else if s.starts_with("node") {
      Ok(Target::Target(TargetOptions::Node(s.replace("node", ""))))
    } else {
      Ok(Target::None)
    }
  }
}
