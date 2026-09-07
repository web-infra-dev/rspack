#[cfg(feature = "plugin")]
pub mod runtime;

use rspack_intern::Atom;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_next_ecma_ast::{
  Ast, CommentKind as NextCommentKind, CommentPosition, Span as NextAstSpan,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RspackComment<'a> {
  pub span: NextAstSpan,
  pub kind: NextCommentKind,
  pub text: &'a str,
}

#[derive(Debug, Default)]
pub struct RspackComments<'a> {
  pub leading: FxHashMap<u32, Vec<RspackComment<'a>>>,
  pub trailing: FxHashMap<u32, Vec<RspackComment<'a>>>,
}

impl<'a> RspackComments<'a> {
  pub fn from_ast(ast: &'a Ast<'a>) -> Self {
    let source = ast.source();
    let mut comments = Self::default();
    for comment in ast.comments() {
      let value = ast.get_utf8(comment.value(source.as_bytes()));
      let item = RspackComment {
        span: comment.span,
        kind: comment.kind,
        text: value,
      };
      match comment.position {
        CommentPosition::Leading => comments
          .leading
          .entry(comment.attached_to)
          .or_default()
          .push(item),
        CommentPosition::Trailing => comments
          .trailing
          .entry(comment.attached_to)
          .or_default()
          .push(item),
      }
    }
    comments
  }

  pub fn has_flag(&self, pos: u32, flag: &str) -> bool {
    self.leading.get(&pos).is_some_and(|comment_list| {
      comment_list.iter().any(|comment| {
        comment.kind == NextCommentKind::Block
          && comment.text.lines().any(|line| {
            let line = line.trim_start_matches(['*', ' ']).trim();
            line.len() == flag.len() + 5
              && (line.starts_with("#__") || line.starts_with("@__"))
              && line.ends_with("__")
              && flag == &line[3..line.len() - 2]
          })
      })
    })
  }
}

pub fn get_swc_next_comments(
  comments: &RspackComments<'_>,
  lo: u32,
  hi: u32,
) -> Vec<(bool, String)> {
  let mut result = vec![];
  let mut visited = FxHashSet::<NextAstSpan>::default();

  for comment in comments
    .leading
    .get(&lo)
    .into_iter()
    .chain(comments.trailing.get(&hi))
    .flatten()
  {
    if visited.insert(comment.span) {
      result.push((
        matches!(comment.kind, NextCommentKind::Line),
        comment.text.to_owned(),
      ));
    }
  }

  result
}

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
