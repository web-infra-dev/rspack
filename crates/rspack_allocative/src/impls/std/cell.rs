/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::{
  borrow::Cow,
  cell::{OnceCell, RefCell},
  sync::OnceLock,
};

use crate::{Allocative, Visitor, impls::common::DATA_NAME};

impl<T: Allocative> Allocative for RefCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Ok(v) = self.try_borrow() {
      visitor.visit_field(DATA_NAME, &*v);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}

impl<T: Allocative> Allocative for OnceCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(v) = self.get() {
      visitor.visit_field::<T>(DATA_NAME, v);
    }
    visitor.exit();
  }
}

impl<T: Allocative> Allocative for OnceLock<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(v) = self.get() {
      visitor.visit_field::<T>(DATA_NAME, v);
    }
    visitor.exit();
  }
}

impl<T> Allocative for Cow<'_, T>
where
  T: Allocative + ToOwned + ?Sized,
  T::Owned: Allocative,
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    match self {
      Cow::Borrowed(_) => (),
      Cow::Owned(v) => v.visit(&mut visitor),
    }
    visitor.exit();
  }
}

#[cfg(test)]
mod tests {
  use std::cell::{OnceCell, RefCell};

  use crate::golden::golden_test;

  #[test]
  fn test_default() {
    golden_test!(&RefCell::new("abc".to_owned()))
  }

  #[test]
  fn test_borrowed() {
    let cell = RefCell::new("abc".to_owned());
    let _lock = cell.borrow_mut();
    golden_test!(&cell)
  }

  #[test]
  fn test_once_cell() {
    let cell = OnceCell::new();
    cell.set("abc".to_owned()).unwrap();
    golden_test!(&cell)
  }
}
