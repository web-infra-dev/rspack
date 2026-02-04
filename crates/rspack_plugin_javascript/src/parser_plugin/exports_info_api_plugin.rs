use rspack_core::ConstDependency;
use rspack_util::SpanExt;
use swc_core::{
  atoms::Atom,
  common::Span,
  ecma::ast::{Ident, MemberExpr},
};

use super::JavascriptParserPlugin;
use crate::{dependency::ExportInfoDependency, visitors::JavascriptParser};

const EXPORTS_INFO: &str = "__webpack_exports_info__";

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
    if len >= 1 && for_name == EXPORTS_INFO {
      let prop = members[len - 1].clone();
      let dep = Box::new(ExportInfoDependency::new(
        member_expr.span.real_lo(),
        member_expr.span.real_hi(),
        members.iter().take(len - 1).cloned().collect::<Vec<_>>(),
        prop,
      ));
      parser.add_presentational_dependency(dep);
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
    if for_name == EXPORTS_INFO {
      let dep = Box::new(ConstDependency::new(expr.span.into(), "true".into()));
      parser.add_presentational_dependency(dep);
      Some(true)
    } else {
      None
    }
  }
}
