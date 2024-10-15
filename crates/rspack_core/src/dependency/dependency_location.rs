use std::{fmt, sync::Arc};

use derivative::Derivative;
use rspack_cacheable::{cacheable, with::Skip};

#[cacheable]
#[derive(Derivative)]
#[derivative(Debug, Clone, Hash)]
pub struct RealDependencyLocation {
  pub end: u32,
  pub start: u32,
  #[cacheable(with=Skip)]
  #[derivative(Debug = "ignore", Hash = "ignore")]
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

impl From<(u32, u32)> for RealDependencyLocation {
  fn from(range: (u32, u32)) -> Self {
    Self {
      start: range.0,
      end: range.1,
      source: None,
    }
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
    if let Some(source) = &self.source {
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
    } else {
      write!(f, "{}:{}", self.start, self.end)
    }
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct SyntheticDependencyLocation {
  pub name: String,
}

impl SyntheticDependencyLocation {
  pub fn new(name: &str) -> Self {
    SyntheticDependencyLocation {
      name: name.to_string(),
    }
  }
}

impl fmt::Display for SyntheticDependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.name)
  }
}

#[cacheable]
#[derive(Debug, Clone)]
pub enum DependencyLocation {
  Real(RealDependencyLocation),
  Synthetic(SyntheticDependencyLocation),
}

impl fmt::Display for DependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let loc = match self {
      DependencyLocation::Real(real) => real.to_string(),
      DependencyLocation::Synthetic(synthetic) => synthetic.to_string(),
    };
    write!(f, "{loc}")
  }
}

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
