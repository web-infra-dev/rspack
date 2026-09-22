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
//! An object implementing [`Allocative`] trait is introspectable, and this crate
//! provides two utilities to work with such objects:
//! * [`FlameGraphBuilder`] to build a flame graph of object tree
//! * [`size_of_unique_allocated_data`] provides estimation
//!   of how much allocated memory the value holds
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
//! * Allocative requires implementation of [`Allocative`] trait for each type
//!   which needs to be measured, and some setup in the program to enable it
//! * Allocative flamegraph shows object by object tree, not by call stack
//! * Allocative shows gaps in allocated memory,
//!   e.g. spare capacity of collections or too large padding in structs or enums
//! * Allocative allows profiling non-malloc allocations (for example, allocations within [bumpalo])
//! * Allocative allows profiling of memory for subset of the process data
//!   (for example, measure the size of RPC response before serialization)
//!
//! [bumpalo]: https://github.com/fitzgen/bumpalo

#![cfg_attr(rust_nightly, feature(const_type_name))]
#![cfg_attr(rust_nightly, feature(never_type))]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(clippy::empty_enums)]

mod allocative_trait;
mod flamegraph;
mod global_root;
pub(crate) mod golden;
mod impls;
mod key;
mod rc_str;
mod size_of;
mod test_derive;
mod visitor;

pub use allocative_derive::{Allocative, root};

pub use crate::{
  allocative_trait::Allocative,
  flamegraph::{FlameGraph, FlameGraphBuilder},
  global_root::register_root,
  key::Key,
  size_of::{size_of_unique, size_of_unique_allocated_data},
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

#[test]
fn ident_key() {
  assert_eq!(ident_key!(foo), Key::new("foo"));
}

/// Explicit adapter for external state whose owned fields cannot be inspected.
pub fn visit_opaque<T: ?Sized>(value: &T, visitor: &mut Visitor<'_>) {
  visitor.visit_opaque(value);
}

/// Count a shared opaque value and its Arc allocation, but report inaccessible captures.
pub fn visit_opaque_arc<T: ?Sized>(value: &std::sync::Arc<T>, visitor: &mut Visitor<'_>) {
  use std::{alloc::Layout, mem::size_of, sync::Arc};
  let mut visitor = visitor.enter_self(value);
  if let Some(mut pointer) = visitor.enter_shared(
    Key::new("ptr"),
    size_of::<*const T>(),
    Arc::as_ptr(value).cast(),
  ) {
    let layout = Layout::new::<[usize; 2]>()
      .extend(Layout::for_value(&**value))
      .expect("a live Arc allocation must have a valid layout")
      .0
      .pad_to_align();
    let mut allocation = pointer.enter(Key::new("ArcInner"), layout.size());
    allocation.visit_opaque(&**value);
    allocation.exit();
    pointer.exit();
  }
  visitor.exit();
}

/// Count an opaque boxed value without pretending to traverse its nested allocations.
// The field adapter needs the Box layout as well as its pointee.
#[allow(clippy::borrowed_box)]
pub fn visit_opaque_box<T: ?Sized>(value: &Box<T>, visitor: &mut Visitor<'_>) {
  let mut visitor = visitor.enter_self(value);
  let mut pointer = visitor.enter_unique(Key::new("ptr"), std::mem::size_of::<*const T>());
  pointer.visit_opaque(&**value);
  pointer.exit();
  visitor.exit();
}

/// For audited leaf values, such as function pointers, which own no heap allocation.
pub fn visit_inline<T: ?Sized>(value: &T, visitor: &mut Visitor<'_>) {
  visitor.enter_self(value).exit();
}

/// Preserve the option's inline storage and deduplicate an opaque shared callback.
pub fn visit_opaque_option_arc<T: ?Sized>(
  value: &Option<std::sync::Arc<T>>,
  visitor: &mut Visitor<'_>,
) {
  let mut visitor = visitor.enter_self(value);
  if let Some(value) = value {
    visit_opaque_arc(value, &mut visitor);
  }
  visitor.exit();
}

/// Traverse an optional boxed array of optional shared opaque callbacks.
pub fn visit_opaque_option_arc_slice<T: ?Sized>(
  value: &Option<Box<[Option<std::sync::Arc<T>>]>>,
  visitor: &mut Visitor<'_>,
) {
  let mut visitor = visitor.enter_self(value);
  if let Some(values) = value {
    let mut pointer = visitor.enter_unique(Key::new("ptr"), std::mem::size_of_val(values));
    let mut data = pointer.enter(Key::new("callbacks"), std::mem::size_of_val(&**values));
    for value in values.iter() {
      visit_opaque_option_arc(value, &mut data);
    }
    data.exit();
    pointer.exit();
  }
  visitor.exit();
}

/// Count an optional opaque callback, including its owned box allocation.
pub fn visit_opaque_option_box<T: ?Sized>(value: &Option<Box<T>>, visitor: &mut Visitor<'_>) {
  let mut visitor = visitor.enter_self(value);
  if let Some(value) = value {
    visit_opaque_box(value, &mut visitor);
  }
  visitor.exit();
}
