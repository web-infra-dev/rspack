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
//! let options = obj.map(GlobOptions::from_ast_object).unwrap_or_default();
//! ```
//!
//! This module provides the [`FromAstExpr`] trait the generated code uses to
//! extract each field value, plus implementations for the common value
//! shapes. Field types only need `FromAstExpr + Default`; an absent or
//! statically unresolvable property always falls back to the field default,
//! matching the previous hand-rolled behavior. Custom value shapes (enums,
//! nested options objects) participate by implementing [`FromAstExpr`], and
//! `Vec<(String, T)>` extracts records with unknown keys in source order.

use rspack_core::ImportAttributes;
use rspack_regex::RspackRegex;
use swc_experimental_ecma_ast::{Bool, Expr, Lit, ObjectLit, PropName, Regex, Str};

use crate::visitors::static_string_from_expr;

pub fn get_value_by_obj_prop<'r, 'ast>(
  obj: &'r ObjectLit<'ast>,
  field: &str,
) -> Option<&'r Expr<'ast>> {
  obj.props.iter().find_map(|p| {
    let prop = p.as_prop()?;
    let kv = prop.as_key_value()?;
    let matched = kv.key.as_ident().is_some_and(|key| key.sym == field)
      || kv
        .key
        .as_str()
        .is_some_and(|key| key.value.as_str() == Some(field));
    matched.then_some(&kv.value)
  })
}

pub fn get_literal_str_by_obj_prop<'r, 'ast>(
  obj: &'r ObjectLit<'ast>,
  field: &str,
) -> Option<&'r Str<'ast>> {
  let lit = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Str(str) => Some(str),
    _ => None,
  }
}

pub fn get_bool_by_obj_prop<'r, 'ast>(obj: &'r ObjectLit<'ast>, field: &str) -> Option<&'r Bool> {
  let lit = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Bool(bool) => Some(bool),
    _ => None,
  }
}

pub fn get_regex_by_obj_prop<'r, 'ast>(
  obj: &'r ObjectLit<'ast>,
  field: &str,
) -> Option<&'r Regex<'ast>> {
  let lit = get_value_by_obj_prop(obj, field).and_then(|e| e.as_lit())?;
  match lit {
    Lit::Regex(regexp) => Some(regexp),
    _ => None,
  }
}

pub fn get_attributes(obj: &ObjectLit<'_>) -> ImportAttributes {
  obj
    .props
    .iter()
    .filter_map(|p| {
      p.as_prop().and_then(|p| p.as_key_value()).and_then(|kv| {
        kv.key
          .as_ident()
          .map(|k| k.sym.as_str())
          .or_else(|| kv.key.as_str().and_then(|k| k.value.as_str()))
          .map(|s| s.to_string())
          .zip(kv.value.as_lit().and_then(|lit| match lit {
            Lit::Str(s) => Some(s.value.to_string_lossy().to_string()),
            _ => None,
          }))
      })
    })
    .collect()
}

/// Look up a value expression by key in an AST object literal.
pub fn get_property<'r, 'ast>(obj: &'r ObjectLit<'ast>, key: &str) -> Option<&'r Expr<'ast>> {
  get_value_by_obj_prop(obj, key)
}

/// Look up a nested value from an AST object literal by a non-empty key path.
pub fn get_from_object<'r, 'ast>(
  object: &'r ObjectLit<'ast>,
  path: &[&str],
) -> Option<&'r Expr<'ast>> {
  let (key, remaining) = path.split_first()?;
  get(get_property(object, key)?, remaining)
}

/// Look up a nested value from an AST object literal and extract it as `T`.
///
/// `#[derive(AstObject)]` delegates field extraction to this helper so derived
/// options and path-based lookups share the same lookup and conversion logic.
pub fn get_value_from_object<'r, 'ast, T: FromAstExpr<'ast>>(
  object: &'r ObjectLit<'ast>,
  path: &[&str],
) -> Option<T> {
  get_from_object(object, path).and_then(T::from_ast_expr)
}

/// Look up a nested value in AST object literals by a key path, like
/// lodash's `get`. Returns the value expression at the path, or `None` if
/// any segment is missing or is not an object literal. An empty path returns
/// the expression itself.
pub fn get<'r, 'ast>(expr: &'r Expr<'ast>, path: &[&str]) -> Option<&'r Expr<'ast>> {
  let mut current = expr;
  for key in path {
    current = get_property(current.as_object()?, key)?;
  }
  Some(current)
}

/// Look up a nested value by a key path and extract it as `T`, combining
/// [`get`] with [`FromAstExpr`].
pub fn get_value<'r, 'ast, T: FromAstExpr<'ast>>(expr: &'r Expr<'ast>, path: &[&str]) -> Option<T> {
  get(expr, path).and_then(|expr| T::from_ast_expr(expr))
}

/// Extract a typed value from an AST expression, if the expression is a
/// statically resolvable representation of the value.
pub trait FromAstExpr<'a>: Sized {
  fn from_ast_expr(expr: &Expr<'a>) -> Option<Self>;
}

impl FromAstExpr<'_> for bool {
  fn from_ast_expr(expr: &Expr<'_>) -> Option<Self> {
    match expr.as_lit()? {
      Lit::Bool(bool) => Some(bool.value),
      _ => None,
    }
  }
}

impl FromAstExpr<'_> for String {
  fn from_ast_expr(expr: &Expr<'_>) -> Option<Self> {
    static_string_from_expr(expr)
  }
}

impl FromAstExpr<'_> for f64 {
  fn from_ast_expr(expr: &Expr<'_>) -> Option<Self> {
    match expr.as_lit()? {
      Lit::Num(num) => Some(num.value),
      _ => None,
    }
  }
}

impl<'a, T: FromAstExpr<'a>> FromAstExpr<'a> for Option<T> {
  fn from_ast_expr(expr: &Expr<'a>) -> Option<Self> {
    T::from_ast_expr(expr).map(Some)
  }
}

impl FromAstExpr<'_> for RspackRegex {
  fn from_ast_expr(expr: &Expr<'_>) -> Option<Self> {
    let Lit::Regex(regex) = expr.as_lit()? else {
      return None;
    };
    RspackRegex::with_flags(regex.exp.as_ref(), regex.flags.as_ref()).ok()
  }
}

/// Extract an object literal into key-value pairs in source order, for
/// options with unknown (dynamic) keys such as records. Every key must be
/// statically resolvable and every value must extract as `T`, otherwise the
/// whole extraction fails.
impl<'a, T: FromAstExpr<'a>> FromAstExpr<'a> for Vec<(String, T)> {
  fn from_ast_expr(expr: &Expr<'a>) -> Option<Self> {
    let obj = expr.as_object()?;
    obj
      .props
      .iter()
      .map(|prop| {
        let kv = prop.as_prop().and_then(|prop| prop.as_key_value())?;
        let key = static_prop_name(&kv.key)?;
        let value = T::from_ast_expr(&kv.value)?;
        Some((key, value))
      })
      .collect()
  }
}

/// Statically resolve an object literal property name, mirroring the key
/// handling of the previous hand-rolled options parsing.
fn static_prop_name(key: &PropName) -> Option<String> {
  match key {
    PropName::Ident(ident) => Some(ident.sym.to_string()),
    PropName::Str(str) => Some(str.value.to_string_lossy().into_owned()),
    PropName::Num(num) => Some(num.value.to_string()),
    PropName::Computed(computed) => {
      if let Some(key) = static_string_from_expr(&computed.expr) {
        return Some(key);
      }
      match computed.expr.as_lit()? {
        Lit::Num(num) => Some(num.value.to_string()),
        Lit::Bool(bool) => Some(bool.value.to_string()),
        Lit::Null(_) => Some("null".to_string()),
        _ => None,
      }
    }
    PropName::BigInt(_) => None,
  }
}

#[cfg(test)]
mod tests {
  use rspack_macros::AstObject;
  use swc_experimental_allocator::Allocator;
  use swc_experimental_ecma_ast::EsVersion;
  use swc_experimental_ecma_parser::{
    EsSyntax, Lexer, Parser, StringSource, Syntax, unstable::Capturing,
  };

  use super::*;

  fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Expr<'a> {
    let lexer = Lexer::new(
      allocator,
      Syntax::Es(EsSyntax::default()),
      EsVersion::EsNext,
      StringSource::new(source),
      None,
    );
    let lexer = Capturing::new(lexer);
    let mut parser = Parser::new_from(allocator, lexer);
    parser
      .parse_expr()
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
    let expr = parse_expr(&allocator, "{ eager: true, caseSensitive: false }");
    let options = TestOptions::from_ast_object(expr.as_object().unwrap());
    assert_eq!(
      options,
      TestOptions {
        eager: true,
        case_sensitive: Some(false),
        import: None,
        skipped: true,
      }
    );

    let expr = parse_expr(&allocator, "{}");
    assert_eq!(
      TestOptions::from_ast_object(expr.as_object().unwrap()),
      TestOptions {
        skipped: true,
        ..Default::default()
      }
    );
  }

  #[test]
  fn unrecognized_values_fall_back_to_default() {
    let allocator = Allocator::new();
    let expr = parse_expr(
      &allocator,
      "{ eager: \"yes\", import: 1, caseSensitive: null }",
    );
    assert_eq!(
      TestOptions::from_ast_object(expr.as_object().unwrap()),
      TestOptions {
        skipped: true,
        ..Default::default()
      }
    );

    let expr = parse_expr(&allocator, "true");
    assert_eq!(TestOptions::from_ast_expr(&expr), None);
  }

  #[test]
  fn resolves_template_literals() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ import: `default` }");
    assert_eq!(
      TestOptions::from_ast_object(expr.as_object().unwrap()).import,
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
    let expr = parse_expr(&allocator, "{ inner: { eager: true } }");
    let options = TestOuter::from_ast_object(expr.as_object().unwrap());
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
    let expr = parse_expr(&allocator, "{ b: '1', a: `2`, [\"c\"]: '3' }");
    let record = Vec::<(String, String)>::from_ast_expr(&expr).unwrap();
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
    let expr = parse_expr(&allocator, "{ a: '1', [Symbol()]: '2' }");
    assert_eq!(Vec::<(String, String)>::from_ast_expr(&expr), None);
  }

  #[derive(Debug, Default, AstObject)]
  #[ast_object(rename_all = "camelCase")]
  struct TestRegexOptions {
    reg_exp: Option<RspackRegex>,
  }

  #[test]
  fn extracts_regex_literals() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ regExp: /abc/gi }");
    let regex = TestRegexOptions::from_ast_object(expr.as_object().unwrap())
      .reg_exp
      .expect("expected a regex");
    assert_eq!(regex.source(), "abc");
    assert_eq!(regex.flags(), "gi");

    let expr = parse_expr(&allocator, "{ regExp: \"nope\" }");
    assert!(
      TestRegexOptions::from_ast_object(expr.as_object().unwrap())
        .reg_exp
        .is_none()
    );
  }
}

#[cfg(test)]
mod get_tests {
  use swc_experimental_allocator::Allocator;
  use swc_experimental_ecma_ast::EsVersion;
  use swc_experimental_ecma_parser::{
    EsSyntax, Lexer, Parser, StringSource, Syntax, unstable::Capturing,
  };

  use super::*;

  fn parse_expr<'a>(allocator: &'a Allocator, source: &'a str) -> Expr<'a> {
    let lexer = Lexer::new(
      allocator,
      Syntax::Es(EsSyntax::default()),
      EsVersion::EsNext,
      StringSource::new(source),
      None,
    );
    let lexer = Capturing::new(lexer);
    let mut parser = Parser::new_from(allocator, lexer);
    parser
      .parse_expr()
      .expect("failed to parse test expression")
  }

  #[test]
  fn gets_nested_values_by_key_path() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ a: { b: { c: 42 } } }");
    assert_eq!(
      get_value_from_object::<f64>(expr.as_object().unwrap(), &["a"]),
      None
    );
    assert_eq!(get_value::<f64>(&expr, &["a", "b", "c"]), Some(42.0));
    assert!(get(&expr, &["a", "b"]).is_some_and(|expr| expr.as_object().is_some()));
    // An empty path is the identity.
    assert!(get(&expr, &[]).is_some());
  }

  #[test]
  fn gets_typed_values_from_object_paths() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ enabled: true, label: 'test' }");
    let object = expr.as_object().unwrap();

    let dynamic_key = "enabled".to_string();
    assert_eq!(
      get_value_from_object::<bool>(object, &[dynamic_key.as_str()]),
      Some(true)
    );
    assert_eq!(
      get_value_from_object::<String>(object, &["label"]),
      Some("test".to_string())
    );
    assert_eq!(get_value_from_object::<bool>(object, &["missing"]), None);
    assert_eq!(get_value_from_object::<bool>(object, &[]), None);
  }

  #[test]
  fn get_returns_none_for_missing_or_non_object_segments() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ a: { b: 1 } }");
    assert_eq!(get_value::<f64>(&expr, &["a", "x"]), None);
    assert_eq!(get_value::<f64>(&expr, &["x"]), None);
    // Intermediate value is not an object.
    assert_eq!(get_value::<f64>(&expr, &["a", "b", "c"]), None);
    // The leaf exists but does not match the requested type.
    assert_eq!(get_value::<String>(&expr, &["a", "b"]), None);
  }
}
