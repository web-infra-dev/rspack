// Modified from fast-glob 1.1.2: https://github.com/oxc-project/fast-glob.
// Originally forked from https://github.com/devongovett/glob-match/blob/d5a6c67/src/lib.rs.
// MIT Licensed; see the notice in lib.rs.
use std::{borrow::Cow, path::is_separator};

use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::{GlobOptions, syntax::skip_class};

pub(super) type States = SmallVec<[usize; 8]>;

#[derive(Debug)]
pub(super) enum Instruction {
  Accept,
  // Byte, continuation, explicit leading dot, source byte offset.
  Literal(u8, usize, bool, usize),
  Any(usize),
  Class {
    ranges: SmallVec<[(u8, u8); 4]>,
    negated: bool,
    next: usize,
  },
  Split(States),
  Star(usize),
  GlobStar,
  GlobStarDirectory {
    next: usize,
    inside: usize,
  },
  InsideGlobStar(usize),
}

pub(super) type Instructions = SmallVec<[Instruction; 16]>;

pub(super) struct Program<'a> {
  pub(super) instructions: &'a [Instruction],
  pub(super) negated: bool,
  pub(super) options: GlobOptions,
}

impl Program<'_> {
  pub(super) fn compile(glob: &[u8]) -> (Instructions, usize, bool) {
    let mut index = 0;
    let mut negated = false;
    while glob.get(index) == Some(&b'!') {
      negated = !negated;
      index += 1;
    }
    let mut compiler = Compiler {
      glob,
      instructions: smallvec::smallvec![Instruction::Accept],
      indices: FxHashMap::default(),
      pending: SmallVec::new(),
    };
    let start = compiler.intern(Position {
      index,
      brace_depth: 0,
      match_start: index,
      component_start: true,
    });
    // A worklist avoids recursion proportional to the pattern length. Only
    // syntactic brace nesting is bounded by fast-glob's validation rules.
    while let Some((id, position)) = compiler.pending.pop() {
      compiler.instructions[id] = compiler.compile_position(id, position);
    }
    (compiler.instructions, start, negated)
  }

  pub(super) fn start(&self, start: usize) -> States {
    let mut scratch = Scratch::new(self.instructions.len());
    scratch.stack.push(start);
    let mut states = States::new();
    self.close(&mut scratch, &mut states);
    states
  }

  pub(super) fn is_match(&self, states: &States) -> bool {
    self.negated ^ states.contains(&0)
  }

  pub(super) fn can_match(&self, states: &States) -> bool {
    if self.negated {
      // A terminal globstar accepts every suffix, making its complement empty.
      self.options.require_literal_leading_dot
        || !states
          .iter()
          .any(|&id| matches!(self.instructions[id], Instruction::GlobStar))
    } else {
      !states.is_empty()
    }
  }

  /// The common literal prefix of every active branch, already unescaped.
  pub(super) fn literal_prefix<'a>(
    &self,
    initial: &States,
    source: &Cow<'a, [u8]>,
  ) -> Cow<'a, [u8]> {
    if self.negated {
      return Cow::Borrowed(&[]);
    }
    let mut states = initial.clone();
    let mut scratch = Scratch::new(self.instructions.len());
    let mut range = 0..0;
    let mut decoded = None::<Vec<u8>>;
    // Literal edges are acyclic. Stop at acceptance or any dynamic transition.
    while let Some(&first) = states.first() {
      let Instruction::Literal(byte, _, _, offset) = self.instructions[first] else {
        break;
      };
      if !states.iter().all(
        |&id| matches!(self.instructions[id], Instruction::Literal(value, _, _, _) if value == byte),
      ) {
        break;
      }
      if let Some(decoded) = &mut decoded {
        decoded.push(byte);
      } else {
        if range.is_empty() {
          range = offset..offset;
        }
        if source.get(range.end) == Some(&byte) {
          range.end += 1;
        } else {
          let mut bytes = source[range.clone()].to_vec();
          bytes.push(byte);
          decoded = Some(bytes);
        }
      }
      for &id in &states {
        if let Instruction::Literal(_, next, _, _) = self.instructions[id] {
          scratch.stack.push(next);
        }
      }
      self.close(&mut scratch, &mut states);
    }
    match decoded {
      Some(bytes) => Cow::Owned(bytes),
      None => match source {
        Cow::Borrowed(bytes) => Cow::Borrowed(&bytes[range]),
        Cow::Owned(bytes) => Cow::Owned(bytes[range].to_vec()),
      },
    }
  }

  /// Enumerate literal next-component alternatives, or require a directory read.
  #[cfg(feature = "fs")]
  pub(super) fn next_components(&self, states: &States) -> Option<Vec<String>> {
    if self.negated || !self.options.case_sensitive {
      return None;
    }
    let mut pending = states
      .iter()
      .map(|&id| (id, Vec::new()))
      .collect::<Vec<_>>();
    let mut names = Vec::new();
    let mut visited = 0;
    while let Some((id, mut name)) = pending.pop() {
      visited += 1;
      // Large brace products fall back to matching directory entries instead
      // of eagerly allocating every possible name.
      if visited > 4096 || pending.len() + names.len() > 256 {
        return None;
      }
      match &self.instructions[id] {
        Instruction::Accept | Instruction::Literal(b'/', _, _, _) => {
          if !name.is_empty() {
            names.push(String::from_utf8(name).ok()?);
          }
        }
        Instruction::Literal(byte, next, _, _) => {
          name.push(*byte);
          pending.push((*next, name));
        }
        Instruction::Split(branches) => {
          pending.extend(branches.iter().map(|&next| (next, name.clone())));
        }
        _ => return None,
      }
    }
    names.sort_unstable();
    names.dedup();
    Some(names)
  }

  pub(super) fn consume(
    &self,
    initial: &States,
    path: &[u8],
    mut component_start: bool,
  ) -> (States, bool) {
    let mut states = initial.clone();
    if path.is_empty() || states.is_empty() {
      return (states, component_start);
    }
    let mut next_states = States::new();
    let mut scratch = Scratch::new(self.instructions.len());
    for &byte in path {
      let separator = is_separator(byte as char);
      let hidden = self.options.require_literal_leading_dot && component_start && byte == b'.';
      for &id in &states {
        let next = match &self.instructions[id] {
          Instruction::Literal(literal, next, explicit_dot, _)
            if if *literal == b'/' {
              separator
            } else {
              *literal == byte
            } && (!hidden || *explicit_dot) =>
          {
            Some(*next)
          }
          Instruction::Any(next) if !separator && !hidden => Some(*next),
          Instruction::Class {
            ranges,
            negated,
            next,
          } if !separator
            && !hidden
            && (ranges
              .iter()
              .any(|&(low, high)| low <= byte && byte <= high)
              != *negated) =>
          {
            Some(*next)
          }
          Instruction::Star(_) if !separator && !hidden => Some(id),
          Instruction::GlobStar if !hidden => Some(id),
          Instruction::GlobStarDirectory { inside, .. } if !hidden => {
            Some(if separator { id } else { *inside })
          }
          Instruction::InsideGlobStar(boundary) if !hidden => {
            Some(if separator { *boundary } else { id })
          }
          _ => None,
        };
        if let Some(next) = next {
          scratch.stack.push(next);
        }
      }
      self.close(&mut scratch, &mut next_states);
      std::mem::swap(&mut states, &mut next_states);
      component_start = separator;
      if states.is_empty() {
        break;
      }
    }
    (states, component_start)
  }

  /// Keeps consuming states and traverses epsilon edges once per input byte.
  fn close(&self, scratch: &mut Scratch, states: &mut States) {
    states.clear();
    if scratch.generation == usize::MAX {
      scratch.seen.fill(0);
      scratch.generation = 0;
    }
    scratch.generation += 1;
    while let Some(id) = scratch.stack.pop() {
      if scratch.seen[id] == scratch.generation {
        continue;
      }
      scratch.seen[id] = scratch.generation;
      match &self.instructions[id] {
        Instruction::Split(branches) => scratch.stack.extend(branches.iter().copied()),
        Instruction::Star(next) | Instruction::GlobStarDirectory { next, .. } => {
          states.push(id);
          scratch.stack.push(*next);
        }
        Instruction::GlobStar => {
          states.push(id);
          scratch.stack.push(0);
        }
        _ => states.push(id),
      }
    }
  }
}

struct Scratch {
  seen: SmallVec<[usize; 16]>,
  generation: usize,
  stack: States,
}

impl Scratch {
  fn new(len: usize) -> Self {
    Self {
      seen: std::iter::repeat_n(0, len).collect(),
      generation: 0,
      stack: States::new(),
    }
  }
}

/// Copying these three offsets lets brace alternatives compile independently.
#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct Position {
  index: usize,
  brace_depth: usize,
  match_start: usize,
  component_start: bool,
}

struct Compiler<'a> {
  glob: &'a [u8],
  instructions: Instructions,
  indices: FxHashMap<Position, usize>,
  pending: SmallVec<[(usize, Position); 16]>,
}

impl Compiler<'_> {
  fn intern(&mut self, mut position: Position) -> usize {
    self.skip_branch_ends(&mut position);
    if position.index == self.glob.len() {
      return 0;
    }
    if let Some(&id) = self.indices.get(&position) {
      return id;
    }
    let id = self.instructions.len();
    self.indices.insert(position, id);
    self.instructions.push(Instruction::Accept);
    self.pending.push((id, position));
    id
  }

  fn compile_position(&mut self, id: usize, mut position: Position) -> Instruction {
    match self.glob[position.index] {
      b'*' => {
        let is_globstar = self.glob.get(position.index + 1) == Some(&b'*');
        if is_globstar {
          // Port of State::skip_globstars: collapse repeated /** segments.
          let mut index = position.index + 2;
          while self.glob.get(index..index + 4) == Some(b"/**/".as_slice()) {
            index += 3;
          }
          if &self.glob[index..] == b"/**" {
            index += 3;
          }
          position.index = index;
          let mut after = position;
          self.skip_branch_ends(&mut after);
          // As in fast-glob, brace branch ends are transparent when deciding
          // whether ** ends a segment. match_start is the selected branch start.
          if (position.index.saturating_sub(position.match_start) < 3
            || self.glob[position.index - 3] == b'/')
            && (after.index == self.glob.len() || self.glob[after.index] == b'/')
          {
            if after.index == self.glob.len() {
              return Instruction::GlobStar;
            }
            after.index += 1;
            after.component_start = true;
            let next = self.intern(after);
            let inside = self.instructions.len();
            self.instructions.push(Instruction::InsideGlobStar(id));
            return Instruction::GlobStarDirectory { next, inside };
          }
        } else {
          position.index += 1;
        }
        position.component_start = false;
        Instruction::Star(self.intern(position))
      }
      b'?' => {
        position.index += 1;
        position.component_start = false;
        Instruction::Any(self.intern(position))
      }
      b'[' => {
        position.index += 1;
        let negated = matches!(self.glob.get(position.index), Some(b'^' | b'!'));
        position.index += usize::from(negated);
        let mut first = true;
        let mut ranges = SmallVec::new();
        while first || self.glob[position.index] != b']' {
          let low = self.unescape(&mut position.index);
          let high = if self.glob.get(position.index) == Some(&b'-')
            && self.glob.get(position.index + 1) != Some(&b']')
          {
            position.index += 1;
            self.unescape(&mut position.index)
          } else {
            low
          };
          ranges.push((low, high));
          first = false;
        }
        position.index += 1;
        position.component_start = false;
        Instruction::Class {
          ranges,
          negated,
          next: self.intern(position),
        }
      }
      b'{' => {
        let mut index = position.index + 1;
        let mut branch = index;
        let mut depth = 1;
        let mut branches = States::new();
        loop {
          match self.glob[index] {
            b'{' => depth += 1,
            b'}' | b',' if depth == 1 => {
              branches.push(self.intern(Position {
                index: branch,
                brace_depth: position.brace_depth + 1,
                match_start: branch,
                component_start: position.component_start,
              }));
              if self.glob[index] == b'}' {
                break;
              }
              branch = index + 1;
            }
            b'}' => depth -= 1,
            b'[' => {
              index = skip_class(self.glob, index).expect("validated character class");
              continue;
            }
            b'\\' => index += 1,
            _ => {}
          }
          index += 1;
        }
        Instruction::Split(branches)
      }
      _ => {
        let offset = position.index + usize::from(self.glob[position.index] == b'\\');
        let literal = self.unescape(&mut position.index);
        let explicit_dot = position.component_start && literal == b'.';
        position.component_start = literal == b'/';
        Instruction::Literal(literal, self.intern(position), explicit_dot, offset)
      }
    }
  }

  fn skip_branch_ends(&self, position: &mut Position) {
    while position.brace_depth > 0 && matches!(self.glob.get(position.index), Some(b',' | b'}')) {
      let end_depth = position.brace_depth - 1;
      while position.index < self.glob.len() {
        match self.glob[position.index] {
          b'{' => position.brace_depth += 1,
          b'}' => {
            position.brace_depth -= 1;
            if position.brace_depth == end_depth {
              position.index += 1;
              break;
            }
          }
          b'[' => {
            position.index =
              skip_class(self.glob, position.index).expect("validated character class");
            continue;
          }
          b'\\' => position.index += 1,
          _ => {}
        }
        position.index += 1;
      }
    }
  }

  fn unescape(&self, index: &mut usize) -> u8 {
    let mut byte = self.glob[*index];
    *index += 1;
    if byte == b'\\' {
      byte = match self.glob[*index] {
        b'a' => b'a',
        b'b' => b'\x08',
        b'n' => b'\n',
        b'r' => b'\r',
        b't' => b'\t',
        byte => byte,
      };
      *index += 1;
    }
    byte
  }
}
