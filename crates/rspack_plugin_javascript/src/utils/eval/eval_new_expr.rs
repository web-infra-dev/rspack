use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_experimental_ecma_ast::NewExpr;

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{evaluate_create_require_new_expression, is_create_require_specifier},
  utils::eval,
  visitors::{CallHooksName, JavascriptParser},
};

#[inline]
pub fn eval_new_expression<'a>(
  scanner: &mut JavascriptParser,
  expr: &'a NewExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let ident = expr.callee.as_ident();
  if scanner.javascript_options.is_create_require_enabled() {
    if let Some(ident) = ident {
      let ident_name = Atom::from(ident.sym.as_str());
      if is_create_require_specifier(scanner, &ident_name) {
        let evaluated = ident_name.call_hooks_name(scanner, |scanner, for_name| {
          evaluate_create_require_new_expression(scanner, for_name, Some(&expr.callee), expr)
        });
        if evaluated.is_some() {
          return evaluated;
        }
      }
    } else if expr.callee.as_member().is_some()
      && let Some(evaluated) =
        evaluate_create_require_new_expression(scanner, "", Some(&expr.callee), expr)
    {
      return Some(evaluated);
    }
  }
  let ident = ident?;
  if ident.sym.as_str() != "RegExp" {
    // FIXME: call hooks
    return None;
  }
  if scanner.get_variable_info(&Atom::from("RegExp")).is_some() {
    return None;
  }
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
