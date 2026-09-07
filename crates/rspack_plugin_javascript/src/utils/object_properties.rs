//! Property lookup and typed extraction utilities for AST object literals.
//!
//! `#[derive(AstObject)]` (from `rspack_macros`) generates a
//! `from_ast_object` constructor for plain data structs, so options objects
//! in parser plugins can be declared once and extracted from a call
//! expression's arguments instead of hand-rolling per-property lookups:
//!
//! ```ignore
//! #[derive(Debug, Default, AstObject)]
//! #[ast_object(rename_all = "camelCase")]
//! struct GlobOptions {
//!   eager: bool,
//!   import: Option<String>,
//!   #[ast_object(skip, default = "default_case_sensitive")]
//!   case_sensitive: bool,
//! }
//!
//! let options = obj
//!   .map(GlobOptions::from_ast_object)
//!   .transpose()?
//!   .unwrap_or_default();
//! ```
//!
//! This module provides the [`FromAstExpr`] trait the generated code uses to
//! extract each field value, plus implementations for the common value
//! shapes. Field types only need `FromAstExpr + Default`; an absent or
//! statically unresolvable property always falls back to the field default,
//! while conversion failures are preserved. Custom value shapes (enums,
//! nested options objects) participate by implementing [`FromAstExpr`], and
//! `Vec<(String, T)>` extracts records with unknown keys in source order.

use rspack_core::{ContextMode, ImportAttributes, try_convert_str_to_context_mode};
use rspack_error::{Error, Label, Result};
use rspack_regex::RspackRegex;
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  Ast, Expr, ExprData, GetSpan, ImportAttribute, ModuleExportNameData, ObjectExpression,
  PropertyKey, PropertyKeyData, TypedSubRange,
};

use crate::visitors::static_string_from_expr;

pub fn get_value_by_obj_prop(ast: &Ast<'_>, object: ObjectExpression, field: &str) -> Option<Expr> {
  object.properties(ast).iter().rev().find_map(|property| {
    let property = ast.get_node_in_sub_range(property);
    let property = property.as_object_property(ast)?;
    (static_property_name(ast, property.key(ast)).as_deref() == Some(field))
      .then(|| property.value(ast))
  })
}

pub fn get_attributes(ast: &Ast<'_>, object: ObjectExpression) -> ImportAttributes {
  object
    .properties(ast)
    .iter()
    .filter_map(|property| {
      let property = ast.get_node_in_sub_range(property);
      let property = property.as_object_property(ast)?;
      let key = static_property_name(ast, property.key(ast))?;
      let ExprData::StringLiteral(value) = ast.expr_data(property.value(ast)) else {
        return None;
      };
      Some((
        key,
        ast
          .get_wtf8(value.value(ast))
          .to_string_lossy()
          .into_owned(),
      ))
    })
    .collect()
}

pub fn get_import_attributes(
  ast: &Ast<'_>,
  attributes: TypedSubRange<ImportAttribute>,
) -> Option<ImportAttributes> {
  if attributes.is_empty() {
    return None;
  }
  Some(
    attributes
      .iter()
      .map(|id| {
        let attribute = ast.get_node_in_sub_range(id);
        let key = match ast.module_export_name_data(attribute.key(ast)) {
          ModuleExportNameData::IdentifierName(identifier) => {
            ast.get_utf8(identifier.name(ast)).to_string()
          }
          ModuleExportNameData::StringLiteral(string) => ast
            .get_wtf8(string.value(ast))
            .to_string_lossy()
            .into_owned(),
        };
        let value = ast
          .get_wtf8(attribute.value(ast).value(ast))
          .to_string_lossy()
          .into_owned();
        (key, value)
      })
      .collect(),
  )
}

/// Look up a value expression by key in an AST object literal.
pub fn get_property(ast: &Ast<'_>, object: ObjectExpression, key: &str) -> Option<Expr> {
  get_value_by_obj_prop(ast, object, key)
}

/// Look up a nested value from an AST object literal by a non-empty key path.
pub fn get_from_object(ast: &Ast<'_>, object: ObjectExpression, path: &[&str]) -> Option<Expr> {
  let (key, remaining) = path.split_first()?;
  get(ast, get_property(ast, object, key)?, remaining)
}

/// Look up a nested value from an AST object literal and extract it as `T`.
///
/// `#[derive(AstObject)]` delegates field extraction to this helper so derived
/// options and path-based lookups share the same lookup and conversion logic.
pub fn get_value_from_object<T: FromAstExpr>(
  ast: &Ast<'_>,
  object: ObjectExpression,
  path: &[&str],
) -> Result<Option<T>> {
  let Some(expr) = get_from_object(ast, object, path) else {
    return Ok(None);
  };
  extract_value(ast, expr)
}

/// Look up a nested value in AST object literals by a key path, like
/// lodash's `get`. Returns the value expression at the path, or `None` if
/// any segment is missing or is not an object literal. An empty path returns
/// the expression itself.
pub fn get(ast: &Ast<'_>, expr: Expr, path: &[&str]) -> Option<Expr> {
  let mut current = expr;
  for key in path {
    current = get_property(ast, current.as_object_expression(ast)?, key)?;
  }
  Some(current)
}

/// Look up a nested value by a key path and extract it as `T`, combining
/// [`get`] with [`FromAstExpr`].
pub fn get_value<T: FromAstExpr>(ast: &Ast<'_>, expr: Expr, path: &[&str]) -> Result<Option<T>> {
  let Some(expr) = get(ast, expr, path) else {
    return Ok(None);
  };
  extract_value(ast, expr)
}

fn extract_value<T: FromAstExpr>(ast: &Ast<'_>, expr: Expr) -> Result<Option<T>> {
  T::from_ast_expr(ast, expr).map_err(|mut error| {
    if error.labels.is_none() {
      let span = expr.span(ast);
      error.labels = Some(vec![Label {
        name: None,
        offset: span.real_lo() as usize,
        len: span.real_hi().saturating_sub(span.real_lo()) as usize,
      }]);
    }
    error
  })
}

/// Extract a typed value from an AST expression.
///
/// `Ok(None)` means that the expression is not a statically resolvable
/// representation of the requested type. `Err` preserves a conversion failure
/// after the expression has been recognized as that type.
pub trait FromAstExpr: Sized {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>>;
}

impl FromAstExpr for bool {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    Ok(match ast.expr_data(expr) {
      ExprData::BooleanLiteral(boolean) => Some(boolean.value(ast)),
      _ => None,
    })
  }
}

impl FromAstExpr for String {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    Ok(static_string_from_expr(ast, expr))
  }
}

impl FromAstExpr for ContextMode {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    let Some(value) = String::from_ast_expr(ast, expr)? else {
      return Ok(None);
    };

    try_convert_str_to_context_mode(&value)
      .map(Some)
      .ok_or_else(|| {
        Error::error(format!(
          r#"Unsupported mode: `mode` expected "sync", "eager", "weak", "async-weak", "lazy" or "lazy-once", but received: "{value}"."#
        ))
      })
  }
}

impl FromAstExpr for f64 {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    Ok(match ast.expr_data(expr) {
      ExprData::NumericLiteral(number) => Some(number.value(ast)),
      _ => None,
    })
  }
}

impl<T: FromAstExpr> FromAstExpr for Option<T> {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    T::from_ast_expr(ast, expr).map(|value| value.map(Some))
  }
}

impl FromAstExpr for RspackRegex {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    let ExprData::RegExpLiteral(regex) = ast.expr_data(expr) else {
      return Ok(None);
    };
    RspackRegex::with_flags(
      ast.get_utf8(regex.pattern(ast)),
      ast.get_utf8(regex.flags(ast)),
    )
    .map(Some)
  }
}

/// Extract an object literal into key-value pairs in source order, for
/// options with unknown (dynamic) keys such as records. Every key must be
/// statically resolvable and every value must extract as `T`, otherwise the
/// whole extraction fails.
impl<T: FromAstExpr> FromAstExpr for Vec<(String, T)> {
  fn from_ast_expr(ast: &Ast<'_>, expr: Expr) -> Result<Option<Self>> {
    let Some(object) = expr.as_object_expression(ast) else {
      return Ok(None);
    };
    object
      .properties(ast)
      .iter()
      .map(|prop| -> Result<Option<(String, T)>> {
        let prop = ast.get_node_in_sub_range(prop);
        let Some(property) = prop.as_object_property(ast) else {
          return Ok(None);
        };
        let Some(key) = static_property_name(ast, property.key(ast)) else {
          return Ok(None);
        };
        let Some(value) = T::from_ast_expr(ast, property.value(ast))? else {
          return Ok(None);
        };
        Ok(Some((key, value)))
      })
      .collect::<Result<Option<Vec<_>>>>()
  }
}

/// Statically resolve an object literal property name, mirroring the key
/// handling of the previous hand-rolled options parsing.
fn static_property_name(ast: &Ast<'_>, key: PropertyKey) -> Option<String> {
  match ast.property_key_data(key) {
    PropertyKeyData::IdentifierName(identifier) => {
      Some(ast.get_utf8(identifier.name(ast)).to_string())
    }
    PropertyKeyData::StringLiteral(string) => Some(
      ast
        .get_wtf8(string.value(ast))
        .to_string_lossy()
        .into_owned(),
    ),
    PropertyKeyData::NumericLiteral(number) => Some(number.value(ast).to_string()),
    PropertyKeyData::Expr(expression) => {
      static_string_from_expr(ast, expression).or_else(|| match ast.expr_data(expression) {
        ExprData::NumericLiteral(number) => Some(number.value(ast).to_string()),
        ExprData::BooleanLiteral(boolean) => Some(boolean.value(ast).to_string()),
        ExprData::NullLiteral(_) => Some("null".to_string()),
        _ => None,
      })
    }
    PropertyKeyData::PrivateIdentifier(_) | PropertyKeyData::BigIntLiteral(_) => None,
  }
}

#[cfg(test)]
mod tests {
  use rspack_macros::AstObject;
  use swc_next_allocator::Allocator;
  use swc_next_ecma_ast::Ast;
  use swc_next_ecma_parser::{FragmentContext, Options, Parser, TokenParserConfig};

  use super::*;

  fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
    Parser::init(allocator, source, Options::default(), TokenParserConfig)
      .parse_expression_fragment(FragmentContext::TopLevel)
      .expect("failed to parse test expression")
  }

  fn default_true() -> bool {
    true
  }

  #[derive(Debug, Default, PartialEq, AstObject)]
  #[ast_object(rename_all = "camelCase")]
  struct TestOptions {
    eager: bool,
    case_sensitive: Option<bool>,
    import: Option<String>,
    #[ast_object(skip, default = "default_true")]
    skipped: bool,
  }

  #[test]
  fn extracts_camel_case_fields_and_defaults() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ eager: true, caseSensitive: false }");
    let expr = ast.root_expression();
    let options =
      TestOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap()).unwrap();
    assert_eq!(
      options,
      TestOptions {
        eager: true,
        case_sensitive: Some(false),
        import: None,
        skipped: true,
      }
    );

    let ast = parse_expr(&allocator, "{}");
    let expr = ast.root_expression();
    assert_eq!(
      TestOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap()).unwrap(),
      TestOptions {
        skipped: true,
        ..Default::default()
      }
    );
  }

  #[test]
  fn unrecognized_values_fall_back_to_default() {
    let allocator = Allocator::new();
    let ast = parse_expr(
      &allocator,
      "{ eager: \"yes\", import: 1, caseSensitive: null }",
    );
    let expr = ast.root_expression();
    assert_eq!(
      TestOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap()).unwrap(),
      TestOptions {
        skipped: true,
        ..Default::default()
      }
    );

    let ast = parse_expr(&allocator, "true");
    assert_eq!(
      TestOptions::from_ast_expr(&ast, ast.root_expression()).unwrap(),
      None
    );
  }

  #[test]
  fn resolves_template_literals() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ import: `default` }");
    let expr = ast.root_expression();
    assert_eq!(
      TestOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap())
        .unwrap()
        .import,
      Some("default".to_string())
    );
  }

  #[derive(Debug, Default, PartialEq, AstObject)]
  struct TestOuter {
    inner: Option<TestOptions>,
  }

  #[test]
  fn extracts_nested_options_objects() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ inner: { eager: true } }");
    let expr = ast.root_expression();
    let options =
      TestOuter::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap()).unwrap();
    assert_eq!(
      options.inner,
      Some(TestOptions {
        eager: true,
        skipped: true,
        ..Default::default()
      })
    );
  }

  #[test]
  fn extracts_records_in_source_order() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ b: '1', a: `2`, [\"c\"]: '3' }");
    let record = Vec::<(String, String)>::from_ast_expr(&ast, ast.root_expression())
      .unwrap()
      .unwrap();
    assert_eq!(
      record,
      vec![
        ("b".to_string(), "1".to_string()),
        ("a".to_string(), "2".to_string()),
        ("c".to_string(), "3".to_string()),
      ]
    );
  }

  #[test]
  fn record_extraction_is_all_or_nothing() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ a: '1', [Symbol()]: '2' }");
    assert_eq!(
      Vec::<(String, String)>::from_ast_expr(&ast, ast.root_expression()).unwrap(),
      None
    );
  }

  #[derive(Debug, Default, AstObject)]
  #[ast_object(rename_all = "camelCase")]
  struct TestRegexOptions {
    reg_exp: Option<RspackRegex>,
    recursive: Option<bool>,
  }

  #[test]
  fn extracts_regex_literals() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ regExp: /abc/gi }");
    let expr = ast.root_expression();
    let regex = TestRegexOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap())
      .unwrap()
      .reg_exp
      .expect("expected a regex");
    assert_eq!(regex.source(), "abc");
    assert_eq!(regex.flags(), "gi");

    let ast = parse_expr(&allocator, "{ regExp: \"nope\" }");
    let expr = ast.root_expression();
    assert!(
      TestRegexOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap())
        .unwrap()
        .reg_exp
        .is_none()
    );
  }

  #[test]
  fn preserves_regex_conversion_errors() {
    let allocator = Allocator::new();
    let ast = parse_expr(
      &allocator,
      "{ regExp: /(?<name>a)(?<name>b)/, recursive: false }",
    );
    let expr = ast.root_expression();
    let error = TestRegexOptions::from_ast_object(&ast, expr.as_object_expression(&ast).unwrap())
      .expect_err("regex conversion failures should not be treated as an absent option");

    assert!(error.to_string().contains("/(?<name>a)(?<name>b)/"));
  }

  #[test]
  fn preserves_valid_fields_when_collecting_conversion_diagnostics() {
    let allocator = Allocator::new();
    let ast = parse_expr(
      &allocator,
      "{ regExp: /(?<name>a)(?<name>b)/, recursive: false }",
    );
    let expr = ast.root_expression();
    let (options, diagnostics) = TestRegexOptions::from_ast_object_with_diagnostics(
      &ast,
      expr.as_object_expression(&ast).unwrap(),
    );

    assert!(options.reg_exp.is_none());
    assert_eq!(options.recursive, Some(false));
    assert_eq!(diagnostics.len(), 1);
    assert!(
      diagnostics[0]
        .to_string()
        .contains("/(?<name>a)(?<name>b)/")
    );
  }
}

#[cfg(test)]
mod get_tests {
  use swc_next_allocator::Allocator;
  use swc_next_ecma_ast::Ast;
  use swc_next_ecma_parser::{FragmentContext, Options, Parser, TokenParserConfig};

  use super::*;

  fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Ast<'a> {
    Parser::init(allocator, source, Options::default(), TokenParserConfig)
      .parse_expression_fragment(FragmentContext::TopLevel)
      .expect("failed to parse test expression")
  }

  #[test]
  fn gets_nested_values_by_key_path() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ a: { b: { c: 42 } } }");
    let expr = ast.root_expression();
    assert_eq!(
      get_value_from_object::<f64>(&ast, expr.as_object_expression(&ast).unwrap(), &["a"]).unwrap(),
      None
    );
    assert_eq!(
      get_value::<f64>(&ast, expr, &["a", "b", "c"]).unwrap(),
      Some(42.0)
    );
    assert!(
      get(&ast, expr, &["a", "b"]).is_some_and(|expr| expr.as_object_expression(&ast).is_some())
    );
    // An empty path is the identity.
    assert!(get(&ast, expr, &[]).is_some());
  }

  #[test]
  fn gets_typed_values_from_object_paths() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ enabled: true, label: 'test' }");
    let expr = ast.root_expression();
    let object = expr.as_object_expression(&ast).unwrap();

    let dynamic_key = "enabled".to_string();
    assert_eq!(
      get_value_from_object::<bool>(&ast, object, &[dynamic_key.as_str()]).unwrap(),
      Some(true)
    );
    assert_eq!(
      get_value_from_object::<String>(&ast, object, &["label"]).unwrap(),
      Some("test".to_string())
    );
    assert_eq!(
      get_value_from_object::<bool>(&ast, object, &["missing"]).unwrap(),
      None
    );
    assert_eq!(
      get_value_from_object::<bool>(&ast, object, &[]).unwrap(),
      None
    );
  }

  #[test]
  fn later_duplicate_properties_take_precedence() {
    let allocator = Allocator::new();
    let ast = parse_expr(
      &allocator,
      "{ enabled: false, enabled: true, nested: { value: 1, value: 2 } }",
    );
    let expr = ast.root_expression();
    let object = expr.as_object_expression(&ast).unwrap();

    assert_eq!(
      get_value_from_object::<bool>(&ast, object, &["enabled"]).unwrap(),
      Some(true)
    );
    assert_eq!(
      get_value::<f64>(&ast, expr, &["nested", "value"]).unwrap(),
      Some(2.0)
    );
  }

  #[test]
  fn get_returns_none_for_missing_or_non_object_segments() {
    let allocator = Allocator::new();
    let ast = parse_expr(&allocator, "{ a: { b: 1 } }");
    let expr = ast.root_expression();
    assert_eq!(get_value::<f64>(&ast, expr, &["a", "x"]).unwrap(), None);
    assert_eq!(get_value::<f64>(&ast, expr, &["x"]).unwrap(), None);
    // Intermediate value is not an object.
    assert_eq!(
      get_value::<f64>(&ast, expr, &["a", "b", "c"]).unwrap(),
      None
    );
    // The leaf exists but does not match the requested type.
    assert_eq!(get_value::<String>(&ast, expr, &["a", "b"]).unwrap(), None);
  }
}
