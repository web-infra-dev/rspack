#![cfg(feature = "camino")]

use std::mem;
use crate::allocative_trait::Allocative;
use crate::visitor::Visitor;
use crate::impls::common::PTR_NAME;

impl Allocative for camino::Utf8PathBuf {
    fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
        let mut visitor = visitor.enter_self_sized::<Self>();
        {
            let mut visitor = visitor.enter_unique(PTR_NAME, mem::size_of::<*const u8>());
            // FIXME use .capacity() ?
            visitor.visit_vec_like_body(self.as_str().as_bytes(), self.as_str().len());
            visitor.exit();
        }
        visitor.exit()
    }
}
