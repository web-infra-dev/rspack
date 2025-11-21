use rspack_util::SpanExt;
use swc_core::ecma::ast::NewExpr;

use super::BasicEvaluatedExpression;
use crate::{utils::eval, visitors::JavascriptParser};

#[inline]
pub fn eval_new_expression<'a>(
  scanner: &mut JavascriptParser,
  expr: &'a NewExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let ident = expr.callee.as_ident()?;
  if ident.sym.as_str() != "RegExp" {
    // FIXME: call hooks
    return None;
  }
  // FIXME: should detect RegExpr variable info
  let Some(args) = &expr.args else {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_regexp(String::new(), String::new());
    return Some(res);
  };

  let Some(arg1) = args.first() else {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
    res.set_regexp(String::new(), String::new());
    return Some(res);
  };

  if arg1.spread.is_some() {
    return None;
  }

  let evaluated_reg_exp = scanner.evaluate_expression(&arg1.expr);
  let reg_exp = evaluated_reg_exp.as_string()?;

  let flags = if let Some(arg2) = args.get(1) {
    if arg2.spread.is_some() {
      return None;
    }
    let evaluated_flags = scanner.evaluate_expression(&arg2.expr);

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

  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.real_hi());
  res.set_regexp(reg_exp, flags);
  Some(res)
}
