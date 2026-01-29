use memchr::{memchr, memchr_iter};

/// Convert byte line, column, length to offset
/// - line is 1-based in bytes
/// - column is 0-based in bytes
pub fn byte_line_column_to_offset(source: &str, line: usize, column: usize) -> Option<usize> {
  if line == 0 {
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
    .map_or(bytes.len(), |rel| line_start + rel);

  // Slice of the line
  let line_slice = &source[line_start..line_end];

  let mut utf16_units = 0usize;
  let mut byte_offset_in_line = 0usize;
  for ch in line_slice.chars() {
    if utf16_units >= column {
      break;
    }
    utf16_units += if (ch as u32) >= 0x1_0000 { 2 } else { 1 };
    byte_offset_in_line += ch.len_utf8();
  }

  if utf16_units < column {
    return None;
  }

  let offset = line_start + byte_offset_in_line;
  if offset > bytes.len() {
    return None;
  }

  Some(offset)
}
