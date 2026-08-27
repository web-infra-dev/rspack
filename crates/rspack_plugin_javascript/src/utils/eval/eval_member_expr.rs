use rspack_util::SpanExt;
use swc_next_ecma_ast::{Expr, GetSpan, MemberExpression};

use super::BasicEvaluatedExpression;
use crate::{
  parser_plugin::{CREATED_REQUIRE_IDENTIFIER_TAG, CreatedRequireTagData, JavascriptParserPlugin},
  visitors::{
    AllowedMemberTypes, ExportedVariableInfo, ExprRef, ExpressionExpressionInfo, JavascriptParser,
    MemberExpressionInfo,
  },
};

pub fn eval_member_expression<'parser>(
  parser: &mut JavascriptParser<'parser>,
  member: MemberExpression,
  expression: Expr,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let result = if let Some(MemberExpressionInfo::Expression(info)) =
    parser.get_member_expression_info(ExprRef::Member(member), AllowedMemberTypes::Expression)
  {
    eval_member_expression_with_info(parser, member, expression, info)
  } else {
    None
  };
  parser.member_expr_in_optional_chain = false;
  result
}

pub fn eval_member_expression_with_info<'parser>(
  parser: &mut JavascriptParser<'parser>,
  member: MemberExpression,
  expression: Expr,
  info: ExpressionExpressionInfo,
) -> Option<BasicEvaluatedExpression<'parser>> {
  let ast = parser.ast.ast;
  let span = member.span(ast);
  let drive = parser.plugin_drive.clone();
  let is_created_require_member = parser.javascript_options.is_create_require_enabled()
    && matches!(
      info.root_info,
      ExportedVariableInfo::VariableInfo(id)
        if parser
          .get_variable_tag_data::<CreatedRequireTagData>(id, CREATED_REQUIRE_IDENTIFIER_TAG)
          .is_some()
    );
  let result = drive
    .evaluate_identifier(
      parser,
      &info.name,
      Some(&info),
      span.real_lo(),
      span.real_hi(),
    )
    .filter(|_| !is_created_require_member)
    .or_else(|| drive.evaluate(parser, expression))
    .or_else(|| {
      let mut evaluated = BasicEvaluatedExpression::with_range(span.real_lo(), span.real_hi());
      evaluated.set_identifier(
        info.name.into(),
        info.root_info,
        Some((info.members, info.members_optionals, info.member_ranges)),
      );
      Some(evaluated)
    });
  parser.member_expr_in_optional_chain = false;
  result
}
