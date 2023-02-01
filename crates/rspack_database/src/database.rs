use std::collections::HashMap;

use super::ukey::Ukey;
use crate::DatabaseItem;

#[derive(Default)]
pub struct Database<Item> {
  stored: HashMap<Ukey<Item>, Item>,
}

impl<Item> Database<Item> {
  pub fn new() -> Self {
    Self {
      stored: HashMap::new(),
    }
  }

  pub fn expect_get(&self, id: &Ukey<Item>) -> &Item {
    self
      .stored
      .get(id)
      .unwrap_or_else(|| panic!("Not found {:?}", id))
  }
  pub fn expect_mut(&mut self, id: &Ukey<Item>) -> &mut Item {
    self
      .stored
      .get_mut(id)
      .unwrap_or_else(|| panic!("Not found {:?}", id))
  }
}

impl<Item: DatabaseItem> Database<Item> {
  pub fn add(&mut self, item: Item) {
    debug_assert!(self.stored.get(&item.ukey()).is_none());
    let ukey = item.ukey();
    self.stored.insert(ukey, item);
  }
}
