use std::{fmt::Debug, hash::Hash, sync::atomic::AtomicUsize};

static NEXT_ID_MAP: Lazy<DashMap<&str, AtomicUsize>> = Lazy::new(|| Default::default());

use dashmap::DashMap;
use once_cell::sync::Lazy;

use crate::Database as Storage;

/// Ukey stands for Unique key
pub struct Ukey<Item>(usize, std::marker::PhantomData<Item>);

impl<Item> Ukey<Item> {
  pub fn new() -> Self {
    let id = NEXT_ID_MAP
      .entry(Self::stored_type())
      .or_default()
      .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Self(id, std::marker::PhantomData)
  }

  const fn stored_type() -> &'static str {
    std::any::type_name::<Item>()
  }

  pub fn as_ref<'db>(&self, db: &'db Storage<Item>) -> &'db Item {
    db.expect_get(self)
  }

  pub fn as_mut<'db>(&self, db: &'db mut Storage<Item>) -> &'db mut Item {
    db.expect_mut(self)
  }

  pub fn as_usize(&self) -> usize {
    self.0
  }
}

impl<Item> Clone for Ukey<Item> {
  fn clone(&self) -> Self {
    Self(self.0, std::marker::PhantomData)
  }
}

impl<Item> PartialOrd for Ukey<Item> {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    self.0.partial_cmp(&other.0)
  }
}

impl<Item> Ord for Ukey<Item> {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.0.cmp(&other.0)
  }
}

impl<Item> Debug for Ukey<Item> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let item_type = Self::stored_type();
    f.debug_tuple(&format!("{item_type}Sid"))
      .field(&self.0)
      .finish()
  }
}

impl<Item> PartialEq for Ukey<Item> {
  fn eq(&self, other: &Self) -> bool {
    self.0 == other.0
  }
}

impl<Item> Eq for Ukey<Item> {}

impl<Item> Hash for Ukey<Item> {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.0.hash(state);
  }
}

impl<Item> Copy for Ukey<Item> {}
