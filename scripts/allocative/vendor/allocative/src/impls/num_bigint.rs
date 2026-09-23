/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

#![cfg(feature = "num-bigint")]

use std::mem;

use num_bigint::BigInt;
use num_bigint::BigUint;

use crate::Allocative;
use crate::Visitor;
use crate::impls::common::DATA_NAME;
use crate::impls::common::PTR_NAME;

impl Allocative for BigInt {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
        let mut visitor = visitor.enter_self_sized::<Self>();
        {
            let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const ()>());
            // We don't know capacity, so just use the length.
            visitor.visit_simple(
                DATA_NAME,
                self.iter_u64_digits().count() * mem::size_of::<u64>(),
            );
            visitor.exit();
        }
        visitor.exit();
    }
}

impl Allocative for BigUint {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
        let mut visitor = visitor.enter_self_sized::<Self>();
        {
            let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const ()>());
            // We don't know capacity, so just use the length.
            visitor.visit_simple(
                DATA_NAME,
                self.iter_u64_digits().count() * mem::size_of::<u64>(),
            );
            visitor.exit();
        }
        visitor.exit();
    }
}
