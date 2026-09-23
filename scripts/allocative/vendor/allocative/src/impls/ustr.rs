#![cfg(feature = "ustr")]


use crate::Allocative;
use crate::Visitor;

impl Allocative for ustr::Ustr {
    fn visit<'a, 'b: 'a>(&self, _visitor: &'a mut Visitor<'b>) {
        // ignore global cache
    }
}
