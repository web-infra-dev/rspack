use std::{
  fmt::Debug,
  ops::{Deref, DerefMut},
};

use rspack_collections::UkeyMap;
use rustc_hash::FxHashMap;

pub type RollbackAtomMap<K, V> = RollbackAtom<FxHashMap<K, V>>;
pub type RollbackAtomUKeyMap<K, V> = RollbackAtom<UkeyMap<K, V>>;
// A simple rollback atom that can checkpoint and recover its state.
#[derive(Debug)]
pub struct RollbackAtom<T: Debug> {
  current: T,
  backup: Option<T>,
}

impl<T> Default for RollbackAtom<T>
where
  T: Clone + Debug + Default,
{
  fn default() -> Self {
    Self {
      current: T::default(),
      backup: None,
    }
  }
}
impl<T> RollbackAtom<T>
where
  T: Clone + Debug + Default,
{
  pub fn new(value: T) -> Self {
    Self {
      current: value,
      backup: None,
    }
  }

  pub fn checkpoint(&mut self) {
    assert!(self.backup.is_none());
    self.backup = Some(self.current.clone());
  }

  pub fn reset(&mut self) {
    assert!(self.backup.is_some());
    if let Some(backup) = self.backup.take() {
      self.current = backup;
    }
  }
}

impl<T> Deref for RollbackAtom<T>
where
  T: Clone + Debug + Default,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.current
  }
}
impl<T> DerefMut for RollbackAtom<T>
where
  T: Clone + Debug + Default,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.current
  }
}

#[cfg(test)]
mod tests {
  use super::RollbackAtom;

  #[test]
  fn reset_restores_checkpointed_value() {
    let mut atom = RollbackAtom::new(1);

    atom.checkpoint();
    *atom = 2;
    atom.reset();

    assert_eq!(*atom, 1);
  }

  #[test]
  fn checkpoint_overrides_previous_backup() {
    let mut atom = RollbackAtom::new("a".to_string());

    atom.checkpoint();
    *atom = "b".to_string();
    atom.reset();

    assert_eq!(atom.as_str(), "a");
  }
}
