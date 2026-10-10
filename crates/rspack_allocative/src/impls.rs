/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! Manual implementations of `Allocative` for various types.

mod atomic_refcell;
mod camino;
pub(crate) mod common;
mod dashmap;
mod either;
pub(crate) mod hashbrown_util;
mod indexmap;
mod lock_api;
mod once_cell;
mod parking_lot;
mod rspack;
mod serde_json;
mod smallvec;
mod std;
mod tokio;
mod triomphe;
mod ustr;

#[cfg(feature = "napi")]
mod napi;

#[cfg(feature = "pnp")]
mod pnp;
