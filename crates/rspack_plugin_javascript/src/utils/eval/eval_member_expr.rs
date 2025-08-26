use rspack_core::SpanExt;
use swc_core::ecma::ast::MemberExpr;

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::JavascriptParserPlugin,
  visitors::{AllowedMemberTypes, JavascriptParser, MemberExpressionInfo},
};

pub fn eval_member_expression<'a>(
  parser: &mut JavascriptParser,
  member: &'a MemberExpr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let ret = if let Some(MemberExpressionInfo::Expression(info)) =
    parser.get_member_expression_info(member, AllowedMemberTypes::Expression)
  {
    parser
      .plugin_drive
      .clone()
      .evaluate_identifier(
        parser,
        &info.name,
        member.span.real_lo(),
        member.span.hi().0,
      )
      .or_else(|| {
        // TODO: fallback with `evaluateDefinedIdentifier`
        let mut eval =
          BasicEvaluatedExpression::with_range(member.span.real_lo(), member.span.hi().0);
        eval.set_identifier(
          info.name.into(),
          info.root_info,
          Some(info.members),
          Some(info.members_optionals),
          Some(info.member_ranges),
        );
        Some(eval)
      })
  } else {
    None
  };
  parser.member_expr_in_optional_chain = false;
  ret
}
