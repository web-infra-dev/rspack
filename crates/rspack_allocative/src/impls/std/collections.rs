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
  collections::{BTreeMap, BTreeSet, HashMap, HashSet},
  mem::size_of,
};

use crate::{Key, allocative_trait::Allocative, visitor::Visitor};

impl<K: Allocative, V: Allocative> Allocative for BTreeMap<K, V> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_generic_map_fields(self);
    visitor.exit();
  }
}

impl<K: Allocative> Allocative for BTreeSet<K> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    visitor.visit_generic_set_fields(self);
    visitor.exit();
  }
}

impl<K: Allocative, V: Allocative, S> Allocative for HashMap<K, V, S> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    let mut allocation = visitor.enter_unique(Key::new("ptr"), size_of::<*const ()>());
    let mut data = allocation.enter(Key::new("capacity"), self.capacity() * size_of::<(K, V)>());
    for (key, value) in self {
      data.visit_field(Key::new("key"), key);
      data.visit_field(Key::new("value"), value);
    }
    data.visit_simple(
      Key::new("unused_capacity"),
      (self.capacity() - self.len()) * size_of::<(K, V)>(),
    );
    data.exit();
    allocation.exit();
    visitor.exit();
  }
}

impl<K: Allocative, S> Allocative for HashSet<K, S> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    let mut allocation = visitor.enter_unique(Key::new("ptr"), size_of::<*const ()>());
    let mut data = allocation.enter(Key::new("capacity"), self.capacity() * size_of::<K>());
    for key in self {
      data.visit_field(Key::new("key"), key);
    }
    data.visit_simple(
      Key::new("unused_capacity"),
      (self.capacity() - self.len()) * size_of::<K>(),
    );
    data.exit();
    allocation.exit();
    visitor.exit();
  }
}
