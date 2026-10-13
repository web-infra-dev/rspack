#[repr(i16)]
pub enum Conditional {
  #[cfg(any())]
  Removed(String) = -10,
  Empty = -9,
  Text(String) = 42,
  Named {
    bytes: Vec<u8>,
  } = 99,
}

macro_rules! generated_enum {
  () => {
    #[derive(allocative::Allocative)]
    pub enum Generated {
      Text(String),
    }
  };
}

generated_enum!();

pub const INLINE_CAPACITY: usize = 4;

pub enum Value {
  Text(String),
}

pub enum Storage<T> {
  Inline(smallvec::SmallVec<[T; INLINE_CAPACITY]>),
  Heap(std::collections::HashSet<T>),
}

pub enum ConstArray {
  Data([String; INLINE_CAPACITY]),
  Nested(Vec<[String; INLINE_CAPACITY + 1]>),
}
