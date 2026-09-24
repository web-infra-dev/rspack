//! Count Rust TSFN handles; Node/V8 callback state remains an explicit boundary.
use napi::{
  Status,
  bindgen_prelude::{FromNapiValue, JsValuesTupleIntoVec},
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionHandle},
};

use crate::{Allocative, Key, Visitor};
impl Allocative for ThreadsafeFunctionHandle {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    visitor.visit_opaque(self);
  }
}
impl<
  T: 'static,
  R: 'static + FromNapiValue,
  A: 'static + JsValuesTupleIntoVec,
  E: AsRef<str> + From<Status>,
  const C: bool,
  const W: bool,
  const Q: usize,
> Allocative for ThreadsafeFunction<T, R, A, E, C, W, Q>
{
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(Key::new("handle"), &self.handle);
    visitor.exit();
  }
}
