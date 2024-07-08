use rspack_core::{Dependency, SpanExt};
use swc_core::{
  common::Spanned,
  ecma::ast::{Expr, ExprOrSpread, MetaPropKind, NewExpr},
};

use super::JavascriptParserPlugin;
use crate::{
  dependency::URLDependency, parser_plugin::inner_graph::plugin::InnerGraphPlugin,
  visitors::JavascriptParser,
};

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
  {
    let chain = parser.extract_member_expression_chain(arg2);
    if let Some(meta) = chain.object.as_meta_prop()
      && matches!(meta.kind, MetaPropKind::ImportMeta)
      && chain.members.len() == 1
      && matches!(chain.members.first(), Some(member) if member == "url")
    {
      return parser
        .evaluate_expression(arg1)
        .as_string()
        .map(|req| (req, arg1.span().real_lo(), arg2.span().real_hi()));
    }
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
    if for_name == "URL"
      && let Some((request, start, end)) = get_url_request(parser, expr)
    {
      let dep = URLDependency::new(
        start,
        end,
        expr.span.real_lo(),
        expr.span.real_hi(),
        request.into(),
        Some(expr.span.into()),
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
      Some(true)
    } else {
      None
    }
  }
}
