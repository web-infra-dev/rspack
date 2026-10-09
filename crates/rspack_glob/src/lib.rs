// License notice for the portions adapted from fast-glob and glob-match.
// MIT License
//
// Copyright (c) 2025-present VoidZero Inc. & Contributors
// Copyright (c) 2024 shulaoda
// Copyright (c) 2023 Devon Govett
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! Glob matching with resumable literal prefix matching.
//!
//! Ported from the Rust crate fast-glob 1.1.2. The byte-oriented syntax,
//! escaping, brace limits and platform path separators follow fast-glob.
//! Its backtracking matcher is adapted into a shared instruction graph so
//! consuming a prefix retains every possible continuation.
//! The legacy [`glob_match`] entry point preserves fast-glob's matching behavior.
//! [`GlobPattern`] retains wildcard alternatives that its single-backtrack
//! interpreter can discard: `**{**/a,b}` matches `aa` with `GlobPattern`, whereas
//! `glob_match` returns false. Negation complements each entry point's own result.
//!
//! ```rust
//! use rspack_glob::GlobPattern;
//!
//! let pattern = GlobPattern::new("@/components/**/*.js")?;
//! let remaining = pattern
//!   .match_prefix("@/components/")
//!   .expect("matching prefix");
//! assert!(remaining.is_match("button/index.js"));
//! # Ok::<(), rspack_glob::Error>(())
//! ```

mod matcher;
mod program;
mod syntax;

use std::{borrow::Cow, sync::Arc};

use cow_utils::CowUtils;
pub use matcher::glob_match;
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
/// Cloning shares the compiled graph and copies the active states so callers
/// can independently traverse sibling directories. Matching operates on bytes:
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
  pub fn new(pattern: impl AsRef<[u8]>) -> Result<Self, Error> {
    Self::new_with_options(pattern, GlobOptions::default())
  }

  /// Compiles a pattern with case and hidden-component matching options.
  pub fn new_with_options(pattern: impl AsRef<[u8]>, options: GlobOptions) -> Result<Self, Error> {
    let pattern = fold_case(pattern.as_ref(), options.case_sensitive);
    validate(&pattern)?;
    let (instructions, start, negated) = Program::compile(&pattern);
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

  /// Matches the entire remaining path, applying any leading `!` negation.
  /// An empty path tests whether the prefix already consumed is a full match.
  pub fn is_match(&self, path: impl AsRef<[u8]>) -> bool {
    let path = fold_case(path.as_ref(), self.options.case_sensitive);
    let program = self.view();
    let (states, _) = program.consume(&self.states, &path, self.component_start);
    program.is_match(&states)
  }

  /// Consumes a literal prefix and returns a matcher for the remaining suffix.
  ///
  /// No separator is inserted and glob metacharacters in the prefix are literal.
  /// For directory traversal, include the trailing separator explicitly.
  /// The returned pattern satisfies `remaining.is_match(suffix) ==
  /// self.is_match(prefix + suffix)` and can itself consume more prefixes.
  ///
  /// `None` means no continuation is possible. Leading `!` patterns conservatively
  /// retain a matcher even if the complemented language is empty; use `is_match`
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
