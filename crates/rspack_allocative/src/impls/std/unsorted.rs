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
  collections::VecDeque,
  convert::Infallible,
  ffi::OsStr,
  marker::PhantomData,
  mem,
  num::{
    NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroIsize, NonZeroU8, NonZeroU16, NonZeroU32,
    NonZeroU64, NonZeroUsize,
  },
  path::PathBuf,
};

use crate::{
  allocative_trait::Allocative,
  impls::common::{DATA_NAME, PTR_NAME, UNUSED_CAPACITY_NAME},
  key::Key,
  visitor::Visitor,
};

impl<T: Allocative + ?Sized> Allocative for &'static T {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let _ = visitor;
  }
}

impl Allocative for str {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple(Key::new("str"), self.len());
  }
}

impl Allocative for OsStr {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple(Key::new("OsStr"), self.len());
  }
}

impl<T: Allocative + ?Sized> Allocative for Box<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    {
      let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const T>());
      (**self).visit(&mut visitor);
    }
    visitor.exit();
  }
}

impl<T: Allocative, E: Allocative> Allocative for Result<T, E> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    match self {
      Ok(v) => visitor.visit_field(Key::new("Ok"), v),
      Err(e) => visitor.visit_field(Key::new("Err"), e),
    }
  }
}

impl<T: Allocative> Allocative for Option<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Some(value) = self {
      visitor.visit_field(Key::new("Some"), value);
    }
  }
}

impl<T: Allocative> Allocative for Vec<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if self.capacity() != 0 && mem::size_of::<T>() != 0 {
      let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const T>());
      visitor.visit_slice(self.as_slice());
      visitor.visit_simple(
        UNUSED_CAPACITY_NAME,
        (self.capacity() - self.len()) * mem::size_of::<T>(),
      );
    }
    visitor.exit();
  }
}

impl<T: Allocative> Allocative for VecDeque<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if self.capacity() != 0 && mem::size_of::<T>() != 0 {
      let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const T>());
      visitor.visit_slice(self.as_slices().0);
      visitor.visit_slice(self.as_slices().1);
      visitor.visit_simple(
        UNUSED_CAPACITY_NAME,
        (self.capacity() - self.len()) * mem::size_of::<T>(),
      );
    }
    visitor.exit();
  }
}

impl Allocative for String {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    {
      let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const u8>());
      visitor.visit_vec_like_body(self.as_bytes(), self.capacity());
      visitor.exit();
    }
    visitor.exit();
  }
}

impl Allocative for PathBuf {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if self.capacity() != 0 {
      let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const u8>());
      visitor.visit_simple(Key::new("path"), self.as_os_str().len());
      visitor.visit_simple(
        UNUSED_CAPACITY_NAME,
        self.capacity() - self.as_os_str().len(),
      );
    }
    visitor.exit();
  }
}

impl Allocative for NonZeroU64 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroU32 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroU16 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroU8 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroI64 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroI32 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroI16 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroI8 {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroUsize {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl Allocative for NonZeroIsize {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_simple_sized::<Self>();
  }
}

impl<T: ?Sized> Allocative for PhantomData<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let _ = visitor;
  }
}

impl Allocative for Infallible {
  fn visit<'a, 'b: 'a>(&self, _visitor: &'a mut Visitor<'b>) {
    match *self {}
  }
}

#[cfg(rust_nightly)]
impl Allocative for ! {
  fn visit<'a, 'b: 'a>(&self, _visitor: &'a mut Visitor<'b>) {
    match *self {}
  }
}

impl<T: Allocative> Allocative for [T] {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter(Key::for_type_name::<T>(), mem::size_of_val::<[T]>(self));
    for item in self {
      // TODO(nga): faster for primitive.
      visitor.visit_field(DATA_NAME, item);
    }
    visitor.exit();
  }
}

impl<T: Allocative, const N: usize> Allocative for [T; N] {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    self.as_slice().visit(visitor);
  }
}
