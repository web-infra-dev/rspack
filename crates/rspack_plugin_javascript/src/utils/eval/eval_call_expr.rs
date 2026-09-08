use swc_next_ecma_ast::{CallExpression, PropertyKeyData};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{
    CREATE_REQUIRE_EVALUATED_TAG, JavascriptParserPlugin, is_create_require_namespace_member,
    is_create_require_specifier,
  },
  visitors::{CallHooksName, JavascriptParser},
};

#[inline]
pub fn eval_call_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  expression: CallExpression,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let drive = parser.plugin_drive.clone();
  let callee = expression.callee(ast);
  if let Some(identifier) = callee.as_identifier_reference(ast) {
    let name = ast.get_utf8(identifier.name(ast));
    let is_create_require = parser.javascript_options.is_create_require_enabled()
      && is_create_require_specifier(parser, name);
    let evaluated = if is_create_require {
      name.call_hooks_name(parser, |parser, for_name| {
        drive.evaluate_call_expression(parser, for_name, expression)
      })
    } else if parser.javascript_options.is_create_require_enabled() {
      let evaluated = parser.evaluate_expression(callee);
      if evaluated.is_identifier() && evaluated.identifier() == CREATE_REQUIRE_EVALUATED_TAG {
        drive.evaluate_call_expression(parser, CREATE_REQUIRE_EVALUATED_TAG, expression)
      } else {
        drive.evaluate_call_expression(parser, name, expression)
      }
    } else {
      drive.evaluate_call_expression(parser, name, expression)
    };
    if evaluated.is_some() {
      return evaluated;
    }
  }
  if let Some(member) = callee.as_member_expression(ast)
    && let PropertyKeyData::IdentifierName(identifier) = ast.property_key_data(member.property(ast))
  {
    let parameter = parser.evaluate_expression(member.object(ast));
    return drive.evaluate_call_expression_member(
      parser,
      ast.get_utf8(identifier.name(ast)),
      expression,
      parameter,
    );
  }
  if parser.javascript_options.is_create_require_enabled()
    && is_create_require_namespace_member(parser, callee)
  {
    return drive.evaluate_call_expression(parser, "", expression);
  }
  None
}
