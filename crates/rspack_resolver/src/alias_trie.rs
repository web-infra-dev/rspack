//! Byte-trie accelerator for [`crate::ResolveOptions::alias`] matching.
//!
//! Replaces the linear `strip_prefix` scan in `load_alias` with a trie walk:
//! descend the trie one byte of the specifier at a time, collect terminal
//! aliases along the way, and yield them in the order they were originally
//! declared.

use crate::Alias;

/// A successful alias-key match against a specifier.
#[derive(Debug, PartialEq, Eq)]
pub struct AliasMatch {
  /// Index of the entry in the original `Alias` vec, used to preserve the
  /// `aliases.iter()` ordering callers rely on.
  pub(crate) index: usize,
  /// Length in bytes of the matched key (after stripping any `$` suffix).
  /// Lets callers compute the specifier tail without re-running the prefix.
  pub(crate) key_len: usize,
  /// True for `$`-suffixed keys — caller should treat the match as exact.
  pub(crate) is_exact: bool,
}

/// Aliases ending at a trie node. In the rare case multiple aliases share the
/// same key string, the loader is expected to try them in declared order.
type TerminalList = Vec<Terminal>;

#[derive(Debug)]
struct Terminal {
  alias_index: usize,
  key_len: usize,
  is_exact: bool,
}

#[derive(Debug, Default)]
struct Node {
  /// Sparse children indexed by edge byte. Low fanout in practice, linear
  /// scan beats a `[Option<...>; 256]` for memory and cache locality.
  children: Vec<(u8, Box<Self>)>,
  terminals: TerminalList,
}

impl Node {
  fn descend(&self, byte: u8) -> Option<&Self> {
    self
      .children
      .iter()
      .find_map(|(b, n)| (*b == byte).then(|| n.as_ref()))
  }

  fn descend_mut_or_insert(&mut self, byte: u8) -> &mut Self {
    if let Some(pos) = self.children.iter().position(|(b, _)| *b == byte) {
      return &mut self.children[pos].1;
    }
    self.children.push((byte, Box::new(Self::default())));
    &mut self
      .children
      .last_mut()
      .expect("children is non-empty after push")
      .1
  }
}

pub struct AliasTrie {
  root: Node,
}

impl AliasTrie {
  pub(crate) fn build(aliases: &Alias) -> Self {
    let mut root = Node::default();
    for (index, (key, _)) in aliases.iter().enumerate() {
      // `$`-suffixed keys are exact-match aliases — index by the stripped key.
      let (effective, is_exact) = key
        .strip_suffix('$')
        .map_or((key.as_str(), false), |stripped| (stripped, true));
      let mut node = &mut root;
      for byte in effective.as_bytes() {
        node = node.descend_mut_or_insert(*byte);
      }
      node.terminals.push(Terminal {
        alias_index: index,
        key_len: effective.len(),
        is_exact,
      });
    }
    Self { root }
  }

  pub(crate) fn matches(&self, specifier: &str) -> Vec<AliasMatch> {
    let bytes = specifier.as_bytes();
    let mut out = Vec::new();
    collect_terminals(&self.root, bytes, 0, &mut out);
    let mut node = &self.root;
    for (i, byte) in bytes.iter().enumerate() {
      let Some(next) = node.descend(*byte) else {
        break;
      };
      node = next;
      collect_terminals(node, bytes, i + 1, &mut out);
    }
    // Trie walk yields matches by key length; callers expect declared order so
    // they can try AliasValue lists in the order the user wrote them.
    if out.len() > 1 {
      out.sort_unstable_by_key(|m| m.index);
    }
    out
  }
}

fn collect_terminals(node: &Node, bytes: &[u8], consumed: usize, out: &mut Vec<AliasMatch>) {
  if node.terminals.is_empty() {
    return;
  }
  let tail = &bytes[consumed..];
  let tail_empty = tail.is_empty();
  let tail_slash = matches!(tail.first(), Some(b'/' | b'\\'));
  for term in &node.terminals {
    let acceptable = if term.is_exact {
      tail_empty
    } else {
      tail_empty || tail_slash
    };
    if acceptable {
      out.push(AliasMatch {
        index: term.alias_index,
        key_len: term.key_len,
        is_exact: term.is_exact,
      });
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::AliasValue;

  fn aliases(entries: &[(&str, &[&str])]) -> Alias {
    entries
      .iter()
      .map(|(k, vs)| {
        (
          (*k).to_string(),
          vs.iter().map(|v| AliasValue::from(*v)).collect(),
        )
      })
      .collect()
  }

  #[test]
  fn empty_trie_yields_no_matches() {
    let aliases: Alias = Vec::new();
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("anything");
    assert!(matches.is_empty(), "expected no matches, got {matches:?}");
  }

  #[test]
  fn matches_prefix_key_with_trailing_slash() {
    // Alias "react" matches specifier "react/foo" (prefix + slash).
    let aliases = aliases(&[("react", &["./src/react"])]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("react/foo");
    assert_eq!(
      matches,
      vec![AliasMatch {
        index: 0,
        key_len: 5,
        is_exact: false
      }]
    );
  }

  #[test]
  fn matches_prefix_key_with_exact_specifier() {
    // Alias "react" also matches the bare specifier "react".
    let aliases = aliases(&[("react", &["./src/react"])]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("react");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].index, 0);
  }

  #[test]
  fn rejects_prefix_followed_by_non_slash() {
    // "react-dom" must NOT match the "react" prefix alias — tail starts with
    // `-`, failing the SLASH_START filter.
    let aliases = aliases(&[("react", &["./src/react"])]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("react-dom");
    assert!(matches.is_empty(), "expected no matches, got {matches:?}");
  }

  #[test]
  fn accepts_prefix_followed_by_backslash() {
    // SLASH_START accepts `\\` too (Windows paths).
    let aliases = aliases(&[("react", &["./src/react"])]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("react\\foo");
    assert_eq!(matches.len(), 1);
    assert_eq!(matches[0].index, 0);
  }

  #[test]
  fn empty_alias_key_matches_slash_prefixed_specifier() {
    // `("", v)` is enhanced-resolve's emergent "match any slash-prefixed
    // specifier" wildcard. The trie must report it without consuming any
    // bytes of the specifier.
    let aliases = aliases(&[("", &["./redirect"])]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("/foo");
    assert_eq!(
      matches,
      vec![AliasMatch {
        index: 0,
        key_len: 0,
        is_exact: false
      }]
    );
  }

  #[test]
  fn matches_preserve_declared_order_long_before_short() {
    // Same path can match both aliases. The caller (load_alias) tries entries
    // in declared order until one succeeds — so the trie must return them in
    // that order even when the trie naturally encounters the shorter key
    // first during the walk.
    let aliases = aliases(&[("a/long/path", &["alpha"]), ("a", &["bravo"])]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("a/long/path/foo");
    let indices: Vec<_> = matches.iter().map(|m| m.index).collect();
    assert_eq!(indices, vec![0, 1], "got {matches:?}");
  }

  #[test]
  fn duplicate_keys_return_all_terminals_in_declared_order() {
    // Same key string registered multiple times lands on the same trie node
    // with multiple `Terminal` entries (see `TerminalList`). `matches` must
    // surface every duplicate and keep them in declared order, so the loader
    // can try each AliasValue list in the order the user wrote them.
    let aliases = aliases(&[
      ("react", &["./alpha"]),     // idx=0
      ("other", &["./unrelated"]), // idx=1, interleaved to ensure sort isn't a no-op
      ("react", &["./bravo"]),     // idx=2
      ("react", &["./charlie"]),   // idx=3
    ]);
    let trie = AliasTrie::build(&aliases);
    let matches = trie.matches("react/foo");
    let indices: Vec<_> = matches.iter().map(|m| m.index).collect();
    assert_eq!(indices, vec![0, 2, 3], "got {matches:?}");
    assert!(
      matches.iter().all(|m| m.key_len == 5 && !m.is_exact),
      "all duplicates of `react` must share key_len=5 and is_exact=false, got {matches:?}"
    );
  }

  #[test]
  fn dollar_exact_key_rejects_specifier_with_tail() {
    // "b$" is exact-match for "b" only; "b/index" must NOT match.
    let aliases = aliases(&[("b$", &["a/index"])]);
    let trie = AliasTrie::build(&aliases);
    let with_tail = trie.matches("b/index");
    assert!(
      with_tail.is_empty(),
      "exact-match alias should not accept tail, got {with_tail:?}"
    );
    let exact = trie.matches("b");
    assert_eq!(
      exact,
      vec![AliasMatch {
        index: 0,
        key_len: 1,
        is_exact: true
      }]
    );
  }
}
