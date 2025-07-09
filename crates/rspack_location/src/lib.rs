use std::fmt::{self, Debug};

pub use itoa::Buffer;
use rspack_cacheable::cacheable;
use swc_core::common::{SourceMap, Span};

#[macro_export]
macro_rules! itoa {
  ($i:expr) => {{
    itoa::Buffer::new().format($i)
  }};
}

/// Represents a position in the source file, including the line number and column number.
#[cacheable]
#[derive(Debug, Clone, Copy)]
pub struct SourcePosition {
  pub line: usize,
  pub column: usize,
}

impl From<(u32, u32)> for SourcePosition {
  fn from(range: (u32, u32)) -> Self {
    Self {
      line: range.0 as usize,
      column: range.1 as usize,
    }
  }
}

/// Represents the real location of a dependency in a source file, including both start and optional end positions.
/// These positions are described in terms of lines and columns in the source code.
#[cacheable]
#[derive(Debug, Clone)]
pub struct RealDependencyLocation {
  pub start: SourcePosition,
  pub end: Option<SourcePosition>,
}

impl RealDependencyLocation {
  pub fn new(start: SourcePosition, end: Option<SourcePosition>) -> Self {
    Self { start, end }
  }

  pub fn from_span(span: &Span, source_map: &SourceMap) -> Self {
    let start_char_pos = source_map.lookup_char_pos(span.lo);
    let end_char_pos = source_map.lookup_char_pos(span.hi);
    RealDependencyLocation::new(
      SourcePosition {
        line: start_char_pos.line,
        column: start_char_pos.col_display,
      },
      Some(SourcePosition {
        line: end_char_pos.line,
        column: end_char_pos.col_display,
      }),
    )
  }
}

impl fmt::Display for RealDependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(end) = self.end {
      if self.start.line == end.line && self.start.column == end.column {
        write!(f, "{}:{}", itoa!(self.start.line), itoa!(self.start.column))
      } else if self.start.line == end.line {
        write!(
          f,
          "{}:{}-{}",
          itoa!(self.start.line),
          itoa!(self.start.column),
          itoa!(end.column)
        )
      } else {
        write!(
          f,
          "{}:{}-{}:{}",
          itoa!(self.start.line),
          itoa!(self.start.column),
          itoa!(end.line),
          itoa!(end.column)
        )
      }
    } else {
      write!(f, "{}:{}", itoa!(self.start.line), itoa!(self.start.column))
    }
  }
}

/// Represents a synthetic dependency location, such as a generated dependency.
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

impl DependencyLocation {
  pub fn from_span(span: &Span, source_map: &SourceMap) -> Self {
    DependencyLocation::Real(RealDependencyLocation::from_span(span, source_map))
  }
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
