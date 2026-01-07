use std::{
  fmt::Debug,
  mem,
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
  pub fn swap_2(&mut self, other: T) -> Option<T> {
    match self.0 {
      None => {
        self.0 = Some(other);
        return None;
      }
      Some(_) => {
        let mut v = Some(other);
        mem::swap(&mut self.0, &mut v);
        return v;
      }
    }
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_take_then_return() {
    let mut v: Option<i32> = None;
    let mut u: Option<i32> = Some(32);
    mem::swap(&mut v, &mut u);

    assert_eq!(v, Some(32));
    assert_eq!(u, None);
  }
}
