use std::collections::HashMap;

use crate::EntryItem;

#[derive(Debug, Default, Clone)]
pub struct CompilerOptions {
  pub entries: HashMap<String, EntryItem>,
  pub root: String,
}
