use std::sync::Arc;

use rspack_core::{
  BoxDependency, ContextDependency, ContextMode, ContextOptions, DependencyCategory,
  JavascriptParserUrl, RuntimeGlobals, RuntimeRequirementsDependency, get_context,
};
use rspack_util::SpanExt;
use swc_next_ecma_ast::{
  ArgumentData, Ast, Expr, GetSpan, MemberExpression, NewExpression, Visit, VisitWith,
};
use url::Url;

use super::{JavascriptParserPlugin, inner_graph::state::InnerGraphUsageOperation};
use crate::{
  InnerGraphParserPlugin,
  dependency::{URLContextDependency, URLDependency},
  magic_comment::{MagicCommentValue, try_extract_magic_comment},
  visitors::{ExprRef, JavascriptParser, RootName, context_reg_exp, create_context_dependency},
};

struct NestedNewUrlVisitor<'a> {
  ast: &'a Ast<'a>,
  has_nested_new_url: bool,
}

impl<'a> Visit<'a> for NestedNewUrlVisitor<'a> {
  fn ast(&self) -> &Ast<'a> {
    self.ast
  }

  fn visit_new_expression(&mut self, expr: NewExpression) {
    if expr
      .callee(self.ast)
      .as_identifier_reference(self.ast)
      .is_some_and(|identifier| self.ast.get_utf8(identifier.name(self.ast)) == "URL")
    {
      self.has_nested_new_url = true;
    }
  }
}

pub fn is_meta_url(parser: &mut JavascriptParser, expr: MemberExpression) -> bool {
  let ast = parser.ast.ast;
  let chain = parser.extract_member_expression_chain(ExprRef::Member(expr));
  if let ExprRef::MetaProp(meta) = chain.object {
    return meta
      .get_root_name(ast)
      .is_some_and(|name| name == "import.meta")
      && chain.members.len() == 1
      && chain.members.first().is_some_and(|member| member == "url");
  }
  false
}

pub fn get_url_request(
  parser: &mut JavascriptParser,
  expr: NewExpression,
) -> Option<(String, u32, u32)> {
  let ast = parser.ast.ast;
  let arguments = expr.arguments(ast);
  let ArgumentData::Expr(arg1) = ast.argument_data(arguments.get_node(ast, 0)?) else {
    return None;
  };
  let arg2 = arguments.get_node(ast, 1);

  if let Some(arg2) = arg2 {
    // new URL(xx, import.meta.url)
    let ArgumentData::Expr(arg2) = ast.argument_data(arg2) else {
      return None;
    };
    let arg2 = arg2.as_member_expression(ast)?;
    if is_meta_url(parser, arg2) {
      return parser
        .evaluate_expression(arg1)
        .as_string()
        .map(|req| (req, arg1.span(ast).real_lo(), arg2.span(ast).real_hi()));
    }
  } else {
    // new URL(import.meta.url)
    let arg1 = arg1.as_member_expression(ast)?;
    if is_meta_url(parser, arg1) {
      return Some((
        Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_string(),
        arg1.span(ast).real_lo(),
        arg1.span(ast).real_hi(),
      ));
    }
  }

  None
}

pub struct URLPlugin {
  pub mode: Option<JavascriptParserUrl>,
  pub import_meta_url_enabled: bool,
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl<'p, 'a> JavascriptParserPlugin<'p, 'a> for URLPlugin {
  fn can_rename(&self, _parser: &mut JavascriptParser<'p>, for_name: &str) -> Option<bool> {
    (for_name == "URL").then_some(true)
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser<'p>,
    expr: NewExpression,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != "URL" {
      return None;
    }

    let ast = parser.ast.ast;
    let arguments = expr.arguments(ast);

    let arg = arguments.get_node(ast, 0)?;
    let magic_comment_options = try_extract_magic_comment(parser, expr.span(ast), arg.span(ast));
    match magic_comment_options.get_ignore_value() {
      Some(MagicCommentValue::Bool(true)) => {
        if arguments.len() != 2 || !self.import_meta_url_enabled {
          return None;
        }
        let arg2 = arguments.get_node(ast, 1)?;
        if let ArgumentData::Expr(arg2_expr) = ast.argument_data(arg2)
          && let Some(arg2) = arg2_expr.as_member_expression(ast)
          && !is_meta_url(parser, arg2)
        {
          return None;
        }
        parser.add_presentational_dependency(Arc::new(RuntimeRequirementsDependency::new(
          arg2.span(ast).into(),
          RuntimeGlobals::BASE_URI,
        )));
        return Some(true);
      }
      Some(MagicCommentValue::Bool(false)) | None => {}
      Some(_) => return None,
    }

    // should not parse new URL(import.meta.url)
    if arguments.len() == 1
      && arguments
        .get_node(ast, 0)
        .and_then(|argument| argument.as_expr(ast))
        .and_then(|expression| expression.as_member_expression(ast))
        .is_some_and(|member| is_meta_url(parser, member))
    {
      return None;
    }

    if let Some((request, start, end)) = get_url_request(parser, expr) {
      if request.starts_with("//") {
        if arguments.len() == 2 {
          parser.walk_arguments(std::iter::once(arguments.get_node(ast, 1)?));
          return Some(true);
        }
        return None;
      }
      let dep = URLDependency::new(
        request.into(),
        expr.span(ast).into(),
        (start, end).into(),
        self.mode,
      );
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(BoxDependency::new(dep));
      InnerGraphParserPlugin::on_usage(parser, InnerGraphUsageOperation::URLDependency(dep_idx));
      return Some(true);
    }

    let ArgumentData::Expr(arg_expression) = ast.argument_data(arg) else {
      return None;
    };
    let mut nested_new_url_visitor = NestedNewUrlVisitor {
      ast,
      has_nested_new_url: false,
    };
    arg_expression.visit_with(&mut nested_new_url_visitor);
    if nested_new_url_visitor.has_nested_new_url {
      return None;
    }

    let arg2 = arguments.get_node(ast, 1)?;
    if !arg2
      .as_expr(ast)
      .and_then(|expression| expression.as_member_expression(ast))
      .is_some_and(|member| is_meta_url(parser, member))
    {
      return None;
    }

    let param = parser.evaluate_expression(arg_expression);
    let result = create_context_dependency(&param, parser);
    let request = result.request();
    let options = ContextOptions {
      mode: ContextMode::Sync,
      recursive: true,
      pattern: context_reg_exp(&result.reg, "", None, parser).into(),
      include: magic_comment_options.get_include(),
      exclude: magic_comment_options.get_exclude(),
      category: DependencyCategory::Url,
      request,
      context: get_context(parser.resource_data).to_string(),
      compiler_context: parser.compiler_options.context.clone(),
      replaces: result.replaces,
      start: expr.span(ast).real_lo(),
      end: expr.span(ast).real_hi(),
      ..Default::default()
    };

    let dep = URLContextDependency::new(
      options,
      expr.span(ast).into(),
      param.range().into(),
      parser.in_try,
    );
    dep.set_critical(result.critical);
    parser.add_dependency(BoxDependency::new(dep));

    Some(true)
  }

  fn is_pure(&self, parser: &mut JavascriptParser<'p>, expr: Expr) -> Option<bool> {
    let ast = parser.ast.ast;
    let expr = expr.as_new_expression(ast)?;
    let callee = expr.callee(ast).as_identifier_reference(ast)?;
    if parser
      .get_free_info_from_variable(ast.get_utf8(callee.name(ast)))
      .is_none()
      || ast.get_utf8(callee.name(ast)) != "URL"
    {
      return None;
    }
    get_url_request(parser, expr)?;
    Some(true)
  }
}
