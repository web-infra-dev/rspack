#[derive(Clone, Debug)]
pub struct RuntimeSourceNode {
  pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct Runtime {
  pub sources: Vec<RuntimeSourceNode>,
}

impl Runtime {
  pub fn generate(&self) -> String {
    let runtime_content = self
      .sources
      .iter()
      .fold(String::new(), |prev, cur| prev + &cur.content);
    format!(r#"(function () {{ {} }})();"#, runtime_content)
  }
}
