use super::Storage;

#[derive(Debug)]
pub struct NoneStorage {}

impl NoneStorage {
  pub fn new() -> Self {
    Self {}
  }
}

impl<Item> Storage<Item> for NoneStorage {
  fn get(&self, _id: &str) -> Option<Item> {
    None
  }
  fn set(&mut self, _id: String, _data: Item) {}
}
