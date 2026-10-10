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
