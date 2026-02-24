use std::fmt::Debug;

use rspack_cacheable::cacheable;
use rspack_util::SpanExt;

/// Represents a range in a dependency, typically used for tracking the span of code in a source file.
/// It stores the start and end positions (as offsets) of the range, typically using base-0 indexing.
#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Default)]
pub struct DependencyRange {
  pub end: u32,
  pub start: u32,
}

impl From<(u32, u32)> for DependencyRange {
  fn from(range: (u32, u32)) -> Self {
    Self {
      start: range.0,
      end: range.1,
    }
  }
}

impl From<swc_core::common::Span> for DependencyRange {
  fn from(span: swc_core::common::Span) -> Self {
    Self {
      start: span.real_lo(),
      end: span.real_hi(),
    }
  }
}

impl DependencyRange {
  pub fn new(start: u32, end: u32) -> Self {
    DependencyRange { end, start }
  }
}
