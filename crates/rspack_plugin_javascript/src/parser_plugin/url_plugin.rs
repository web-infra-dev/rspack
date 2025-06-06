use rspack_core::{ConstDependency, Dependency, RuntimeGlobals, SpanExt};
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
      expr: box Expr::Member(arg2),
    }) = args.get(1)
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
  pub relative: bool,
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
    let magic_comment_options = try_extract_webpack_magic_comment(
      parser.source_file,
      &parser.comments,
      expr.span,
      arg.span(),
      &mut parser.warning_diagnostics,
    );
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
        expr: box Expr::Member(arg2),
      } = arg2
        && !is_meta_url(parser, arg2)
      {
        return None;
      }
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
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
        self.relative,
      );
      let dep_id = *dep.id();
      parser.dependencies.push(Box::new(dep));
      InnerGraphPlugin::on_usage(
        parser,
        Box::new(move |parser, used_by_exports| {
          if let Some(dep) = parser
            .dependencies
            .iter_mut()
            .find(|dep| dep.id() == &dep_id)
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
