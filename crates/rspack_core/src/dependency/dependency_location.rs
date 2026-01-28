use std::fmt::{self, Debug};

use rspack_cacheable::cacheable;
use rspack_location::{DependencyLocation, RealDependencyLocation, SourcePosition};
use rspack_util::SpanExt;

/// Represents a range in a dependency, typically used for tracking the span of code in a source file.
/// It stores the start and end positions (as offsets) of the range, typically using base-0 indexing.
#[cacheable]
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
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

  /// Converts the `DependencyRange` into a `DependencyLocation`.
  /// The `source` parameter is an optional source map used to resolve the exact position in the source file.
  pub fn to_loc<T: AsLoc>(&self, source: Option<T>) -> Option<DependencyLocation> {
    source
      .and_then(|s| s.as_loc().look_up_range_pos(self.start, self.end))
      .map(|(start, end)| {
        DependencyLocation::Real(if start.line == end.line && start.column == end.column {
          RealDependencyLocation::new(start, None)
        } else {
          RealDependencyLocation::new(start, Some(end))
        })
      })
  }
}

/// Trait representing a source map that can resolve the positions of code ranges to source file positions.
pub trait SourceLocation: Send + Sync {
  fn look_up_range_pos(&self, start: u32, end: u32) -> Option<(SourcePosition, SourcePosition)>;
}

impl Debug for dyn SourceLocation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("SourceMap").finish()
  }
}

impl SourceLocation for &str {
  fn look_up_range_pos(&self, start: u32, end: u32) -> Option<(SourcePosition, SourcePosition)> {
    let s = *self;
    let len = s.len();
    let start_idx = start as usize;
    let end_idx = end as usize;

    if start_idx > len || end_idx > len {
      return None;
    }

    // Fast paths
    if end_idx == 0 {
      let p = SourcePosition { line: 1, column: 1 };
      return Some((p, p));
    }

    let bytes = s.as_bytes();

    // Single pass over [..end_idx], tracking:
    // - total lines up to end
    // - last newline before start
    // - last newline before end
    let mut line_end = 1usize;
    let mut last_nl_before_start: Option<usize> = None;
    let mut last_nl_before_end: Option<usize> = None;

    for idx in memchr::memchr_iter(b'\n', &bytes[..end_idx]) {
      line_end += 1;
      // update last newline before end
      last_nl_before_end = Some(idx);
      // update last newline before start only if newline is strictly before start_idx
      if idx < start_idx {
        last_nl_before_start = Some(idx);
      }
    }

    // Derive line for start by counting newlines before start:
    // It's line_end minus the number of newlines after start within [..end_idx]
    // Simpler: count lines up to start in the same loop using last_nl_before_start.
    let line_start = if start_idx == 0 {
      1
    } else {
      // Count lines up to start: number of '\n' before start + 1
      let mut count_lines_before_start = 1usize;
      // If we need exact count, we can recompute cheaply with memchr on [..start_idx].
      // But to ensure single pass, use the information we tracked:
      // We don't have the count, so do a memchr over the shorter slice only when start < end.
      // This is still <= one full scan of the string in worst case.
      if start_idx > 0 {
        count_lines_before_start = 1 + memchr::memchr_iter(b'\n', &bytes[..start_idx]).count();
      }
      count_lines_before_start
    };

    // Compute line start byte offsets
    let start_line_start_byte = last_nl_before_start.map_or(0, |p| p + 1);
    let end_line_start_byte = last_nl_before_end.map_or(0, |p| p + 1);

    // UTF-16 columns are 1-based
    let start_seg = &s[start_line_start_byte..start_idx];
    let end_seg = &s[end_line_start_byte..end_idx];

    let start_col_utf16 = start_seg.encode_utf16().count() + 1;
    let end_col_utf16 = end_seg.encode_utf16().count() + 1;

    let start_pos = SourcePosition {
      line: line_start,
      column: start_col_utf16,
    };
    let end_pos = SourcePosition {
      line: line_end,
      column: end_col_utf16,
    };

    Some((start_pos, end_pos))
  }
}

pub trait AsLoc {
  fn as_loc(&self) -> &dyn SourceLocation;
}

impl AsLoc for &str {
  #[inline]
  fn as_loc(&self) -> &dyn SourceLocation {
    let loc: &dyn SourceLocation = self;
    loc
  }
}
