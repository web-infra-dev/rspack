use rspack_util::SpanExt;
use swc_core::ecma::ast::{Expr, MemberExpr};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{CREATED_REQUIRE_IDENTIFIER_TAG, CreatedRequireTagData, JavascriptParserPlugin},
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, ExprRef, JavascriptParser, MemberExpressionInfo,
  },
};

pub fn eval_member_expression<'a>(
  parser: &mut JavascriptParser,
  member: &'a MemberExpr,
  expr: &'a Expr,
) -> Option<BasicEvaluatedExpression<'a>> {
  let drive = parser.plugin_drive.clone();
  let ret = if let Some(MemberExpressionInfo::Expression(info)) =
    parser.get_member_expression_info(ExprRef::Member(member), AllowedMemberTypes::Expression)
  {
    let is_created_require_member = matches!(
      info.root_info,
      ExportedVariableInfo::VariableInfo(id)
        if parser
          .get_variable_tag_data::<CreatedRequireTagData>(id, CREATED_REQUIRE_IDENTIFIER_TAG)
          .is_some()
    );
    drive
      .evaluate_identifier(
        parser,
        &info.name,
        member.span.real_lo(),
        member.span.real_hi(),
      )
      .filter(|_| !is_created_require_member)
      .or_else(|| drive.evaluate(parser, expr))
      .or_else(|| {
        // TODO: fallback with `evaluateDefinedIdentifier`
        let mut eval =
          BasicEvaluatedExpression::with_range(member.span.real_lo(), member.span.real_hi());
        eval.set_identifier(
          info.name.into(),
          info.root_info,
          Some(info.members.into_vec()),
          Some(info.members_optionals.into_vec()),
          Some(info.member_ranges.into_vec()),
        );
        Some(eval)
      })
  } else {
    None
  };
  parser.member_expr_in_optional_chain = false;
  ret
}
