use std::{
  fmt::Debug,
  ops::{Deref, DerefMut},
};

#[derive(Debug)]
pub struct RollbackSingle<T: Debug> {
  current: T,
  backup: Option<T>,
}

impl<T> Default for RollbackSingle<T>
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
impl<T> RollbackSingle<T>
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

  pub fn recover_from_last_checkpoint(&mut self) {
    if let Some(backup) = self.backup.take() {
      self.current = backup;
    }
  }
}

impl<T> Deref for RollbackSingle<T>
where
  T: Clone + Debug + Default,
{
  type Target = T;

  fn deref(&self) -> &Self::Target {
    &self.current
  }
}
impl<T> DerefMut for RollbackSingle<T>
where
  T: Clone + Debug + Default,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.current
  }
}
