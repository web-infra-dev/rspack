/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use crate::{
  Allocative, Key, Visitor,
  visitor::{NodeKind, VisitorImpl},
};

/// Size of data allocated in unique pointers in the struct.
///
/// * Exclude self
/// * Exclude shared pointers
/// * For unique pointers, include the size of the pointee plus this function recursively
///
/// # Example
///
/// ```
/// use allocative::Allocative;
///
/// #[derive(Allocative)]
/// struct Foo {
///   data: Vec<u8>,
/// }
///
/// assert_eq!(
///   3,
///   allocative::size_of_unique_allocated_data(&Foo {
///     data: vec![10, 20, 30]
///   })
/// );
/// ```
pub fn size_of_unique_allocated_data(root: &dyn Allocative) -> usize {
  struct SizeOfUniqueAllocatedDataVisitor {
    /// Size we return.
    size: usize,
  }

  impl VisitorImpl for SizeOfUniqueAllocatedDataVisitor {
    fn enter_inline_impl(&mut self, _name: Key, size: usize, parent: NodeKind) {
      if let NodeKind::Unique = parent {
        self.size += size;
      }
    }

    fn enter_unique_impl(&mut self, _name: Key, _size: usize, _parent: NodeKind) {}

    fn enter_shared_impl(
      &mut self,
      _name: Key,
      _size: usize,
      _ptr: *const (),
      _parent: NodeKind,
    ) -> bool {
      false
    }

    fn exit_inline_impl(&mut self) {}

    fn exit_unique_impl(&mut self) {}

    fn exit_shared_impl(&mut self) {
      unreachable!("shared pointers are not visited")
    }

    fn exit_root_impl(&mut self) {}
  }

  let mut visitor_impl = SizeOfUniqueAllocatedDataVisitor { size: 0 };
  let mut visitor = Visitor {
    visitor: &mut visitor_impl,
    node_kind: NodeKind::Root,
  };
  root.visit(&mut visitor);
  visitor.exit();
  visitor_impl.size
}

/// Size of a piece of data and data allocated in unique pointers in the struct.
///
/// * Excludes shared pointers
///
/// # Example
///
/// ```
/// use allocative::Allocative;
///
/// #[derive(Allocative)]
/// struct Foo {
///   data: Vec<u8>,
/// }
///
/// assert_eq!(
///   3 + std::mem::size_of::<Vec<u8>>(),
///   allocative::size_of_unique(&Foo {
///     data: vec![10, 20, 30]
///   })
/// );
/// ```
pub fn size_of_unique<T>(root: &T) -> usize
where
  T: Allocative,
{
  std::mem::size_of::<T>() + size_of_unique_allocated_data(root)
}
