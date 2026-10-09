//! Glob matching with resumable literal prefix matching.
//!
//! Byte-oriented syntax, escaping, brace limits and platform path separators
//! follow fast-glob. Patterns compile into a shared instruction graph so
//! consuming a prefix retains every possible continuation.
//!
//! ```rust
//! use rspack_glob::{GlobMatch, GlobPattern};
//!
//! let pattern = GlobPattern::new(b"@/components/**/*.js")?;
//! let GlobMatch::MatchPartial(remaining) = pattern.match_path("@/components/") else {
//!   panic!("matching directory prefix");
//! };
//! assert!(matches!(
//!   remaining.match_path("button/index.js"),
//!   GlobMatch::MatchExact
//! ));
//! # Ok::<(), rspack_glob::Error>(())
//! ```

mod path;
mod program;
mod syntax;
#[cfg(feature = "fs")]
mod walk;

use std::{
  borrow::Cow,
  sync::{Arc, OnceLock},
};

use cow_utils::CowUtils;
pub use path::{
  extract_glob_base_dir, glob_base_dir_end, is_glob_metacharacter, normalize_path_separators,
  normalize_path_separators_for_path, unescape_glob_path,
};
use program::{Instructions, Program, States};
pub use syntax::{Error, ErrorKind, validate};

/// Options applied while compiling and consuming a glob.
#[derive(Debug, Clone, Copy)]
pub struct GlobOptions {
  /// Match the spelling exactly; otherwise fold Unicode case before matching.
  pub case_sensitive: bool,
  /// Require an explicit leading `.` in each hidden path component.
  pub require_literal_leading_dot: bool,
}

impl Default for GlobOptions {
  fn default() -> Self {
    Self {
      case_sensitive: true,
      require_literal_leading_dot: false,
    }
  }
}

/// A compiled glob and the matching states remaining after a literal prefix.
///
/// Cloning shares the compiled graph, source and cached prefix, and copies the
/// active states so callers can independently traverse sibling directories.
/// The source borrows the input unless case folding changes its bytes.
/// Matching operates on bytes:
/// `?` consumes one non-separator byte, including within UTF-8 strings.
#[derive(Debug, Clone)]
pub struct GlobPattern<'a> {
  program: Arc<Instructions>,
  source: Arc<Cow<'a, [u8]>>,
  literal_prefix: OnceLock<Arc<Cow<'a, [u8]>>>,
  states: States,
  negated: bool,
  options: GlobOptions,
  component_start: bool,
}

impl<'a> GlobPattern<'a> {
  /// Compiles a pattern, rejecting invalid syntax with its original byte offset.
  pub fn new(pattern: &'a [u8]) -> Result<Self, Error> {
    Self::new_with_options(pattern, GlobOptions::default())
  }

  /// Compiles a pattern with case and hidden-component matching options.
  pub fn new_with_options(pattern: &'a [u8], options: GlobOptions) -> Result<Self, Error> {
    validate(pattern)?;
    let source = Arc::new(fold_case(pattern, options.case_sensitive));
    let (instructions, start, negated) = Program::compile(&source);
    let program = Program {
      instructions: &instructions,
      negated,
      options,
    };
    let states = program.start(start);
    Ok(Self {
      program: Arc::new(instructions),
      source,
      literal_prefix: OnceLock::new(),
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
  /// Returns no prefix for a complemented pattern. Matching still operates on
  /// bytes, so the prefix may end inside a UTF-8 character.
  /// Contiguous literals borrow the source; decoded or joined literals are
  /// cached once and shared by clones. Repeated queries allocate no prefix bytes.
  pub fn literal_prefix(&self) -> &[u8] {
    self
      .literal_prefix
      .get_or_init(|| Arc::new(self.view().literal_prefix(&self.states, &self.source)))
  }

  /// Matches a path, applying any leading `!` negation.
  ///
  /// A full match takes precedence over a partial match, even when more input
  /// could also match (for example `**`). Use [`Self::match_prefix`] to retain
  /// continuations in that case. An empty path tests the already consumed prefix.
  pub fn match_path(&self, path: impl AsRef<[u8]>) -> GlobMatch<'a> {
    let path = fold_case(path.as_ref(), self.options.case_sensitive);
    let program = self.view();
    let (states, component_start) = program.consume(&self.states, &path, self.component_start);
    if program.is_match(&states) {
      GlobMatch::MatchExact
    } else if program.can_match(&states) {
      GlobMatch::MatchPartial(Self {
        program: Arc::clone(&self.program),
        source: self.source.clone(),
        literal_prefix: OnceLock::new(),
        states,
        negated: self.negated,
        options: self.options,
        component_start,
      })
    } else {
      GlobMatch::NotMatch
    }
  }

  /// Consumes a literal prefix and returns a matcher for the remaining suffix.
  ///
  /// No separator is inserted and glob metacharacters in the prefix are literal.
  /// For directory traversal, include the trailing separator explicitly.
  /// The returned pattern satisfies `remaining.match_path(suffix).is_exact() ==
  /// self.match_path(prefix + suffix).is_exact()` and can itself consume more prefixes.
  ///
  /// `None` means no continuation is possible. Leading `!` patterns conservatively
  /// retain a matcher even if the complemented language is empty; use `match_path`
  /// to test completed paths. A negated pattern whose positive states die still
  /// matches every suffix, so it must remain usable during traversal.
  pub fn match_prefix(&self, prefix: impl AsRef<[u8]>) -> Option<Self> {
    let prefix = fold_case(prefix.as_ref(), self.options.case_sensitive);
    let program = self.view();
    let (states, component_start) = program.consume(&self.states, &prefix, self.component_start);
    if !program.can_match(&states) {
      return None;
    }
    Some(Self {
      program: Arc::clone(&self.program),
      source: self.source.clone(),
      literal_prefix: OnceLock::new(),
      states,
      negated: self.negated,
      options: self.options,
      component_start,
    })
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
    Cow::Owned(bytes.to_ascii_lowercase())
  }
}

/// The result of consuming a path with a compiled glob.
#[derive(Debug)]
pub enum GlobMatch<'a> {
  /// The entire path matches; longer paths may also match.
  MatchExact,
  /// The path is a matching prefix; the payload matches its remaining suffix.
  MatchPartial(GlobPattern<'a>),
  /// No matching continuation is possible.
  NotMatch,
}

impl GlobMatch<'_> {
  /// Whether the entire path matched.
  pub fn is_exact(&self) -> bool {
    matches!(self, Self::MatchExact)
  }
}
