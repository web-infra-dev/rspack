use std::sync::LazyLock;

use itertools::Itertools;
use regex::Captures;
use rspack_error::miette::{Diagnostic, Severity};
use rspack_regex::RspackRegex;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::comments::{Comment, CommentKind, Comments};
use swc_core::common::{SourceFile, Span};

use crate::visitors::create_traceable_error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebpackComment {
  ChunkName,
  Prefetch,
  Preload,
  Ignore,
  FetchPriority,
  IncludeRegexp,
  IncludeFlags,
  ExcludeRegexp,
  ExcludeFlags,
  Mode,
  Exports,
}

#[derive(Debug)]
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

  pub fn get_fetch_priority(&self) -> Option<&String> {
    self.0.get(&WebpackComment::FetchPriority)
  }

  pub fn get_webpack_include(&self) -> Option<RspackRegex> {
    self.0.get(&WebpackComment::IncludeRegexp).map(|expr| {
      let flags = self
        .0
        .get(&WebpackComment::IncludeFlags)
        .map(|x| x.as_str())
        .unwrap_or_default();

      RspackRegex::with_flags(expr, flags).unwrap_or_else(|_| {
        // test when capture
        unreachable!();
      })
    })
  }

  pub fn get_webpack_exclude(&self) -> Option<RspackRegex> {
    self.0.get(&WebpackComment::ExcludeRegexp).map(|expr| {
      let flags = self
        .0
        .get(&WebpackComment::ExcludeFlags)
        .map(|x| x.as_str())
        .unwrap_or_default();

      RspackRegex::with_flags(expr, flags).unwrap_or_else(|_| {
        // test when capture
        unreachable!();
      })
    })
  }

  pub fn get_webpack_exports(&self) -> Option<Vec<String>> {
    self.0.get(&WebpackComment::Exports).map(|expr| {
      expr
        .split(',')
        .filter_map(|x| {
          if x.is_empty() {
            None
          } else {
            Some(x.to_owned())
          }
        })
        .collect_vec()
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
    .with_severity(Severity::Warning)
    .with_hide_stack(Some(true)),
  ))
}

// Using vm.runInNewContext in webpack
// _0 for name
// _1 for "xxx"
// _2 for 'xxx'
// _3 for `xxx`
// _4 for number
// _5 for true/false
// _6 for regexp
// _7 for array
// _8 for identifier
static WEBPACK_MAGIC_COMMENT_REGEXP: LazyLock<regex::Regex> = LazyLock::new(|| {
  regex::Regex::new(r#"(?P<_0>webpack[a-zA-Z\d_-]+)\s*:\s*("(?P<_1>[^"]+)"|'(?P<_2>[^']+)'|`(?P<_3>[^`]+)`|(?P<_4>[\d.-]+)|(?P<_5>true|false)|(?P<_6>/([^,]+)/([dgimsuvy]*))|\[(?P<_7>[^\]]+)|(?P<_8>([^,]+)))"#)
    .expect("invalid regex")
});

static WEBAPCK_EXPORT_NAME_REGEXP: LazyLock<regex::Regex> =
  LazyLock::new(|| regex::Regex::new(r#"^["`'](\w+)["`']$"#).expect("invalid regex"));

pub fn try_extract_webpack_magic_comment(
  source_file: &SourceFile,
  comments: &Option<&dyn Comments>,
  error_span: Span,
  span: Span,
  warning_diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
) -> WebpackCommentMap {
  let mut result = WebpackCommentMap::new();
  comments.with_leading(span.lo, |comments| {
    analyze_comments(
      source_file,
      comments,
      error_span,
      warning_diagnostics,
      &mut result,
    )
  });
  comments.with_trailing(span.hi, |comments| {
    analyze_comments(
      source_file,
      comments,
      error_span,
      warning_diagnostics,
      &mut result,
    )
  });
  result
}

fn analyze_comments(
  source_file: &SourceFile,
  comments: &[Comment],
  error_span: Span,
  warning_diagnostics: &mut Vec<Box<dyn Diagnostic + Send + Sync>>,
  result: &mut WebpackCommentMap,
) {
  // TODO: remove this, parser.comments contains two same block comment
  let mut parsed_comment = FxHashSet::<Span>::default();
  for comment in comments
    .iter()
    .rev()
    .filter(|c| matches!(c.kind, CommentKind::Block))
  {
    if parsed_comment.contains(&comment.span) {
      continue;
    }
    parsed_comment.insert(comment.span);
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
              continue;
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              "a string",
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackPrefetch" => {
            if let Some(item_value_match) = captures.name("_4").or(captures.name("_5")) {
              result.insert(
                WebpackComment::Prefetch,
                item_value_match.as_str().to_string(),
              );
              continue;
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              "true or a number",
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackPreload" => {
            if let Some(item_value_match) = captures.name("_4").or(captures.name("_5")) {
              result.insert(
                WebpackComment::Preload,
                item_value_match.as_str().to_string(),
              );
              continue;
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              "true or a number",
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackIgnore" => {
            if let Some(item_value_match) = captures.name("_5") {
              result.insert(
                WebpackComment::Ignore,
                item_value_match.as_str().to_string(),
              );
              continue;
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              "true or false",
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackMode" => {
            if let Some(item_value_match) = captures
              .name("_1")
              .or(captures.name("_2"))
              .or(captures.name("_3"))
            {
              result.insert(WebpackComment::Mode, item_value_match.as_str().to_string());
              continue;
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              "a string",
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackFetchPriority" => {
            if let Some(item_value_match) = captures
              .name("_1")
              .or(captures.name("_2"))
              .or(captures.name("_3"))
            {
              let priority = item_value_match.as_str();
              if priority == "low" || priority == "high" || priority == "auto" {
                result.insert(WebpackComment::FetchPriority, priority.to_string());
                continue;
              }
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#""low", "high" or "auto""#,
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackInclude" => {
            if captures.name("_6").is_some() {
              if let Some(regexp) = captures.get(9).map(|x| x.as_str()) {
                let flags = captures.get(10).map(|x| x.as_str()).unwrap_or_default();
                if RspackRegex::with_flags(regexp, flags).is_ok() {
                  result.insert(WebpackComment::IncludeRegexp, regexp.to_string());
                  result.insert(WebpackComment::IncludeFlags, flags.to_string());
                  continue;
                }
              }
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#"a regular expression"#,
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackExclude" => {
            if captures.name("_6").is_some() {
              if let Some(regexp) = captures.get(9).map(|x| x.as_str()) {
                let flags = captures.get(10).map(|x| x.as_str()).unwrap_or_default();
                if RspackRegex::with_flags(regexp, flags).is_ok() {
                  result.insert(WebpackComment::ExcludeRegexp, regexp.to_string());
                  result.insert(WebpackComment::ExcludeFlags, flags.to_string());
                  continue;
                }
              }
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#"a regular expression"#,
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          "webpackExports" => {
            if let Some(item_value_match) = captures
              .name("_1")
              .or(captures.name("_2"))
              .or(captures.name("_3"))
            {
              result.insert(
                WebpackComment::Exports,
                item_value_match.as_str().trim().to_string(),
              );
              return;
            } else if let Some(item_value_match) = captures.name("_7") {
              if let Some(exports) =
                item_value_match
                  .as_str()
                  .split(',')
                  .try_fold("".to_string(), |acc, item| {
                    WEBAPCK_EXPORT_NAME_REGEXP
                      .captures(item.trim())
                      .and_then(|matched| matched.get(1).map(|x| x.as_str()))
                      .map(|name| format!("{acc},{name}"))
                  })
              {
                result.insert(WebpackComment::Exports, exports);
                return;
              }
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#"a string or an array of strings"#,
              &captures,
              warning_diagnostics,
              error_span,
            );
          }
          _ => {}
        }
      }
    }
  }
}
