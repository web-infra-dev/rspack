#[derive(Debug, Clone)]
pub struct Entry {
  pub name: Option<String>,
  pub src: String,
}

impl From<String> for Entry {
  fn from(src: String) -> Self {
    Self { name: None, src }
  }
}
