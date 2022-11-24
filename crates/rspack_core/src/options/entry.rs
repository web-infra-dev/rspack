use std::collections::HashMap;

pub type BundleEntries = HashMap<String, EntryItem>;

#[derive(Debug, Clone)]
pub struct EntryItem {
  pub import: Vec<String>,
  pub runtime: Option<String>,
}
