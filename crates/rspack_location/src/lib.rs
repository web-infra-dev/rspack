use std::fmt::{self, Debug};

pub use itoa::Buffer;
use rspack_cacheable::cacheable;

/// Represents a position within a source file (line and column).
/// Semantics match V8 Error stack positions:
/// - Both line and column are 1-based.
/// - Column counts UTF-16 code units (not Unicode scalar values or UTF-8 bytes).
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

  /// Convert byte line, column, length to js style location
  /// - line is 1-based in bytes
  /// - column is 0-based in bytes
  /// - length in bytes
  pub fn from_byte_location(
    source: &str,
    line: usize,
    column: usize,
    length: Option<usize>,
  ) -> Option<Self> {
    if line == 0 {
      return None;
    }

    let bytes = source.as_bytes();
    let target_line_idx = line - 1;

    // 1. Quickly locate the byte index of the line start.
    // If it's the first line, the offset is 0; otherwise, search for the (line-1)th newline.
    let line_start_offset = if target_line_idx == 0 {
      0
    } else {
      let mut iter = memchr::memchr_iter(b'\n', bytes);
      match iter.nth(target_line_idx - 1) {
        Some(idx) => idx + 1,
        None => return None, // Line number exceeds file lines
      }
    };

    // 2. Validate start position.
    let start_byte = line_start_offset + column;
    if start_byte > bytes.len() {
      return None;
    }

    // Ensure start_byte doesn't cross into the next line (find the next newline of the current line).
    let current_line_end = memchr::memchr(b'\n', &bytes[line_start_offset..])
      .map(|rel| line_start_offset + rel)
      .unwrap_or(bytes.len());
    if start_byte > current_line_end {
      return None;
    }

    // 3. Calculate the UTF-16 length of the start column.
    // This avoids the overhead of constructing the surrogate pair iterator and only performs numerical accumulation.
    let start_line_slice = source.get(line_start_offset..start_byte)?;
    let start_utf16_col = start_line_slice.encode_utf16().count() + 1; // 1-based

    let start = SourcePosition {
      line,
      column: start_utf16_col,
    };

    // 4. Calculate end position (if length is present).
    let end = if let Some(len) = length {
      let end_byte = start_byte + len;

      if end_byte > bytes.len() {
        // If it exceeds the file range, keep start and discard end (matching original logic).
        return Some(Self { start, end: None });
      }

      let Some(span_slice) = source.get(start_byte..end_byte) else {
        return Some(Self { start, end: None });
      };
      let newlines_in_span = memchr::memchr_iter(b'\n', span_slice.as_bytes()).count();

      let end_line = line + newlines_in_span;

      let end_column = if newlines_in_span == 0 {
        start_utf16_col + span_slice.encode_utf16().count()
      } else {
        #[allow(clippy::unwrap_used)]
        let last_newline_pos = span_slice.rfind('\n').unwrap();
        let text_after_last_newline = &span_slice[last_newline_pos + 1..];
        text_after_last_newline.encode_utf16().count() + 1 // 1-based
      };

      Some(SourcePosition {
        line: end_line,
        column: end_column,
      })
    } else {
      None
    };

    Some(Self { start, end })
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
  pub fn from_byte_location(
    source: &str,
    line: usize,
    column: usize,
    length: Option<usize>,
  ) -> Option<Self> {
    RealDependencyLocation::from_byte_location(source, line, column, length)
      .map(DependencyLocation::Real)
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_from_byte_location_ascii() {
    let source = "hello world\nfoo bar baz";

    // Test basic ASCII string, first line
    let loc = RealDependencyLocation::from_byte_location(source, 1, 6, Some(5));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 7); // "world" starts at byte 6, UTF-16 column 7 (1-based)
    assert_eq!(loc.end.as_ref().unwrap().line, 1);
    assert_eq!(loc.end.as_ref().unwrap().column, 12); // 7 + 5 = 12
  }

  #[test]
  fn test_from_byte_location_second_line() {
    let source = "hello world\nfoo bar baz";

    // Test second line
    let loc = RealDependencyLocation::from_byte_location(source, 2, 4, Some(3));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 2);
    assert_eq!(loc.start.column, 5); // "bar" starts at byte 4, UTF-16 column 5 (1-based)
    assert_eq!(loc.end.as_ref().unwrap().line, 2);
    assert_eq!(loc.end.as_ref().unwrap().column, 8); // 5 + 3 = 8
  }

  #[test]
  fn test_from_byte_location_utf8_multibyte() {
    // Test with multi-byte UTF-8 characters
    // "你好" = 2 chars, 6 bytes, 2 UTF-16 code units
    // "世界" = 2 chars, 6 bytes, 2 UTF-16 code units
    let source = "你好世界abc";

    // Start at byte 0, length 6 (first two characters "你好")
    let loc = RealDependencyLocation::from_byte_location(source, 1, 0, Some(6));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 1); // 1-based
    assert_eq!(loc.end.as_ref().unwrap().line, 1);
    assert_eq!(loc.end.as_ref().unwrap().column, 3); // 1 + 2 UTF-16 units = 3
  }

  #[test]
  fn test_from_byte_location_utf8_emoji() {
    // Test with emoji (4-byte UTF-8, 2 UTF-16 code units)
    // "😀" = 1 grapheme, 4 bytes, 2 UTF-16 code units
    let source = "hello😀world";

    // Start at "😀", byte offset 5, length 4
    let loc = RealDependencyLocation::from_byte_location(source, 1, 5, Some(4));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 6); // "hello" = 5 UTF-16 units, so emoji starts at 6 (1-based)
    assert_eq!(loc.end.as_ref().unwrap().line, 1);
    assert_eq!(loc.end.as_ref().unwrap().column, 8); // 6 + 2 UTF-16 units = 8
  }

  #[test]
  fn test_from_byte_location_no_length() {
    let source = "hello world";

    // Test without length (end is None)
    let loc = RealDependencyLocation::from_byte_location(source, 1, 0, None);
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 1);
    assert!(loc.end.is_none());
  }

  #[test]
  fn test_from_byte_location_invalid_line() {
    let source = "hello world";

    // Test with line 0 (invalid)
    let loc = RealDependencyLocation::from_byte_location(source, 0, 0, None);
    assert!(loc.is_none());
  }

  #[test]
  fn test_from_byte_location_line_out_of_bounds() {
    let source = "hello world\nfoo bar";

    // Test with line number that doesn't exist
    let loc = RealDependencyLocation::from_byte_location(source, 10, 0, None);
    assert!(loc.is_none());
  }

  #[test]
  fn test_from_byte_location_column_out_of_bounds() {
    let source = "hello";

    // Test with column beyond line length
    let loc = RealDependencyLocation::from_byte_location(source, 1, 100, None);
    assert!(loc.is_none());
  }

  #[test]
  fn test_from_byte_location_empty_line() {
    let source = "hello\n\nworld";

    // Test empty line (line 2)
    let loc = RealDependencyLocation::from_byte_location(source, 2, 0, None);
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 2);
    assert_eq!(loc.start.column, 1);
  }

  #[test]
  fn test_from_byte_location_length_exceeds_line() {
    let source = "hello world";

    // Test with length that exceeds the line
    let loc = RealDependencyLocation::from_byte_location(source, 1, 6, Some(100));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 7);
    // Should clamp to end of line: "world" = 5 chars
    assert!(loc.end.is_none()); // 7 + 5 = 12
  }

  #[test]
  fn test_from_byte_location_mixed_content() {
    // Mix of ASCII, multi-byte UTF-8, and emoji
    let source = "abc你好😀xyz\nline2\nline3";

    // Start at "😀" (byte offset: 3 + 6 = 9), length 4
    let loc = RealDependencyLocation::from_byte_location(source, 1, 9, Some(4));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    // "abc" = 3, "你好" = 2, so start at UTF-16 position 6 (1-based)
    assert_eq!(loc.start.column, 6);
    assert_eq!(loc.end.as_ref().unwrap().line, 1);
    assert_eq!(loc.end.as_ref().unwrap().column, 8);
  }

  #[test]
  fn test_from_byte_location_multiline() {
    // Test length spanning multiple lines
    let source = "hello world\nfoo bar baz\nend";

    // Start at "world" (byte 6 on line 1), length 18 (to "end" on line 3)
    let loc = RealDependencyLocation::from_byte_location(source, 1, 6, Some(18));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 7);
    assert_eq!(loc.end.as_ref().unwrap().line, 3);
    assert_eq!(loc.end.as_ref().unwrap().column, 1);
  }

  #[test]
  fn test_from_byte_location_multiline_three_lines() {
    // Test length spanning three lines
    let source = "abc\ndefg\nhij\nklm";

    // Start at byte 2 on line 1 ("c"), length 10
    // "c\ndefg\nhi" = 1 + 1 + 4 + 1 + 2 = 9 bytes, so 10 includes "j"
    let loc = RealDependencyLocation::from_byte_location(source, 1, 2, Some(10));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 3); // "c" is at column 3 (1-based)
    assert_eq!(loc.end.as_ref().unwrap().line, 3);
    assert_eq!(loc.end.as_ref().unwrap().column, 4); // "j" is at column 3 on line 3
  }

  #[test]
  fn test_from_byte_location_multiline_utf8() {
    // Test multiline with UTF-8 characters
    let source = "你好\n世界abc\n测试";

    // Start at byte 0 on line 1, length 12
    // "你好\n世界" = 6 + 1 + 6 = 13 bytes, so 12 doesn't include the last character
    let loc = RealDependencyLocation::from_byte_location(source, 1, 0, Some(13));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 1);
    assert_eq!(loc.end.as_ref().unwrap().line, 2);
    assert_eq!(loc.end.as_ref().unwrap().column, 3); // "世界" = 2 UTF-16 units, column 3 (1-based)
  }

  #[test]
  fn test_from_byte_location_multiline_exact_line_end() {
    // Test when length ends exactly at a line boundary
    let source = "hello\nworld";

    // Start at byte 0, length 5 (exactly "hello")
    let loc = RealDependencyLocation::from_byte_location(source, 1, 0, Some(5));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 1);
    assert_eq!(loc.end.as_ref().unwrap().line, 1);
    assert_eq!(loc.end.as_ref().unwrap().column, 6); // End of "hello"
  }

  #[test]
  fn test_from_byte_location_multiline_including_newline() {
    // Test when length includes the newline character
    let source = "hello\nworld";

    // Start at byte 0, length 6 (includes newline)
    let loc = RealDependencyLocation::from_byte_location(source, 1, 0, Some(6));
    assert!(loc.is_some());
    let loc = loc.unwrap();
    assert_eq!(loc.start.line, 1);
    assert_eq!(loc.start.column, 1);
    assert_eq!(loc.end.as_ref().unwrap().line, 2);
    assert_eq!(loc.end.as_ref().unwrap().column, 1); // First position of line 2
  }
}
