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
    self
      .sources
      .iter()
      .fold(String::from("(function () { "), |prev, cur| {
        prev + &cur.content
      })
      + " })();"
  }
}
