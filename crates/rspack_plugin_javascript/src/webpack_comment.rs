use once_cell::sync::Lazy;
use regex::Captures;
use rspack_error::miette::{Diagnostic, Severity};
use rustc_hash::FxHashMap;
use swc_core::common::comments::{CommentKind, Comments};
use swc_core::common::{SourceFile, Span};

use crate::visitors::create_traceable_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebpackComment {
  ChunkName,
  Prefetch,
  Preload,
  Ignore,
  Mode,
}

pub struct WebpackCommentMap(FxHashMap<WebpackComment, String>);

impl WebpackCommentMap {
  fn new() -> Self {
    Self(Default::default())
  }

  fn insert(&mut self, key: WebpackComment, value: String) {
    self.0.insert(key, value);
  }

  pub fn get_webpack_mode(&self) -> Option<&String> {
    self.0.get(&WebpackComment::Mode)
  }

  pub fn get_webpack_chunk_name(&self) -> Option<&String> {
    self.0.get(&WebpackComment::ChunkName)
  }

  pub fn get_webpack_prefetch(&self) -> Option<&String> {
    self.0.get(&WebpackComment::Prefetch)
  }

  pub fn get_webpack_preload(&self) -> Option<&String> {
    self.0.get(&WebpackComment::Preload)
  }

  pub fn get_webpack_ignore(&self) -> Option<bool> {
    self.0.get(&WebpackComment::Ignore).and_then(|item| {
      if item == "true" {
        Some(true)
      } else if item == "false" {
        Some(false)
      } else {
        None
      }
    })
  }
}

fn add_magic_comment_warning(
  source_file: &SourceFile,
  comment_name: &str,
  comment_type: &str,
  captures: &Captures,
  warning_diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  span: Span,
) {
  warning_diagnostics.push(Box::new(
    create_traceable_error(
      "Magic comments parse failed".into(),
      format!(
        "`{comment_name}` expected {comment_type}, but received: {}.",
        captures.get(2).map_or("", |m| m.as_str())
      ),
      source_file,
      span.into(),
    )
    .with_severity(Severity::Warning),
  ))
}

// Using vm.runInNewContext in webpack
// _0 for name
// _1 for "xxx"
// _2 for 'xxx'
// _3 for `xxx`
// _4 for number
// _5 for true/false
// TODO: regexp/array
static WEBPACK_MAGIC_COMMENT_REGEXP: Lazy<regex::Regex> = Lazy::new(|| {
  regex::Regex::new(r#"(?P<_0>webpack[a-zA-Z\d_-]+)\s*:\s*("(?P<_1>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)"|'(?P<_2>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)'|`(?P<_3>(\./)?([\w0-9_\-\[\]\(\)]+/)*?[\w0-9_\-\[\]\(\)]+)`|(?P<_4>[\d.-]+)|(?P<_5>true|false))"#)
    .expect("invalid regex")
});

pub fn try_extract_webpack_magic_comment(
  source_file: &SourceFile,
  comments: &Option<&dyn Comments>,
  import_span: Span,
  span: Span,
  warning_diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
) -> WebpackCommentMap {
  let mut result = WebpackCommentMap::new();
  comments.with_leading(span.lo, |comments| {
    for comment in comments
      .iter()
      .rev()
      .filter(|c| matches!(c.kind, CommentKind::Block))
    {
      for captures in WEBPACK_MAGIC_COMMENT_REGEXP.captures_iter(&comment.text) {
        if let Some(item_name_match) = captures.name("_0") {
          let item_name = item_name_match.as_str();
          match item_name {
            "webpackChunkName" => {
              if let Some(item_value_match) = captures
                .name("_1")
                .or(captures.name("_2"))
                .or(captures.name("_3"))
              {
                result.insert(
                  WebpackComment::ChunkName,
                  item_value_match.as_str().to_string(),
                );
              } else {
                add_magic_comment_warning(
                  source_file,
                  item_name,
                  "a string",
                  &captures,
                  warning_diagnostics,
                  import_span,
                );
              }
            }
            "webpackPrefetch" => {
              if let Some(item_value_match) = captures.name("_4").or(captures.name("_5")) {
                result.insert(
                  WebpackComment::Prefetch,
                  item_value_match.as_str().to_string(),
                );
              } else {
                add_magic_comment_warning(
                  source_file,
                  item_name,
                  "true or a number",
                  &captures,
                  warning_diagnostics,
                  import_span,
                );
              }
            }
            "webpackPreload" => {
              if let Some(item_value_match) = captures.name("_4").or(captures.name("_5")) {
                result.insert(
                  WebpackComment::Preload,
                  item_value_match.as_str().to_string(),
                );
              } else {
                add_magic_comment_warning(
                  source_file,
                  item_name,
                  "true or a number",
                  &captures,
                  warning_diagnostics,
                  import_span,
                );
              }
            }
            "webpackIgnore" => {
              if let Some(item_value_match) = captures.name("_5") {
                result.insert(
                  WebpackComment::Ignore,
                  item_value_match.as_str().to_string(),
                );
              } else {
                add_magic_comment_warning(
                  source_file,
                  item_name,
                  "true or false",
                  &captures,
                  warning_diagnostics,
                  import_span,
                );
              }
            }
            "webpackMode" => {
              if let Some(item_value_match) = captures
                .name("_1")
                .or(captures.name("_2"))
                .or(captures.name("_3"))
              {
                result.insert(WebpackComment::Mode, item_value_match.as_str().to_string());
              } else {
                add_magic_comment_warning(
                  source_file,
                  item_name,
                  "a string",
                  &captures,
                  warning_diagnostics,
                  import_span,
                );
              }
            }
            _ => {
              // TODO: other magic comment
            }
          }
        }
      }
    }
  });
  result
}
