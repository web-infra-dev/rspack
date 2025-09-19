use rspack_plugin_javascript::{
  JavascriptParserPlugin, utils::eval::BasicEvaluatedExpression, visitors::JavascriptParser,
};
use swc_core::{
  atoms::Atom,
  ecma::ast::{AssignExpr, CallExpr, Ident, MemberExpr, UnaryExpr},
};

#[derive(PartialEq, Debug, Default)]
pub struct RslibParserPlugin {
  pub intercept_api_plugin: bool,
}

impl RslibParserPlugin {
  pub fn new(intercept_api_plugin: bool) -> Self {
    Self {
      intercept_api_plugin,
    }
  }
}

impl JavascriptParserPlugin for RslibParserPlugin {
  fn assign_member_chain(
    &self,
    parser: &mut JavascriptParser,
    _assign_expr: &AssignExpr,
    _remaining: &[Atom],
    for_name: &str,
  ) -> Option<bool> {
    if parser.is_esm && for_name == "module" {
      return Some(true);
    }

    None
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    _ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    // Intercept CommonJsExportsParsePlugin, not APIPlugin, but put it here.
    // crates/rspack_plugin_javascript/src/parser_plugin/common_js_exports_parse_plugin.rs
    if for_name == "module" && parser.is_esm {
      return Some(true);
    }

    None
  }

  fn member(
    &self,
    parser: &mut JavascriptParser,
    _member_expr: &MemberExpr,
    for_name: &str,
  ) -> Option<bool> {
    if parser.is_esm && (for_name == "module" || for_name.starts_with("module.")) {
      return Some(true);
    }

    if for_name == "require.cache"
      || for_name == "require.extensions"
      || for_name == "require.config"
      || for_name == "require.version"
      || for_name == "require.include"
      || for_name == "require.onError"
    {
      return Some(true);
    }
    None
  }

  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    _expr: &MemberExpr,
    for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[swc_core::common::Span],
  ) -> Option<bool> {
    if parser.is_esm && for_name == "module" {
      return Some(true);
    }

    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    _expr: &CallExpr,
    for_name: &str,
    _members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[swc_core::common::Span],
  ) -> Option<bool> {
    if parser.is_esm && for_name == "module" {
      return Some(true);
    }

    None
  }

  fn evaluate_typeof<'a>(
    &self,
    parser: &mut JavascriptParser,
    _expr: &'a UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression<'a>> {
    if parser.is_esm && for_name == "module" {
      let span = _expr.span;
      let mut eval = BasicEvaluatedExpression::with_range(span.lo.0, span.hi.0);
      eval.set_side_effects(false);
      return Some(eval);
    }

    None
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    _expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    if parser.is_esm && for_name == "module" {
      return Some(true);
    }

    None
  }
}
