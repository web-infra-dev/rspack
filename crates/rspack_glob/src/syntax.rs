use std::fmt;

use smallvec::SmallVec;

const MAX_BRACE_NESTING: usize = 10;
const MAX_BRACE_GROUPS: usize = 10;

/// An error describing why a glob pattern is invalid, returned by [`validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Error {
  /// The kind of invalid construct that was found.
  pub kind: ErrorKind,
  /// Byte offset in the pattern of the offending character.
  pub index: usize,
}

/// The kind of invalid construct described by an [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
  /// A `{` is never closed by a matching `}`.
  UnclosedBrace,
  /// A `[` is never closed by a matching `]`.
  UnclosedBracket,
  /// A `\` at the end of the pattern has no character to escape.
  TrailingBackslash,
  /// Brace expansions nest deeper than the supported 10 levels.
  BraceNestingTooDeep,
  /// A pattern contains more than the supported 10 brace groups.
  TooManyBraceGroups,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let index = self.index;
    match self.kind {
      ErrorKind::UnclosedBrace => write!(
        f,
        "unclosed brace expansion at byte {index}; missing '}}' (to match a literal '{{', escape it as '\\{{' or '[{{]')"
      ),
      ErrorKind::UnclosedBracket => write!(
        f,
        "unclosed character class at byte {index}; missing ']' (to match a literal '[', escape it as '\\[' or '[[]')"
      ),
      ErrorKind::TrailingBackslash => write!(
        f,
        "trailing backslash at byte {index} has no character to escape (to match a literal '\\', use '\\\\')"
      ),
      ErrorKind::BraceNestingTooDeep => write!(
        f,
        "brace expansion at byte {index} nests deeper than the supported {MAX_BRACE_NESTING} levels"
      ),
      ErrorKind::TooManyBraceGroups => write!(
        f,
        "brace expansion at byte {index} exceeds the supported limit of {MAX_BRACE_GROUPS} groups"
      ),
    }
  }
}

impl std::error::Error for Error {}

/// Checks the pattern using fast-glob's validation rules.
pub fn validate(glob: impl AsRef<[u8]>) -> Result<(), Error> {
  let glob = glob.as_ref();
  let mut index = 0;

  // Leading `!` characters negate the glob and are not part of the pattern.
  while index < glob.len() && glob[index] == b'!' {
    index += 1;
  }

  let mut open_braces = SmallVec::<[usize; MAX_BRACE_NESTING]>::new();
  let mut brace_groups = 0;

  while index < glob.len() {
    match glob[index] {
      b'\\' => {
        if index + 1 >= glob.len() {
          return Err(Error {
            kind: ErrorKind::TrailingBackslash,
            index,
          });
        }
        index += 2;
      }
      b'[' => match skip_class(glob, index) {
        Some(next) => index = next,
        None => {
          return Err(Error {
            kind: ErrorKind::UnclosedBracket,
            index,
          });
        }
      },
      b'{' => {
        if open_braces.len() == MAX_BRACE_NESTING {
          return Err(Error {
            kind: ErrorKind::BraceNestingTooDeep,
            index,
          });
        }
        open_braces.push(index);
        brace_groups += 1;
        if brace_groups > MAX_BRACE_GROUPS {
          return Err(Error {
            kind: ErrorKind::TooManyBraceGroups,
            index,
          });
        }
        index += 1;
      }
      // A `}` without a matching `{` is an ordinary character.
      b'}' => {
        open_braces.pop();
        index += 1;
      }
      _ => index += 1,
    }
  }

  if let Some(&index) = open_braces.first() {
    return Err(Error {
      kind: ErrorKind::UnclosedBrace,
      index,
    });
  }

  Ok(())
}

/// Returns the index just past the `]` closing the character class opened by
/// the `[` at `index`, or `None` if the class is unclosed. Mirrors the class
/// parsing in the compiler: an optional `^`/`!` prefix, then the first
/// character is a literal member (so a leading `]` does not close the class),
/// and `\` escapes the next character.
pub(super) fn skip_class(glob: &[u8], index: usize) -> Option<usize> {
  let mut index = index + 1;
  if matches!(glob.get(index), Some(b'^' | b'!')) {
    index += 1;
  }

  let mut first = true;
  loop {
    match glob.get(index)? {
      b']' if !first => return Some(index + 1),
      b'\\' => index += 1,
      _ => {}
    }
    first = false;
    index += 1;
  }
}
