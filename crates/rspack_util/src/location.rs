use std::fmt::Display;

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

/// Convert line, column, length to a [`Location`]
///
/// Line are 1-based, column are 0-based in bytes
///
/// Return `None` if any value is out of bounds
pub fn try_line_column_length_to_location(
  rope: &ropey::Rope,
  line: usize,
  column: usize,
  length: usize,
) -> Option<Location> {
  let sl = line.saturating_sub(1);
  let sc = column;

  let sb = rope.try_line_to_byte(sl).ok()?;
  let end_byte = sb + sc + length;
  let el = rope.try_byte_to_line(end_byte).ok()?;
  let ec = end_byte - rope.try_line_to_byte(el).ok()?;

  Some(Location {
    sl: sl as u32,
    sc: sc as u32,
    el: el as u32,
    ec: ec as u32,
  })
}

/// Convert line, column, length to a (offset, length)
///
/// Offset is 0-based in bytes
///
/// Return `None` if any value is out of bounds
pub fn try_line_column_length_to_offset_length(
  rope: &ropey::Rope,
  line: usize,
  column: usize,
  length: usize,
) -> Option<(usize, usize)> {
  let line = line.saturating_sub(1);
  let sb = rope.try_line_to_byte(line).ok()?;
  let offset = sb + column;
  Some((offset, length))
}
