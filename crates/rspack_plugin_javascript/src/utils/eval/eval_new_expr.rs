use rspack_util::SpanExt;
use swc_core::atoms::Atom;
use swc_experimental_ecma_ast::{GetSpan, NewExpr};

use super::BasicEvaluatedExpression;
use crate::{utils::eval, visitors::JavascriptParser};

#[inline]
pub fn eval_new_expression(
  scanner: &mut JavascriptParser,
  expr: NewExpr,
) -> Option<BasicEvaluatedExpression> {
  let ident = expr.callee(&scanner.ast).as_ident()?;
  if scanner.ast.get_utf8(ident.sym(&scanner.ast)) != "RegExp" {
    // FIXME: call hooks
    return None;
  }
  if scanner.get_variable_info(&Atom::from("RegExp")).is_some() {
    return None;
  }
  let Some(args) = expr.args(&scanner.ast) else {
    let mut res = BasicEvaluatedExpression::with_range(
      expr.span(&scanner.ast).real_lo(),
      expr.span(&scanner.ast).real_hi(),
    );
    res.set_regexp(String::new(), String::new());
    return Some(res);
  };

  let Some(arg1) = args.first() else {
    let mut res = BasicEvaluatedExpression::with_range(
      expr.span(&scanner.ast).real_lo(),
      expr.span(&scanner.ast).real_hi(),
    );
    res.set_regexp(String::new(), String::new());
    return Some(res);
  };

  let arg1 = scanner.ast.get_node_in_sub_range(arg1);
  if arg1.spread(&scanner.ast).is_some() {
    return None;
  }

  let evaluated_reg_exp = scanner.evaluate_expression(arg1.expr(&scanner.ast));
  let reg_exp = evaluated_reg_exp.as_string()?;

  let flags = if let Some(arg2) = args.get(1) {
    let arg2 = scanner.ast.get_node_in_sub_range(arg2);
    if arg2.spread(&scanner.ast).is_some() {
      return None;
    }
    let evaluated_flags = scanner.evaluate_expression(arg2.expr(&scanner.ast));

    if let Some(flags) = evaluated_flags.as_string()
      && eval::is_valid_reg_exp_flags(&flags)
    {
      flags
    } else {
      return None;
    }
  } else {
    String::new()
  };

  let mut res = BasicEvaluatedExpression::with_range(
    expr.span(&scanner.ast).real_lo(),
    expr.span(&scanner.ast).real_hi(),
  );
  res.set_regexp(reg_exp, flags);
  Some(res)
}
