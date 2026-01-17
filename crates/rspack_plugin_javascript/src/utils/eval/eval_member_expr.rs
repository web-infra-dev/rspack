use rspack_util::SpanExt;
use swc_core::ecma::ast::{Expr, MemberExpr};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::JavascriptParserPlugin,
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, ExprRef, JavascriptParser, MemberExpressionInfo,
  },
};

pub fn eval_member_expression<'a>(
  parser: &mut JavascriptParser,
  member: &'a MemberExpr,
  expr: &'a Expr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let ret = if let Some(MemberExpressionInfo::Expression(info)) =
    parser.get_member_expression_info(ExprRef::Member(member), AllowedMemberTypes::Expression)
  {
    if let ExportedVariableInfo::VariableInfo(_) = info.root_info {
      None
    } else {
      parser.plugin_drive.clone().evaluate_identifier(
        parser,
        &info.name,
        member.span.real_lo(),
        member.span.real_hi(),
      )
    }
    .or_else(|| parser.plugin_drive.clone().evaluate(parser, expr))
    .or_else(|| {
      // TODO: fallback with `evaluateDefinedIdentifier`
      let mut eval =
        BasicEvaluatedExpression::with_range(member.span.real_lo(), member.span.real_hi());
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
