use std::collections::HashMap;

pub type BundleEntries = HashMap<String, EntryItem>;

#[derive(Debug, Clone)]
pub struct EntryItem {
  pub src: String,
}

impl From<String> for EntryItem {
  fn from(src: String) -> Self {
    Self { src }
  }
}
