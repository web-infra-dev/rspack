/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is dual-licensed under either the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree or the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree. You may select, at your option, one of the
 * above-listed licenses.
 */

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use crate::{allocative_trait::Allocative, visitor::Visitor};

impl<K: Allocative, V: Allocative> Allocative for BTreeMap<K, V> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    let bytes = self.len() * std::mem::size_of::<(K, V)>();
    let mut storage = visitor.enter_unique(crate::Key::new("ptr"), std::mem::size_of::<usize>());
    let mut entries = storage.enter(crate::Key::new("entries_lower_bound"), bytes);
    for (k, v) in self {
      entries.visit_field(crate::Key::new("key"), k);
      entries.visit_field(crate::Key::new("value"), v);
    }
    entries.exit();
    storage.exit();
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
    if self.capacity() != 0 {
      let buckets = (self.capacity() + 1).next_power_of_two();
      let pair_size = std::mem::size_of::<(K, V)>();
      let mut storage = visitor.enter_unique(crate::Key::new("ptr"), std::mem::size_of::<usize>());
      let mut table = storage.enter(crate::Key::new("table"), buckets * pair_size + buckets + 16);
      for (k, v) in self {
        table.visit_field(crate::Key::new("key"), k);
        table.visit_field(crate::Key::new("value"), v);
      }
      table.visit_simple(
        crate::Key::new("unused_capacity"),
        (buckets - self.len()) * pair_size,
      );
      table.exit();
      storage.exit();
    }
    visitor.exit();
  }
}
impl<K: Allocative, S> Allocative for HashSet<K, S> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if self.capacity() != 0 {
      let buckets = (self.capacity() + 1).next_power_of_two();
      let key_size = std::mem::size_of::<K>();
      let mut storage = visitor.enter_unique(crate::Key::new("ptr"), std::mem::size_of::<usize>());
      let mut table = storage.enter(crate::Key::new("table"), buckets * key_size + buckets + 16);
      for k in self {
        table.visit_field(crate::Key::new("key"), k);
      }
      table.visit_simple(
        crate::Key::new("unused_capacity"),
        (buckets - self.len()) * key_size,
      );
      table.exit();
      storage.exit();
    }
    visitor.exit();
  }
}
