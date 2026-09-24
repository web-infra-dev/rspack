#![cfg(feature = "camino")]

use std::mem;

use crate::{allocative_trait::Allocative, impls::common::PTR_NAME, visitor::Visitor};

impl Allocative for camino::Utf8PathBuf {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    {
      let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const u8>());
      visitor.visit_vec_like_body(self.as_str().as_bytes(), self.capacity());
      visitor.exit();
    }
    visitor.exit()
  }
}

impl Allocative for camino::Utf8Path {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.enter_self(self).exit();
  }
}
