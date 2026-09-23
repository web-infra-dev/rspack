/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use allocative_derive::Allocative;

use crate as allocative;
use crate::Visitor;

struct DoesNotImplementAllocative<T>(T);

fn visit_does_not_implement_allocative<T>(
    _field: &DoesNotImplementAllocative<T>,
    visitor: &mut Visitor<'_>,
) {
    visitor
        .enter_self_sized::<DoesNotImplementAllocative<T>>()
        .exit();
}

#[derive(Allocative)]
struct MyStruct {
    #[allocative(visit = visit_does_not_implement_allocative)]
    field: DoesNotImplementAllocative<String>,
}
