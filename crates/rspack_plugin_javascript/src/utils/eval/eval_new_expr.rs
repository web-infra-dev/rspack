use std::borrow::Cow;

use rspack_core::SpanExt;
use swc_core::ecma::ast::NewExpr;

use super::BasicEvaluatedExpression;
use crate::parser_plugin::JavaScriptParserPluginDrive;
use crate::utils::eval;
use crate::visitors::JavascriptParser;

pub fn eval_new_expression<'ast, 'parser>(
  scanner: &mut JavascriptParser<'parser>,
  expr: &'ast NewExpr,
  plugin_drive: &JavaScriptParserPluginDrive<'ast, 'parser>,
) -> Option<BasicEvaluatedExpression<'ast>> {
  let Some(ident) = expr.callee.as_ident() else {
    return None;
  };
  if ident.sym.as_str() != "RegExp" {
    // FIXME: call hooks
    return None;
  }
  // FIXME: should detect RegExpr variable info
  let Some(args) = &expr.args else {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
    res.set_regexp(Default::default(), Default::default());
    return Some(res);
  };

  let Some(arg1) = args.first() else {
    let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
    res.set_regexp(Default::default(), Default::default());
    return Some(res);
  };

  if arg1.spread.is_some() {
    return None;
  }

  let evaluated_reg_exp = scanner.evaluate_expression(&arg1.expr, plugin_drive);
  let Some(reg_exp) = evaluated_reg_exp.as_string() else {
    return None;
  };

  let flags = if let Some(arg2) = args.get(1) {
    if arg2.spread.is_some() {
      return None;
    }
    let evaluated_flags = scanner.evaluate_expression(&arg2.expr, plugin_drive);

    if let Some(flags) = evaluated_flags.as_string()
      && eval::is_valid_reg_exp_flags(flags)
    {
      flags.to_string()
    } else {
      return None;
    }
  } else {
    Default::default()
  };

  let mut res = BasicEvaluatedExpression::with_range(expr.span.real_lo(), expr.span.hi().0);
  res.set_regexp(Cow::Owned(reg_exp.to_string()), Cow::Owned(flags));
  Some(res)
}
