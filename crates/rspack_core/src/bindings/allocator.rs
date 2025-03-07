use crate::{bindings::Root, Compilation};

pub trait Allocator {
  fn allocate_compilation(&self, val: Compilation) -> Root<Compilation>;
}
