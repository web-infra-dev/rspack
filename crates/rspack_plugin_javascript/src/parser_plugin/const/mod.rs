mod if_stmt;
mod logic_expr;

use rspack_core::{CachedConstDependency, ConstDependency};
use swc_experimental_ecma_ast::{BinExpr, CondExpr, GetSpan, Ident, IfStmt};

pub use self::logic_expr::is_logic_op;
use super::JavascriptParserPlugin;
use crate::{
  utils::eval::{BasicEvaluatedExpression, evaluate_to_string},
  visitors::JavascriptParser,
};

pub struct ConstPlugin;

const RESOURCE_FRAGMENT: &str = "__resourceFragment";
const RESOURCE_QUERY: &str = "__resourceQuery";

impl JavascriptParserPlugin for ConstPlugin {
  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: BinExpr,
  ) -> Option<bool> {
    self::logic_expr::expression_logic_operator(parser, expr)
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser,
    expression: CondExpr,
  ) -> Option<bool> {
    let param = parser.evaluate_expression(expression.test(&parser.ast));
    if let Some(bool) = param.as_bool() {
      if !param.could_have_side_effects() {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          param.range().into(),
          format!(" {bool}").into(),
        )));
      } else {
        parser.walk_expression(expression.test(&parser.ast));
      }
      if bool {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          expression.alt(&parser.ast).span(&parser.ast).into(),
          "0".into(),
        )));
      } else {
        parser.add_presentational_dependency(Box::new(ConstDependency::new(
          expression.cons(&parser.ast).span(&parser.ast).into(),
          "0".into(),
        )));
      }
      Some(bool)
    } else {
      None
    }
  }

  fn statement_if(&self, parser: &mut JavascriptParser, expr: IfStmt) -> Option<bool> {
    self::if_stmt::statement_if(parser, expr)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: Ident,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      RESOURCE_FRAGMENT => {
        let resource_fragment = parser.resource_data.fragment().unwrap_or("");
        parser.add_presentational_dependency(Box::new(CachedConstDependency::new(
          ident.span(&parser.ast).into(),
          "__resourceFragment".into(),
          serde_json::to_string(resource_fragment)
            .expect("should render module id")
            .into(),
        )));
        Some(true)
      }
      RESOURCE_QUERY => {
        let resource_query = parser.resource_data.query().unwrap_or("");
        parser.add_presentational_dependency(Box::new(CachedConstDependency::new(
          ident.span(&parser.ast).into(),
          "__resourceQuery".into(),
          serde_json::to_string(resource_query)
            .expect("should render module id")
            .into(),
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    for_name: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    match for_name {
      RESOURCE_QUERY => Some(evaluate_to_string(
        parser
          .resource_data
          .query()
          .map(ToOwned::to_owned)
          .unwrap_or_default(),
        start,
        end,
      )),
      RESOURCE_FRAGMENT => Some(evaluate_to_string(
        parser
          .resource_data
          .fragment()
          .map(ToOwned::to_owned)
          .unwrap_or_default(),
        start,
        end,
      )),
      _ => None,
    }
  }
}
