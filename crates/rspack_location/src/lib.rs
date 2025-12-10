use std::fmt::{self, Debug};

pub use itoa::Buffer;
use ropey::Rope;
use rspack_cacheable::cacheable;
use swc_core::common::Span;

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

  pub fn from_span(span: &Span, source_rope: &Rope) -> Self {
    let start_char_offset = source_rope.byte_to_char(span.lo.0 as usize);
    let end_char_offset = source_rope.byte_to_char(span.hi.0 as usize);

    let start_line = source_rope.char_to_line(start_char_offset);
    let start_column = start_char_offset - source_rope.line_to_char(start_line);
    let end_line = source_rope.char_to_line(end_char_offset);
    let end_column = end_char_offset - source_rope.line_to_char(end_line);

    RealDependencyLocation::new(
      SourcePosition {
        line: start_line + 1,
        column: start_column,
      },
      Some(SourcePosition {
        line: end_line + 1,
        column: end_column,
      }),
    )
  }
}

impl fmt::Display for RealDependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(end) = self.end {
      let mut start_line_buffer = itoa::Buffer::new();
      let start_line = start_line_buffer.format(self.start.line);
      let mut start_col_buffer = itoa::Buffer::new();
      let start_col = start_col_buffer.format(self.start.column);
      if self.start.line == end.line && self.start.column == end.column {
        write!(f, "{}:{}", start_line, start_col)
      } else if self.start.line == end.line {
        let mut end_col_buffer = itoa::Buffer::new();
        let end_col = end_col_buffer.format(end.column);
        write!(f, "{}:{}-{}", start_line, start_col, end_col)
      } else {
        let mut end_line_buffer = itoa::Buffer::new();
        let end_line = end_line_buffer.format(end.line);
        let mut end_col_buffer = itoa::Buffer::new();
        let end_col = end_col_buffer.format(end.column);
        write!(f, "{}:{}-{}:{}", start_line, start_col, end_line, end_col)
      }
    } else {
      let mut start_line_buffer = itoa::Buffer::new();
      let start_line = start_line_buffer.format(self.start.line);
      let mut start_col_buffer = itoa::Buffer::new();
      let start_col = start_col_buffer.format(self.start.column);
      write!(f, "{}:{}", start_line, start_col)
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
  pub fn from_span(span: &Span, source_rope: &Rope) -> Self {
    DependencyLocation::Real(RealDependencyLocation::from_span(span, source_rope))
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
