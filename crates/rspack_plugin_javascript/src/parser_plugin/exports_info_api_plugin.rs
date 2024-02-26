use rspack_core::{extract_member_expression_chain, ConstDependency, SpanExt};
use swc_core::atoms::Atom;

use super::JavascriptParserPlugin;
use crate::dependency::ExportInfoApiDependency;

const WEBPACK_EXPORTS_INFO: &str = "__webpack_exports_info__";

pub struct ExportsInfoApiPlugin;

impl JavascriptParserPlugin for ExportsInfoApiPlugin {
  fn member(
    &self,
    parser: &mut crate::visitors::JavascriptParser,
    member_expr: &swc_core::ecma::ast::MemberExpr,
    _name: &str,
  ) -> Option<bool> {
    let expression_info = extract_member_expression_chain(member_expr);
    let member_chain = expression_info.members();
    if !member_chain.is_empty()
      && member_chain[0].0 == WEBPACK_EXPORTS_INFO
      && parser.is_unresolved_ident(&Atom::new(WEBPACK_EXPORTS_INFO))
    {
      let len = member_chain.len();
      if len >= 3 {
        let prop = member_chain[len - 1].0.clone();
        let dep = Box::new(ExportInfoApiDependency::new(
          member_expr.span.real_lo(),
          member_expr.span.real_hi(),
          member_chain
            .into_iter()
            .skip(1)
            .take(len - 2)
            .map(|item| item.0.clone())
            .collect::<Vec<_>>(),
          prop,
        ));
        parser.presentational_dependencies.push(dep);
        Some(true)
      } else {
        // TODO: support other __webpack_exports_info__
        None
      }
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
