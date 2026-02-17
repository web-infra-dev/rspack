use rspack_core::{
  ContextDependency, ContextMode, ContextNameSpaceObject, ContextOptions, DependencyCategory,
  JavascriptParserUrl, RuntimeGlobals, RuntimeRequirementsDependency,
};
use rspack_util::SpanExt;
use swc_experimental_ecma_ast::{
  Ast, Expr, GetSpan, MemberExpr, MetaPropKind, NewExpr, Visit, VisitWith,
};
use url::Url;

use super::JavascriptParserPlugin;
use crate::{
  InnerGraphPlugin,
  dependency::{URLContextDependency, URLDependency},
  magic_comment::try_extract_magic_comment,
  parser_plugin::inner_graph::state::InnerGraphUsageOperation,
  visitors::{JavascriptParser, context_reg_exp, create_context_dependency},
};

struct NestedNewUrlVisitor<'a> {
  ast: &'a Ast,
  has_nested_new_url: bool,
}

impl Visit for NestedNewUrlVisitor<'_> {
  fn ast(&self) -> &Ast {
    self.ast
  }

  fn visit_new_expr(&mut self, expr: NewExpr) {
    if expr
      .callee(self.ast)
      .as_ident()
      .is_some_and(|ident| self.ast.get_utf8(ident.sym(self.ast)).eq("URL"))
    {
      self.has_nested_new_url = true;
    }
  }
}

pub fn is_meta_url(parser: &mut JavascriptParser, expr: MemberExpr) -> bool {
  let chain = parser.extract_member_expression_chain(Expr::Member(expr));
  if let Expr::MetaProp(meta) = chain.object {
    return meta.kind(&parser.ast) == MetaPropKind::ImportMeta
      && chain.members.len() == 1
      && chain.members.first().is_some_and(|member| member == "url");
  }
  false
}

pub fn get_url_request(parser: &mut JavascriptParser, expr: NewExpr) -> Option<(String, u32, u32)> {
  let args = expr.args(&parser.ast)?;
  let expr_or_spread = parser.ast.get_node_in_sub_range(args.first()?);
  if expr_or_spread.spread(&parser.ast).is_some() {
    return None;
  }

  let arg1 = expr_or_spread.expr(&parser.ast);
  if let Some(arg2) = args.get(1) {
    // new URL(xx, import.meta.url)
    let arg2 = parser.ast.get_node_in_sub_range(arg2);
    if arg2.spread(&parser.ast).is_some() {
      return None;
    };

    let arg2 = arg2.expr(&parser.ast);
    let Expr::Member(arg2) = arg2 else {
      return None;
    };
    if is_meta_url(parser, arg2) {
      return parser.evaluate_expression(arg1).as_string().map(|req| {
        (
          req,
          arg1.span(&parser.ast).real_lo(),
          arg2.span(&parser.ast).real_hi(),
        )
      });
    }
  } else {
    // new URL(import.meta.url)
    let Expr::Member(arg1) = arg1 else {
      return None;
    };
    if is_meta_url(parser, arg1) {
      return Some((
        Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_string(),
        arg1.span(&parser.ast).real_lo(),
        arg1.span(&parser.ast).real_hi(),
      ));
    }
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
    expr: NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != "URL" {
      return None;
    }

    let args = expr.args(&parser.ast)?;

    let arg = parser.ast.get_node_in_sub_range(args.first()?);
    let magic_comment_options =
      try_extract_magic_comment(parser, expr.span(&parser.ast), arg.span(&parser.ast));
    if magic_comment_options.get_ignore().unwrap_or_default() {
      if args.len() != 2 {
        return None;
      }

      let arg2 = parser.ast.get_node_in_sub_range(args.get(1)?);
      if arg2.spread(&parser.ast).is_some() {
        return None;
      }

      let arg2 = arg2.expr(&parser.ast);
      let Expr::Member(arg2) = arg2 else {
        return None;
      };
      if !is_meta_url(parser, arg2) {
        return None;
      }
      parser.add_presentational_dependency(Box::new(RuntimeRequirementsDependency::new(
        arg2.span(&parser.ast).into(),
        RuntimeGlobals::BASE_URI,
      )));
      return Some(true);
    }

    // should not parse new URL(import.meta.url)
    if expr.args(&parser.ast).is_some_and(|args| {
      args.len() == 1
        && parser
          .ast
          .get_node_in_sub_range(args.get(0).unwrap())
          .expr(&parser.ast)
          .as_member()
          .is_some_and(|member| is_meta_url(parser, member))
    }) {
      return None;
    }

    if let Some((request, start, end)) = get_url_request(parser, expr) {
      let dep = URLDependency::new(
        request.into(),
        expr.span(&parser.ast).into(),
        (start, end).into(),
        self.mode,
      );
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(Box::new(dep));
      InnerGraphPlugin::on_usage(parser, InnerGraphUsageOperation::URLDependency(dep_idx));
      return Some(true);
    }

    let mut nested_new_url_visitor = NestedNewUrlVisitor {
      ast: &parser.ast,
      has_nested_new_url: false,
    };
    arg
      .expr(&parser.ast)
      .visit_with(&mut nested_new_url_visitor);
    if nested_new_url_visitor.has_nested_new_url {
      return None;
    }

    let arg2 = parser.ast.get_node_in_sub_range(args.get(1)?);
    if !arg2
      .expr(&parser.ast)
      .as_member()
      .is_some_and(|member| is_meta_url(parser, member))
    {
      return None;
    }

    let param = parser.evaluate_expression(arg.expr(&parser.ast));
    let result = create_context_dependency(&param, parser);
    let options = ContextOptions {
      mode: ContextMode::Sync,
      recursive: true,
      reg_exp: context_reg_exp(&result.reg, "", None, parser),
      include: magic_comment_options.get_include(),
      exclude: magic_comment_options.get_exclude(),
      category: DependencyCategory::Url,
      request: format!("{}{}{}", result.context, result.query, result.fragment),
      context: result.context,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      replaces: result.replaces,
      start: expr.span(&parser.ast).real_lo(),
      end: expr.span(&parser.ast).real_hi(),
      referenced_exports: None,
      attributes: None,
      phase: None,
    };

    let mut dep = URLContextDependency::new(
      options,
      expr.span(&parser.ast).into(),
      param.range().into(),
      parser.in_try,
    );
    *dep.critical_mut() = result.critical;
    parser.add_dependency(Box::new(dep));

    Some(true)
  }

  fn is_pure(&self, parser: &mut JavascriptParser, expr: Expr) -> Option<bool> {
    let expr = expr.as_new()?;
    let callee = expr.callee(&parser.ast).as_ident()?;
    if parser
      .get_free_info_from_variable(&parser.ast.get_atom(callee.sym(&parser.ast)))
      .is_none()
      || !parser.ast.get_utf8(callee.sym(&parser.ast)).eq("URL")
    {
      return None;
    }
    get_url_request(parser, expr)?;
    Some(true)
  }
}
