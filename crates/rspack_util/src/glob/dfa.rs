use std::fmt::Write;

use regex_automata::{
  Anchored, Input, MatchKind,
  dfa::{Automaton, StartKind, dense},
  nfa::thompson,
  util::{primitives::StateID, syntax},
};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

/// A byte-oriented, anchored glob expression ready to be compiled. Literal
/// directory prefixes come from the filesystem and bypass glob parsing.
pub struct GlobExpression(String);

impl GlobExpression {
  /// Patterns with special backtracking semantics can use the existing matcher.
  pub fn new(directory: &str, glob: &str) -> Option<Self> {
    fast_glob::validate(glob).ok()?;
    if glob.starts_with('!') {
      return None;
    }
    let mut regex = String::from("(?-u:\\A");
    for byte in directory.bytes() {
      push_literal(&mut regex, byte);
    }
    let bytes = glob.as_bytes();
    let mut index = 0;
    let mut braces = 0;
    while let Some(&byte) = bytes.get(index) {
      let start = index;
      index += 1;
      match byte {
        b'*' => {
          // The legacy interpreter backtracks across brace alternatives
          // containing stars. Keep those patterns on that implementation.
          if braces != 0 {
            return None;
          }
          if bytes.get(index) == Some(&b'*') {
            index += 1;
            while bytes[index..].starts_with(b"/**/") {
              index += 3;
            }
            if &bytes[index..] == b"/**" {
              index += 3;
            }
            if (start == 0 || bytes[index - 3] == b'/')
              && (index == bytes.len() || bytes[index] == b'/')
            {
              if index == bytes.len() {
                regex.push_str("[\\x00-\\xff]*");
              } else {
                index += 1;
                regex.push_str("(?:[^/]*/)*");
              }
              continue;
            }
          }
          regex.push_str("[^/]*");
        }
        b'?' => regex.push_str("[^/]"),
        b'[' => push_class(&mut regex, bytes, &mut index),
        b'{' => {
          braces += 1;
          regex.push_str("(?:");
        }
        b',' if braces > 0 => regex.push('|'),
        b'}' if braces > 0 => {
          braces -= 1;
          regex.push(')');
        }
        b'\\' => push_literal(&mut regex, escaped_byte(bytes, &mut index)),
        _ => push_literal(&mut regex, byte),
      }
    }
    regex.push_str("\\z)");
    Some(Self(regex))
  }
}

fn push_literal(regex: &mut String, byte: u8) {
  write!(regex, "\\x{byte:02x}").expect("writing to a String cannot fail");
}

fn escaped_byte(bytes: &[u8], index: &mut usize) -> u8 {
  let byte = bytes[*index];
  *index += 1;
  match byte {
    b'b' => b'\x08',
    b'n' => b'\n',
    b'r' => b'\r',
    b't' => b'\t',
    byte => byte,
  }
}

fn class_byte(bytes: &[u8], index: &mut usize) -> u8 {
  let byte = bytes[*index];
  *index += 1;
  if byte == b'\\' {
    escaped_byte(bytes, index)
  } else {
    byte
  }
}

fn push_class(regex: &mut String, bytes: &[u8], index: &mut usize) {
  let negated = matches!(bytes[*index], b'^' | b'!');
  *index += usize::from(negated);
  let mut included = [false; 256];
  loop {
    let low = class_byte(bytes, index);
    let high = if bytes[*index] == b'-' && bytes[*index + 1] != b']' {
      *index += 1;
      class_byte(bytes, index)
    } else {
      low
    };
    for byte in low..=high {
      included[usize::from(byte)] = true;
    }
    if bytes[*index] == b']' {
      *index += 1;
      break;
    }
  }

  regex.push('[');
  let mut runs = (0..=u8::MAX)
    .filter(|&byte| byte != b'/' && included[usize::from(byte)] != negated)
    .peekable();
  if runs.peek().is_none() {
    regex.push_str("^\\x00-\\xff");
  }
  while let Some(low) = runs.next() {
    let mut high = low;
    while let Some(&next) = runs.peek()
      && next == high.saturating_add(1)
    {
      high = next;
      runs.next();
    }
    push_literal(regex, low);
    if high != low {
      regex.push('-');
      push_literal(regex, high);
    }
  }
  regex.push(']');
}

/// An anchored automaton with one accept label per input pattern. Includes
/// precede excludes so reachability can prune a subtree using includes alone.
pub struct GlobDfa {
  dfa: dense::DFA<Vec<u32>>,
  start: StateID,
  includes: Option<FxHashSet<StateID>>,
}

impl GlobDfa {
  /// Limit the cost of determinization for user-supplied patterns.
  pub fn new(patterns: &[GlobExpression], include_count: usize) -> Option<Self> {
    const DFA_SIZE_LIMIT: usize = 256 * 1024;
    const BUILD_SIZE_LIMIT: usize = 1024 * 1024;
    let expressions = patterns
      .iter()
      .map(|pattern| pattern.0.as_str())
      .collect::<Vec<_>>();
    let dfa = dense::Builder::new()
      .configure(
        dense::Config::new()
          .start_kind(StartKind::Anchored)
          .match_kind(MatchKind::All)
          .dfa_size_limit(Some(DFA_SIZE_LIMIT))
          .determinize_size_limit(Some(BUILD_SIZE_LIMIT)),
      )
      .syntax(syntax::Config::new().utf8(false))
      .thompson(
        thompson::Config::new()
          .utf8(false)
          .nfa_size_limit(Some(BUILD_SIZE_LIMIT)),
      )
      .build_many(&expressions)
      .ok()?;
    let start = dfa
      .start_state_forward(&Input::new(b"").anchored(Anchored::Yes))
      .ok()?;
    let includes =
      (include_count < patterns.len()).then(|| reachable_includes(&dfa, start, include_count));
    Some(Self {
      dfa,
      start,
      includes,
    })
  }

  pub fn start(&self) -> GlobDfaState<'_> {
    GlobDfaState {
      program: self,
      state: self.start,
    }
  }
}

/// A cursor can be copied into a child without copying its transition table.
#[derive(Clone, Copy)]
pub struct GlobDfaState<'a> {
  program: &'a GlobDfa,
  state: StateID,
}

impl<'a> GlobDfaState<'a> {
  pub fn advance(mut self, path: &str) -> Self {
    for byte in path.bytes() {
      self.state = self.program.dfa.next_state(self.state, byte);
      if self.program.dfa.is_dead_state(self.state) {
        break;
      }
    }
    self
  }

  pub fn can_include(self) -> bool {
    self.program.includes.as_ref().map_or_else(
      || !self.program.dfa.is_dead_state(self.state),
      |includes| includes.contains(&self.state),
    )
  }

  pub fn matches(self) -> impl Iterator<Item = usize> + 'a {
    matching_patterns(&self.program.dfa, self.state)
  }
}

fn matching_patterns(dfa: &dense::DFA<Vec<u32>>, state: StateID) -> impl Iterator<Item = usize> {
  let end = dfa.next_eoi_state(state);
  let count = if dfa.is_match_state(end) {
    dfa.match_len(end)
  } else {
    0
  };
  (0..count).map(move |index| dfa.match_pattern(end, index).as_usize())
}

fn reachable_includes(
  dfa: &dense::DFA<Vec<u32>>,
  start: StateID,
  include_count: usize,
) -> FxHashSet<StateID> {
  let bytes = dfa
    .byte_classes()
    .representatives(..)
    .filter_map(|unit| unit.as_u8())
    .collect::<Vec<_>>();
  let mut incoming = FxHashMap::<StateID, SmallVec<[StateID; 4]>>::default();
  let mut visited = FxHashSet::default();
  let mut pending = vec![start];
  let mut accepting = Vec::new();
  while let Some(state) = pending.pop() {
    if dfa.is_dead_state(state) || !visited.insert(state) {
      continue;
    }
    if matching_patterns(dfa, state).any(|pattern| pattern < include_count) {
      accepting.push(state);
    }
    let mut destinations = bytes
      .iter()
      .map(|&byte| dfa.next_state(state, byte))
      .collect::<SmallVec<[_; 16]>>();
    destinations.sort_unstable();
    destinations.dedup();
    for next in destinations {
      if !dfa.is_dead_state(next) {
        incoming.entry(next).or_default().push(state);
        pending.push(next);
      }
    }
  }
  visited.clear();
  while let Some(state) = accepting.pop() {
    if visited.insert(state)
      && let Some(predecessors) = incoming.get(&state)
    {
      accepting.extend(predecessors);
    }
  }
  visited
}
