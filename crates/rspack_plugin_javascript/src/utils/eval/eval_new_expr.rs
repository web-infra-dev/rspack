use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_next_ecma_ast::{ArgumentData, GetSpan, NewExpression};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{evaluate_create_require_new_expression, is_create_require_specifier},
  utils::eval,
  visitors::{CallHooksName, JavascriptParser},
};

#[inline]
pub fn eval_new_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  expression: NewExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let callee = expression.callee(ast);
  let identifier = callee.as_identifier_reference(ast);
  if parser.javascript_options.is_create_require_enabled() {
    if let Some(identifier) = identifier {
      let name = Atom::from(ast.get_utf8(identifier.name(ast)));
      if is_create_require_specifier(parser, &name) {
        let evaluated = name.call_hooks_name(parser, |parser, for_name| {
          evaluate_create_require_new_expression(parser, for_name, Some(callee), expression)
        });
        if evaluated.is_some() {
          return evaluated;
        }
      }
    } else if callee.as_member_expression(ast).is_some()
      && let Some(evaluated) =
        evaluate_create_require_new_expression(parser, "", Some(callee), expression)
    {
      return Some(evaluated);
    }
  }
  let identifier = identifier?;
  if ast.get_utf8(identifier.name(ast)) != "RegExp"
    || parser.get_variable_info(&Atom::from("RegExp")).is_some()
  {
    return None;
  }
  let arguments = expression.arguments(ast);
  let span = expression.span(ast);
  let Some(first) = arguments.get_node(ast, 0) else {
    let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
    result.set_regexp(String::new(), String::new());
    return Some(result);
  };
  let ArgumentData::Expr(first) = ast.argument_data(first) else {
    return None;
  };
  let regexp = parser.evaluate_expression(first).as_string()?;
  let flags = if let Some(second) = arguments.get_node(ast, 1) {
    let ArgumentData::Expr(second) = ast.argument_data(second) else {
      return None;
    };
    let flags = parser.evaluate_expression(second).as_string()?;
    eval::is_valid_reg_exp_flags(&flags).then_some(flags)?
  } else {
    String::new()
  };
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  result.set_regexp(regexp, flags);
  Some(result)
}
