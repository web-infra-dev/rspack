//! `fast-glob` is a high-performance glob matching crate for Rust, originally forked from [`devongovett/glob-match`](https://github.com/devongovett/glob-match).
//! This crate provides efficient glob pattern matching with support for multi-pattern matching and brace expansion.
//!
//! ## Key Features
//!
//! - Up to 60% performance improvement.
//! - Support for more complex and efficient brace expansion.
//! - Fixed matching issues with wildcard and globstar [`glob-match/issues#9`](https://github.com/devongovett/glob-match/issues/9).
//!
//! ## Examples
//!
//! ### Simple Match
//!
//! Note that simple matching does not support `brace expansion`, but all other syntaxes do.
//!
//! ```rust
//! use fast_glob::glob_match;
//!
//! let glob = "some/**/n*d[k-m]e?txt";
//! let path = "some/a/bigger/path/to/the/crazy/needle.txt";
//!
//! assert!(glob_match(glob, path));
//! ```
//!
//! ### Brace Expansion
//!
//! Brace expansion is supported using `glob_match_with_brace`, allowing for more complex matching patterns:
//!
//! ```rust
//! use fast_glob::glob_match_with_brace;
//!
//! let glob = "some/**/{the,crazy}/?*.{png,txt}";
//! let path = "some/a/bigger/path/to/the/crazy/needle.txt";
//!
//! assert!(glob_match_with_brace(glob, path));
//! ```
//!
//! ### Multi-Pattern Matching
//!
//! `Glob` instances can handle multiple patterns efficiently:
//!
//! ```rust
//! use fast_glob::Glob;
//!
//! let mut glob = Glob::default();
//! assert!(glob.add("*.txt"));
//! assert!(glob.is_match("file.txt"));
//! ```
//!
//! ## Syntax
//!
//! `fast-glob` supports the following glob pattern syntax:
//!
//! | Syntax  | Meaning                                                                                                                                                                                             |
//! | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
//! | `?`     | Matches any single character.                                                                                                                                                                       |
//! | `*`     | Matches zero or more characters, except for path separators (e.g., `/`).                                                                                                                             |
//! | `**`    | Matches zero or more characters, including path separators. Must match a complete path segment (i.e., followed by a `/` or the end of the pattern).                                                  |
//! | `[ab]`  | Matches one of the characters contained in the brackets. Character ranges, e.g., `[a-z]`, are also supported. Use `[!ab]` or `[^ab]` to match any character _except_ those contained in the brackets. |
//! | `{a,b}` | Matches one of the patterns contained in the braces. Any of the wildcard characters can be used in the sub-patterns. Braces may be nested up to 10 levels deep.                                     |
//! | `!`     | When at the start of the glob, this negates the result. Multiple `!` characters negate the glob multiple times.                                                                                     |
//! | `\`     | A backslash character may be used to escape any of the above special characters.                                                                                                                    |
//!
//! ---
//!
//! For detailed usage and API reference, refer to the specific function and struct documentation.
//!
//! For any issues or contributions, please visit the [GitHub repository](https://github.com/shulaoda/fast-glob).
mod brace;
mod glob;

use brace::Pattern;
use glob::glob_match_normal;

/// `Glob` represents a glob pattern matcher with support for multi-pattern matching.
#[derive(Debug, Default, Clone)]
pub struct Glob {
  glob: Vec<u8>,
  pattern: Option<Pattern>,
}

impl PartialEq for Glob {
  fn eq(&self, other: &Self) -> bool {
    self.glob == other.glob
  }
}

impl Eq for Glob {}

impl Glob {
  /// Creates a new `Glob` instance from a given glob pattern.
  ///
  /// Returns `Some(Glob)` if successful, `None` otherwise.
  ///
  /// # Example
  ///
  /// ```
  /// use fast_glob::Glob;
  ///
  /// let glob = Glob::new("*.txt");
  /// assert!(glob.is_some());
  /// ```
  pub fn new(glob: &str) -> Self {
    let mut value = Vec::with_capacity(glob.len() + 2);
    value.push(b'{');
    value.extend(glob.as_bytes());
    value.push(b'}');

    let pattern = Pattern::new(&value[1..value.len() - 1]);
    if let Some(mut pattern) = Pattern::new(&value[1..value.len() - 1]) {
      pattern.value.extend_from_slice(&value[1..value.len() - 1]);
      pattern.branch.push((0, 1));
      pattern.shadow.push((0, 0));
    }

    Glob {
      glob: value,
      pattern,
    }
  }

  /// Adds a new glob pattern to match against.
  ///
  /// Returns `true` if the pattern was successfully added, `false` otherwise.
  ///
  /// # Example
  ///
  /// ```
  /// use fast_glob::Glob;
  ///
  /// let mut glob = Glob::default();
  /// assert!(glob.add("*.txt"));
  /// ```
  //   pub fn add(&mut self, glob: &str) -> bool {
  //     if self.glob.len() == 0 {
  //       if let Some(c) = Self::new(glob) {
  //         *self = c;
  //         return true;
  //       }
  //       return false;
  //     }

  //     let glob = glob.as_bytes();
  //     if let Some(branch) = Pattern::parse(glob) {
  //       self.pattern.branch[0].1 += 1;
  //       self.pattern.branch.extend(branch);
  //       self.glob.reserve_exact(glob.len() + 1);

  //       let index = self.glob.len() - 1;
  //       self.glob[index] = b',';
  //       self.glob.extend(glob);
  //       self.glob.push(b'}');

  //       return true;
  //     }
  //     false
  //   }

  /// Checks if any of the glob patterns matches the given path.
  ///
  /// Returns `true` if a match is found, `false` otherwise.
  ///
  /// # Example
  ///
  /// ```
  /// use fast_glob::Glob;
  ///
  /// let mut glob = Glob::new("*.txt").unwrap();
  /// assert!(glob.is_match("file.txt"));
  /// ```
  pub fn is_match(&self, path: &str) -> bool {
    match self.pattern.clone() {
      Some(mut pattern) => {
        let mut flag = false;
        loop {
          let (result, longest_index) = glob_match_normal(&pattern.value, path.as_bytes());
          if result || !pattern.trigger(&self.glob, longest_index) {
            if flag {
              pattern.restore();
              pattern.track(&self.glob);
            }
            return result;
          }
          flag = true;
        }
      }
      None => {
        let (matched, _) = glob_match_normal(&self.glob, path.as_bytes());
        matched
      }
    }
  }
}

/// Performs glob pattern matching for a simple glob pattern.
///
/// Returns `true` if `glob` matches `path`, `false` otherwise.
///
/// # Example
///
/// ```
/// use fast_glob::glob_match;
///
/// let glob = "**/*.txt";
/// let path = "file.txt";
///
/// assert!(glob_match(glob, path));
/// ```
pub fn glob_match(glob: &str, path: &str) -> bool {
  glob_match_normal(glob.as_bytes(), path.as_bytes()).0
}

/// Performs glob pattern matching for a glob pattern with brace expansion.
///
/// Returns `true` if `glob` matches `path`, `false` otherwise.
///
/// # Example
///
/// ```
/// use fast_glob::glob_match_with_brace;
///
/// let glob = "some/**/{the,crazy}/?*.{png,txt}";
/// let path = "some/a/bigger/path/to/the/crazy/needle.txt";
///
/// assert!(glob_match_with_brace(glob, path));
/// ```
pub fn glob_match_with_brace(glob: &str, path: &str) -> bool {
  let glob = glob.as_bytes();
  let path = path.as_bytes();

  if let Some(pattern) = &mut Pattern::new(glob) {
    if pattern.branch.is_empty() {
      return glob_match_normal(glob, path).0;
    }

    loop {
      let (result, longest_index) = glob_match_normal(&pattern.value, path);

      if result || !pattern.trigger(glob, longest_index) {
        return result;
      }
    }
  }
  false
}
