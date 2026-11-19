//! Glob matching with resumable literal prefix matching.
//!
//! Byte-oriented syntax, escaping, brace limits and platform path separators
//! follow fast-glob. Patterns compile into a shared instruction graph so
//! consuming a prefix retains every possible continuation.
//!
//! ```rust
//! use rspack_util::glob::GlobPattern;
//!
//! let pattern = GlobPattern::new(b"@/components/**/*.js")?;
//! let remaining = pattern
//!   .match_prefix("@/components/")
//!   .expect("matching directory");
//! assert!(remaining.is_match("button/index.js"));
//! # Ok::<(), rspack_util::glob::Error>(())
//! ```

mod program;
mod syntax;

use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;
use program::{Instructions, Program, States};
pub use syntax::{BackslashMode, Error, ErrorKind, validate};

/// Normalizes Windows-form glob separators while retaining glob escapes.
/// Invalid patterns keep their original spelling.
pub fn normalize_glob_path(pattern: &str) -> Cow<'_, str> {
  if !pattern.as_bytes().contains(&b'\\') {
    return Cow::Borrowed(pattern);
  }
  match syntax::parse(pattern.as_bytes(), BackslashMode::WindowsPath) {
    Ok(Cow::Owned(bytes)) => {
      Cow::Owned(String::from_utf8(bytes).expect("separator normalization preserves UTF-8"))
    }
    Ok(Cow::Borrowed(_)) | Err(_) => Cow::Borrowed(pattern),
  }
}

/// The directory spelled out before the first glob construct in a positive
/// pattern (without a leading `!` negation prefix) with normalized `/`
/// separators. Escaped metacharacters are literal path characters.
pub fn literal_directory_prefix(pattern: &str) -> Cow<'_, str> {
  let mut bytes = pattern.bytes().enumerate();
  let mut end = 0;
  while let Some((index, byte)) = bytes.next() {
    match byte {
      b'*' | b'?' | b'[' | b'{' => break,
      b'/' => end = index + 1,
      b'\\' => {
        bytes.next();
      }
      _ => {}
    }
  }

  let prefix = &pattern[..end];
  if !prefix.contains('\\') {
    return Cow::Borrowed(prefix);
  }
  let mut chars = prefix.chars();
  Cow::Owned(
    std::iter::from_fn(|| match chars.next()? {
      '\\' => chars.next(),
      ch => Some(ch),
    })
    .collect(),
  )
}

/// Options applied while compiling and consuming a glob.
#[derive(Debug, Clone, Copy)]
pub struct GlobOptions {
  /// Match the spelling exactly; otherwise fold Unicode case before matching.
  pub case_sensitive: bool,
  /// Require an explicit leading `.` in each hidden path component.
  pub require_literal_leading_dot: bool,
  /// How backslashes outside character classes are interpreted.
  pub backslash: BackslashMode,
}

impl Default for GlobOptions {
  fn default() -> Self {
    Self {
      case_sensitive: true,
      require_literal_leading_dot: false,
      backslash: BackslashMode::Escape,
    }
  }
}

/// A compiled glob and the matching states remaining after a literal prefix.
///
/// Cloning shares the compiled instruction graph and copies the active states
/// so callers can independently traverse sibling directories.
/// Matching operates on bytes:
/// `?` consumes one non-separator byte, including within UTF-8 strings.
#[derive(Debug, Clone)]
pub struct GlobPattern {
  program: Arc<Instructions>,
  states: States,
  negated: bool,
  options: GlobOptions,
  component_start: bool,
}

impl GlobPattern {
  /// Compiles a pattern, rejecting invalid syntax with its original byte offset.
  pub fn new(pattern: &[u8]) -> Result<Self, Error> {
    Self::new_with_options(pattern, GlobOptions::default())
  }

  /// Compiles a pattern with case, backslash, and hidden-component matching options.
  pub fn new_with_options(pattern: &[u8], options: GlobOptions) -> Result<Self, Error> {
    let source = syntax::parse(pattern, options.backslash)?;
    let source = fold_pattern_case(&source, options.case_sensitive);
    let (instructions, start, negated) = Program::compile(&source);
    let program = Program {
      instructions: &instructions,
      negated,
      options,
    };
    let states = program.start(start);
    Ok(Self {
      program: Arc::new(instructions),
      states,
      negated,
      options,
      component_start: true,
    })
  }

  fn view(&self) -> Program<'_> {
    Program {
      instructions: &self.program,
      negated: self.negated,
      options: self.options,
    }
  }

  /// The unescaped literal prefix shared by every remaining matching branch.
  /// Returns an empty vector for a complemented pattern. The result may end inside UTF-8.
  pub fn literal_prefix(&self) -> Vec<u8> {
    self.view().literal_prefix(&self.states)
  }

  /// The common literal prefix ending at a directory boundary, already unescaped.
  /// Returns an empty vector when no complete directory component is shared.
  /// Case-insensitive patterns return folded bytes; filesystem scans only use
  /// structural components for direct lookup to preserve the actual spelling.
  pub fn directory_prefix(&self) -> Vec<u8> {
    let mut prefix = self.literal_prefix();
    let end = prefix
      .iter()
      .rposition(|&byte| byte == b'/')
      .map_or(0, |index| index + 1);
    prefix.truncate(end);
    prefix
  }

  /// Whether the remaining pattern can match a path containing a separator.
  /// Complemented patterns return true.
  pub fn is_recursive(&self) -> bool {
    self.view().is_recursive(&self.states)
  }

  /// Tests a complete path against the remaining pattern.
  pub fn is_match(&self, path: impl AsRef<[u8]>) -> bool {
    self.matches_bytes(path.as_ref())
  }

  #[inline(never)]
  fn matches_bytes(&self, path: &[u8]) -> bool {
    let path = fold_case(path, self.options.case_sensitive);
    let program = self.view();
    let (states, _) = program.consume(&self.states, &path, self.component_start);
    program.is_match(&states)
  }

  /// Consumes a literal prefix and returns a matcher for the remaining suffix.
  ///
  /// No separator is inserted and glob metacharacters in the prefix are literal.
  /// For directory traversal, include the trailing separator explicitly.
  /// When `case_sensitive` is `false`, `prefix` should end on a path separator
  /// (or complete component boundary) so Unicode case folding has full component context.
  /// The returned pattern satisfies `remaining.is_match(suffix) ==
  /// self.is_match(prefix + suffix)` and can itself consume more prefixes.
  ///
  /// `None` means no continuation is possible. Leading `!` patterns conservatively
  /// retain a matcher even if the complemented language is empty; use `is_match`
  /// to test completed paths. A negated pattern whose positive states die still
  /// matches every suffix, so it must remain usable during traversal.
  pub fn match_prefix(&self, prefix: impl AsRef<[u8]>) -> Option<Self> {
    self.consume_prefix(prefix.as_ref(), false)
  }

  /// Consumes a filesystem entry name followed by a separator or the end of the path.
  #[doc(hidden)]
  pub fn match_component(&self, name: &str) -> Option<Self> {
    self.consume_prefix(name.as_bytes(), true)
  }

  #[inline(never)]
  fn consume_prefix(&self, prefix: &[u8], component_boundary: bool) -> Option<Self> {
    let prefix = fold_case(prefix, self.options.case_sensitive);
    let program = self.view();
    let (states, component_start) = program.consume(&self.states, &prefix, self.component_start);
    let alive = if component_boundary {
      program.can_match_component(&states)
    } else {
      program.can_match(&states)
    };
    if !alive {
      return None;
    }
    Some(Self {
      program: Arc::clone(&self.program),
      states,
      negated: self.negated,
      options: self.options,
      component_start,
    })
  }
}

fn fold_pattern_case(bytes: &[u8], case_sensitive: bool) -> Cow<'_, [u8]> {
  if case_sensitive {
    return Cow::Borrowed(bytes);
  }
  // Uppercase \B, \N, \R, \T are literal escapes, not control-character escapes.
  // Strip their backslash before lowercasing so they don't become \b/\n/\r/\t.
  let mut unescaped: Option<Vec<u8>> = None;
  let mut index = 0;
  while index < bytes.len() {
    if bytes[index] == b'\\' && index + 1 < bytes.len() {
      let next = bytes[index + 1];
      if matches!(next, b'B' | b'N' | b'R' | b'T') {
        let out = unescaped.get_or_insert_with(|| bytes[..index].to_vec());
        out.push(next);
      } else if let Some(out) = &mut unescaped {
        out.extend_from_slice(&bytes[index..index + 2]);
      }
      index += 2;
    } else {
      if let Some(out) = &mut unescaped {
        out.push(bytes[index]);
      }
      index += 1;
    }
  }
  match unescaped {
    Some(bytes) => Cow::Owned(fold_case(&bytes, false).into_owned()),
    None => fold_case(bytes, false),
  }
}

fn fold_case(bytes: &[u8], case_sensitive: bool) -> Cow<'_, [u8]> {
  if case_sensitive {
    return Cow::Borrowed(bytes);
  }
  if let Ok(value) = std::str::from_utf8(bytes) {
    match value.cow_to_lowercase() {
      Cow::Borrowed(value) => Cow::Borrowed(value.as_bytes()),
      Cow::Owned(value) => Cow::Owned(value.into_bytes()),
    }
  } else {
    let mut folded = Vec::with_capacity(bytes.len());
    for chunk in bytes.utf8_chunks() {
      folded.extend_from_slice(chunk.valid().cow_to_lowercase().as_bytes());
      folded.extend_from_slice(chunk.invalid());
    }
    Cow::Owned(folded)
  }
}
