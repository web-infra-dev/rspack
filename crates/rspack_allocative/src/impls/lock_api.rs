/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

#![cfg(feature = "parking_lot")]

use parking_lot::lock_api::{Mutex, RawMutex, RawRwLock, RwLock};

use crate::{allocative_trait::Allocative, key::Key, visitor::Visitor};

impl<R: RawMutex + 'static, T: Allocative + ?Sized> Allocative for Mutex<R, T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Some(data) = self.try_lock() {
      visitor.visit_field(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}

impl<R: RawRwLock + 'static, T: Allocative + ?Sized> Allocative for RwLock<R, T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Some(data) = self.try_read() {
      visitor.visit_field(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}
