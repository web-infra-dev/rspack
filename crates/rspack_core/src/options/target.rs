use std::str::FromStr;

#[derive(Debug)]
pub enum TargetOptions {
  Web,
  WebWorker,
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
  type Err = anyhow::Error;

  fn from_str(s: &str) -> anyhow::Result<Target> {
    if s.eq("web") {
      Ok(Target::Target(TargetOptions::Web))
    } else if s.starts_with("node") {
      Ok(Target::Target(TargetOptions::Node(s.replace("node", ""))))
    } else if s.eq("webworker") {
      Ok(Target::Target(TargetOptions::WebWorker))
    } else {
      Ok(Target::None)
    }
  }
}
