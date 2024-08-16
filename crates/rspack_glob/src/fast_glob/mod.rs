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

use crate::MatchOptions;

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
pub fn glob_match_with_brace(glob: &str, path: &str, options: &MatchOptions) -> bool {
  let glob = glob.chars().collect::<Vec<_>>();
  let path = path.chars().collect::<Vec<_>>();

  if let Some(pattern) = &mut Pattern::new(&glob) {
    if pattern.branch.is_empty() {
      return glob_match_normal(&glob, &path, options).0;
    }

    loop {
      let (result, longest_index) = glob_match_normal(&pattern.value, &path, options);

      if result || !pattern.trigger(&glob, longest_index) {
        return result;
      }
    }
  }
  false
}
