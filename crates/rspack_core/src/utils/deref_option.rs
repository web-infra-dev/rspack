use std::{
  fmt::Debug,
  ops::{Deref, DerefMut},
};

#[derive(Debug)]
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
  pub fn is_none(&self) -> bool {
    self.0.is_none()
  }
  pub fn take(&mut self) -> T {
    self
      .0
      .take()
      .unwrap_or_else(|| panic!("should set in compilation first"))
  }
  pub fn swap(&mut self, other: &mut T) {
    std::mem::swap(
      self
        .0
        .as_mut()
        .unwrap_or_else(|| panic!("should set in compilation first")),
      other,
    );
  }

  pub fn replace(&mut self, value: T) -> Option<T> {
    self.0.replace(value)
  }
}
impl<T> Deref for DerefOption<T> {
  type Target = T;
  fn deref(&self) -> &Self::Target {
    self
      .0
      .as_ref()
      .unwrap_or_else(|| panic!("should set in compilation first"))
  }
}
impl<T> DerefMut for DerefOption<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    self
      .0
      .as_mut()
      .unwrap_or_else(|| panic!("should set in compilation first"))
  }
}
