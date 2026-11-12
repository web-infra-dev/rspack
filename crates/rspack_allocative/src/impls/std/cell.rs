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
  cell::{OnceCell, RefCell, UnsafeCell},
  sync::OnceLock,
};

use crate::{Allocative, Visit, Visitor, impls::common::DATA_NAME};

// The wrapper may be shared while its contents are mutating or uninitialized.
// Only a higher-level adapter holding the appropriate guard may visit its data.
impl<T: ?Sized> Allocative for UnsafeCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<T: Visit> Allocative for RefCell<T> {
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

impl<T: Visit> Allocative for OnceCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(v) = self.get() {
      visitor.visit_field::<T>(DATA_NAME, v);
    }
    visitor.exit();
  }
}

impl<T: Visit> Allocative for OnceLock<T> {
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
  T: Visit + ToOwned + ?Sized,
  T::Owned: Visit,
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    match self {
      Cow::Borrowed(_) => (),
      Cow::Owned(v) => v.visit_memory(&mut visitor),
    }
    visitor.exit();
  }
}
