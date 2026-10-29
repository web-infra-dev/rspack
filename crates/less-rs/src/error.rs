//! Errors and warnings.
//!
//! [`LessError`] is the internal error value that is propagated with `?`
//! through the parser and evaluator; it mirrors the plain objects that less.js
//! throws (`{ type, message, index, filename }`). At the API boundary it is
//! converted into the public [`Error`], whose `Display` output is identical to
//! less.js's `LessError.prototype.toString()`.

use std::fmt;

use cow_utils::CowUtils;

use crate::tree::Pos;

/// Arena bytes a compilation may use for `source_len` bytes of source:
/// 256 per source byte, at least 64 MiB. Real stylesheets need under 50
/// (semantic-ui: 40 MB for about a megabyte of source); only input that grows
/// without bound gets near it, and less.js runs out of heap on such input.
pub(crate) fn arena_budget(source_len: usize) -> usize {
  source_len.saturating_mul(256).max(64 << 20)
}

pub(crate) fn out_of_memory(budget: usize) -> Box<LessError> {
  LessError::new(
    "Runtime",
    format!(
      "Out of memory: the compilation needs more than {} MiB",
      budget >> 20
    ),
  )
}

#[derive(Debug, Clone)]
pub struct LessError {
  pub typ: String,
  pub message: String,
  pub pos: Option<Pos>,
  /// A file id when the error names a file but no index.
  pub file: Option<u32>,
  /// less.js `LessError` instances carry own `line`/`column` properties;
  /// `Call.eval` passes those through without wrapping.
  pub has_line_col: bool,
  /// `true` once the error has been given a definitive index (e.g. by
  /// `Declaration.eval`), mirroring `typeof err.index === 'number'`.
  pub index_set: bool,
  /// A native JS `Error` (its `message` is not enumerable, so object spread
  /// in `MixinCall.eval` drops it).
  pub native: bool,
}

pub type LResult<T> = Result<T, Box<LessError>>;

impl LessError {
  pub fn new(typ: &str, message: impl Into<String>) -> Box<LessError> {
    Box::new(LessError {
      typ: typ.to_string(),
      message: message.into(),
      pos: None,
      file: None,
      has_line_col: false,
      index_set: false,
      native: false,
    })
  }
  /// A native JS error (`new Error(...)`, `TypeError`, `RangeError`).
  pub fn native(message: impl Into<String>) -> Box<LessError> {
    let mut e = LessError::new("", message);
    e.native = true;
    e
  }
  pub fn at(typ: &str, message: impl Into<String>, pos: Pos) -> Box<LessError> {
    let mut e = LessError::new(typ, message);
    e.set_pos(pos);
    e
  }
  /// An error without a `type` (a JS `Error` or a plain `{ message }`
  /// object): reported as `Syntax`, or `Runtime` inside a function call.
  pub fn syntax(message: impl Into<String>) -> Box<LessError> {
    LessError::new("", message)
  }
  pub fn set_pos(&mut self, pos: Pos) {
    if pos.is_none() {
      self.pos = None;
    } else {
      self.pos = Some(pos);
    }
    self.index_set = true;
  }
}

/// The kind of a [`Error`]; corresponds to less.js's error `type`.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
  Parse,
  Syntax,
  Name,
  Runtime,
  Operation,
  Argument,
  File,
  Value,
  Unit,
  Plugin,
  JsUnsupported,
  Io,
  Other,
}

impl ErrorKind {
  pub(crate) fn from_type(t: &str) -> ErrorKind {
    match t {
      "Parse" => ErrorKind::Parse,
      "Syntax" => ErrorKind::Syntax,
      "Name" => ErrorKind::Name,
      "Runtime" => ErrorKind::Runtime,
      "Operation" => ErrorKind::Operation,
      "Argument" => ErrorKind::Argument,
      "File" => ErrorKind::File,
      "Value" => ErrorKind::Value,
      "Unit" => ErrorKind::Unit,
      "Plugin" => ErrorKind::Plugin,
      _ => ErrorKind::Other,
    }
  }
}

/// A compilation error.
///
/// `Display` renders exactly what less.js's `err.toString()` renders (without
/// colors), including the three-line source extract.
#[derive(Clone, Debug)]
pub struct Error {
  pub(crate) kind: ErrorKind,
  pub(crate) type_name: String,
  pub(crate) message: String,
  pub(crate) filename: Option<String>,
  pub(crate) line: Option<u32>,
  pub(crate) column: Option<u32>,
  pub(crate) index: Option<u32>,
  pub(crate) extract: [Option<String>; 3],
}

impl Error {
  pub fn kind(&self) -> ErrorKind {
    self.kind
  }
  /// The less.js error type (`"Syntax"`, `"Name"`, `"Parse"`, ...).
  pub fn type_name(&self) -> &str {
    &self.type_name
  }
  pub fn message(&self) -> &str {
    &self.message
  }
  pub fn filename(&self) -> Option<&str> {
    self.filename.as_deref()
  }
  /// 1-based line and 0-based column (as less.js reports them).
  pub fn location(&self) -> Option<(u32, u32)> {
    match (self.line, self.column) {
      (Some(l), Some(c)) => Some((l, c)),
      _ => None,
    }
  }
  pub fn index(&self) -> Option<u32> {
    self.index
  }
  pub fn extract(&self) -> &[Option<String>; 3] {
    &self.extract
  }

  pub(crate) fn simple(kind: ErrorKind, type_name: &str, message: String) -> Error {
    Error {
      kind,
      type_name: type_name.to_string(),
      message,
      filename: None,
      line: None,
      column: None,
      index: None,
      extract: [None, None, None],
    }
  }
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&format_less_error(
      &self.type_name,
      &self.message,
      self.filename.as_deref(),
      self.line,
      self.column,
      &self.extract,
    ))
  }
}

impl std::error::Error for Error {}

/// Pretty terminal diagnostics (feature `miette`); `Display` stays the
/// less.js-compatible text.
#[cfg(feature = "miette")]
impl miette::Diagnostic for Error {
  fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    Some(Box::new(format!("less::{}", self.type_name)))
  }
  /// The three-line extract around the error, as less.js prints it.
  fn help<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
    let lines: Vec<&str> = self.extract.iter().flatten().map(|s| s.as_str()).collect();
    (!lines.is_empty()).then(|| Box::new(lines.join("\n")) as Box<dyn std::fmt::Display>)
  }
}

/// `LessError.prototype.toString` with an identity `stylize`.
pub(crate) fn format_less_error(
  type_name: &str,
  message: &str,
  filename: Option<&str>,
  line: Option<u32>,
  column: Option<u32>,
  extract: &[Option<String>; 3],
) -> String {
  format_less_error_styled(
    type_name,
    message,
    filename,
    line,
    column,
    extract,
    &|s, _| s.to_string(),
  )
}

/// lessc's ANSI `stylize`.
pub(crate) fn ansi(s: &str, style: &str) -> String {
  let (a, b) = match style {
    "reset" => (0, 0),
    "bold" => (1, 22),
    "inverse" => (7, 27),
    "underline" => (4, 24),
    "yellow" => (33, 39),
    "green" => (32, 39),
    "red" => (31, 39),
    _ => (90, 39),
  };
  format!("\x1b[{}m{}\x1b[{}m", a, s, b)
}

pub(crate) fn format_less_error_styled(
  type_name: &str,
  message: &str,
  filename: Option<&str>,
  line: Option<u32>,
  column: Option<u32>,
  extract: &[Option<String>; 3],
  stylize: &dyn Fn(&str, &str) -> String,
) -> String {
  let is_warning = type_name.cow_to_lowercase().contains("warning");
  let typ = if is_warning {
    type_name.to_string()
  } else {
    format!("{}Error", type_name)
  };
  let color = if is_warning { "yellow" } else { "red" };
  let mut error = String::new();
  if let Some(line) = line {
    let mut parts: Vec<String> = Vec::new();
    if !is_warning && let Some(e0) = &extract[0] {
      parts.push(stylize(&format!("{} {}", line as i64 - 1, e0), "grey"));
    }
    if let Some(e1) = &extract[1] {
      let mut t = format!("{} ", line);
      if !e1.is_empty() {
        let col = column.unwrap_or(0) as usize;
        let units: Vec<u16> = e1.encode_utf16().collect();
        let a = String::from_utf16_lossy(&units[..col.min(units.len())]);
        let b = String::from_utf16_lossy(&units[col.min(units.len())..(col + 1).min(units.len())]);
        let c = String::from_utf16_lossy(&units[(col + 1).min(units.len())..]);
        t.push_str(&a);
        t.push_str(&stylize(
          &stylize(&format!("{}{}", stylize(&b, "bold"), c), "red"),
          "inverse",
        ));
      }
      parts.push(t);
    }
    if !is_warning && let Some(e2) = &extract[2] {
      parts.push(stylize(&format!("{} {}", line + 1, e2), "grey"));
    }
    error = format!("{}{}\n", parts.join("\n"), stylize("", "reset"));
  }
  let mut out = stylize(&format!("{}: {}", typ, message), color);
  if let Some(f) = filename {
    out.push_str(&stylize(" in ", color));
    out.push_str(f);
  }
  if let Some(l) = line
    && l != 0
  {
    out.push_str(&stylize(
      &format!(
        " on line {}, column {}:",
        l,
        column.map(|c| c as i64 + 1).unwrap_or(0)
      ),
      "grey",
    ));
  }
  out.push('\n');
  out.push_str(&error);
  if filename.is_none() {
    out.push('\n');
  }
  out
}

impl Error {
  /// `err.toString({ stylize })` as printed by `lessc` (ANSI colors).
  pub fn to_string_colored(&self) -> String {
    format_less_error_styled(
      &self.type_name,
      &self.message,
      self.filename.as_deref(),
      self.line,
      self.column,
      &self.extract,
      &|s, st| ansi(s, st),
    )
  }
}

/// A warning (deprecations, unmatched extends, ...).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Warning {
  /// The stable deprecation id, when this is a deprecation warning.
  pub id: Option<&'static str>,
  /// The full text less.js would log.
  pub message: String,
}

impl fmt::Display for Warning {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.message)
  }
}
