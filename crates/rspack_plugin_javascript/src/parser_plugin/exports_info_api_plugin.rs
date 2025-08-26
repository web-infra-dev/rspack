use rspack_core::{ConstDependency, SpanExt};
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{Ident, MemberExpr},
};

use super::JavascriptParserPlugin;
use crate::{dependency::ExportInfoDependency, visitors::JavascriptParser};

const WEBPACK_EXPORTS_INFO: &str = "__webpack_exports_info__";

pub struct ExportsInfoApiPlugin;

impl JavascriptParserPlugin for ExportsInfoApiPlugin {
  fn member_chain(
    &self,
    parser: &mut JavascriptParser,
    member_expr: &MemberExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    let len = members.len();
    if len >= 1 && for_name == WEBPACK_EXPORTS_INFO {
      let prop = members[len - 1].clone();
      let dep = Box::new(ExportInfoDependency::new(
        member_expr.span.real_lo(),
        member_expr.span.real_hi(),
        members.iter().take(len - 1).cloned().collect::<Vec<_>>(),
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
    expr: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == WEBPACK_EXPORTS_INFO {
      let dep = Box::new(ConstDependency::new(expr.span.into(), "true".into(), None));
      parser.presentational_dependencies.push(dep);
      Some(true)
    } else {
      None
    }
  }
}
