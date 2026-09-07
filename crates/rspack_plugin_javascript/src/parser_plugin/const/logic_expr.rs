use std::sync::Arc;

use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_next_ecma_ast::{GetSpan, LogicalExpression, LogicalOperator};

use crate::visitors::JavascriptParser;

pub fn expression_logic_operator(
  scanner: &mut JavascriptParser,
  expr: LogicalExpression,
) -> Option<bool> {
  let ast = scanner.ast.ast;
  let operator = expr.operator(ast);
  if matches!(operator, LogicalOperator::And | LogicalOperator::Or) {
    let param = scanner.evaluate_expression(expr.left(ast));
    let boolean = param.as_bool();
    let boolean = boolean?;
    let keep_right = if boolean {
      operator == LogicalOperator::And
    } else {
      operator == LogicalOperator::Or
    };
    if !param.could_have_side_effects() && (keep_right || param.is_bool()) {
      scanner.add_presentational_dependency(Arc::new(ConstDependency::new(
        param.range().into(),
        format!(" {boolean}").into(),
      )));
    } else {
      scanner.walk_expression(expr.left(ast));
    }

    if !keep_right {
      scanner.add_presentational_dependency(Arc::new(ConstDependency::new(
        {
          let span = expr.right(ast).span(ast);
          (span.real_lo(), span.real_hi()).into()
        },
        "0".into(),
      )));
    }
    Some(keep_right)
  } else if operator == LogicalOperator::NullishCoalescing {
    let param = scanner.evaluate_expression(expr.left(ast));
    if let Some(keep_right) = param.as_nullish() {
      if !param.could_have_side_effects() && keep_right {
        scanner.add_presentational_dependency(Arc::new(ConstDependency::new(
          param.range().into(),
          " null".into(),
        )));
      } else {
        scanner.add_presentational_dependency(Arc::new(ConstDependency::new(
          {
            let span = expr.right(ast).span(ast);
            (span.real_lo(), span.real_hi()).into()
          },
          "0".into(),
        )));
        scanner.walk_expression(expr.left(ast));
      }
      Some(keep_right)
    } else {
      None
    }
  } else {
    unreachable!()
  }
}
