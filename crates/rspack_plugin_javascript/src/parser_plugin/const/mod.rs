mod if_stmt;
mod logic_expr;

use rspack_core::{ConstDependency, SpanExt};

pub use self::logic_expr::is_logic_op;
use super::{api_plugin::WEBPACK_RESOURCE_QUERY, JavascriptParserPlugin};
use crate::{utils::eval::evaluate_to_string, visitors::JavascriptParser};

pub struct ConstPlugin;

impl JavascriptParserPlugin for ConstPlugin {
  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::BinExpr,
  ) -> Option<bool> {
    self::logic_expr::expression_logic_operator(parser, expr)
  }

  fn statement_if(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::IfStmt,
  ) -> Option<bool> {
    self::if_stmt::statement_if(parser, expr)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &swc_core::ecma::ast::Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == WEBPACK_RESOURCE_FRAGMENT {
      let resource_fragment = parser
        .resource_data
        .resource_fragment
        .as_deref()
        .unwrap_or("");
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          ident.span.real_lo(),
          ident.span.real_hi(),
          serde_json::to_string(resource_fragment)
            .expect("should render module id")
            .into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression> {
    if ident == WEBPACK_RESOURCE_QUERY {
      Some(evaluate_to_string(
        parser
          .resource_data
          .resource_query
          .clone()
          .unwrap_or_default(),
        start,
        end,
      ))
    } else if ident == WEBPACK_RESOURCE_FRAGMENT {
      Some(evaluate_to_string(
        parser
          .resource_data
          .resource_fragment
          .clone()
          .unwrap_or_default(),
        start,
        end,
      ))
    } else {
      None
    }
  }
}

const WEBPACK_RESOURCE_FRAGMENT: &str = "__resourceFragment";
