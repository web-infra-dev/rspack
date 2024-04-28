/*
 * The following code is modified based on
 * https://github.com/swc-project/swc/blob/7114530e16467ce1f74a1ececacbda68e70bc38a/crates/swc/src/dropped_comments_preserver.rs
 *
 * Copyright (c) 2021 kdy1(Donny/강동윤), kwonoj(OJ Kwon), XiNiHa(Cosmo Shin (신의하)), beaumontjonathan(Jonathan Beaumont)
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
use swc_core::common::{
  comments::{Comment, Comments},
  BytePos, Span, DUMMY_SP,
};
use swc_core::ecma::ast::{Module, Script};
use swc_core::ecma::visit::{as_folder, noop_visit_mut_type, Fold, VisitMut, VisitMutWith};
use swc_node_comments::SwcComments;

/// Preserves comments that would otherwise be dropped.
///
/// If during compilation an ast node associated with
/// a comment is dropped, the comment will not appear in the final emitted
/// output. This can create problems in the JavaScript ecosystem, particularly
/// around istanbul coverage and other tooling that relies on comment
/// directives.
///
/// This transformer shifts orphaned comments to the next closest known span
/// while making a best-effort to preserve the "general orientation" of
/// comments.

pub fn dropped_comments_preserver(comments: Option<SwcComments>) -> impl Fold + VisitMut {
  as_folder(DroppedCommentsPreserver {
    comments,
    is_first_span: true,
    known_spans: Vec::new(),
  })
}

struct DroppedCommentsPreserver {
  comments: Option<SwcComments>,
  is_first_span: bool,
  known_spans: Vec<Span>,
}

type CommentEntries = Vec<(BytePos, Vec<Comment>)>;

impl VisitMut for DroppedCommentsPreserver {
  noop_visit_mut_type!();

  fn visit_mut_module(&mut self, module: &mut Module) {
    module.visit_mut_children_with(self);
    self
      .known_spans
      .sort_by(|span_a, span_b| span_a.lo.cmp(&span_b.lo));
    self.shift_comments_to_known_spans();
  }

  fn visit_mut_script(&mut self, script: &mut Script) {
    script.visit_mut_children_with(self);
    self
      .known_spans
      .sort_by(|span_a, span_b| span_a.lo.cmp(&span_b.lo));
    self.shift_comments_to_known_spans();
  }

  fn visit_mut_span(&mut self, span: &mut Span) {
    if span.is_dummy() || self.is_first_span {
      self.is_first_span = false;
      return;
    }

    self.known_spans.push(*span);
    span.visit_mut_children_with(self)
  }
}

impl DroppedCommentsPreserver {
  fn shift_comments_to_known_spans(&self) {
    if let Some(comments) = &self.comments {
      let trailing_comments = self.shift_leading_comments(comments);

      self.shift_trailing_comments(trailing_comments);
    }
  }

  /// We'll be shifting all comments to known span positions, so drain the
  /// current comments first to limit the amount of look ups needed into
  /// the hashmaps.
  ///
  /// This way, we only need to take the comments once, and then add them back
  /// once.
  fn collect_existing_comments(&self, comments: &SwcComments) -> CommentEntries {
    let (leading_comments, trailing_comments) = (&comments.leading, &comments.trailing);
    let mut existing_comments: CommentEntries = leading_comments
      .iter()
      .map(|c| (*c.key(), c.value().clone()))
      .chain(
        trailing_comments
          .iter()
          .map(|c| (*c.key(), c.value().clone())),
      )
      .collect();

    leading_comments.clear();
    trailing_comments.clear();

    existing_comments.sort_by(|(bp_a, _), (bp_b, _)| bp_a.cmp(bp_b));

    existing_comments
  }

  /// Shift all comments to known leading positions.
  /// This prevents trailing comments from ending up associated with
  /// nodes that will not emit trailing comments, while
  /// preserving any comments that might show up after all code positions.
  ///
  /// This maintains the highest fidelity between existing comment positions
  /// of pre and post compiled code.
  fn shift_leading_comments(&self, comments: &SwcComments) -> CommentEntries {
    let mut existing_comments = self.collect_existing_comments(comments);

    existing_comments.sort_by(|(bp_a, _), (bp_b, _)| bp_a.cmp(bp_b));

    for span in self.known_spans.iter() {
      let cut_point = existing_comments.partition_point(|(bp, _)| *bp <= span.lo);
      let collected_comments = existing_comments
        .drain(..cut_point)
        .flat_map(|(_, c)| c)
        .collect::<Vec<Comment>>();
      self
        .comments
        .add_leading_comments(span.lo, collected_comments)
    }

    existing_comments
  }

  /// These comments trail all known span lo byte positions.
  /// Therefore, by shifting them to trail the highest known hi position, we
  /// ensure that any remaining trailing comments are emitted in a
  /// similar location
  fn shift_trailing_comments(&self, remaining_comment_entries: CommentEntries) {
    let last_trailing = self
      .known_spans
      .iter()
      .max_by_key(|span| span.hi)
      .cloned()
      .unwrap_or(DUMMY_SP);

    self.comments.add_trailing_comments(
      last_trailing.hi,
      remaining_comment_entries
        .into_iter()
        .flat_map(|(_, c)| c)
        .collect(),
    );
  }
}
