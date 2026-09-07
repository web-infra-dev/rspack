use std::sync::Arc;

use rspack_util::SpanExt;
mod if_stmt;
mod logic_expr;

use rspack_core::{CachedConstDependency, ConstDependency};
use swc_next_ecma_ast::{ConditionalExpression, GetSpan, IfStatement, LogicalExpression};

use super::JavascriptParserPlugin;
use crate::{
  utils::eval::evaluate_to_string,
  visitors::{Identifier, JavascriptParser, Statement},
};

pub struct ConstPlugin;

const RESOURCE_FRAGMENT: &str = "__resourceFragment";
const RESOURCE_QUERY: &str = "__resourceQuery";

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for ConstPlugin {
  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: LogicalExpression,
  ) -> Option<bool> {
    self::logic_expr::expression_logic_operator(parser, expr)
  }

  fn expression_conditional_operation(
    &self,
    parser: &mut JavascriptParser<'p>,
    expression: ConditionalExpression,
  ) -> Option<bool> {
    let ast = parser.ast.ast;
    let param = parser.evaluate_expression(expression.test(ast));
    if let Some(bool) = param.as_bool() {
      if !param.could_have_side_effects() {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          param.range().into(),
          format!(" {bool}").into(),
        )));
      } else {
        parser.walk_expression(expression.test(ast));
      }
      if bool {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          expression.alternate(ast).span(ast).into(),
          "0".into(),
        )));
      } else {
        parser.add_presentational_dependency(Arc::new(ConstDependency::new(
          expression.consequent(ast).span(ast).into(),
          "0".into(),
        )));
      }
      Some(bool)
    } else {
      None
    }
  }

  fn statement_if(&self, parser: &mut JavascriptParser<'p>, expr: IfStatement) -> Option<bool> {
    self::if_stmt::statement_if(parser, expr)
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    ident: &Identifier,
    for_name: &str,
  ) -> Option<bool> {
    match for_name {
      RESOURCE_FRAGMENT => {
        let resource_fragment = parser.resource_data.fragment().unwrap_or("");
        parser.add_presentational_dependency(Arc::new(CachedConstDependency::new(
          ident.span().into(),
          "__resourceFragment".into(),
          rspack_util::json_stringify_str(resource_fragment).into(),
        )));
        Some(true)
      }
      RESOURCE_QUERY => {
        let resource_query = parser.resource_data.query().unwrap_or("");
        parser.add_presentational_dependency(Arc::new(CachedConstDependency::new(
          ident.span().into(),
          "__resourceQuery".into(),
          rspack_util::json_stringify_str(resource_query).into(),
        )));
        Some(true)
      }
      _ => None,
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser<'p>,
    for_name: &str,
    _member_expr_info: Option<&crate::visitors::ExpressionExpressionInfo>,
    start: u32,
    end: u32,
  ) -> Option<crate::utils::eval::BasicEvaluatedExpression<'p>> {
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

  fn unused_statement(&self, parser: &mut JavascriptParser<'p>, stmt: Statement) -> Option<bool> {
    // Skip top level scope to align with webpack's ConstPlugin behavior.
    if parser.is_top_level_scope() {
      return None;
    }

    // Compute hoisted declarations from the dead statement without cloning the AST.
    let include_function_declarations = !parser.is_strict();
    let declarations =
      self::if_stmt::get_hoisted_declarations(parser.ast.ast, stmt, include_function_declarations);

    let replacement_body = if declarations.is_empty() {
      "{}".to_string()
    } else {
      let mut names: Vec<&str> = declarations.iter().copied().collect();
      names.sort_unstable();
      format!("{{ var {} }}", names.join(", "))
    };

    // Prepend the same comment as webpack for easier debugging.
    let mut replacement = String::from("// removed by dead control flow\n");
    replacement.push_str(&replacement_body);

    let span = stmt.span(parser.ast.ast);
    parser.add_presentational_dependency(Arc::new(ConstDependency::new(
      (span.real_lo(), span.real_hi()).into(),
      replacement.into_boxed_str(),
    )));

    Some(true)
  }
}
