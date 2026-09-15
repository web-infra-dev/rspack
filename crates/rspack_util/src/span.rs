pub trait SpanExt {
  fn real_lo(&self) -> u32;

  fn real_hi(&self) -> u32;
}

impl SpanExt for swc_core::common::Span {
  #[inline]
  fn real_lo(&self) -> u32 {
    self.lo().0.saturating_sub(1)
  }

  #[inline]
  fn real_hi(&self) -> u32 {
    self.hi().0.saturating_sub(1)
  }
}

impl SpanExt for swc_experimental_ecma_ast::Span {
  fn real_lo(&self) -> u32 {
    self.start.saturating_sub(1)
  }

  fn real_hi(&self) -> u32 {
    self.end.saturating_sub(1)
  }
}
