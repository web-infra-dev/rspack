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
      AtomicBool, AtomicI8, AtomicI16, AtomicI32, AtomicI64, AtomicIsize, AtomicU8, AtomicU16,
      AtomicU32, AtomicU64, AtomicUsize,
    },
  },
};

use crate::{allocative_trait::Allocative, impls::common::PTR_NAME, key::Key, visitor::Visitor};

impl<T: Allocative, F: FnOnce() -> T> Allocative for LazyLock<T, F> {
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

impl<T: Allocative + ?Sized> Allocative for RwLock<T> {
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
    let val_layout = Layout::for_value(val);
    // See rcbox_layout_for_value_layout in std
    Layout::new::<RcBox<()>>()
      .extend(val_layout)
      .expect("a live shared allocation must have a valid layout")
      .0
      .pad_to_align()
  }
}

impl<T: Allocative + ?Sized> Allocative for Arc<T> {
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
          val.visit(&mut visitor);
          visitor.exit();
        }
        visitor.exit();
      }
    }
    visitor.exit();
  }
}

impl<T: Allocative + ?Sized> Allocative for Weak<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    {
      if let Some(arc) = self.upgrade() {
        arc.visit(&mut visitor);
      }
    }
    visitor.exit();
  }
}

impl<T: Allocative> Allocative for Rc<T> {
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
          val.visit(&mut visitor);
          visitor.exit();
        }
        visitor.exit();
      }
    }
    visitor.exit();
  }
}

impl<T: Allocative> Allocative for rc::Weak<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    {
      if let Some(rc) = self.upgrade() {
        rc.visit(&mut visitor);
      }
    }
    visitor.exit();
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

impl<T: Allocative + ?Sized> Allocative for Mutex<T> {
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
