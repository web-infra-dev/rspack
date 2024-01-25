use itertools::Itertools;
use rspack_core::{DependencyLocation, SpanExt};
use swc_core::{
  atoms::Atom,
  common::Spanned,
  ecma::ast::{Expr, MemberExpr},
};

use super::JavascriptParserPlugin;
use crate::{dependency::ProvideDependency, visitors::JavascriptParser};

const SOURCE_DOT: &str = r#"."#;
const MODULE_DOT: &str = r#"_dot_"#;

fn dep(parser: &JavascriptParser, target: &str, start: u32, end: u32) -> Option<ProvideDependency> {
  for (name, requests) in &parser.compiler_options.builtins.provide {
    if name != target {
      continue;
    }
    let name_identifier = if name.contains(SOURCE_DOT) {
      format!("__webpack_provide_{}", name.replace(SOURCE_DOT, MODULE_DOT))
    } else {
      name.to_string()
    };
    return Some(ProvideDependency::new(
      start,
      end,
      Atom::from(requests[0].as_str()),
      name_identifier,
      requests[1..]
        .iter()
        .map(|s| Atom::from(s.as_str()))
        .collect_vec(),
    ));
  }
  None
}

fn get_nested_identifier_name_from_member(member: &MemberExpr) -> Option<String> {
  let Some(mut obj) = get_nested_identifier_name(&member.obj) else {
    return None;
  };
  if let Some(ident_prop) = member.prop.as_ident() {
    obj.push('.');
    obj.push_str(&ident_prop.sym);
  }
  Some(obj)
}

fn get_nested_identifier_name(expr: &Expr) -> Option<String> {
  match expr {
    Expr::Member(member) => get_nested_identifier_name_from_member(member),
    Expr::Ident(ident) => Some(ident.sym.to_string()),
    Expr::This(_) => Some("this".to_string()),
    _ => None,
  }
}

pub struct ProviderPlugin;

impl JavascriptParserPlugin for ProviderPlugin {
  fn call(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    expr: &swc_core::ecma::ast::CallExpr,
  ) -> Option<bool> {
    let Some(s) = expr
      .callee
      .as_expr()
      .and_then(|expr| get_nested_identifier_name(expr))
    else {
      return None;
    };
    dep(
      parser,
      &s,
      expr.callee.span().real_lo(),
      expr.callee.span().real_hi(),
    )
    .map(|dep| {
      parser.dependencies.push(Box::new(dep));
      // FIXME: webpack use `walk_expression` here
      parser.walk_expr_or_spread(&expr.args);
      true
    })
  }

  fn member(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    expr: &swc_core::ecma::ast::MemberExpr,
  ) -> Option<bool> {
    let Some(s) = get_nested_identifier_name_from_member(expr) else {
      return None;
    };
    dep(parser, &s, expr.span().real_lo(), expr.span().real_hi()).map(|dep| {
      // FIXME: temp
      parser.ignored.insert(DependencyLocation::new(
        expr.span.real_lo(),
        expr.span.real_hi(),
      ));
      parser.dependencies.push(Box::new(dep));
      true
    })
  }

  fn identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
  ) -> Option<bool> {
    dep(
      parser,
      ident.sym.as_str(),
      ident.span.real_lo(),
      ident.span.real_hi(),
    )
    .map(|dep| {
      parser.dependencies.push(Box::new(dep));
      true
    })
  }
}
