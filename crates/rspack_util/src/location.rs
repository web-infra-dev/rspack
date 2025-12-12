use std::fmt::Display;

use memchr::{memchr, memchr_iter};

#[derive(Debug)]
pub struct Location {
  /// Start line, 0-based
  pub sl: u32,
  /// Start column, 0-based
  pub sc: u32,
  /// End line, 0-based
  pub el: u32,
  /// End column, 0-based
  pub ec: u32,
}

impl Display for Location {
  /// Print location in human readable format
  ///
  /// Lines are 1-based, columns are 0-based
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.sl == self.el {
      if self.sc == self.ec {
        return write!(f, "{}:{}", self.sl + 1, self.sc);
      }
      return write!(f, "{}:{}-{}", self.sl + 1, self.sc, self.ec);
    }
    write!(f, "{}:{}-{}:{}", self.sl + 1, self.sc, self.el + 1, self.ec)
  }
}

/// Convert a (line, column) to a byte offset within the source.
/// - Line is 1-based.
/// - Column is 0-based.
/// - Column is measured in UTF-8 bytes within the line.
pub fn utf8_line_column_to_offset(source: &str, line: usize, column: usize) -> Option<usize> {
  if line == 0 {
    return None;
  }

  let bytes = source.as_bytes();
  let target_line = line - 1;

  // Find start of target line using memchr
  let line_start = if target_line == 0 {
    0
  } else {
    let mut count = 0usize;
    let mut pos = None;
    for idx in memchr::memchr_iter(b'\n', bytes) {
      count += 1;
      if count == target_line {
        pos = Some(idx + 1);
        break;
      }
    }
    pos?
  };

  // End of line (exclusive) to validate column
  let line_end = memchr::memchr(b'\n', &bytes[line_start..])
    .map(|rel| line_start + rel)
    .unwrap_or(bytes.len());

  // Column is 1-based byte offset within the line
  let byte_offset_in_line = column;

  // Validate column within line bounds
  if line_start + byte_offset_in_line > line_end {
    return None;
  }

  let offset = line_start + byte_offset_in_line;

  Some(offset)
}

/// Convert utf16 line, column, length to a (offset, length)
/// Semantics match V8 Error stack positions:
/// - Both line and column are 1-based.
/// - Column counts UTF-16 code units (not Unicode scalar values or UTF-8 bytes).
pub fn v8_line_column_length_to_offset_length(
  source: &str,
  line: usize,
  column: usize,
  utf16_len: usize,
) -> Option<(usize, usize)> {
  if line == 0 || column == 0 {
    return None;
  }

  let bytes = source.as_bytes();
  let target_line = line - 1;

  // Find start of target line with memchr
  let line_start = if target_line == 0 {
    0
  } else {
    let mut count = 0usize;
    let mut pos = None;
    for idx in memchr_iter(b'\n', bytes) {
      count += 1;
      if count == target_line {
        pos = Some(idx + 1);
        break;
      }
    }
    pos?
  };

  // End of line (exclusive)
  let line_end = memchr(b'\n', &bytes[line_start..])
    .map(|rel| line_start + rel)
    .unwrap_or(bytes.len());

  // Slice of the line
  let line_slice = &source[line_start..line_end];

  // Convert 1-based UTF-16 column to 0-based byte offset in line
  let mut utf16_units = 0usize;
  let mut byte_offset_in_line = 0usize;
  for ch in line_slice.chars() {
    if utf16_units >= column - 1 {
      break;
    }
    utf16_units += if (ch as u32) >= 0x1_0000 { 2 } else { 1 };
    byte_offset_in_line += ch.len_utf8();
  }

  if utf16_units < column - 1 {
    return None;
  }

  let start_byte = line_start + byte_offset_in_line;
  if start_byte > bytes.len() {
    return None;
  }

  // Advance 'length' UTF-16 units to compute end_byte
  let mut remaining_units = utf16_len;
  let mut end_byte = start_byte;
  if remaining_units > 0 {
    let mut it = source[end_byte..].chars();
    while remaining_units > 0 {
      if let Some(ch) = it.next() {
        let units = if (ch as u32) >= 0x1_0000 { 2 } else { 1 };
        if remaining_units >= units {
          remaining_units -= units;
          end_byte += ch.len_utf8();
        } else {
          // Do not split a non-BMP code point
          break;
        }
      } else {
        break;
      }
    }
  }

  if end_byte > bytes.len() {
    return None;
  }

  Some((start_byte, end_byte - start_byte))
}

/// Convert utf16 line, column, length to a [`Location`]
/// Semantics match V8 Error stack positions:
/// - Both line and column are 1-based.
/// - Column counts UTF-16 code units (not Unicode scalar values or UTF-8 bytes).
pub fn v8_line_column_to_byte_location(
  source: &str,
  line: usize,
  column: usize,
  utf16_len: usize,
) -> Option<Location> {
  if line == 0 || column == 0 {
    return None;
  }

  let bytes = source.as_bytes();
  let target_line = line - 1;

  // Find the start byte index of the target line using memchr
  let line_start = if target_line == 0 {
    0
  } else {
    let mut count = 0usize;
    let mut pos = None;
    for idx in memchr_iter(b'\n', bytes) {
      count += 1;
      if count == target_line {
        pos = Some(idx + 1);
        break;
      }
    }
    pos?
  };

  // Determine end-of-line (exclusive) for bounds on column
  let line_end = memchr(b'\n', &bytes[line_start..])
    .map(|rel| line_start + rel)
    .unwrap_or(bytes.len());

  // Slice of the line
  let line_slice = &source[line_start..line_end];

  // Convert 1-based UTF-16 column to a 0-based byte offset within the line.
  let mut utf16_units = 0usize;
  let mut byte_offset_in_line = 0usize;

  for ch in line_slice.chars() {
    if utf16_units >= column - 1 {
      break;
    }
    utf16_units += if (ch as u32) >= 0x1_0000 { 2 } else { 1 };
    byte_offset_in_line += ch.len_utf8();
  }

  // Column out of bounds
  if utf16_units < column - 1 {
    return None;
  }

  // Start byte position
  let start_byte = line_start + byte_offset_in_line;
  if start_byte > bytes.len() {
    return None;
  }

  // Compute end_byte by advancing utf16_len UTF-16 code units from start_byte.
  // This can cross line boundaries; we iterate chars from start_byte onward.
  let mut remaining_units = utf16_len;
  let mut end_byte = start_byte;

  if remaining_units > 0 {
    // Safe slicing from a valid UTF-8 boundary
    let mut it = source[end_byte..].chars();
    while remaining_units > 0 {
      if let Some(ch) = it.next() {
        let units = if (ch as u32) >= 0x1_0000 { 2 } else { 1 };
        if remaining_units >= units {
          remaining_units -= units;
          end_byte += ch.len_utf8();
          // continue to next char
        } else {
          // Partial surrogate pair requested; cannot split a code point
          // V8 positions count code units, but slices must end on code point boundary.
          // If a non-BMP char (2 units) is only partially requested, we stop before it.
          break;
        }
      } else {
        // Reached end of source before consuming requested units
        break;
      }
    }
  }

  if end_byte > bytes.len() {
    return None;
  }

  // Compute end line index by counting newlines up to end_byte
  let mut el = 0usize;
  for _ in memchr_iter(b'\n', &bytes[..end_byte]) {
    el += 1;
  }
  let sl = target_line;

  // Start column (0-based byte column within start line)
  let sc = byte_offset_in_line;

  // End column: byte offset within end line
  let end_line_start = if el == 0 {
    0
  } else {
    let mut count = 0usize;
    let mut start = 0usize;
    for idx in memchr_iter(b'\n', bytes) {
      count += 1;
      start = idx + 1;
      if count == el {
        break;
      }
    }
    start
  };
  let ec = end_byte - end_line_start;

  Some(Location {
    sl: sl as u32,
    sc: sc as u32,
    el: el as u32,
    ec: ec as u32,
  })
}
