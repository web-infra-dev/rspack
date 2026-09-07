#[derive(Debug, Default)]
pub struct CompilerPlatform {
  pub web: Option<bool>,
  pub browser: Option<bool>,
  pub webworker: Option<bool>,
  pub node: Option<bool>,
  pub nwjs: Option<bool>,
  pub electron: Option<bool>,
}

impl CompilerPlatform {
  pub fn is_neutral(&self) -> bool {
    !self.is_web() && !self.is_node()
  }

  pub fn is_node(&self) -> bool {
    self.node.unwrap_or_default()
  }

  pub fn is_web(&self) -> bool {
    self.web.unwrap_or_default()
  }
}
