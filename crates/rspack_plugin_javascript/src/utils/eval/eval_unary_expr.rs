use rspack_util::SpanExt;
use swc_atoms::Atom;
use swc_next_ecma_ast::{ExprData, GetSpan, UnaryExpression, UnaryOperator};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::JavascriptParserPlugin,
  visitors::{CallHooksName, JavascriptParser, RootName},
};

fn eval_typeof<'parser>(
  parser: &mut JavascriptParser<'parser>,
  expression: UnaryExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  debug_assert_eq!(expression.operator(ast), UnaryOperator::Typeof);
  let argument = expression.argument(ast);
  let hook_result = match ast.expr_data(argument) {
    ExprData::IdentifierReference(identifier) => Atom::from(ast.get_utf8(identifier.name(ast)))
      .call_hooks_name(parser, |parser, name| {
        parser
          .plugin_drive
          .clone()
          .evaluate_typeof(parser, expression, name)
      }),
    ExprData::MetaProperty(meta) => meta.get_root_name(ast).and_then(|name| {
      name.call_hooks_name(parser, |parser, name| {
        parser
          .plugin_drive
          .clone()
          .evaluate_typeof(parser, expression, name)
      })
    }),
    ExprData::MemberExpression(member) => member.call_hooks_name(parser, |parser, name| {
      parser
        .plugin_drive
        .clone()
        .evaluate_typeof(parser, expression, name)
    }),
    ExprData::ChainExpression(chain) => chain.call_hooks_name(parser, |parser, name| {
      parser
        .plugin_drive
        .clone()
        .evaluate_typeof(parser, expression, name)
    }),
    _ => None,
  };
  if hook_result.is_some() {
    return hook_result;
  }
  let span = expression.span(ast);
  if argument.is_function(ast) {
    let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
    result.set_string("function".to_string());
    return Some(result);
  }
  let argument_eval = parser.evaluate_expression(argument);
  let type_name = if argument_eval.is_unknown() {
    match ast.expr_data(argument) {
      ExprData::Function(_) | ExprData::Class(_) => Some("function"),
      ExprData::UnaryExpression(unary)
        if matches!(
          unary.operator(ast),
          UnaryOperator::Negate | UnaryOperator::Positive
        ) && unary.argument(ast).is_numeric_literal(ast) =>
      {
        Some("number")
      }
      _ => None,
    }
  } else if argument_eval.is_string() || argument_eval.is_wrapped() {
    Some("string")
  } else if argument_eval.is_undefined() {
    Some("undefined")
  } else if argument_eval.is_number() {
    Some("number")
  } else if argument_eval.is_null() || argument_eval.is_regexp() || argument_eval.is_array() {
    Some("object")
  } else if argument_eval.is_bool() {
    Some("boolean")
  } else if argument_eval.is_bigint() {
    Some("bigint")
  } else {
    None
  }?;
  let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
  result.set_string(type_name.to_string());
  if argument_eval.is_wrapped() {
    result.set_side_effects(argument_eval.could_have_side_effects());
  }
  Some(result)
}

#[inline]
pub fn eval_unary_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  expression: UnaryExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let span = expression.span(ast);
  let argument = expression.argument(ast);
  match expression.operator(ast) {
    UnaryOperator::Typeof => eval_typeof(parser, expression),
    UnaryOperator::LogicalNot => {
      let argument = parser.evaluate_expression(argument);
      if argument.is_dependency() {
        let side_effects = argument.could_have_side_effects();
        let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
        result.set_dependency(argument.into_dependency().not());
        result.set_side_effects(side_effects);
        return Some(result);
      }
      let value = argument.as_bool()?;
      let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
      result.set_bool(!value);
      result.set_side_effects(argument.could_have_side_effects());
      Some(result)
    }
    UnaryOperator::BitwiseNot => {
      let argument = parser.evaluate_expression(argument);
      let value = argument.as_int()?;
      let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
      result.set_number(!value as f64);
      result.set_side_effects(argument.could_have_side_effects());
      Some(result)
    }
    UnaryOperator::Negate | UnaryOperator::Positive => {
      let argument = parser.evaluate_expression(argument);
      let value = argument.as_number()?;
      let value = if expression.operator(ast) == UnaryOperator::Negate {
        -value
      } else {
        value
      };
      let mut result = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
      result.set_number(value);
      result.set_side_effects(argument.could_have_side_effects());
      Some(result)
    }
    UnaryOperator::Void | UnaryOperator::Delete => None,
  }
}
