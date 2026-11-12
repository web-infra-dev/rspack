/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

#![cfg(feature = "once_cell")]

use crate::{Visit, allocative_trait::Allocative, visitor::Visitor};

impl<T: Visit> Allocative for once_cell::sync::OnceCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(val) = self.get() {
      val.visit_memory(&mut visitor);
    }
    visitor.exit();
  }
}

impl<T: Visit> Allocative for once_cell::sync::Lazy<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(val) = once_cell::sync::Lazy::get(self) {
      val.visit_memory(&mut visitor);
    }
    visitor.exit();
  }
}

impl<T: Visit> Allocative for once_cell::unsync::OnceCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(val) = self.get() {
      val.visit_memory(&mut visitor);
    }
    visitor.exit();
  }
}

impl<T: Visit> Allocative for once_cell::unsync::Lazy<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(val) = once_cell::unsync::Lazy::get(self) {
      val.visit_memory(&mut visitor);
    }
    visitor.exit();
  }
}
