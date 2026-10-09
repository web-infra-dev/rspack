// Ported from fast-glob 1.1.2: https://github.com/oxc-project/fast-glob.
// Originally forked from https://github.com/devongovett/glob-match/blob/d5a6c67/src/lib.rs.
// MIT Licensed; see the notice in lib.rs.
use std::path::is_separator;

use smallvec::SmallVec;

use super::syntax::{skip_class, validate};

const MAX_BRACE_GROUPS: usize = 10;

/// Copies the backtracking offsets when trying independent brace branches.
#[derive(Clone, Debug, Default)]
struct State {
  path_index: usize,
  glob_index: usize,
  brace_depth: usize,

  wildcard: Wildcard,
  globstar: Wildcard,
}

#[derive(Clone, Copy, Debug, Default)]
struct Wildcard {
  glob_index: u32,
  path_index: u32,
  brace_depth: u32,
}

type BraceStack = SmallVec<[(u32, u32); MAX_BRACE_GROUPS]>;

/// Performs glob pattern matching for `glob` against `path`.
///
/// `glob` is expected to be a valid pattern. An invalid pattern — an unclosed
/// `{` or `[`, a trailing `\`, more than 10 brace groups, or brace expansions
/// nested deeper than 10 levels — cannot be reported here and its result is
/// unspecified: typically it matches nothing, and it never matches through
/// `!` negation, but the exact behavior may change between releases. Callers
/// accepting user-written patterns should reject invalid ones up front with
/// [`validate`].
pub fn glob_match(glob: impl AsRef<[u8]>, path: impl AsRef<[u8]>) -> bool {
  let (matched, invalid_pattern) = glob_match_internal(glob.as_ref(), path.as_ref());
  matched && !invalid_pattern
}

/// Returns the match result (with negation applied) alongside whether the
/// pattern was detected as invalid, so tests can check the latter against
/// [`validate`].
fn glob_match_internal(glob: &[u8], path: &[u8]) -> (bool, bool) {
  let mut state = State::default();

  let mut negated = false;
  while state.glob_index < glob.len() && glob[state.glob_index] == b'!' {
    negated = !negated;
    state.glob_index += 1;
  }

  let mut brace_stack = BraceStack::new();
  let mut invalid_pattern = false;
  let match_start = state.glob_index;
  let matched = state.glob_match_from(
    glob,
    path,
    match_start,
    &mut brace_stack,
    &mut invalid_pattern,
  );

  // A negated glob matches every path its pattern does not — for an invalid
  // pattern that would be every path, even when the matcher never reaches
  // the invalid construct (e.g. after an early literal mismatch). Gate the
  // negation flip on validity instead of relying on lazy detection.
  if negated && !matched && !invalid_pattern && validate(glob).is_err() {
    return (false, true);
  }

  (negated ^ matched, invalid_pattern)
}

#[inline(always)]
fn unescape(c: &mut u8, glob: &[u8], state: &mut State, invalid_pattern: &mut bool) -> bool {
  if *c == b'\\' {
    state.glob_index += 1;
    if state.glob_index >= glob.len() {
      // A trailing backslash has nothing to escape.
      *invalid_pattern = true;
      return false;
    }
    *c = match glob[state.glob_index] {
      b'a' => b'\x61',
      b'b' => b'\x08',
      b'n' => b'\n',
      b'r' => b'\r',
      b't' => b'\t',
      c => c,
    }
  }
  true
}

impl State {
  #[inline(always)]
  fn backtrack(&mut self) {
    self.glob_index = self.wildcard.glob_index as usize;
    self.path_index = self.wildcard.path_index as usize;
    self.brace_depth = self.wildcard.brace_depth as usize;
  }

  #[inline(always)]
  fn skip_globstars(&mut self, glob: &[u8]) {
    let mut glob_index = self.glob_index + 2;

    while glob_index + 4 <= glob.len() && &glob[glob_index..glob_index + 4] == b"/**/" {
      glob_index += 3;
    }

    if &glob[glob_index..] == b"/**" {
      glob_index += 3;
    }

    self.glob_index = glob_index - 2;
  }

  #[inline(always)]
  fn skip_to_separator(&mut self, path: &[u8], is_end_invalid: bool) {
    if self.path_index == path.len() {
      self.wildcard.path_index += 1;
      return;
    }

    let mut path_index = self.path_index;
    while path_index < path.len() && !is_separator(path[path_index] as char) {
      path_index += 1;
    }

    if is_end_invalid || path_index != path.len() {
      path_index += 1;
    }

    self.wildcard.path_index = path_index as u32;
    self.globstar = self.wildcard;
  }

  #[inline(always)]
  fn skip_branch(&mut self, glob: &[u8]) {
    let end_brace_depth = self.brace_depth - 1;
    while self.glob_index < glob.len() {
      match glob[self.glob_index] {
        b'{' => self.brace_depth += 1,
        b'}' => {
          self.brace_depth -= 1;
          if self.brace_depth == end_brace_depth {
            self.glob_index += 1;
            return;
          }
        }
        b'[' => {
          // An unclosed class swallows the rest of the glob.
          self.glob_index = skip_class(glob, self.glob_index).unwrap_or(glob.len());
          continue;
        }
        b'\\' => self.glob_index += 1,
        _ => (),
      }
      self.glob_index += 1;
    }
  }

  #[inline(always)]
  fn skip_branch_ends(&mut self, glob: &[u8]) {
    while self.brace_depth > 0 && matches!(glob.get(self.glob_index), Some(b',' | b'}')) {
      self.skip_branch(glob);
    }
  }

  fn match_brace_branch(
    &self,
    glob: &[u8],
    path: &[u8],
    open_brace_index: usize,
    branch_index: usize,
    brace_stack: &mut BraceStack,
    invalid_pattern: &mut bool,
  ) -> bool {
    // Gracefully reject patterns with more groups than BraceStack capacity.
    if brace_stack.len() == MAX_BRACE_GROUPS {
      *invalid_pattern = true;
      return false;
    }

    brace_stack.push((open_brace_index as u32, branch_index as u32));
    let mut branch_state = self.clone();
    branch_state.glob_index = branch_index;
    // The stack also contains choices for earlier sequential groups, so
    // derive syntactic nesting from the parent state instead of its length.
    branch_state.brace_depth = self.brace_depth + 1;

    let matched =
      branch_state.glob_match_from(glob, path, branch_index, brace_stack, invalid_pattern);

    brace_stack.pop();

    matched
  }

  fn match_brace(
    &mut self,
    glob: &[u8],
    path: &[u8],
    brace_stack: &mut BraceStack,
    invalid_pattern: &mut bool,
  ) -> bool {
    let mut brace_depth = 0;
    let mut has_closing_brace = false;
    let mut matched = false;

    let open_brace_index = self.glob_index;

    let mut branch_index = 0;

    while self.glob_index < glob.len() {
      match glob[self.glob_index] {
        b'{' => {
          brace_depth += 1;
          if brace_depth == 1 {
            branch_index = self.glob_index + 1;
          }
        }
        b'}' => {
          brace_depth -= 1;
          if brace_depth == 0 {
            has_closing_brace = true;
            if self.match_brace_branch(
              glob,
              path,
              open_brace_index,
              branch_index,
              brace_stack,
              invalid_pattern,
            ) {
              matched = true;
            }
            break;
          }
        }
        b',' if brace_depth == 1 => {
          if self.match_brace_branch(
            glob,
            path,
            open_brace_index,
            branch_index,
            brace_stack,
            invalid_pattern,
          ) {
            matched = true;
          }
          branch_index = self.glob_index + 1;
        }
        b'[' => {
          // An unclosed class swallows the rest of the glob,
          // leaving the brace unclosed as well.
          self.glob_index = skip_class(glob, self.glob_index).unwrap_or(glob.len());
          continue;
        }
        b'\\' => self.glob_index += 1,
        _ => (),
      }
      self.glob_index += 1;
    }

    if !has_closing_brace {
      *invalid_pattern = true;
      return false;
    }

    matched
  }

  #[inline(always)]
  fn glob_match_from(
    &mut self,
    glob: &[u8],
    path: &[u8],
    match_start: usize,
    brace_stack: &mut BraceStack,
    invalid_pattern: &mut bool,
  ) -> bool {
    while self.glob_index < glob.len() || self.path_index < path.len() {
      if self.glob_index < glob.len() {
        match glob[self.glob_index] {
          b'*' => {
            let is_globstar = self.glob_index + 1 < glob.len() && glob[self.glob_index + 1] == b'*';
            if is_globstar {
              self.skip_globstars(glob);
            }

            self.wildcard.glob_index = self.glob_index as u32;
            self.wildcard.path_index = self.path_index as u32 + 1;
            self.wildcard.brace_depth = self.brace_depth as u32;

            let mut in_globstar = false;
            if is_globstar {
              self.glob_index += 2;

              // A selected brace branch ends at `,` or `}`, but
              // brace expansion makes the suffix after the group
              // logically adjacent to this branch. Resolve those
              // boundaries before deciding whether `**` occupies
              // a complete path segment.
              let mut after_globstar = self.clone();
              after_globstar.skip_branch_ends(glob);
              let is_end_invalid = after_globstar.glob_index < glob.len();

              if (self.glob_index.saturating_sub(match_start) < 3
                || glob[self.glob_index - 3] == b'/')
                && (after_globstar.glob_index >= glob.len()
                  || glob[after_globstar.glob_index] == b'/')
              {
                self.glob_index = after_globstar.glob_index;
                self.brace_depth = after_globstar.brace_depth;
                if is_end_invalid {
                  self.glob_index += 1;
                }

                self.skip_to_separator(path, is_end_invalid);
                in_globstar = true;
              }
            } else {
              self.glob_index += 1;
            }

            if !in_globstar
              && self.path_index < path.len()
              && is_separator(path[self.path_index] as char)
            {
              self.wildcard = self.globstar;
            }

            continue;
          }
          b'?' if self.path_index < path.len() && !is_separator(path[self.path_index] as char) => {
            self.glob_index += 1;
            self.path_index += 1;
            continue;
          }
          b'[' if self.path_index < path.len() => {
            self.glob_index += 1;

            let mut negated = false;
            if self.glob_index < glob.len() && matches!(glob[self.glob_index], b'^' | b'!') {
              negated = true;
              self.glob_index += 1;
            }

            let mut first = true;
            let mut is_match = false;
            let c = path[self.path_index];
            while self.glob_index < glob.len() && (first || glob[self.glob_index] != b']') {
              let mut low = glob[self.glob_index];
              if !unescape(&mut low, glob, self, invalid_pattern) {
                return false;
              }

              self.glob_index += 1;

              let high = if self.glob_index + 1 < glob.len()
                && glob[self.glob_index] == b'-'
                && glob[self.glob_index + 1] != b']'
              {
                self.glob_index += 1;

                let mut high = glob[self.glob_index];
                if !unescape(&mut high, glob, self, invalid_pattern) {
                  return false;
                }

                self.glob_index += 1;
                high
              } else {
                low
              };

              if low <= c && c <= high {
                is_match = true;
              }

              first = false;
            }

            if self.glob_index >= glob.len() {
              *invalid_pattern = true;
              return false;
            }

            self.glob_index += 1;
            if is_match != negated && !is_separator(c as char) {
              self.path_index += 1;
              continue;
            }
          }
          b'{' => {
            if let Some((_, branch_index)) = brace_stack
              .iter()
              .find(|(open_brace_index, _)| *open_brace_index == self.glob_index as u32)
            {
              self.glob_index = *branch_index as usize;
              self.brace_depth += 1;
              continue;
            }
            return self.match_brace(glob, path, brace_stack, invalid_pattern);
          }
          b',' | b'}' if self.brace_depth > 0 => {
            self.skip_branch(glob);
            continue;
          }
          mut c if self.path_index < path.len() => {
            if !unescape(&mut c, glob, self, invalid_pattern) {
              return false;
            }

            let is_match = if c == b'/' {
              is_separator(path[self.path_index] as char)
            } else {
              path[self.path_index] == c
            };

            if is_match {
              self.glob_index += 1;
              self.path_index += 1;

              if c == b'/' {
                self.wildcard = self.globstar;
              }

              continue;
            }
          }
          _ => {}
        }
      }

      if self.wildcard.path_index > 0 && self.wildcard.path_index <= path.len() as u32 {
        self.backtrack();
        continue;
      }

      return false;
    }

    true
  }
}
