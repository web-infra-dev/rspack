use swc_core::common::Span;

pub trait SpanExt {
  fn real_lo(&self) -> u32;

  fn real_hi(&self) -> u32;
}

impl SpanExt for Span {
  #[inline]
  fn real_lo(&self) -> u32 {
    self.lo().0 - 1
  }

  #[inline]
  fn real_hi(&self) -> u32 {
    self.hi().0 - 1
  }
}
