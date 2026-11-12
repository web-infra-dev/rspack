use std::fmt::{self, Write};

use camino::Utf8PathBuf;
use rspack_cacheable::{cacheable, utils::PortablePath, with::As};
use simd_utf16_len::utf16_len;

use crate::{DependencyLocation, RealDependencyLocation, SourcePosition};

/// How a certified path literal is written, so a path under another root gets the bytes its
/// producer writes there.
#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PathSpelling {
  /// JSON escapes in double quotes, with every non-ASCII UTF-16 code unit as `\uXXXX`. The
  /// fallback for a literal that neither SWC rule writes.
  Ascii,
  /// SWC's string literal for a target above ES5, which keeps non-ASCII characters.
  Swc,
  /// SWC's string literal for ES5 and below, which also escapes U+007F to U+00FF and characters
  /// above U+FFFF.
  SwcEs5,
}

impl PathSpelling {
  /// The spelling that writes `value` as `raw`. Both SWC rules write a value the same way unless
  /// it has a character they treat differently, so `es5` says which rule the producer follows.
  pub fn of(raw: &str, value: &str, es5: bool) -> Self {
    let rules = if es5 {
      [Self::SwcEs5, Self::Swc]
    } else {
      [Self::Swc, Self::SwcEs5]
    };
    rules
      .into_iter()
      .find(|rule| rule.quote(value) == raw)
      .unwrap_or(Self::Ascii)
  }

  pub fn quote(self, value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    let _ = self.write(value, &mut quoted);
    quoted
  }

  /// The width of the quoted value in UTF-16 code units, which source columns count.
  fn width(self, value: &str) -> usize {
    struct Width(usize);
    impl Write for Width {
      fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0 += utf16_len(text);
        Ok(())
      }
    }
    let mut width = Width(0);
    let _ = self.write(value, &mut width);
    width.0
  }

  fn write(self, value: &str, quoted: &mut impl Write) -> fmt::Result {
    match self {
      Self::Ascii => write_quoted_ascii(value, quoted),
      Self::Swc => write_quoted_swc(value, false, quoted),
      Self::SwcEs5 => write_quoted_swc(value, true, quoted),
    }
  }
}

/// A single-line quoted path replacement in canonical source coordinates.
/// Cloned only into locations whose endpoints depend on this replacement's width; dependency
/// locations themselves need Clone when copied between dependencies and graph origins.
#[cacheable]
#[derive(Debug, Clone)]
pub struct PortableLocationShift {
  pub start: SourcePosition,
  pub canonical_width: u32,
  #[cacheable(with=As<PortablePath>)]
  pub path: Utf8PathBuf,
  pub spelling: PathSpelling,
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
        let width = shift.spelling.width(shift.path.as_str()) as i64;
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

/// The spelling of canonical literals: JSON escapes with every non-ASCII UTF-16 code unit written
/// as `\uXXXX`. The result is always single-line ASCII.
pub fn quote_ascii(value: &str) -> String {
  PathSpelling::Ascii.quote(value)
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

/// SWC's `get_quoted_utf16` for valid Unicode without its `ascii_only` option. It takes the quote
/// that needs fewer escapes and keeps every character that needs none. For ES5 and below it also
/// writes U+007F to U+00FF as `\xNN` and a character above U+FFFF as a surrogate pair.
fn write_quoted_swc(value: &str, es5: bool, quoted: &mut impl Write) -> fmt::Result {
  let count = |quote| memchr::memchr_iter(quote, value.as_bytes()).count();
  let quote = if count(b'"') > count(b'\'') {
    '\''
  } else {
    '"'
  };
  quoted.write_char(quote)?;
  let mut chars = value.chars().peekable();
  while let Some(c) = chars.next() {
    match c {
      '\0' if chars.peek().is_some_and(|next| next.is_ascii_digit()) => {
        quoted.write_str("\\x00")?
      }
      '\0' => quoted.write_str("\\0")?,
      '\u{8}' => quoted.write_str("\\b")?,
      '\u{c}' => quoted.write_str("\\f")?,
      '\n' => quoted.write_str("\\n")?,
      '\r' => quoted.write_str("\\r")?,
      '\u{b}' => quoted.write_str("\\v")?,
      '\\' => quoted.write_str("\\\\")?,
      c if c == quote => {
        quoted.write_char('\\')?;
        quoted.write_char(c)?;
      }
      '\u{1}'..='\u{1f}' if c != '\t' => write!(quoted, "\\x{:02x}", c as u8)?,
      '\u{7f}'..='\u{ff}' if es5 => write!(quoted, "\\x{:x}", c as u8)?,
      '\u{2028}' => quoted.write_str("\\u2028")?,
      '\u{2029}' => quoted.write_str("\\u2029")?,
      '\u{feff}' => quoted.write_str("\\uFEFF")?,
      c if es5 && c > '\u{ffff}' => {
        let c = c as u32 - 0x10000;
        write!(
          quoted,
          "\\u{:04X}\\u{:04X}",
          c / 0x400 + 0xd800,
          c % 0x400 + 0xdc00
        )?;
      }
      c => quoted.write_char(c)?,
    }
  }
  quoted.write_char(quote)
}
