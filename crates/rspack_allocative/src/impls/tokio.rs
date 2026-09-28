/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

#![cfg(feature = "tokio")]

use tokio::sync::{Mutex, RwLock};

use crate::{Allocative, Key, Visitor};

impl<T: Allocative + ?Sized> Allocative for RwLock<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Ok(data) = self.try_read() {
      visitor.visit_field::<T>(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}

impl<T: Allocative + ?Sized> Allocative for Mutex<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Ok(data) = self.try_lock() {
      visitor.visit_field::<T>(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}

impl<T> Allocative for tokio::sync::mpsc::UnboundedSender<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}
impl<T> Allocative for tokio::sync::oneshot::Receiver<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}
impl Allocative for tokio::sync::Notify {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}
impl<T: Allocative> Allocative for tokio::sync::OnceCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Some(value) = self.get() {
      visitor.visit_field(Key::new("data"), value);
    }
    visitor.exit();
  }
}
