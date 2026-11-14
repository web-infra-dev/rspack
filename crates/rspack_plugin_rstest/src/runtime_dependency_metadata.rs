use rspack_core::{DependenciesBlock, Dependency};
use rspack_plugin_javascript::{
  RawMagicComment, try_extract_magic_comment_from_comments, visitors::JavascriptParser,
};
use rspack_util::swc::AstSubRangeExt;
use serde_json::{Value, json};
use swc_next_ecma_ast::{Ast, BinaryOperator, CommentKind, Expr, ExprData, Span};

const METADATA_KEY: &str = "rstestRuntimeDependencies";

pub fn reset(parser: &mut JavascriptParser) {
  // BuildInfo can outlive one parse during rebuilds. Replace only our field.
  parser.build_info.extras.insert(
    METADATA_KEY.to_string(),
    json!({ "version": 1, "calls": [] }),
  );
}

pub fn record_native_module_capability(parser: &mut JavascriptParser) {
  let acquires_native_module = |dependency: &dyn Dependency| {
    dependency
      .as_module_dependency()
      .is_some_and(|dependency| matches!(dependency.request(), "module" | "node:module"))
  };
  let has_native_module = parser
    .get_dependencies()
    .iter()
    .any(|dependency| acquires_native_module(dependency.as_ref()))
    || (0..parser.next_block_idx()).any(|index| {
      parser.get_block_mut(index).is_some_and(|block| {
        block
          .get_dependencies()
          .iter()
          .any(|dependency| acquires_native_module(dependency.as_ref()))
      })
    });
  if has_native_module {
    // createRequire can escape through aliases, properties, or re-exports even
    // when its parser support is disabled. Observe the existing dependency;
    // an empty call list must not imply a complete native dependency closure.
    record(
      parser,
      "require",
      "source",
      None,
      Some("native-module-capability"),
    );
  }
}

pub fn has_ignore_comment(parser: &JavascriptParser, error_span: Span, span: Span) -> bool {
  let comments = parser
    .ast
    .comments
    .leading
    .get(&span.start)
    .into_iter()
    .chain(parser.ast.comments.trailing.get(&span.end))
    .flatten()
    .filter(|comment| matches!(comment.kind, CommentKind::Block))
    .map(|comment| RawMagicComment {
      text: comment.text,
      span: comment.span.into(),
    })
    .collect::<Vec<_>>();
  if comments.is_empty() {
    return false;
  }
  // The ordinary require parser owns diagnostics. Observing a call must not
  // emit the same magic-comment warning a second time.
  try_extract_magic_comment_from_comments(parser.source(), &comments, error_span.into())
    .0
    .get_ignore()
    .unwrap_or_default()
}

fn append_literal(parts: &mut Vec<Option<String>>, value: String) {
  if let Some(Some(previous)) = parts.last_mut() {
    previous.push_str(&value);
  } else {
    parts.push(Some(value));
  }
}

fn append_hole(parts: &mut Vec<Option<String>>) {
  if !matches!(parts.last(), Some(None)) {
    parts.push(None);
  }
}

fn is_string_expression(ast: &Ast<'_>, expr: Expr) -> bool {
  match ast.expr_data(expr) {
    ExprData::StringLiteral(_) | ExprData::TemplateLiteral(_) => true,
    ExprData::BinaryExpression(binary) if binary.operator(ast) == BinaryOperator::Add => {
      is_string_expression(ast, binary.left(ast)) || is_string_expression(ast, binary.right(ast))
    }
    _ => false,
  }
}

fn append_expression(ast: &Ast<'_>, expr: Expr, parts: &mut Vec<Option<String>>) {
  match ast.expr_data(expr) {
    ExprData::StringLiteral(literal) => {
      append_literal(
        parts,
        ast
          .get_wtf8(literal.value(ast))
          .to_string_lossy()
          .into_owned(),
      );
    }
    ExprData::TemplateLiteral(template) => {
      let mut expressions = ast.nodes(template.expressions(ast));
      for quasi in ast.nodes(template.quasis(ast)) {
        if quasi.is_cooked_undefined(ast) {
          append_hole(parts);
        } else {
          append_literal(
            parts,
            ast
              .get_wtf8(quasi.cooked(ast))
              .to_string_lossy()
              .into_owned(),
          );
        }
        if let Some(expression) = expressions.next() {
          append_expression(ast, expression, parts);
        }
      }
    }
    ExprData::BinaryExpression(binary)
      if binary.operator(ast) == BinaryOperator::Add && is_string_expression(ast, expr) =>
    {
      append_expression(ast, binary.left(ast), parts);
      append_expression(ast, binary.right(ast), parts);
    }
    _ => append_hole(parts),
  }
}

pub fn is_static_request(ast: &Ast<'_>, expression: Expr) -> bool {
  let mut parts = Vec::new();
  append_expression(ast, expression, &mut parts);
  parts.iter().all(Option::is_some)
}

pub fn record(
  parser: &mut JavascriptParser,
  kind: &str,
  origin: &str,
  expression: Option<Expr>,
  unsupported: Option<&str>,
) {
  let mut parts = Vec::new();
  if let Some(expression) = expression {
    append_expression(parser.ast.ast, expression, &mut parts);
  }
  let unsupported = unsupported.or_else(|| {
    (!parts
      .iter()
      .any(|part| part.as_ref().is_some_and(|literal| !literal.is_empty())))
    .then_some("opaque-request")
  });
  let mut call = json!({ "kind": kind, "origin": origin, "parts": parts });
  if let Some(reason) = unsupported {
    call["unsupported"] = Value::String(reason.to_string());
  }
  parser
    .build_info
    .extras
    .get_mut(METADATA_KEY)
    .and_then(|metadata| metadata.get_mut("calls"))
    .and_then(Value::as_array_mut)
    .expect("Rstest runtime metadata is initialized before walking the program")
    .push(call);
}
