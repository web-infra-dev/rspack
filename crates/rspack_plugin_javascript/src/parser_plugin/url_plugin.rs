use rspack_core::{ConstDependency, JavascriptParserUrl, RuntimeGlobals};
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::ast::{Expr, ExprOrSpread, MemberExpr, MetaPropKind, NewExpr},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::URLDependency, parser_plugin::inner_graph::plugin::InnerGraphPlugin,
  visitors::JavascriptParser, webpack_comment::try_extract_webpack_magic_comment,
};

pub fn is_meta_url(parser: &mut JavascriptParser, expr: &MemberExpr) -> bool {
  let chain = parser.extract_member_expression_chain(expr);
  chain.object.as_meta_prop().is_some_and(|meta| {
    meta.kind == MetaPropKind::ImportMeta
      && chain.members.len() == 1
      && chain.members.first().is_some_and(|member| member == "url")
  })
}

pub fn get_url_request(
  parser: &mut JavascriptParser,
  expr: &NewExpr,
) -> Option<(String, u32, u32)> {
  if let Some(args) = &expr.args
    && let Some(ExprOrSpread {
      spread: None,
      expr: arg1,
    }) = args.first()
    && let Some(ExprOrSpread {
      spread: None,
      expr: arg2_expr,
    }) = args.get(1)
    && let Expr::Member(arg2) = &**arg2_expr
    && is_meta_url(parser, arg2)
  {
    return parser
      .evaluate_expression(arg1)
      .as_string()
      .map(|req| (req, arg1.span().real_lo(), arg2.span().real_hi()));
  }
  None
}

pub struct URLPlugin {
  pub mode: Option<JavascriptParserUrl>,
}

impl JavascriptParserPlugin for URLPlugin {
  fn can_rename(&self, _parser: &mut JavascriptParser, for_name: &str) -> Option<bool> {
    (for_name == "URL").then_some(true)
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != "URL" {
      return None;
    }

    let args = expr.args.as_ref()?;
    let arg = args.first()?;
    let magic_comment_options = try_extract_webpack_magic_comment(parser, expr.span, arg.span());
    if magic_comment_options
      .get_webpack_ignore()
      .unwrap_or_default()
    {
      if args.len() != 2 {
        return None;
      }
      let arg2 = args.get(1)?;
      if let ExprOrSpread {
        spread: None,
        expr: arg2_expr,
      } = arg2
        && let Expr::Member(arg2) = &**arg2_expr
        && !is_meta_url(parser, arg2)
      {
        return None;
      }
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        arg2.span().into(),
        RuntimeGlobals::BASE_URI.name().into(),
        Some(RuntimeGlobals::BASE_URI),
      )));
      return Some(true);
    }

    if let Some((request, start, end)) = get_url_request(parser, expr) {
      let dep = URLDependency::new(
        request.into(),
        expr.span.into(),
        (start, end).into(),
        self.mode,
      );
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(Box::new(dep));
      InnerGraphPlugin::on_usage(
        parser,
        Box::new(move |parser, used_by_exports| {
          if let Some(dep) = parser.get_dependency_mut(dep_idx)
            && let Some(dep) = dep.downcast_mut::<URLDependency>()
          {
            dep.set_used_by_exports(used_by_exports);
          }
        }),
      );
      return Some(true);
    }

    None
  }
}
