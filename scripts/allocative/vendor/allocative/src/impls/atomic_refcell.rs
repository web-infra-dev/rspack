#![cfg(feature = "atomic_refcell")]

use crate::allocative_trait::Allocative;
use crate::key::Key;
use crate::visitor::Visitor;

impl<T: Allocative> Allocative for atomic_refcell::AtomicRefCell<T> {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
        let mut visitor = visitor.enter_self_sized::<Self>();
        if let Ok(data) = self.try_borrow() {
            visitor.visit_field(Key::new("data"), &*data);
        }
        visitor.exit();
    }
}
