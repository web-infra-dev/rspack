use std::fmt::{self, Debug};

pub use itoa::Buffer;
use rspack_cacheable::cacheable;

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

impl fmt::Display for DependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let loc = match self {
      DependencyLocation::Real(real) => real.to_string(),
      DependencyLocation::Synthetic(synthetic) => synthetic.to_string(),
    };
    write!(f, "{loc}")
  }
}
