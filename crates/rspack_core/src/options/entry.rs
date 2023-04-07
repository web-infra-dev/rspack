use indexmap::IndexMap;

pub type BundleEntries = IndexMap<String, EntryItem>;

#[derive(Debug, Clone)]
pub struct EntryItem {
  pub import: Vec<String>,
  pub runtime: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EntryOptions {
  pub runtime: Option<String>,
}
