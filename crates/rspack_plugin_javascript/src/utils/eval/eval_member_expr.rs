use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{Expr, GetSpan, MemberExpr};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::JavascriptParserPlugin,
  visitors::{AllowedMemberTypes, JavascriptParser, MemberExpressionInfo},
};

pub fn eval_member_expression(
  parser: &mut JavascriptParser,
  member: MemberExpr,
  expr: Expr,
) -> Option<BasicEvaluatedExpression> {
  let ret = if let Some(MemberExpressionInfo::Expression(info)) =
    parser.get_member_expression_info(Expr::Member(member), AllowedMemberTypes::Expression)
  {
    parser
      .plugin_drive
      .clone()
      .evaluate_identifier(
        parser,
        &info.name,
        member.span(&parser.ast).real_lo(),
        member.span(&parser.ast).real_hi(),
      )
      .or_else(|| parser.plugin_drive.clone().evaluate(parser, expr))
      .or_else(|| {
        // TODO: fallback with `evaluateDefinedIdentifier`
        let mut eval = BasicEvaluatedExpression::with_range(
          member.span(&parser.ast).real_lo(),
          member.span(&parser.ast).real_hi(),
        );
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
