use std::borrow::Cow;

use rspack_core::DependencyRange;
use rspack_error::{Diagnostic, Error, Severity};
use rspack_regex::RspackRegex;
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::{
  Span,
  comments::{Comment, CommentKind, Comments},
};
use swc_experimental_ecma_ast::{
  Ast, EsVersion, Expr, GetSpan, Lit, Prop, PropName, PropOrSpread, StringAllocator, UnaryOp,
};
use swc_experimental_ecma_parser::{EsSyntax, Syntax, parse_file_as_expr};

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
pub struct RspackCommentMap(FxHashMap<RspackComment, MagicCommentValue>);

impl RspackCommentMap {
  fn new() -> Self {
    Self(Default::default())
  }

  fn insert(&mut self, key: RspackComment, value: MagicCommentValue) {
    self.0.insert(key, value);
  }

  pub fn get_ignore_value(&self) -> Option<&MagicCommentValue> {
    self.0.get(&RspackComment::Ignore)
  }

  pub fn get_mode(&self) -> Option<&String> {
    match self.0.get(&RspackComment::Mode) {
      Some(MagicCommentValue::String(value)) => Some(value),
      _ => None,
    }
  }

  pub fn get_chunk_name(&self) -> Option<&String> {
    match self.0.get(&RspackComment::ChunkName) {
      Some(MagicCommentValue::String(value)) => Some(value),
      _ => None,
    }
  }

  pub fn get_prefetch(&self) -> Option<Cow<'_, str>> {
    match self.0.get(&RspackComment::Prefetch) {
      Some(MagicCommentValue::Bool(true)) => Some(Cow::Borrowed("true")),
      Some(MagicCommentValue::Number(value)) => Some(Cow::Borrowed(value.as_str())),
      _ => None,
    }
  }

  pub fn get_preload(&self) -> Option<Cow<'_, str>> {
    match self.0.get(&RspackComment::Preload) {
      Some(MagicCommentValue::Bool(true)) => Some(Cow::Borrowed("true")),
      Some(MagicCommentValue::Number(value)) => Some(Cow::Borrowed(value.as_str())),
      _ => None,
    }
  }

  pub fn get_ignore(&self) -> Option<bool> {
    match self.0.get(&RspackComment::Ignore) {
      Some(MagicCommentValue::Bool(value)) => Some(*value),
      _ => None,
    }
  }

  pub fn get_fetch_priority(&self) -> Option<&String> {
    match self.0.get(&RspackComment::FetchPriority) {
      Some(MagicCommentValue::String(value)) => Some(value),
      _ => None,
    }
  }

  pub fn get_include(&self) -> Option<RspackRegex> {
    self
      .0
      .get(&RspackComment::IncludeRegexp)
      .and_then(|value| match value {
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
      .and_then(|value| match value {
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
    match self.0.get(&RspackComment::Exports) {
      Some(MagicCommentValue::String(value)) => Some(vec![value.clone()]),
      Some(MagicCommentValue::Array(value)) => Some(value.clone()),
      _ => None,
    }
  }
}

fn add_magic_comment_warning(
  source: &str,
  comment_name: &str,
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
  parser.comments.with_leading(span.lo, |comments| {
    analyze_comments(
      parser.source,
      comments,
      error_span,
      &mut warning_diagnostics,
      &mut result,
    )
  });
  parser.comments.with_trailing(span.hi, |comments| {
    analyze_comments(
      parser.source,
      comments,
      error_span,
      &mut warning_diagnostics,
      &mut result,
    )
  });
  parser.add_warnings(warning_diagnostics);
  result
}

/// Convert a value span from the synthetic object literal into a source span.
///
/// Block comment text does not include `/*` and `*/`. We parse it by wrapping
/// it as `({<comment_text>})`, so synthetic expression spans are offset by the
/// two-byte `({` prefix.
fn value_span_to_error_span(
  comment_span: Span,
  value_span: swc_experimental_ecma_ast::Span,
) -> Option<DependencyRange> {
  // Block comment format: /* comment_text */
  // The comment_text doesn't include the "/*" and "*/" delimiters
  // So we need to add 2 bytes for "/*" to get the actual position in source
  const BLOCK_COMMENT_START_LEN: usize = 2; // Length of "/*"
  const OBJECT_LITERAL_PREFIX_LEN: usize = 2; // Length of "({"

  let value_start = value_span
    .start
    .checked_sub(OBJECT_LITERAL_PREFIX_LEN as u32 + 1)? as usize;
  let value_end = value_span
    .end
    .checked_sub(OBJECT_LITERAL_PREFIX_LEN as u32 + 1)? as usize;

  let comment_start = comment_span.real_lo() as usize;
  let start = comment_start + BLOCK_COMMENT_START_LEN + value_start;
  let end = comment_start + BLOCK_COMMENT_START_LEN + value_end;

  Some(DependencyRange::new(start as u32, end as u32))
}

fn value_span_to_comment_offsets(
  comment_text: &str,
  value_span: swc_experimental_ecma_ast::Span,
) -> Option<(usize, usize)> {
  const OBJECT_LITERAL_PREFIX_LEN: usize = 2; // Length of "({"

  let source_start = value_span.start.checked_sub(1)? as usize;
  let source_end = value_span.end.checked_sub(1)? as usize;
  let start = source_start.checked_sub(OBJECT_LITERAL_PREFIX_LEN)?;
  let end = source_end.checked_sub(OBJECT_LITERAL_PREFIX_LEN)?;

  (start <= end && end <= comment_text.len()).then_some((start, end))
}

fn raw_value<'a>(ast: &Ast, comment_text: &'a str, value: Expr) -> Option<&'a str> {
  let (start, end) = value_span_to_comment_offsets(comment_text, value.span(ast))?;
  comment_text.get(start..end).map(str::trim)
}

fn parse_magic_comment_object(comment_text: &str) -> Option<(Ast, Expr)> {
  let source = format!("({{{comment_text}}})");
  let mut ast = Ast::new(source.len(), StringAllocator::default());
  let expr = parse_file_as_expr(
    &mut ast,
    &source,
    Syntax::Es(EsSyntax::default()),
    EsVersion::EsNext,
    None,
  )
  .ok()?;
  let Expr::Paren(paren) = expr else {
    return Some((ast, expr));
  };
  let expr = paren.expr(&ast);
  Some((ast, expr))
}

fn prop_name_to_str<'a>(ast: &'a Ast, name: PropName) -> Option<Cow<'a, str>> {
  match name {
    PropName::Ident(ident) => Some(Cow::Borrowed(ast.get_utf8(ident.sym(ast)))),
    PropName::Str(str) => Some(ast.get_wtf8(str.value(ast)).to_string_lossy()),
    _ => None,
  }
}

fn expr_to_str<'a>(ast: &'a Ast, expr: Expr) -> Option<Cow<'a, str>> {
  match expr {
    Expr::Lit(Lit::Str(str)) => Some(ast.get_wtf8(str.value(ast)).to_string_lossy()),
    Expr::Tpl(tpl) if tpl.exprs(ast).is_empty() && tpl.quasis(ast).len() == 1 => {
      let quasi = ast.get_node_in_sub_range(tpl.quasis(ast).first()?);
      Some(Cow::Borrowed(ast.get_utf8(quasi.raw(ast))))
    }
    _ => None,
  }
}

fn expr_to_bool(ast: &Ast, expr: Expr) -> Option<bool> {
  match expr {
    Expr::Lit(Lit::Bool(bool)) => Some(bool.value(ast)),
    _ => None,
  }
}

fn is_number_expr(ast: &Ast, expr: Expr) -> bool {
  match expr {
    Expr::Lit(Lit::Num(_)) => true,
    Expr::Unary(unary) if matches!(unary.op(ast), UnaryOp::Minus) => {
      matches!(unary.arg(ast), Expr::Lit(Lit::Num(_)))
    }
    _ => false,
  }
}

fn expr_to_order_str<'a>(ast: &Ast, comment_text: &'a str, expr: Expr) -> Option<&'a str> {
  if expr_to_bool(ast, expr).is_some() || is_number_expr(ast, expr) {
    raw_value(ast, comment_text, expr)
  } else {
    None
  }
}

fn expr_to_regexp<'a>(ast: &'a Ast, expr: Expr) -> Option<(&'a str, &'a str)> {
  match expr {
    Expr::Lit(Lit::Regex(regex)) => {
      Some((ast.get_utf8(regex.exp(ast)), ast.get_utf8(regex.flags(ast))))
    }
    _ => None,
  }
}

fn expr_to_magic_comment_value(
  ast: &Ast,
  comment_text: &str,
  expr: Expr,
) -> Option<MagicCommentValue> {
  if let Some(value) = expr_to_bool(ast, expr) {
    return Some(MagicCommentValue::Bool(value));
  }

  if let Some(value) = expr_to_str(ast, expr) {
    return Some(MagicCommentValue::String(value.into_owned()));
  }

  if is_number_expr(ast, expr) {
    return raw_value(ast, comment_text, expr)
      .map(|value| MagicCommentValue::Number(value.to_string()));
  }

  if let Some((source, flags)) = expr_to_regexp(ast, expr) {
    return Some(MagicCommentValue::RegExp {
      source: source.to_string(),
      flags: flags.to_string(),
    });
  }

  let Expr::Array(array) = expr else {
    return None;
  };
  let mut items = Vec::new();
  for elem in array.elems(ast).iter() {
    let elem = ast.get_node_in_sub_range(elem)?;
    if elem.spread(ast).is_some() {
      return None;
    }
    items.push(expr_to_str(ast, elem.expr(ast))?.into_owned());
  }
  Some(MagicCommentValue::Array(items))
}

fn expr_to_exports(ast: &Ast, expr: Expr) -> Option<MagicCommentValue> {
  if let Some(string) = expr_to_str(ast, expr) {
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
  for elem in array.elems(ast).iter() {
    let elem = ast.get_node_in_sub_range(elem)?;
    if elem.spread(ast).is_some() {
      return None;
    }
    exports.push(expr_to_str(ast, elem.expr(ast))?.into_owned());
  }

  Some(MagicCommentValue::Array(exports))
}

fn analyze_comments(
  source: &str,
  comments: &[Comment],
  error_span: Span,
  warning_diagnostics: &mut Vec<Diagnostic>,
  result: &mut RspackCommentMap,
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
    let Some((ast, expr)) = parse_magic_comment_object(&comment.text) else {
      continue;
    };
    let Expr::Object(object) = expr else {
      continue;
    };
    for prop in object.props(&ast).iter() {
      let PropOrSpread::Prop(prop) = ast.get_node_in_sub_range(prop) else {
        continue;
      };
      let Prop::KeyValue(prop) = prop else {
        continue;
      };
      if let Some(item_name) = prop_name_to_str(&ast, prop.key(&ast)) {
        let value = prop.value(&ast);
        let received = raw_value(&ast, &comment.text, value).unwrap_or_default();
        let error_span =
          || value_span_to_error_span(comment.span, value.span(&ast)).unwrap_or(error_span.into());
        match item_name.as_ref() {
          "webpackChunkName" => {
            if let Some(value) = expr_to_str(&ast, value) {
              result.insert(
                RspackComment::ChunkName,
                MagicCommentValue::String(value.into_owned()),
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              "a string",
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackPrefetch" => {
            if let Some(value) = expr_to_order_str(&ast, &comment.text, value) {
              result.insert(
                RspackComment::Prefetch,
                if value == "true" {
                  MagicCommentValue::Bool(true)
                } else {
                  MagicCommentValue::Number(value.to_string())
                },
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              "true or a number",
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackPreload" => {
            if let Some(value) = expr_to_order_str(&ast, &comment.text, value) {
              result.insert(
                RspackComment::Preload,
                if value == "true" {
                  MagicCommentValue::Bool(true)
                } else {
                  MagicCommentValue::Number(value.to_string())
                },
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              "true or a number",
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackIgnore" => {
            if let Some(value) = expr_to_bool(&ast, value) {
              result.insert(RspackComment::Ignore, MagicCommentValue::Bool(value));
              continue;
            }
            result.insert(
              RspackComment::Ignore,
              expr_to_magic_comment_value(&ast, &comment.text, value)
                .unwrap_or(MagicCommentValue::Unknown),
            );
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              "a boolean",
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackMode" => {
            if let Some(value) = expr_to_str(&ast, value) {
              result.insert(
                RspackComment::Mode,
                MagicCommentValue::String(value.into_owned()),
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              "a string",
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackFetchPriority" => {
            if let Some(priority) = expr_to_str(&ast, value)
              && matches!(priority.as_ref(), "low" | "high" | "auto")
            {
              result.insert(
                RspackComment::FetchPriority,
                MagicCommentValue::String(priority.into_owned()),
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              r#""low", "high" or "auto""#,
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackInclude" => {
            if let Some((regexp, flags)) = expr_to_regexp(&ast, value)
              && RspackRegex::with_flags(regexp, flags).is_ok()
            {
              result.insert(
                RspackComment::IncludeRegexp,
                MagicCommentValue::RegExp {
                  source: regexp.to_string(),
                  flags: flags.to_string(),
                },
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              r#"a regular expression"#,
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackExclude" => {
            if let Some((regexp, flags)) = expr_to_regexp(&ast, value)
              && RspackRegex::with_flags(regexp, flags).is_ok()
            {
              result.insert(
                RspackComment::ExcludeRegexp,
                MagicCommentValue::RegExp {
                  source: regexp.to_string(),
                  flags: flags.to_string(),
                },
              );
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              r#"a regular expression"#,
              received,
              warning_diagnostics,
              error_span(),
            );
          }
          "webpackExports" => {
            if let Some(exports) = expr_to_exports(&ast, value) {
              result.insert(RspackComment::Exports, exports);
              continue;
            }
            add_magic_comment_warning(
              source,
              item_name.as_ref(),
              r#"a string or an array of strings"#,
              received,
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
mod tests_extract_magic_comment_object {
  use super::*;

  fn find_value(raw: &str, name: &str) -> Option<(Ast, Expr)> {
    let (ast, expr) = parse_magic_comment_object(raw)?;
    let Expr::Object(object) = expr else {
      return None;
    };
    for prop in object.props(&ast).iter() {
      let PropOrSpread::Prop(prop) = ast.get_node_in_sub_range(prop) else {
        continue;
      };
      let Prop::KeyValue(prop) = prop else {
        continue;
      };
      if prop_name_to_str(&ast, prop.key(&ast)).as_deref() == Some(name) {
        let value = prop.value(&ast);
        return Some((ast, value));
      }
    }
    None
  }

  fn try_match_string(raw: &str) -> Option<(String, String)> {
    let name = "webpackInclude";
    let (ast, value) = find_value(raw, name)?;
    Some((name.to_string(), expr_to_str(&ast, value)?.into_owned()))
  }

  fn try_match_order(raw: &str) -> Option<(String, String)> {
    let name = "webpackInclude";
    let (ast, value) = find_value(raw, name)?;
    Some((
      name.to_string(),
      expr_to_order_str(&ast, raw, value)?.to_string(),
    ))
  }

  fn try_match_regex(raw: &str) -> Option<(String, String, String)> {
    let name = "webpackInclude";
    let (ast, value) = find_value(raw, name)?;
    let (regexp, flags) = expr_to_regexp(&ast, value)?;
    Some((name.to_string(), regexp.to_string(), flags.to_string()))
  }

  fn try_match_error_range(raw: &str, name: &str) -> Option<DependencyRange> {
    let (ast, value) = find_value(raw, name)?;
    value_span_to_error_span(
      Span::new(
        swc_core::common::BytePos(100),
        swc_core::common::BytePos(100),
      ),
      value.span(&ast),
    )
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
    let value = find_value("webpackExports: [\"a\", `b`, 'c']", "webpackExports");
    assert_eq!(
      value.and_then(|(ast, value)| expr_to_exports(&ast, value)),
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
  fn test_extract_magic_comment_object() {
    test_extract_string();
    test_extract_number();
    test_extract_boolean();
    test_extract_array();
    test_extract_regexp();
    assert_eq!(
      try_match_error_range("webpackPrefetch: \"aaa\"", "webpackPrefetch"),
      Some(DependencyRange::new(118, 123))
    );
  }
}
