use regex_automata::{
  Anchored, Input, MatchKind,
  dfa::{Automaton, StartKind, dense},
  nfa::thompson,
  util::{primitives::StateID, syntax},
};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

/// An anchored state machine with one accept label per input pattern. Includes
/// precede excludes so reachability can prune a subtree using includes alone.
pub struct PathStateMachine {
  dfa: dense::DFA<Vec<u32>>,
  start: StateID,
  includes: Option<FxHashSet<StateID>>,
}

impl PathStateMachine {
  /// Limit the cost of determinization for user-supplied patterns.
  pub fn new(patterns: &[impl AsRef<str>], include_count: usize) -> Option<Self> {
    const DFA_SIZE_LIMIT: usize = 256 * 1024;
    const BUILD_SIZE_LIMIT: usize = 1024 * 1024;
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
      .build_many(patterns)
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

  pub fn start(&self) -> PathMatchState<'_> {
    PathMatchState {
      program: self,
      state: self.start,
    }
  }
}

/// A cursor can be copied into a child without copying its transition table.
#[derive(Clone, Copy)]
pub struct PathMatchState<'a> {
  program: &'a PathStateMachine,
  state: StateID,
}

impl<'a> PathMatchState<'a> {
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
