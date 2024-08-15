use std::sync::Arc;

use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct DependencyRange {
  pub end: u32,
  pub start: u32,
  #[derivative(Debug = "ignore")]
  source: Option<Arc<dyn SourceLocation>>,
}

impl DependencyRange {
  pub fn new(start: u32, end: u32) -> Self {
    DependencyRange {
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

impl From<swc_core::common::Span> for DependencyRange {
  fn from(span: swc_core::common::Span) -> Self {
    Self {
      start: span.lo.0.saturating_sub(1),
      end: span.hi.0.saturating_sub(1),
      source: None,
    }
  }
}

#[derive(Debug, Clone, Copy)]
pub struct SourcePosition {
  pub line: usize,
  pub column: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct DependencyLocation {
  pub end: SourcePosition,
  pub start: SourcePosition,
}

pub trait SourceLocation: Send + Sync {
  fn look_up_range_pos(&self, start: u32, end: u32) -> DependencyLocation;
}

impl SourceLocation for swc_core::common::SourceMap {
  fn look_up_range_pos(&self, start: u32, end: u32) -> DependencyLocation {
    let lo = self.lookup_char_pos(swc_core::common::BytePos(start + 1));
    let hi = self.lookup_char_pos(swc_core::common::BytePos(end + 1));

    DependencyLocation {
      start: SourcePosition {
        line: lo.line,
        column: lo.col_display,
      },
      end: SourcePosition {
        line: hi.line,
        column: hi.col_display,
      },
    }
  }
}

impl DependencyRange {
  pub fn to_loc(&self) -> Option<String> {
    if let Some(source) = &self.source {
      let loc = source.look_up_range_pos(self.start, self.end);

      if loc.start.line == loc.end.line {
        if loc.start.column == loc.end.column {
          return Some(format!("{}:{}", loc.start.line, loc.start.column));
        }

        return Some(format!(
          "{}:{}-{}",
          loc.start.line, loc.start.column, loc.end.column
        ));
      }

      return Some(format!(
        "{}:{}-{}:{}",
        loc.start.line, loc.start.column, loc.end.line, loc.end.column
      ));
    }

    None
  }
}
