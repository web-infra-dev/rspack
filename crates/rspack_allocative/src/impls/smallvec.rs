/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

#![cfg(feature = "smallvec")]

use smallvec::{Array, SmallVec};

use crate::{allocative_trait::Allocative, impls::common::PTR_NAME, key::Key, visitor::Visitor};

impl<A> Allocative for SmallVec<A>
where
  A: Array,
  A::Item: Allocative,
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if self.spilled() {
      let mut visitor = visitor.enter_unique(PTR_NAME, std::mem::size_of::<*const A::Item>());
      for item in self {
        visitor.visit_field(Key::new("data"), item);
      }
      // TODO(nga): spare capacity.
      visitor.exit();
    } else {
      for item in self {
        visitor.visit_field(Key::new("data"), item);
      }
    }
    visitor.exit();
  }
}
