use std::{fmt, sync::Arc};

use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct RealDependencyLocation {
  pub end: u32,
  pub start: u32,
  #[derivative(Debug = "ignore")]
  source: Option<Arc<dyn SourceLocation>>,
}

impl RealDependencyLocation {
  pub fn new(start: u32, end: u32) -> Self {
    RealDependencyLocation {
      end,
      start,
      source: None,
    }
  }

  pub fn with_source(mut self, source: Arc<dyn SourceLocation>) -> Self {
    self.source = Some(source);
    self
  }
}

impl From<swc_core::common::Span> for RealDependencyLocation {
  fn from(span: swc_core::common::Span) -> Self {
    Self {
      start: span.lo.0.saturating_sub(1),
      end: span.hi.0.saturating_sub(1),
      source: None,
    }
  }
}

impl fmt::Display for RealDependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let source = &self.source.clone().expect("missing sourcemap");
    let (start, end) = source.look_up_range_pos(self.start, self.end);

    if start.line == end.line {
      if start.column == end.column {
        return write!(f, "{}:{}", start.line, start.column);
      }

      return write!(f, "{}:{}-{}", start.line, start.column, end.column);
    }

    write!(
      f,
      "{}:{}-{}:{}",
      start.line, start.column, end.line, end.column
    )
  }
}

#[derive(Debug, Clone)]
pub struct SyntheticDependencyName {
  pub name: String,
}

impl SyntheticDependencyName {
  pub fn new(name: &str) -> Self {
    SyntheticDependencyName {
      name: name.to_string(),
    }
  }
}

impl fmt::Display for SyntheticDependencyName {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}

// #[derive(Debug, Clone)]
// pub enum DependencyLocation {
//   Real(RealDependencyLocation),
//   Synthetic(SyntheticDependencyName),
// }

#[derive(Debug, Clone, Copy)]
pub struct SourcePosition {
  pub line: usize,
  pub column: usize,
}

pub trait SourceLocation: Send + Sync {
  fn look_up_range_pos(&self, start: u32, end: u32) -> (SourcePosition, SourcePosition);
}

impl SourceLocation for swc_core::common::SourceMap {
  fn look_up_range_pos(&self, start: u32, end: u32) -> (SourcePosition, SourcePosition) {
    let lo = self.lookup_char_pos(swc_core::common::BytePos(start + 1));
    let hi = self.lookup_char_pos(swc_core::common::BytePos(end + 1));

    (
      SourcePosition {
        line: lo.line,
        column: lo.col_display,
      },
      SourcePosition {
        line: hi.line,
        column: hi.col_display,
      },
    )
  }
}
