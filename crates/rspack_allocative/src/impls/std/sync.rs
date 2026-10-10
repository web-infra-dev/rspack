/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::{
  alloc::Layout,
  mem, rc,
  rc::Rc,
  sync::{
    Arc, LazyLock, Mutex, RwLock, Weak,
    atomic::{
      AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicPtr, AtomicU8,
      AtomicU16, AtomicU32, AtomicU64, AtomicUsize,
    },
  },
};

use crate::{
  Visit, allocative_trait::Allocative, impls::common::PTR_NAME, key::Key,
  reflection::try_visit_inner, visitor::Visitor,
};

impl<T: Visit + ?Sized> crate::reflection::VisitInner for Arc<T> {
  fn visit_inner<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let value: &T = self;
    value.visit_memory(visitor);
  }
}

impl<T: Visit + ?Sized> crate::reflection::VisitInner for Rc<T> {
  fn visit_inner<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let value: &T = self;
    value.visit_memory(visitor);
  }
}

impl<T: Visit, F: FnOnce() -> T> Allocative for LazyLock<T, F> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Some(value) = LazyLock::get(self) {
      visitor.visit_field(Key::new("data"), value);
    } else if std::mem::size_of::<F>() != 0
      && std::any::type_name::<F>() != std::any::type_name::<fn() -> T>()
    {
      // Do not run the initializer just to inspect its captures.
      visitor.report_opaque::<F>();
    }
    visitor.exit();
  }
}

impl<T: Visit + ?Sized> Allocative for RwLock<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Ok(data) = self.try_read() {
      visitor.visit_field(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}

#[allow(dead_code)] // Only used for its size
#[repr(C)] // as does std
struct RcBox<T: ?Sized> {
  a: usize,
  b: usize,
  t: T,
}

impl<T: ?Sized> RcBox<T> {
  fn layout(val: &T) -> Layout {
    Self::layout_for_value_layout(Layout::for_value(val))
  }

  fn layout_for_value_layout(val_layout: Layout) -> Layout {
    // See rcbox_layout_for_value_layout in std
    Layout::new::<RcBox<()>>()
      .extend(val_layout)
      .expect("a live shared allocation must have a valid layout")
      .0
      .pad_to_align()
  }
}

// The pinned std implementation returns the same empty sentinel for every T,
// including an empty Weak coerced to a DST. Compare addresses before using its
// metadata: an empty trait-object Weak can carry a vtable without an allocation.
fn visit_retained_weak<T: ?Sized>(
  ptr: *const T,
  empty: *const (),
  inner: Key,
  visitor: &mut Visitor<'_>,
) {
  if std::ptr::addr_eq(ptr, empty) {
    return;
  }
  if let Some(mut shared) = visitor.enter_shared(PTR_NAME, mem::size_of::<*const T>(), ptr.cast()) {
    // SAFETY: this nonempty Weak keeps the original allocation and its pointer
    // metadata alive. The original sized/slice/trait-object layout still fits
    // in isize, even though T has been dropped. This reads only metadata, never
    // creates &T, and never visits the freed allocations formerly owned by T.
    let layout = unsafe { Layout::for_value_raw(ptr) };
    let size = RcBox::<T>::layout_for_value_layout(layout).size();
    let mut allocation = shared.enter(inner, size);
    allocation.visit_simple(Key::new("retained"), size);
    allocation.exit();
    shared.exit();
  }
}

impl<T: 'static + ?Sized> Allocative for Arc<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    {
      let visitor = visitor.enter_shared(
        PTR_NAME,
        // Possibly fat pointer for size
        mem::size_of::<*const T>(),
        // Force thin pointer for identity, as fat pointers can have
        // different VTable addresses attached to the same memory
        Arc::as_ptr(self) as *const (),
      );
      if let Some(mut visitor) = visitor {
        {
          let val: &T = self;
          let mut visitor = visitor.enter(Key::new("ArcInner"), RcBox::layout(val).size());
          if !try_visit_inner(self, &mut visitor) {
            visitor.visit_opaque(val);
          }
          visitor.exit();
        }
        visitor.exit();
      }
    }
    visitor.exit();
  }
}

impl<T: 'static + ?Sized> Allocative for Weak<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    {
      if let Some(arc) = self.upgrade() {
        arc.visit(&mut visitor);
      } else {
        visit_retained_weak(
          self.as_ptr(),
          Weak::<()>::new().as_ptr().cast(),
          Key::new("ArcInner"),
          &mut visitor,
        );
      }
    }
    visitor.exit();
  }
}

impl<T: 'static + ?Sized> Allocative for Rc<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    {
      let visitor = visitor.enter_shared(
        PTR_NAME,
        // Possibly fat pointer for size
        mem::size_of::<*const T>(),
        // Force thin pointer for identity, as fat pointers can have
        // different VTable addresses attached to the same memory
        Rc::as_ptr(self) as *const (),
      );
      if let Some(mut visitor) = visitor {
        {
          let val: &T = self;
          let mut visitor = visitor.enter(Key::new("RcInner"), RcBox::layout(val).size());
          if !try_visit_inner(self, &mut visitor) {
            visitor.visit_opaque(val);
          }
          visitor.exit();
        }
        visitor.exit();
      }
    }
    visitor.exit();
  }
}

impl<T: 'static + ?Sized> Allocative for rc::Weak<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    {
      if let Some(rc) = self.upgrade() {
        rc.visit(&mut visitor);
      } else {
        visit_retained_weak(
          self.as_ptr(),
          rc::Weak::<()>::new().as_ptr().cast(),
          Key::new("RcInner"),
          &mut visitor,
        );
      }
    }
    visitor.exit();
  }
}

// An atomic pointer does not expose ownership of its target. Count its storage
// without loading the pointer or borrowing potentially invalid pointee memory.
impl<T> Allocative for AtomicPtr<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicU8 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicU16 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicU32 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicU64 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicUsize {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicI8 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicI16 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicI32 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicI64 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicBool {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl Allocative for AtomicIsize {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}

impl<T: Visit + ?Sized> Allocative for Mutex<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    if let Ok(data) = self.try_lock() {
      visitor.visit_field(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}
