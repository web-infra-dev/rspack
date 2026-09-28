#![cfg(feature = "atomic_refcell")]

use crate::{allocative_trait::Allocative, key::Key, visitor::Visitor};

impl<T: Allocative> Allocative for atomic_refcell::AtomicRefCell<T> {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self_sized::<Self>();
    if let Ok(data) = self.try_borrow() {
      visitor.visit_field(Key::new("data"), &*data);
    } else {
      visitor.report_opaque::<Self>();
    }
    visitor.exit();
  }
}
