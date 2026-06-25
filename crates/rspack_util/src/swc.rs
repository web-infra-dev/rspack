#[cfg(feature = "plugin")]
pub mod runtime;

use rustc_hash::FxHashSet;
use swc_core::atoms::Atom;
use swc_experimental_ecma_ast::{CommentKind, Comments, Span as AstSpan};

pub fn normalize_custom_filename(source: &str) -> &str {
  if source.starts_with('<') && source.ends_with('>') {
    &source[1..source.len() - 1] // remove '<' and '>' for swc FileName::Custom
  } else {
    source
  }
}

pub fn join_atom<'a, T: Iterator<Item = &'a Atom>>(mut iter: T, separator: &str) -> String {
  let mut ret = String::new();
  if let Some(item) = iter.next() {
    ret.push_str(item);
  }
  for item in iter {
    ret.push_str(separator);
    ret.push_str(item);
  }
  ret
}

#[test]
fn test_normalize_custom_filename() {
  let input = "<custom_filename>";
  let expected_output = "custom_filename";
  assert_eq!(normalize_custom_filename(input), expected_output);
}

pub fn get_swc_comments(comments: &Comments<'_>, lo: u32, hi: u32) -> Vec<(bool, String)> {
  let mut result = vec![];
  let mut visited = FxHashSet::<AstSpan>::default();

  if let Some(comments) = comments.leading.get(&lo) {
    for comment in comments {
      if !visited.insert(comment.span) {
        continue;
      }

      result.push((
        matches!(comment.kind, CommentKind::Line),
        comment.text.to_string(),
      ));
    }
  }

  if let Some(comments) = comments.trailing.get(&hi) {
    for comment in comments {
      if !visited.insert(comment.span) {
        continue;
      }

      result.push((
        matches!(comment.kind, CommentKind::Line),
        comment.text.to_string(),
      ));
    }
  }

  result
}
