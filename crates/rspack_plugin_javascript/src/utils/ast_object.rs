//! Lightweight typed extraction of options objects from AST object literals.
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

use swc_experimental_ecma_ast::{Expr, Lit, PropName, Span};

use crate::visitors::static_string_from_expr;

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

/// A regex literal extracted from an options object, with its source span
/// preserved for later diagnostics.
#[derive(Debug)]
pub struct AstRegex {
  pub exp: String,
  pub flags: String,
  pub span: Span,
}

impl FromAstExpr<'_> for AstRegex {
  fn from_ast_expr(expr: &Expr<'_>) -> Option<Self> {
    match expr.as_lit()? {
      Lit::Regex(regex) => Some(Self {
        exp: regex.exp.to_string(),
        flags: regex.flags.to_string(),
        span: regex.span,
      }),
      _ => None,
    }
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
    reg_exp: Option<AstRegex>,
  }

  #[test]
  fn extracts_regex_literals_with_span() {
    let allocator = Allocator::new();
    let expr = parse_expr(&allocator, "{ regExp: /abc/gi }");
    let regex = TestRegexOptions::from_ast_object(expr.as_object().unwrap())
      .reg_exp
      .expect("expected a regex");
    assert_eq!(regex.exp, "abc");
    assert_eq!(regex.flags, "gi");
    assert_eq!(regex.span.end - regex.span.start, "/abc/gi".len() as u32);

    let expr = parse_expr(&allocator, "{ regExp: \"nope\" }");
    assert!(
      TestRegexOptions::from_ast_object(expr.as_object().unwrap())
        .reg_exp
        .is_none()
    );
  }
}
