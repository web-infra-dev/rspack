#[cfg(feature = "plugin")]
pub mod runtime;

use rustc_hash::FxHashSet;
use swc_next_allocator::{hash_map::HashMap as ArenaHashMap, vec::Vec as ArenaVec};
use swc_next_ecma_ast::{
  Ast, Comment, CommentKind as NextCommentKind, CommentPosition, Span as NextAstSpan,
};

use crate::atom::Atom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RspackComment<'a> {
  pub span: NextAstSpan,
  pub kind: NextCommentKind,
  pub text: &'a str,
}

pub struct RspackComments<'a> {
  ast: &'a Ast<'a>,
  comments: ArenaHashMap<'a, u32, ArenaVec<'a, Comment>>,
}

impl<'a> RspackComments<'a> {
  pub fn from_ast(ast: &'a Ast<'a>) -> Self {
    Self {
      ast,
      comments: ast.create_comments_map(),
    }
  }

  fn at(
    &self,
    pos: u32,
    position: CommentPosition,
  ) -> impl DoubleEndedIterator<Item = RspackComment<'_>> {
    self
      .comments
      .get(&pos)
      .into_iter()
      .flat_map(|comments| comments.iter())
      .filter(move |comment| comment.position == position)
      .map(|comment| RspackComment {
        span: comment.span,
        kind: comment.kind,
        text: self
          .ast
          .get_utf8(comment.value(self.ast.source().as_bytes())),
      })
  }

  pub fn leading(&self, pos: u32) -> impl DoubleEndedIterator<Item = RspackComment<'_>> {
    self.at(pos, CommentPosition::Leading)
  }

  pub fn trailing(&self, pos: u32) -> impl DoubleEndedIterator<Item = RspackComment<'_>> {
    self.at(pos, CommentPosition::Trailing)
  }

  pub fn has_flag(&self, pos: u32, flag: &str) -> bool {
    self.leading(pos).any(|comment| {
      comment.kind == NextCommentKind::Block
        && comment.text.lines().any(|line| {
          let line = line.trim_start_matches(['*', ' ']).trim();
          line.len() == flag.len() + 5
            && (line.starts_with("#__") || line.starts_with("@__"))
            && line.ends_with("__")
            && flag == &line[3..line.len() - 2]
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

  for comment in comments.leading(lo).chain(comments.trailing(hi)) {
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
