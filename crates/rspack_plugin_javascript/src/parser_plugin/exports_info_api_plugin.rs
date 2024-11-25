use rspack_core::{ConstDependency, SpanExt};

use super::JavascriptParserPlugin;
use crate::{
  dependency::ExportInfoDependency,
  visitors::{AllowedMemberTypes, ExportedVariableInfo, JavascriptParser, MemberExpressionInfo},
};

const WEBPACK_EXPORTS_INFO: &str = "__webpack_exports_info__";

pub struct ExportsInfoApiPlugin;

impl JavascriptParserPlugin for ExportsInfoApiPlugin {
  fn member(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    _name: &str,
  ) -> Option<bool> {
    let (members, root) = parser
      .get_member_expression_info(member_expr, AllowedMemberTypes::Expression)
      .and_then(|info| match info {
        MemberExpressionInfo::Call(_) => None,
        MemberExpressionInfo::Expression(info) => {
          if let ExportedVariableInfo::Name(root) = info.root_info {
            Some((info.members, root))
          } else {
            None
          }
        }
      })?;

    let len = members.len();
    if len >= 1 && root == WEBPACK_EXPORTS_INFO && parser.is_unresolved_ident(WEBPACK_EXPORTS_INFO)
    {
      let prop = members[len - 1].clone();
      let dep = Box::new(ExportInfoDependency::new(
        member_expr.span.real_lo(),
        member_expr.span.real_hi(),
        members.into_iter().take(len - 1).collect::<Vec<_>>(),
        prop,
      ));
      parser.presentational_dependencies.push(dep);
      Some(true)
    } else {
      None
    }
  }

  fn identifier(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    expr: &swc_core::ecma::ast::Ident,
    _for_name: &str,
  ) -> Option<bool> {
    if expr.sym == WEBPACK_EXPORTS_INFO {
      let dep = Box::new(ConstDependency::new(
        expr.span.real_lo(),
        expr.span.real_hi(),
        "true".into(),
        None,
      ));
      parser.presentational_dependencies.push(dep);
      Some(true)
    } else {
      None
    }
  }
}
