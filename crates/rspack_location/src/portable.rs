use std::fmt::{self, Write};

use camino::Utf8PathBuf;
use rspack_cacheable::{cacheable, utils::PortablePath, with::As};

use crate::{DependencyLocation, RealDependencyLocation, SourcePosition};

/// A single-line ASCII-quoted path replacement in canonical source coordinates.
/// Cloned only into locations whose endpoints depend on this replacement's width; dependency
/// locations themselves need Clone when copied between dependencies and graph origins.
#[cacheable]
#[derive(Debug, Clone)]
pub struct PortableLocationShift {
  pub start: SourcePosition,
  pub canonical_width: u32,
  #[cacheable(with=As<PortablePath>)]
  pub path: Utf8PathBuf,
}

impl PortableLocationShift {
  fn affects(&self, point: SourcePosition) -> bool {
    self.start.line == point.line && self.start.column < point.column
  }
}

/// Canonical endpoints plus only the path replacements affecting their columns.
/// Clone is required by DependencyLocation; unaffected locations retain the ordinary Real variant.
#[cacheable]
#[derive(Debug, Clone)]
pub struct PortableDependencyLocation {
  canonical: RealDependencyLocation,
  shifts: Vec<PortableLocationShift>,
}

impl PortableDependencyLocation {
  /// `shifts` must be sorted by canonical position, with disjoint single-line ranges.
  pub fn from_canonical(
    canonical: RealDependencyLocation,
    shifts: &[PortableLocationShift],
  ) -> DependencyLocation {
    let affects = |shift: &PortableLocationShift| {
      shift.affects(canonical.start) || canonical.end.is_some_and(|end| shift.affects(end))
    };
    // Search only the endpoint lines. Earlier lines affect offsets but never these columns.
    let on_line = |line| {
      let first = shifts.partition_point(|shift| shift.start.line < line);
      shifts[first..]
        .iter()
        .take_while(move |shift| shift.start.line == line)
    };
    let end_line = canonical.end.map_or(canonical.start.line, |end| end.line);
    let shifts: Vec<_> = on_line(canonical.start.line)
      .chain(on_line(end_line).filter(|_| end_line != canonical.start.line))
      .filter(|shift| affects(shift))
      .cloned()
      .collect();
    if shifts.is_empty() {
      DependencyLocation::Real(canonical)
    } else {
      DependencyLocation::Portable(Box::new(Self { canonical, shifts }))
    }
  }

  pub fn resolve(&self) -> RealDependencyLocation {
    let position = |point: SourcePosition, end: bool| {
      let mut delta = 0i64;
      for shift in self.shifts.iter().filter(|shift| shift.affects(point)) {
        let width = quoted_ascii_width(shift.path.as_str()) as i64;
        if point.column < shift.start.column + shift.canonical_width {
          // Match SourceView's outward snapping for an endpoint inside a replacement.
          return SourcePosition {
            line: point.line,
            column: (i64::from(shift.start.column) + delta + if end { width } else { 0 }) as u32,
          };
        }
        delta += width - i64::from(shift.canonical_width);
      }
      SourcePosition {
        line: point.line,
        column: (i64::from(point.column) + delta) as u32,
      }
    };
    let start = position(self.canonical.start, false);
    let end = self
      .canonical
      .end
      .map(|end| position(end, end != self.canonical.start));
    RealDependencyLocation::new(start, end)
  }
}

/// One spelling shared by path rendering and portable location widths: JSON escapes with every
/// non-ASCII UTF-16 code unit written as `\uXXXX`. The result is always single-line ASCII.
pub fn quote_ascii(value: &str) -> String {
  let mut quoted = String::with_capacity(value.len() + 2);
  let _ = write_quoted_ascii(value, &mut quoted);
  quoted
}

fn quoted_ascii_width(value: &str) -> usize {
  struct Width(usize);
  impl Write for Width {
    fn write_str(&mut self, text: &str) -> fmt::Result {
      self.0 += text.len();
      Ok(())
    }
  }
  let mut width = Width(0);
  let _ = write_quoted_ascii(value, &mut width);
  width.0
}

fn write_quoted_ascii(value: &str, quoted: &mut impl Write) -> fmt::Result {
  quoted.write_char('"')?;
  for c in value.chars() {
    match c {
      '"' => quoted.write_str("\\\"")?,
      '\\' => quoted.write_str("\\\\")?,
      '\n' => quoted.write_str("\\n")?,
      '\r' => quoted.write_str("\\r")?,
      '\t' => quoted.write_str("\\t")?,
      '\u{8}' => quoted.write_str("\\b")?,
      '\u{c}' => quoted.write_str("\\f")?,
      ' '..='~' => quoted.write_char(c)?,
      _ => {
        let mut units = [0; 2];
        for unit in c.encode_utf16(&mut units) {
          write!(quoted, "\\u{unit:04x}")?;
        }
      }
    }
  }
  quoted.write_char('"')
}
