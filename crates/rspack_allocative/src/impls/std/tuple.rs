/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use crate::{Visit, allocative_trait::Allocative, key::Key, visitor::Visitor};

impl Allocative for () {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl<A: Visit> Allocative for (A,) {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_field(Key::new("0"), &self.0);
  }
}

impl<A: Visit, B: Visit> Allocative for (A, B) {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_field(Key::new("0"), &self.0);
    visitor.visit_field(Key::new("1"), &self.1);
  }
}

impl<A: Visit, B: Visit, C: Visit> Allocative for (A, B, C) {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_field(Key::new("0"), &self.0);
    visitor.visit_field(Key::new("1"), &self.1);
    visitor.visit_field(Key::new("2"), &self.2);
  }
}

impl<A: Visit, B: Visit, C: Visit, D: Visit> Allocative for (A, B, C, D) {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_field(Key::new("0"), &self.0);
    visitor.visit_field(Key::new("1"), &self.1);
    visitor.visit_field(Key::new("2"), &self.2);
    visitor.visit_field(Key::new("3"), &self.3);
  }
}

impl<A: Visit, B: Visit, C: Visit, D: Visit, E: Visit> Allocative for (A, B, C, D, E) {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_field(Key::new("0"), &self.0);
    visitor.visit_field(Key::new("1"), &self.1);
    visitor.visit_field(Key::new("2"), &self.2);
    visitor.visit_field(Key::new("3"), &self.3);
    visitor.visit_field(Key::new("4"), &self.4);
  }
}
