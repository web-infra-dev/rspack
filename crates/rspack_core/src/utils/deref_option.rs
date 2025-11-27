use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct DerefOption<T>(Option<T>);

impl<T> From<T> for DerefOption<T> {
  fn from(value: T) -> Self {
    Self::new(value)
  }
}
impl<T> DerefOption<T> {
  pub fn new(value: T) -> Self {
    Self(Some(value))
  }
  pub fn take(&mut self) -> T {
    self.0.take().unwrap()
  }
  pub fn swap(&mut self, other: &mut T) {
    std::mem::swap(self.0.as_mut().unwrap(), other);
  }
}
impl<T> Deref for DerefOption<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    self.0.as_ref().unwrap()
  }
}
impl<T> DerefMut for DerefOption<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.0.as_mut().unwrap()
  }
}
