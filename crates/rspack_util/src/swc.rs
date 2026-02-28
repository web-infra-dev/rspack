#[cfg(feature = "plugin")]
pub mod runtime;

use rustc_hash::FxHashSet;
use swc_config::types::BoolOr;
use swc_core::{
  atoms::Atom,
  base::config::JsMinifyCommentOption,
  common::{
    BytePos,
    comments::{Comment, CommentKind, Comments, SingleThreadedComments},
  },
};

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

/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/e6fc5327b1a309eae840fe1ec3a2367adab37430/crates/swc_compiler_base/src/lib.rs#L342
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
pub fn minify_file_comments(
  comments: &SingleThreadedComments,
  preserve_comments: &BoolOr<JsMinifyCommentOption>,
  preserve_annotations: bool,
) {
  match preserve_comments {
    BoolOr::Bool(true) | BoolOr::Data(JsMinifyCommentOption::PreserveAllComments) => {}

    BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments) => {
      let preserve_excl = |_: &BytePos, vc: &mut std::vec::Vec<Comment>| -> bool {
        // Preserve license comments.
        //
        // See https://github.com/terser/terser/blob/798135e04baddd94fea403cfaab4ba8b22b1b524/lib/output.js#L175-L181
        vc.retain(|c: &Comment| {
          c.text.contains("@lic")
            || c.text.contains("@preserve")
            || c.text.contains("@copyright")
            || c.text.contains("@cc_on")
            || (preserve_annotations
              && (c.text.contains("__PURE__")
                || c.text.contains("__INLINE__")
                || c.text.contains("__NOINLINE__")
                || c.text.contains("@vite-ignore")))
            || (c.kind == CommentKind::Block && c.text.starts_with('!'))
        });
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();

      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }

    BoolOr::Bool(false) => {
      let (mut l, mut t) = comments.borrow_all_mut();
      l.clear();
      t.clear();
    }
    BoolOr::Data(JsMinifyCommentOption::PreserveRegexComments { regex }) => {
      let preserve_excl = |_: &BytePos, vc: &mut std::vec::Vec<Comment>| -> bool {
        // Preserve comments that match the regex
        //
        // See https://github.com/terser/terser/blob/798135e04baddd94fea403cfaab4ba8b22b1b524/lib/output.js#L286
        vc.retain(|c: &Comment| regex.find(&c.text).is_some());
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();
      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }
  }
}

pub fn get_swc_comments(
  comments: Option<&dyn Comments>,
  lo: BytePos,
  hi: BytePos,
) -> Vec<(bool, String)> {
  let mut result = vec![];
  let mut visited = FxHashSet::default();

  comments.with_leading(lo, |comments| {
    for comment in comments {
      if !visited.insert(comment.span) {
        continue;
      }

      result.push((
        matches!(comment.kind, CommentKind::Line),
        comment.text.to_string(),
      ));
    }
  });

  comments.with_trailing(hi, |comments| {
    for comment in comments {
      if !visited.insert(comment.span) {
        continue;
      }

      result.push((
        matches!(comment.kind, CommentKind::Line),
        comment.text.to_string(),
      ));
    }
  });

  result
}
