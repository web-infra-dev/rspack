use std::{
  fmt::{self, Debug},
  sync::Arc,
};

use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, Clone, Hash)]
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
      start: span.lo.0.saturating_sub(1),
      end: span.hi.0.saturating_sub(1),
    }
  }
}

impl DependencyRange {
  pub fn new(start: u32, end: u32) -> Self {
    DependencyRange { end, start }
  }

  pub fn to_loc(&self, source: Option<&Arc<dyn SourceLocation>>) -> DependencyLocation {
    DependencyLocation::Real(match source {
      Some(source) => {
        let (start, end) = source.look_up_range_pos(self.start, self.end);

        if start.line == end.line && start.column == end.column {
          RealDependencyLocation::new(start, None)
        } else {
          RealDependencyLocation::new(start, Some(end))
        }
      }
      None => RealDependencyLocation::new(
        SourcePosition {
          line: self.start as usize,
          column: self.end as usize,
        },
        None,
      ),
    })
  }
}

#[derive(Debug, Clone)]
pub struct RealDependencyLocation {
  start: SourcePosition,
  end: Option<SourcePosition>,
}

impl RealDependencyLocation {
  pub fn new(start: SourcePosition, end: Option<SourcePosition>) -> Self {
    Self { start, end }
  }
}

impl fmt::Display for RealDependencyLocation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Some(end) = self.end {
      if self.start.line == end.line {
        write!(
          f,
          "{}:{}-{}",
          self.start.line, self.start.column, end.column
        )
      } else {
        write!(
          f,
          "{}:{}-{}:{}",
          self.start.line, self.start.column, end.line, end.column
        )
      }
    } else {
      write!(f, "{}:{}", self.start.line, self.start.column)
    }
  }
}

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
  line: usize,
  column: usize,
}

impl From<(u32, u32)> for SourcePosition {
  fn from(range: (u32, u32)) -> Self {
    Self {
      line: range.0 as usize,
      column: range.1 as usize,
    }
  }
}

pub trait SourceLocation: Send + Sync {
  fn look_up_range_pos(&self, start: u32, end: u32) -> (SourcePosition, SourcePosition);
}

impl Debug for dyn SourceLocation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SourceMap").finish()
  }
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

pub type SharedSourceMap = Arc<dyn SourceLocation>;
