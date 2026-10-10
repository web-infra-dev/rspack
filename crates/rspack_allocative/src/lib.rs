/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

//! # Allocative
//!
//! Crate implements lightweight memory profiler which allows
//! object traversal and size introspection.
//!
//! Explicit [`Allocative`] adapters describe enums, containers and custom field policies.
//! [`Visit`] provides the common object-safe traversal interface. This crate
//! provides [`FlameGraphBuilder`] to build a flame graph of the object tree.
//!
//! ## Allocative overhead
//!
//! When allocative is used, binary size is slightly increased due to implementations
//! of [`Allocative`] trait, but it has no runtime/memory overhead when it is not used.
//!
//! ## How it is different from other call-stack malloc profilers like jemalloc heap profiler
//!
//! Allocative is not a substitute for call stack malloc profiler,
//! it provides a different view on memory usage.
//!
//! Here are some differences between allocative and call-stack malloc profiler:
//!
//! * Explicit [`Allocative`] adapters describe object ownership and field traversal
//! * Allocative flamegraph shows object by object tree, not by call stack
//! * Allocative shows gaps in allocated memory,
//!   e.g. spare capacity of collections or too large padding in structs or enums
//! * Allocative allows profiling of memory for subset of the process data
//!   (for example, measure the size of RPC response before serialization)

#![feature(const_type_name, never_type)]
#![feature(layout_for_ptr)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(clippy::empty_enums)]

mod allocative_trait;
mod flamegraph;
mod global_root;
mod impls;
mod key;
mod visitor;

pub use allocative_derive::{Allocative, root};

pub use crate::{
  allocative_trait::{Allocative, Visit},
  flamegraph::{FlameGraph, FlameGraphBuilder},
  global_root::register_root,
  key::Key,
  visitor::Visitor,
};

#[doc(hidden)]
pub mod __macro_refs {
  pub use ctor;
}

/// Create a `const` of type `Key` with the provided `ident` as the value and
/// return that value. This allows the keys to be placed conveniently inline
/// without any performance hit because unlike calling `Key::new` this is
/// guaranteed to be evaluated at compile time.
///
/// The main use case is manual implementations of [`Allocative`], like so:
///
/// ```
/// use allocative::{Allocative, Visitor, ident_key};
///
/// struct MyStruct {
///   foo: usize,
///   bar: Vec<()>,
/// }
///
/// impl Allocative for MyStruct {
///   fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
///     let mut visitor = visitor.enter_self(self);
///     visitor.visit_field(ident_key!(foo), &self.foo);
///     visitor.visit_field(ident_key!(bar), &self.bar);
///     visitor.exit();
///   }
/// }
/// ```
#[macro_export]
macro_rules! ident_key {
  ($name:ident) => {{
    const KEY: $crate::Key = $crate::Key::new(stringify!($name));
    KEY
  }};
}
