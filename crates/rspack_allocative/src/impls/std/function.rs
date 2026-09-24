/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use crate::{allocative_trait::Allocative, visitor::Visitor};

impl<R> Allocative for fn() -> R {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl<R, A> Allocative for fn(A) -> R {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}
impl<R, A, B> Allocative for fn(A, B) -> R {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}
impl<R, A, B, C> Allocative for fn(A, B, C) -> R {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl<R, A, B, C, D> Allocative for fn(A, B, C, D) -> R {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl<R, A, B, C, D, E> Allocative for fn(A, B, C, D, E) -> R {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl<R> Allocative for dyn Fn() -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0> Allocative for dyn Fn(A0) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1> Allocative for dyn Fn(A0, A1) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2> Allocative for dyn Fn(A0, A1, A2) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3> Allocative for dyn Fn(A0, A1, A2, A3) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4> Allocative for dyn Fn(A0, A1, A2, A3, A4) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5> Allocative for dyn Fn(A0, A1, A2, A3, A4, A5) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5, A6> Allocative for dyn Fn(A0, A1, A2, A3, A4, A5, A6) -> R + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R> Allocative for dyn Fn() -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0> Allocative for dyn Fn(A0) -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1> Allocative for dyn Fn(A0, A1) -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2> Allocative for dyn Fn(A0, A1, A2) -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3> Allocative for dyn Fn(A0, A1, A2, A3) -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4> Allocative for dyn Fn(A0, A1, A2, A3, A4) -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5> Allocative for dyn Fn(A0, A1, A2, A3, A4, A5) -> R + Send + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5, A6> Allocative
  for dyn Fn(A0, A1, A2, A3, A4, A5, A6) -> R + Send + '_
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R> Allocative for dyn Fn() -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0> Allocative for dyn Fn(A0) -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1> Allocative for dyn Fn(A0, A1) -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2> Allocative for dyn Fn(A0, A1, A2) -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3> Allocative for dyn Fn(A0, A1, A2, A3) -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4> Allocative for dyn Fn(A0, A1, A2, A3, A4) -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5> Allocative for dyn Fn(A0, A1, A2, A3, A4, A5) -> R + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5, A6> Allocative
  for dyn Fn(A0, A1, A2, A3, A4, A5, A6) -> R + Sync + '_
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R> Allocative for dyn Fn() -> R + Send + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0> Allocative for dyn Fn(A0) -> R + Send + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1> Allocative for dyn Fn(A0, A1) -> R + Send + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2> Allocative for dyn Fn(A0, A1, A2) -> R + Send + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3> Allocative for dyn Fn(A0, A1, A2, A3) -> R + Send + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4> Allocative for dyn Fn(A0, A1, A2, A3, A4) -> R + Send + Sync + '_ {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5> Allocative
  for dyn Fn(A0, A1, A2, A3, A4, A5) -> R + Send + Sync + '_
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}

impl<R, A0, A1, A2, A3, A4, A5, A6> Allocative
  for dyn Fn(A0, A1, A2, A3, A4, A5, A6) -> R + Send + Sync + '_
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}
