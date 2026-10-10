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

macro_rules! impl_dyn_fn {
  (($($arg:ident),*) $($bound:ident),*) => {
    impl<R, $($arg),*> Allocative for dyn Fn($($arg),*) -> R $(+ $bound)* + '_ {
      fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
        visitor.visit_opaque(self);
      }
    }

  };
}

macro_rules! impl_fn {
  ($($arg:ident),*) => {
    impl<R, $($arg),*> Allocative for fn($($arg),*) -> R {
      fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
        visitor.visit_simple_sized::<Self>();
      }
    }
    impl_dyn_fn!(($($arg),*));
    impl_dyn_fn!(($($arg),*) Send);
    impl_dyn_fn!(($($arg),*) Sync);
    impl_dyn_fn!(($($arg),*) Send, Sync);
  };
}

impl_fn!();
impl_fn!(A0);
impl_fn!(A0, A1);
impl_fn!(A0, A1, A2);
impl_fn!(A0, A1, A2, A3);
impl_fn!(A0, A1, A2, A3, A4);
impl_fn!(A0, A1, A2, A3, A4, A5);
impl_fn!(A0, A1, A2, A3, A4, A5, A6);
