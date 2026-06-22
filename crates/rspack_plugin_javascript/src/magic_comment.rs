use std::{borrow::Cow, fmt};

use rspack_core::DependencyRange;
use rspack_error::{Diagnostic, Error, Severity};
use rspack_regex::RspackRegex;
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{
  Comment, CommentKind, EsVersion, Expr, GetSpan, Lit, Prop, PropName, PropOrSpread, Span, UnaryOp,
};
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_expr};
use swc_experimental_ecma_transforms_base::remove_paren::remove_paren;

use crate::visitors::{JavascriptParser, create_traceable_error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RspackComment {
  ChunkName,
  Prefetch,
  Preload,
  Ignore,
  FetchPriority,
  IncludeRegexp,
  ExcludeRegexp,
  Mode,
  Exports,
}

impl RspackComment {
  fn prefixed_name(self, prefix: MagicCommentPrefix) -> String {
    format!("{prefix}{self}")
  }
}

impl fmt::Display for RspackComment {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::ChunkName => "ChunkName",
      Self::Prefetch => "Prefetch",
      Self::Preload => "Preload",
      Self::Ignore => "Ignore",
      Self::FetchPriority => "FetchPriority",
      Self::IncludeRegexp => "Include",
      Self::ExcludeRegexp => "Exclude",
      Self::Mode => "Mode",
      Self::Exports => "Exports",
    })
  }
}

impl TryFrom<&str> for RspackComment {
  type Error = ();

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      "ChunkName" => Ok(Self::ChunkName),
      "Prefetch" => Ok(Self::Prefetch),
      "Preload" => Ok(Self::Preload),
      "Ignore" => Ok(Self::Ignore),
      "FetchPriority" => Ok(Self::FetchPriority),
      "Include" => Ok(Self::IncludeRegexp),
      "Exclude" => Ok(Self::ExcludeRegexp),
      "Mode" => Ok(Self::Mode),
      "Exports" => Ok(Self::Exports),
      _ => Err(()),
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MagicCommentValue {
  Bool(bool),
  String(String),
  Number(String),
  RegExp { source: String, flags: String },
  Array(Vec<String>),
  Unknown,
}

#[derive(Debug)]
pub struct RspackCommentMap(FxHashMap<RspackComment, MagicCommentItem>);

impl RspackCommentMap {
  fn new() -> Self {
    Self(Default::default())
  }

  fn insert(&mut self, key: RspackComment, value: MagicCommentItem) {
    self.0.insert(key, value);
  }

  fn push_conflict_warning(
    source: &str,
    ignored_comment_name: impl fmt::Display,
    preferred_comment_name: impl fmt::Display,
    item: &MagicCommentItem,
    warning_diagnostics: &mut Vec<Diagnostic>,
  ) {
    let mut error: Error = create_traceable_error(
      "Magic comments conflict".into(),
      format!(
        "`{ignored_comment_name}` is ignored because `{preferred_comment_name}` is also specified. Prefer `{preferred_comment_name}`."
      ),
      source.to_owned(),
      item.span,
    );
    error.severity = Severity::Warning;
    error.hide_stack = Some(true);
    warning_diagnostics.push(error.into())
  }

  fn insert_with_conflict_warning(
    &mut self,
    source: &str,
    rspack_comment: RspackComment,
    item: MagicCommentItem,
    warning_diagnostics: &mut Vec<Diagnostic>,
  ) {
    if let Some(existing) = self.0.get_mut(&rspack_comment) {
      match (existing.prefix, item.prefix) {
        (MagicCommentPrefix::Rspack, MagicCommentPrefix::Webpack) => {
          Self::push_conflict_warning(
            source,
            rspack_comment.prefixed_name(MagicCommentPrefix::Webpack),
            rspack_comment.prefixed_name(MagicCommentPrefix::Rspack),
            &item,
            warning_diagnostics,
          );
        }
        (MagicCommentPrefix::Webpack, MagicCommentPrefix::Rspack) => {
          Self::push_conflict_warning(
            source,
            rspack_comment.prefixed_name(MagicCommentPrefix::Webpack),
            rspack_comment.prefixed_name(MagicCommentPrefix::Rspack),
            existing,
            warning_diagnostics,
          );
          *existing = item;
        }
        _ => {
          Self::push_conflict_warning(
            source,
            rspack_comment.prefixed_name(item.prefix),
            rspack_comment.prefixed_name(existing.prefix),
            &item,
            warning_diagnostics,
          );
        }
      }
    } else {
      self.insert(rspack_comment, item);
    }
  }

  pub fn get_ignore_value(&self) -> Option<&MagicCommentValue> {
    self.0.get(&RspackComment::Ignore).map(|item| &item.value)
  }

  pub fn get_mode(&self) -> Option<&String> {
    match self.0.get(&RspackComment::Mode).map(|item| &item.value) {
      Some(MagicCommentValue::String(value)) => Some(value),
      _ => None,
    }
  }

  pub fn get_chunk_name(&self) -> Option<&String> {
    match self
      .0
      .get(&RspackComment::ChunkName)
      .map(|item| &item.value)
    {
      Some(MagicCommentValue::String(value)) => Some(value),
      _ => None,
    }
  }

  pub fn get_prefetch(&self) -> Option<Cow<'_, str>> {
    match self.0.get(&RspackComment::Prefetch).map(|item| &item.value) {
      Some(MagicCommentValue::Bool(true)) => Some(Cow::Borrowed("true")),
      Some(MagicCommentValue::Number(value)) => Some(Cow::Borrowed(value.as_str())),
      _ => None,
    }
  }

  pub fn get_preload(&self) -> Option<Cow<'_, str>> {
    match self.0.get(&RspackComment::Preload).map(|item| &item.value) {
      Some(MagicCommentValue::Bool(true)) => Some(Cow::Borrowed("true")),
      Some(MagicCommentValue::Number(value)) => Some(Cow::Borrowed(value.as_str())),
      _ => None,
    }
  }

  pub fn get_ignore(&self) -> Option<bool> {
    match self.0.get(&RspackComment::Ignore).map(|item| &item.value) {
      Some(MagicCommentValue::Bool(value)) => Some(*value),
      _ => None,
    }
  }

  pub fn get_fetch_priority(&self) -> Option<&String> {
    match self
      .0
      .get(&RspackComment::FetchPriority)
      .map(|item| &item.value)
    {
      Some(MagicCommentValue::String(value)) => Some(value),
      _ => None,
    }
  }

  pub fn get_include(&self) -> Option<RspackRegex> {
    self
      .0
      .get(&RspackComment::IncludeRegexp)
      .and_then(|item| match &item.value {
        MagicCommentValue::RegExp { source, flags } => {
          Some(RspackRegex::with_flags(source, flags).unwrap_or_else(|_| {
            // test when capture
            unreachable!();
          }))
        }
        _ => None,
      })
  }

  pub fn get_exclude(&self) -> Option<RspackRegex> {
    self
      .0
      .get(&RspackComment::ExcludeRegexp)
      .and_then(|item| match &item.value {
        MagicCommentValue::RegExp { source, flags } => {
          Some(RspackRegex::with_flags(source, flags).unwrap_or_else(|_| {
            // test when capture
            unreachable!();
          }))
        }
        _ => None,
      })
  }

  pub fn get_exports(&self) -> Option<Vec<String>> {
    match self.0.get(&RspackComment::Exports).map(|item| &item.value) {
      Some(MagicCommentValue::String(value)) => Some(vec![value.clone()]),
      Some(MagicCommentValue::Array(value)) => Some(value.clone()),
      _ => None,
    }
  }
}

fn push_magic_comment_parse_warning(
  source: &str,
  comment_name: impl fmt::Display,
  comment_type: &str,
  received: &str,
  warning_diagnostics: &mut Vec<Diagnostic>,
  span: DependencyRange,
) {
  let mut error: Error = create_traceable_error(
    "Magic comments parse failed".into(),
    format!("`{comment_name}` expected {comment_type}, but received: {received}."),
    source.to_owned(),
    span,
  );
  error.severity = Severity::Warning;
  error.hide_stack = Some(true);
  warning_diagnostics.push(error.into())
}

pub fn try_extract_magic_comment(
  parser: &mut JavascriptParser,
  error_span: Span,
  span: Span,
) -> RspackCommentMap {
  let mut result = RspackCommentMap::new();
  let mut warning_diagnostics = Vec::new();
  let allocator = parser.ast.allocator;
  if let Some(comments) = parser.ast.comments.leading.get(&span.start) {
    analyze_comments(
      allocator,
      parser.source,
      comments,
      error_span,
      &mut warning_diagnostics,
      &mut result,
    );
  }
  if let Some(comments) = parser.ast.comments.trailing.get(&span.end) {
    analyze_comments(
      allocator,
      parser.source,
      comments,
      error_span,
      &mut warning_diagnostics,
      &mut result,
    );
  }
  parser.add_warnings(warning_diagnostics);
  result
}

/// Convert a value span from the synthetic object literal into a source span.
///
/// Block comment text does not include `/*` and `*/`. We parse it by wrapping
/// it as `({<comment_text>})`, so synthetic expression spans are offset by the
/// two-byte `({` prefix.
fn value_span_to_error_span(comment_span: Span, value_span: Span) -> Option<DependencyRange> {
  // Block comment format: /* comment_text */
  // The comment_text doesn't include the "/*" and "*/" delimiters
  // So we need to add 2 bytes for "/*" to get the actual position in source
  const BLOCK_COMMENT_START_LEN: usize = 2; // Length of "/*"
  const OBJECT_LITERAL_PREFIX_LEN: usize = 2; // Length of "({"

  let value_start = value_span.real_lo() as usize;
  let value_end = value_span.real_hi() as usize;
  if value_start < OBJECT_LITERAL_PREFIX_LEN || value_end < OBJECT_LITERAL_PREFIX_LEN {
    return None;
  }

  let comment_start = comment_span.real_lo() as usize;
  let start = comment_start + BLOCK_COMMENT_START_LEN + value_start - OBJECT_LITERAL_PREFIX_LEN;
  let end = comment_start + BLOCK_COMMENT_START_LEN + value_end - OBJECT_LITERAL_PREFIX_LEN;

  Some(DependencyRange::new(start as u32, end as u32))
}

fn value_span_to_comment_offsets(comment_text: &str, value_span: Span) -> Option<(usize, usize)> {
  const OBJECT_LITERAL_PREFIX_LEN: usize = 2; // Length of "({"

  let start = value_span
    .real_lo()
    .checked_sub(OBJECT_LITERAL_PREFIX_LEN as u32)? as usize;
  let end = value_span
    .real_hi()
    .checked_sub(OBJECT_LITERAL_PREFIX_LEN as u32)? as usize;

  (start <= end && end <= comment_text.len()).then_some((start, end))
}

fn raw_value<'a>(comment_text: &'a str, value: &Expr) -> Option<&'a str> {
  let (start, end) = value_span_to_comment_offsets(comment_text, value.span())?;
  comment_text.get(start..end).map(str::trim)
}

fn parse_magic_comment_object<'a>(
  allocator: &'a Allocator,
  comment_text: &str,
) -> Option<Expr<'a>> {
  let source = format!("({{{comment_text}}})");
  let source = allocator.alloc_str(&source);
  let mut expr = parse_file_as_expr(
    allocator,
    source,
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
  )
  .ok()?;
  remove_paren(&mut expr, allocator, None);
  Some(expr)
}

fn prop_name_to_str<'a>(name: &'a PropName<'a>) -> Option<Cow<'a, str>> {
  match name {
    PropName::Ident(ident) => Some(Cow::Borrowed(ident.sym.as_str())),
    PropName::Str(str) => Some(str.value.to_string_lossy()),
    _ => None,
  }
}

fn expr_to_str<'a>(expr: &'a Expr<'a>) -> Option<Cow<'a, str>> {
  match expr {
    Expr::Lit(lit) => match &**lit {
      Lit::Str(str) => Some(str.value.to_string_lossy()),
      _ => None,
    },
    Expr::Tpl(tpl) if tpl.exprs.is_empty() && tpl.quasis.len() == 1 => {
      tpl.quasis.first().map(|el| Cow::Borrowed(el.raw.as_ref()))
    }
    _ => None,
  }
}

fn expr_to_bool(expr: &Expr) -> Option<bool> {
  match expr {
    Expr::Lit(lit) => match &**lit {
      Lit::Bool(bool) => Some(bool.value),
      _ => None,
    },
    _ => None,
  }
}

fn is_number_expr(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(lit) => matches!(&**lit, Lit::Num(_)),
    Expr::Unary(unary) if matches!(unary.op, UnaryOp::Minus) => {
      matches!(&unary.arg, Expr::Lit(lit) if matches!(&**lit, Lit::Num(_)))
    }
    _ => false,
  }
}

fn expr_to_order_str<'a>(comment_text: &'a str, expr: &Expr) -> Option<&'a str> {
  if expr_to_bool(expr).is_some() || is_number_expr(expr) {
    raw_value(comment_text, expr)
  } else {
    None
  }
}

fn expr_to_regexp<'a>(expr: &'a Expr<'a>) -> Option<(&'a str, &'a str)> {
  match expr {
    Expr::Lit(lit) => match &**lit {
      Lit::Regex(regex) => Some((regex.exp.as_str(), regex.flags.as_str())),
      _ => None,
    },
    _ => None,
  }
}

fn expr_to_magic_comment_value(comment_text: &str, expr: &Expr) -> Option<MagicCommentValue> {
  if let Some(value) = expr_to_bool(expr) {
    return Some(MagicCommentValue::Bool(value));
  }

  if let Some(value) = expr_to_str(expr) {
    return Some(MagicCommentValue::String(value.into_owned()));
  }

  if is_number_expr(expr) {
    return raw_value(comment_text, expr).map(|value| MagicCommentValue::Number(value.to_string()));
  }

  if let Some((source, flags)) = expr_to_regexp(expr) {
    return Some(MagicCommentValue::RegExp {
      source: source.to_string(),
      flags: flags.to_string(),
    });
  }

  let Expr::Array(array) = expr else {
    return None;
  };
  let mut items = Vec::new();
  for elem in &array.elems {
    let elem = elem.as_ref()?;
    if elem.spread.is_some() {
      return None;
    }
    items.push(expr_to_str(&elem.expr)?.into_owned());
  }
  Some(MagicCommentValue::Array(items))
}

fn expr_to_exports(expr: &Expr) -> Option<MagicCommentValue> {
  if let Some(string) = expr_to_str(expr) {
    let trimmed = string.trim();
    if trimmed.len() == string.len() {
      return Some(MagicCommentValue::String(string.into_owned()));
    }
    return Some(MagicCommentValue::String(trimmed.to_string()));
  }

  let Expr::Array(array) = expr else {
    return None;
  };

  let mut exports = Vec::new();
  for elem in &array.elems {
    let elem = elem.as_ref()?;
    if elem.spread.is_some() {
      return None;
    }
    exports.push(expr_to_str(&elem.expr)?.into_owned());
  }

  Some(MagicCommentValue::Array(exports))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MagicCommentPrefix {
  Rspack,
  Webpack,
}

impl fmt::Display for MagicCommentPrefix {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(match self {
      Self::Rspack => "rspack",
      Self::Webpack => "webpack",
    })
  }
}

#[derive(Debug)]
struct MagicCommentItem {
  prefix: MagicCommentPrefix,
  value: MagicCommentValue,
  span: DependencyRange,
}

fn parse_magic_comment_name(name: &str) -> Option<(RspackComment, MagicCommentPrefix)> {
  let (name, prefix) = if let Some(name) = name.strip_prefix("rspack") {
    (name, MagicCommentPrefix::Rspack)
  } else if let Some(name) = name.strip_prefix("webpack") {
    (name, MagicCommentPrefix::Webpack)
  } else {
    return None;
  };

  Some((RspackComment::try_from(name).ok()?, prefix))
}

fn analyze_comments(
  allocator: &Allocator,
  source: &str,
  comments: &[Comment],
  error_span: Span,
  warning_diagnostics: &mut Vec<Diagnostic>,
  result: &mut RspackCommentMap,
) {
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
    let Some(expr) = parse_magic_comment_object(allocator, &comment.text) else {
      continue;
    };
    let Expr::Object(object) = &expr else {
      continue;
    };
    for prop in &object.props {
      let PropOrSpread::Prop(prop) = prop else {
        continue;
      };
      let Prop::KeyValue(prop) = &**prop else {
        continue;
      };
      let Some(item_name) = prop_name_to_str(&prop.key) else {
        continue;
      };
      let Some((rspack_comment, prefix)) = parse_magic_comment_name(item_name.as_ref()) else {
        continue;
      };
      let value = &prop.value;
      let item_name = rspack_comment.prefixed_name(prefix);
      let received = raw_value(&comment.text, value).unwrap_or_default();
      let item_span =
        value_span_to_error_span(comment.span, value.span()).unwrap_or(error_span.into());
      let push_parse_warning = |comment_type| {
        push_magic_comment_parse_warning(
          source,
          item_name,
          comment_type,
          received,
          warning_diagnostics,
          item_span,
        );
      };

      let value = match rspack_comment {
        RspackComment::ChunkName => {
          if let Some(value) = expr_to_str(value) {
            MagicCommentValue::String(value.into_owned())
          } else {
            push_parse_warning("a string");
            continue;
          }
        }
        RspackComment::Prefetch => {
          if let Some(value) = expr_to_order_str(&comment.text, value) {
            if value == "true" {
              MagicCommentValue::Bool(true)
            } else {
              MagicCommentValue::Number(value.to_string())
            }
          } else {
            push_parse_warning("true or a number");
            continue;
          }
        }
        RspackComment::Preload => {
          if let Some(value) = expr_to_order_str(&comment.text, value) {
            if value == "true" {
              MagicCommentValue::Bool(true)
            } else {
              MagicCommentValue::Number(value.to_string())
            }
          } else {
            push_parse_warning("true or a number");
            continue;
          }
        }
        RspackComment::Ignore => {
          if let Some(value) = expr_to_bool(value) {
            MagicCommentValue::Bool(value)
          } else {
            let value = expr_to_magic_comment_value(&comment.text, value)
              .unwrap_or(MagicCommentValue::Unknown);
            push_parse_warning("a boolean");
            value
          }
        }
        RspackComment::Mode => {
          if let Some(value) = expr_to_str(value) {
            MagicCommentValue::String(value.into_owned())
          } else {
            push_parse_warning("a string");
            continue;
          }
        }
        RspackComment::FetchPriority => {
          if let Some(priority) = expr_to_str(value)
            && matches!(priority.as_ref(), "low" | "high" | "auto")
          {
            MagicCommentValue::String(priority.into_owned())
          } else {
            push_parse_warning(r#""low", "high" or "auto""#);
            continue;
          }
        }
        RspackComment::IncludeRegexp => {
          if let Some((regexp, flags)) = expr_to_regexp(value)
            && RspackRegex::with_flags(regexp, flags).is_ok()
          {
            MagicCommentValue::RegExp {
              source: regexp.to_string(),
              flags: flags.to_string(),
            }
          } else {
            push_parse_warning(r#"a regular expression"#);
            continue;
          }
        }
        RspackComment::ExcludeRegexp => {
          if let Some((regexp, flags)) = expr_to_regexp(value)
            && RspackRegex::with_flags(regexp, flags).is_ok()
          {
            MagicCommentValue::RegExp {
              source: regexp.to_string(),
              flags: flags.to_string(),
            }
          } else {
            push_parse_warning(r#"a regular expression"#);
            continue;
          }
        }
        RspackComment::Exports => {
          if let Some(exports) = expr_to_exports(value) {
            exports
          } else {
            push_parse_warning(r#"a string or an array of strings"#);
            continue;
          }
        }
      };
      let item = MagicCommentItem {
        prefix,
        value,
        span: item_span,
      };
      result.insert_with_conflict_warning(source, rspack_comment, item, warning_diagnostics);
    }
  }
}

#[cfg(test)]
mod tests_extract_magic_comment_object {
  use swc_experimental_ecma_ast::DUMMY_SP;

  use super::*;

  fn with_value<R>(raw: &str, name: &str, f: impl FnOnce(&Expr<'_>) -> Option<R>) -> Option<R> {
    let allocator = Allocator::new();
    let expr = parse_magic_comment_object(&allocator, raw)?;
    let Expr::Object(object) = &expr else {
      return None;
    };
    for prop in &object.props {
      let PropOrSpread::Prop(prop) = prop else {
        continue;
      };
      let Prop::KeyValue(prop) = &**prop else {
        continue;
      };
      if prop_name_to_str(&prop.key).as_deref() == Some(name) {
        return f(&prop.value);
      }
    }
    None
  }

  fn extract(raw: &str) -> (RspackCommentMap, Vec<Diagnostic>) {
    let mut result = RspackCommentMap::new();
    let mut warning_diagnostics = Vec::new();
    let allocator = Allocator::new();
    analyze_comments(
      &allocator,
      "",
      &[Comment {
        kind: CommentKind::Block,
        span: DUMMY_SP,
        text: swc_experimental_allocator::atom::Atom::new_in(raw, &allocator),
      }],
      DUMMY_SP,
      &mut warning_diagnostics,
      &mut result,
    );
    (result, warning_diagnostics)
  }

  fn try_match_string(raw: &str) -> Option<(String, String)> {
    let name = "webpackInclude";
    with_value(raw, name, |value| {
      Some((name.to_string(), expr_to_str(value)?.into_owned()))
    })
  }

  fn try_match_order(raw: &str) -> Option<(String, String)> {
    let name = "webpackInclude";
    with_value(raw, name, |value| {
      Some((name.to_string(), expr_to_order_str(raw, value)?.to_string()))
    })
  }

  fn try_match_regex(raw: &str) -> Option<(String, String, String)> {
    let name = "webpackInclude";
    with_value(raw, name, |value| {
      let (regexp, flags) = expr_to_regexp(value)?;
      Some((name.to_string(), regexp.to_string(), flags.to_string()))
    })
  }

  fn test_extract_string() {
    assert_eq!(
      try_match_string("webpackInclude: \"abc\""),
      Some(("webpackInclude".to_string(), "abc".to_string()))
    );
    assert_eq!(
      try_match_string("webpackInclude: 'abc'"),
      Some(("webpackInclude".to_string(), "abc".to_string()))
    );
    assert_eq!(
      try_match_string("webpackInclude: `abc`"),
      Some(("webpackInclude".to_string(), "abc".to_string()))
    );
    assert_eq!(
      try_match_string("webpackInclude: \"abc_-|123\""),
      Some(("webpackInclude".to_string(), "abc_-|123".to_string()))
    );
  }

  fn test_extract_number() {
    assert_eq!(
      try_match_order("webpackInclude: 123"),
      Some(("webpackInclude".to_string(), "123".to_string()))
    );
    assert_eq!(
      try_match_order("webpackInclude: 123.456"),
      Some(("webpackInclude".to_string(), "123.456".to_string()))
    );
    assert_eq!(
      try_match_order("webpackInclude: -123.456"),
      Some(("webpackInclude".to_string(), "-123.456".to_string()))
    );
  }

  fn test_extract_boolean() {
    assert_eq!(
      try_match_order("webpackInclude: true"),
      Some(("webpackInclude".to_string(), "true".to_string()))
    );
    assert_eq!(
      try_match_order("webpackInclude: false"),
      Some(("webpackInclude".to_string(), "false".to_string()))
    );
  }

  fn test_extract_array() {
    assert_eq!(
      with_value(
        "webpackExports: [\"a\", `b`, 'c']",
        "webpackExports",
        expr_to_exports
      ),
      Some(MagicCommentValue::Array(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string()
      ]))
    );
  }

  fn test_extract_regexp() {
    assert_eq!(
      try_match_regex("webpackInclude: /abc/"),
      Some((
        "webpackInclude".to_string(),
        "abc".to_string(),
        String::new()
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
        String::new()
      ))
    );
    assert_eq!(
      try_match_regex(r#"webpackInclude: /components[/\\][^/\\]+\.vue$/"#),
      Some((
        "webpackInclude".to_string(),
        r#"components[/\\][^/\\]+\.vue$"#.to_string(),
        String::new()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /^.{2,}$/"),
      Some((
        "webpackInclude".to_string(),
        "^.{2,}$".to_string(),
        String::new()
      ))
    );
    assert_eq!(
      try_match_regex("webpackInclude: /^.{2,}$/, webpackExclude: /^.{3,}$/"),
      Some((
        "webpackInclude".to_string(),
        "^.{2,}$".to_string(),
        String::new()
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
        String::new()
      ))
    );
  }

  #[test]
  fn test_rspack_magic_comment_name_aliases() {
    assert_eq!(
      parse_magic_comment_name("rspackChunkName").map(|(comment, _)| comment),
      Some(RspackComment::ChunkName)
    );
    assert_eq!(
      parse_magic_comment_name("rspackPrefetch").map(|(comment, _)| comment),
      Some(RspackComment::Prefetch)
    );
    assert_eq!(
      parse_magic_comment_name("rspackPreload").map(|(comment, _)| comment),
      Some(RspackComment::Preload)
    );
    assert_eq!(
      parse_magic_comment_name("rspackIgnore").map(|(comment, _)| comment),
      Some(RspackComment::Ignore)
    );
    assert_eq!(
      parse_magic_comment_name("rspackMode").map(|(comment, _)| comment),
      Some(RspackComment::Mode)
    );
    assert_eq!(
      parse_magic_comment_name("rspackFetchPriority").map(|(comment, _)| comment),
      Some(RspackComment::FetchPriority)
    );
    assert_eq!(
      parse_magic_comment_name("rspackInclude").map(|(comment, _)| comment),
      Some(RspackComment::IncludeRegexp)
    );
    assert_eq!(
      parse_magic_comment_name("rspackExclude").map(|(comment, _)| comment),
      Some(RspackComment::ExcludeRegexp)
    );
    assert_eq!(
      parse_magic_comment_name("rspackExports").map(|(comment, _)| comment),
      Some(RspackComment::Exports)
    );
  }

  #[test]
  fn test_extract_rspack_prefixed_magic_comments() {
    let (comments, warnings) = extract(
      r#"
        rspackChunkName: "chunk",
        rspackPrefetch: 1,
        rspackPreload: true,
        rspackIgnore: true,
        rspackMode: "eager",
        rspackFetchPriority: "high",
        rspackInclude: /\.js$/,
        rspackExclude: /\.test\.js$/,
        rspackExports: ["a", "b"]
      "#,
    );

    assert!(warnings.is_empty());
    assert_eq!(comments.get_chunk_name(), Some(&"chunk".to_string()));
    assert_eq!(comments.get_prefetch().as_deref(), Some("1"));
    assert_eq!(comments.get_preload().as_deref(), Some("true"));
    assert_eq!(comments.get_ignore(), Some(true));
    assert_eq!(comments.get_mode(), Some(&"eager".to_string()));
    assert_eq!(comments.get_fetch_priority(), Some(&"high".to_string()));
    assert!(comments.get_include().is_some());
    assert!(comments.get_exclude().is_some());
    assert_eq!(comments.get_exports(), Some(vec!["a".into(), "b".into()]));
  }

  #[test]
  fn test_rspack_prefixed_magic_comments_override_webpack_prefixed_comments() {
    let (comments, warnings) = extract(
      r#"
        webpackChunkName: "webpack-chunk",
        rspackChunkName: "rspack-chunk",
        rspackMode: "eager",
        webpackMode: "lazy",
        webpackPrefetch: 1,
        rspackPrefetch: true,
        rspackPreload: 2,
        webpackPreload: true,
        webpackIgnore: false,
        rspackIgnore: true,
        rspackFetchPriority: "high",
        webpackFetchPriority: "low",
        webpackInclude: /\.jsx$/,
        rspackInclude: /\.js$/,
        rspackExclude: /\.test\.js$/,
        webpackExclude: /\.spec\.js$/,
        webpackExports: ["webpack"],
        rspackExports: ["rspack"]
      "#,
    );

    assert_eq!(comments.get_chunk_name(), Some(&"rspack-chunk".to_string()));
    assert_eq!(comments.get_mode(), Some(&"eager".to_string()));
    assert_eq!(comments.get_prefetch().as_deref(), Some("true"));
    assert_eq!(comments.get_preload().as_deref(), Some("2"));
    assert_eq!(comments.get_ignore(), Some(true));
    assert_eq!(comments.get_fetch_priority(), Some(&"high".to_string()));
    assert_eq!(comments.get_include().unwrap().source(), r#"\.js$"#);
    assert_eq!(comments.get_exclude().unwrap().source(), r#"\.test\.js$"#);
    assert_eq!(comments.get_exports(), Some(vec!["rspack".into()]));
    assert_eq!(warnings.len(), 9);
    for webpack_name in [
      "webpackChunkName",
      "webpackMode",
      "webpackPrefetch",
      "webpackPreload",
      "webpackIgnore",
      "webpackFetchPriority",
      "webpackInclude",
      "webpackExclude",
      "webpackExports",
    ] {
      assert!(
        warnings.iter().any(|warning| warning
          .message
          .contains(&format!("`{webpack_name}` is ignored"))),
        "missing conflict warning for {webpack_name}"
      );
    }
  }

  #[test]
  fn test_rspack_prefixed_magic_comments_override_webpack_prefixed_comments_in_any_order() {
    let (comments, warnings) = extract(
      r#"
        rspackChunkName: "rspack-chunk",
        webpackChunkName: "webpack-chunk"
      "#,
    );

    assert_eq!(comments.get_chunk_name(), Some(&"rspack-chunk".to_string()));
    assert_eq!(warnings.len(), 1);
    assert!(
      warnings[0]
        .message
        .contains("`webpackChunkName` is ignored")
    );
  }

  #[test]
  fn test_repeated_magic_comments_with_same_prefix_keep_first_value() {
    let (comments, warnings) = extract(
      r#"
        webpackChunkName: "first-webpack-chunk",
        webpackChunkName: "second-webpack-chunk",
        rspackMode: "eager",
        rspackMode: "lazy"
      "#,
    );

    assert_eq!(
      comments.get_chunk_name(),
      Some(&"first-webpack-chunk".to_string())
    );
    assert_eq!(comments.get_mode(), Some(&"eager".to_string()));
    assert_eq!(warnings.len(), 2);
    assert!(
      warnings[0]
        .message
        .contains("`webpackChunkName` is ignored")
    );
    assert!(warnings[1].message.contains("`rspackMode` is ignored"));
  }

  #[test]
  fn test_extract_magic_comment_object() {
    test_extract_string();
    test_extract_number();
    test_extract_boolean();
    test_extract_array();
    test_extract_regexp();
  }
}
