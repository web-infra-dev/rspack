use std::sync::LazyLock;

use itertools::Itertools;
use regex::Captures;
use rspack_core::DependencyRange;
use rspack_error::{Diagnostic, Error, Severity};
use rspack_regex::RspackRegex;
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::{
  SourceFile, Span,
  comments::{Comment, CommentKind, Comments},
};

use crate::visitors::{JavascriptParser, create_traceable_error};

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
  warning_diagnostics: &mut Vec<Diagnostic>,
  span: DependencyRange,
) {
  let mut error: Error = create_traceable_error(
    "Magic comments parse failed".into(),
    format!(
      "`{comment_name}` expected {comment_type}, but received: {}.",
      captures.get(2).map_or("", |m| m.as_str())
    ),
    source_file,
    span,
  );
  error.severity = Severity::Warning;
  error.hide_stack = Some(true);
  warning_diagnostics.push(error.into())
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
// _9 for item value as a whole
static WEBPACK_MAGIC_COMMENT_REGEXP: LazyLock<regex::Regex> = LazyLock::new(|| {
  regex::Regex::new(r#"(?P<_0>webpack[a-zA-Z\d_-]+)\s*:\s*(?P<_9>"(?P<_1>[^"]+)"|'(?P<_2>[^']+)'|`(?P<_3>[^`]+)`|(?P<_4>[\d.-]+)|(?P<_5>true|false)|(?P<_6>/((?:(?:[^\\/\]\[]+)|(?:\[[^\]]+\])|(?:\\/)|(?:\\.))*)/([dgimsuvy]*))|\[(?P<_7>[^\]]*)|(?P<_8>([^,]+)))"#)
    .expect("invalid regex")
});

static WEBAPCK_EXPORT_NAME_REGEXP: LazyLock<regex::Regex> =
  LazyLock::new(|| regex::Regex::new(r#"^["`'](\w+)["`']$"#).expect("invalid regex"));

pub fn try_extract_webpack_magic_comment(
  parser: &mut JavascriptParser,
  error_span: Span,
  span: Span,
) -> WebpackCommentMap {
  let mut result = WebpackCommentMap::new();
  let mut warning_diagnostics = Vec::new();
  parser.comments.with_leading(span.lo, |comments| {
    analyze_comments(
      parser.source_file,
      comments,
      error_span,
      &mut warning_diagnostics,
      &mut result,
    )
  });
  parser.comments.with_trailing(span.hi, |comments| {
    analyze_comments(
      parser.source_file,
      comments,
      error_span,
      &mut warning_diagnostics,
      &mut result,
    )
  });
  parser.add_warnings(warning_diagnostics);
  result
}

#[derive(Debug)]
struct Location {
  /// Start line
  sl: u32,
  /// Start column
  sc: u32,
  /// End line
  el: u32,
  /// End column
  ec: u32,
}

impl Location {
  /// Block comment should be within the location of the source location.
  fn merge_with_block_comment_location(&self, block_comment: &Location) -> Self {
    let sl = self.sl + block_comment.sl;
    let sc = if block_comment.sl == 0 {
      self.sc + block_comment.sc + 2 // Length of `/*`
    } else {
      block_comment.sc
    };
    let el = self.sl + block_comment.el;
    let ec = if block_comment.el == 0 {
      self.sc + block_comment.ec + 2 // Length of `/*`
    } else {
      block_comment.ec
    };
    Location { sl, sc, el, ec }
  }
}

/// # Panics
///
/// Panics if `start` or `end` is out-of-bound.
fn byte_offset_to_location(rope: &ropey::Rope, start: usize, end: usize) -> Location {
  let char_index = rope.byte_to_char(start);
  let sl = rope.char_to_line(char_index);
  let sc = char_index - rope.line_to_char(sl);

  let char_index = rope.byte_to_char(end);
  let el = rope.char_to_line(char_index);
  let ec = char_index - rope.line_to_char(el);

  Location {
    sl: sl as u32,
    sc: sc as u32,
    el: el as u32,
    ec: ec as u32,
  }
}

/// Convert match item to error span within the source
///
/// # Panics
///
/// Panics if `comment_span` is out-of-bound of `source`.
/// Panics if either `match_start` or `match_end` is out-of-bound of `comment_text`.
fn match_item_to_error_span(
  source: &str,
  comment_span: Span,
  comment_text: &str,
  match_start: usize,
  match_end: usize,
) -> DependencyRange {
  let s = ropey::Rope::from_str(source);
  // SAFETY: `comment_span` is always within the bound of `source`.
  let s_loc = byte_offset_to_location(
    &s,
    comment_span.real_lo() as usize,
    comment_span.real_hi() as usize,
  );
  let c = ropey::Rope::from_str(comment_text);
  // SAFETY: `match_start` or `match_end` is always within the bound of `comment_text`.
  let c_loc = byte_offset_to_location(&c, match_start, match_end);

  let Location { sl, sc, el, ec } = s_loc.merge_with_block_comment_location(&c_loc);
  let start = s.line_to_byte(sl as usize) + sc as usize;
  let end = s.line_to_byte(el as usize) + ec as usize;
  DependencyRange::new(start as u32, end as u32)
}

fn analyze_comments(
  source_file: &SourceFile,
  comments: &[Comment],
  error_span: Span,
  warning_diagnostics: &mut Vec<Diagnostic>,
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
        let error_span = || {
          captures
            .name("_9")
            .map(|item| {
              match_item_to_error_span(
                &source_file.src,
                comment.span,
                &comment.text,
                item.start(),
                item.end(),
              )
            })
            .unwrap_or(error_span.into())
        };
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
              error_span(),
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
              error_span(),
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
              error_span(),
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
              "a boolean",
              &captures,
              warning_diagnostics,
              error_span(),
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
              error_span(),
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
              error_span(),
            );
          }
          "webpackInclude" => {
            if captures.name("_6").is_some()
              && let Some(regexp) = captures.get(9).map(|x| x.as_str())
            {
              let flags = captures.get(10).map(|x| x.as_str()).unwrap_or_default();
              if RspackRegex::with_flags(regexp, flags).is_ok() {
                result.insert(WebpackComment::IncludeRegexp, regexp.to_string());
                result.insert(WebpackComment::IncludeFlags, flags.to_string());
                continue;
              }
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#"a regular expression"#,
              &captures,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackExclude" => {
            if captures.name("_6").is_some()
              && let Some(regexp) = captures.get(9).map(|x| x.as_str())
            {
              let flags = captures.get(10).map(|x| x.as_str()).unwrap_or_default();
              if RspackRegex::with_flags(regexp, flags).is_ok() {
                result.insert(WebpackComment::ExcludeRegexp, regexp.to_string());
                result.insert(WebpackComment::ExcludeFlags, flags.to_string());
                continue;
              }
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#"a regular expression"#,
              &captures,
              warning_diagnostics,
              error_span(),
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
              continue;
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
              }
              continue;
            }
            add_magic_comment_warning(
              source_file,
              item_name,
              r#"a string or an array of strings"#,
              &captures,
              warning_diagnostics,
              error_span(),
            );
          }
          _ => {}
        }
      }
    }
  }
}

#[cfg(test)]
mod tests_extract_regex {
  use super::*;

  fn try_match(raw: &str, index: usize) -> Option<(String, String)> {
    let captures = WEBPACK_MAGIC_COMMENT_REGEXP.captures(raw)?;
    let item_name = captures.name("_0").map(|x| x.as_str().to_string())?;
    let item_value = captures
      .name(&format!("_{index}"))
      .map(|x| x.as_str().to_string())?;
    Some((item_name, item_value))
  }

  fn try_match_regex(raw: &str) -> Option<(String, String, String)> {
    let captures = WEBPACK_MAGIC_COMMENT_REGEXP.captures(raw)?;
    let item_name = captures.name("_0").map(|x| x.as_str().to_string())?;
    if let Some(regexp) = captures.get(9).map(|x| x.as_str()) {
      let flags = captures.get(10).map(|x| x.as_str()).unwrap_or_default();
      Some((item_name, regexp.to_string(), flags.to_string()))
    } else {
      None
    }
  }

  fn test_extract_string() {
    assert_eq!(
      try_match("webpackInclude: \"abc\"", 1),
      Some(("webpackInclude".to_string(), "abc".to_string()))
    );
    assert_eq!(
      try_match("webpackInclude: 'abc'", 2),
      Some(("webpackInclude".to_string(), "abc".to_string()))
    );
    assert_eq!(
      try_match("webpackInclude: `abc`", 3),
      Some(("webpackInclude".to_string(), "abc".to_string()))
    );
    assert_eq!(
      try_match("webpackInclude: \"abc_-|123\"", 1),
      Some(("webpackInclude".to_string(), "abc_-|123".to_string()))
    );
  }

  fn test_extract_number() {
    assert_eq!(
      try_match("webpackInclude: 123", 4),
      Some(("webpackInclude".to_string(), "123".to_string()))
    );
    assert_eq!(
      try_match("webpackInclude: 123.456", 4),
      Some(("webpackInclude".to_string(), "123.456".to_string()))
    );
    assert_eq!(
      try_match("webpackInclude: -123.456", 4),
      Some(("webpackInclude".to_string(), "-123.456".to_string()))
    );
  }

  fn test_extract_boolean() {
    assert_eq!(
      try_match("webpackInclude: true", 5),
      Some(("webpackInclude".to_string(), "true".to_string()))
    );
    assert_eq!(
      try_match("webpackInclude: false", 5),
      Some(("webpackInclude".to_string(), "false".to_string()))
    );
  }

  fn test_extract_array() {
    assert_eq!(
      try_match("webpackInclude: [\"a\", `b`, 'c']", 7),
      Some(("webpackInclude".to_string(), "\"a\", `b`, 'c'".to_string()))
    );
  }

  fn test_extract_regexp() {
    assert_eq!(
      try_match_regex("webpackInclude: /abc/"),
      Some((
        "webpackInclude".to_string(),
        "abc".to_string(),
        "".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /abc/ig"),
      Some((
        "webpackInclude".to_string(),
        "abc".to_string(),
        "ig".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /[^,+]/ig"),
      Some((
        "webpackInclude".to_string(),
        "[^,+]".to_string(),
        "ig".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /a\\/b\\/c/ig"),
      Some((
        "webpackInclude".to_string(),
        "a\\/b\\/c".to_string(),
        "ig".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /components[\\/][^\\/]+\\.vue$/"),
      Some((
        "webpackInclude".to_string(),
        "components[\\/][^\\/]+\\.vue$".to_string(),
        "".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /components[/\\][^/\\]+\\.vue$/"),
      Some((
        "webpackInclude".to_string(),
        "components[/\\][^/\\]+\\.vue$".to_string(),
        "".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /^.{2,}$/"),
      Some((
        "webpackInclude".to_string(),
        "^.{2,}$".to_string(),
        "".to_string()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /^.{2,}$/, webpackExclude: /^.{3,}$/"),
      Some((
        "webpackInclude".to_string(),
        "^.{2,}$".to_string(),
        "".to_string()
      ))
    );
    // https://github.com/web-infra-dev/rspack/issues/10195
    assert_eq!(
      try_match_regex(
        "webpackInclude: /(?!.*node_modules)(?:\\/src\\/(?!\\.)(?=.)[^/]*?\\.stories\\.tsx)$/"
      ),
      Some((
        "webpackInclude".to_string(),
        "(?!.*node_modules)(?:\\/src\\/(?!\\.)(?=.)[^/]*?\\.stories\\.tsx)$".to_string(),
        "".to_string()
      ))
    );
  }

  #[test]
  fn test_extract_regex() {
    test_extract_string();
    test_extract_number();
    test_extract_boolean();
    test_extract_array();
    test_extract_regexp();
  }
}
