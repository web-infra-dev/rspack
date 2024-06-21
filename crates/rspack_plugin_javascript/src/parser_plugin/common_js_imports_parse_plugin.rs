use itertools::Itertools;
use rspack_core::{
  context_reg_exp, ConstDependency, ContextMode, DependencyCategory, ErrorSpan, SpanExt,
};
use rspack_core::{ContextNameSpaceObject, ContextOptions};
use rspack_error::Severity;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{CallExpr, Expr, Ident, Lit, MemberExpr, UnaryExpr};

use super::JavascriptParserPlugin;
use crate::dependency::RequireHeaderDependency;
use crate::dependency::{CommonJsFullRequireDependency, CommonJsRequireContextDependency};
use crate::dependency::{CommonJsRequireDependency, RequireResolveDependency};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::{
  create_context_dependency, create_traceable_error, expr_matcher, expr_name, JavascriptParser,
};
use crate::visitors::{extract_require_call_info, is_require_call_start};

fn create_commonjs_require_context_dependency(
  parser: &mut JavascriptParser,
  param: &BasicEvaluatedExpression,
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  span: Option<ErrorSpan>,
) -> CommonJsRequireContextDependency {
  let result = create_context_dependency(param, parser);
  let options = ContextOptions {
    mode: ContextMode::Sync,
    recursive: true,
    reg_exp: context_reg_exp(&result.reg, ""),
    include: None,
    exclude: None,
    category: DependencyCategory::CommonJS,
    request: format!("{}{}{}", result.context, result.query, result.fragment),
    context: result.context,
    namespace_object: ContextNameSpaceObject::Unset,
    group_options: None,
    replaces: result.replaces,
    start: callee_start,
    end: callee_end,
  };
  CommonJsRequireContextDependency::new(
    callee_start,
    callee_end,
    args_end,
    options,
    span,
    parser.in_try,
  )
}

pub struct CommonJsImportsParserPlugin;

impl CommonJsImportsParserPlugin {
  fn add_require_resolve(&self, parser: &mut JavascriptParser, node: &CallExpr, weak: bool) {
    if !node.args.is_empty()
      && let Some(Lit::Str(str)) = node.args.first().and_then(|x| x.expr.as_lit())
    {
      parser
        .dependencies
        .push(Box::new(RequireResolveDependency::new(
          node.span.real_lo(),
          node.span.real_hi(),
          str.value.to_string(),
          weak,
          node.span.into(),
          parser.in_try,
        )));
    }
  }

  fn chain_handler(
    &self,
    parser: &mut JavascriptParser,
    mem_expr: &MemberExpr,
    is_call: bool,
  ) -> Option<CommonJsFullRequireDependency> {
    let expr = Expr::Member(mem_expr.to_owned());

    let is_require_member_chain = is_require_call_start(&expr)
      && !expr_matcher::is_require(&expr)
      && !expr_matcher::is_module_require(&expr)
      && parser.is_unresolved_ident("require");
    if !is_require_member_chain {
      return None;
    }

    let Some((members, first_arg, loc)) = extract_require_call_info(&expr) else {
      return None;
    };

    let param = parser.evaluate_expression(&first_arg.expr);
    param.is_string().then(|| {
      CommonJsFullRequireDependency::new(
        param.string().to_string(),
        members.iter().map(|i| i.to_owned()).collect_vec(),
        loc,
        Some(mem_expr.span.into()),
        is_call,
        parser.in_try,
        !parser.is_asi_position(mem_expr.span_lo()),
      )
    })
  }

  fn process_require_item(
    &self,
    parser: &mut JavascriptParser,
    span: Span,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    param.is_string().then(|| {
      let dep = CommonJsRequireDependency::new(
        param.string().to_string(),
        Some(span.into()),
        param.range().0,
        param.range().1,
        parser.in_try,
      );
      parser.dependencies.push(Box::new(dep));
      true
    })
  }

  fn process_require_context(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    param: &BasicEvaluatedExpression,
  ) -> Option<bool> {
    let Some(argument_expr) = &call_expr.args.first().map(|expr| expr.expr.as_ref()) else {
      unreachable!("ensure require includes arguments")
    };
    let dep = create_commonjs_require_context_dependency(
      parser,
      param,
      call_expr.callee.span().real_lo(),
      call_expr.callee.span().real_hi(),
      call_expr.span.real_hi(),
      Some(call_expr.span.into()),
    );
    parser.dependencies.push(Box::new(dep));
    // FIXME: align `parser.walk_expression` to webpack, which put into `context_dependency_helper`
    parser.walk_expression(argument_expr);
    Some(true)
  }

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    let is_require_expr = for_name == expr_name::REQUIRE
      || call_expr
        .callee
        .as_expr()
        .is_some_and(|expr| expr_matcher::is_module_require(&**expr)); // FIXME: remove `module.require`

    if !is_require_expr || call_expr.args.len() != 1 {
      return None;
    }

    let argument_expr = &call_expr.args[0].expr;
    let param = parser.evaluate_expression(argument_expr);
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if self
          .process_require_item(parser, call_expr.span(), p)
          .is_none()
        {
          is_expression = true;
        }
      }
      if !is_expression {
        parser
          .presentational_dependencies
          .push(Box::new(RequireHeaderDependency::new(
            call_expr.callee.span().real_lo(),
            call_expr.callee.span().hi().0,
          )));
        return Some(true);
      }
    }

    // FIXME: should support `LocalModuleDependency`
    if self
      .process_require_item(parser, call_expr.span, &param)
      .is_none()
    {
      self.process_require_context(parser, call_expr, &param);
    } else {
      parser
        .presentational_dependencies
        .push(Box::new(RequireHeaderDependency::new(
          call_expr.callee.span().real_lo(),
          call_expr.callee.span_hi().0,
        )));
    }
    Some(true)
  }

  fn require_as_expression_handler(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
  ) -> Option<bool> {
    let dep = CommonJsRequireContextDependency::new(
      ident.span().real_lo(),
      ident.span().real_hi(),
      ident.span().real_hi(),
      ContextOptions {
        mode: ContextMode::Sync,
        recursive: true,
        reg_exp: None,
        include: None,
        exclude: None,
        category: DependencyCategory::Unknown,
        request: ".".to_string(),
        context: ".".to_string(),
        namespace_object: ContextNameSpaceObject::Unset,
        group_options: None,
        replaces: Vec::new(),
        start: ident.span().real_lo(),
        end: ident.span().real_hi(),
      },
      Some(ident.span().into()),
      parser.in_try,
    );
    parser.warning_diagnostics.push(Box::new(
      create_traceable_error(
        "Critical dependency".into(),
        "require function is used in a way in which dependencies cannot be statically extracted"
          .to_string(),
        parser.source_file,
        ident.span().into(),
      )
      .with_severity(Severity::Warn),
    ));
    parser.dependencies.push(Box::new(dep));
    Some(true)
  }
}

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    if str == expr_name::REQUIRE && parser.is_unresolved_ident(str) {
      Some(true)
    } else {
      None
    }
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    if str == expr_name::REQUIRE && parser.is_unresolved_ident(str) {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "undefined".into(),
          None,
        )));
      Some(false)
    } else {
      None
    }
  }

  fn evaluate_typeof(
    &self,
    _parser: &mut JavascriptParser,
    expr: &UnaryExpr,
    for_name: &str,
  ) -> Option<BasicEvaluatedExpression> {
    (for_name == expr_name::REQUIRE
      || for_name == expr_name::REQUIRE_RESOLVE
      || for_name == expr_name::REQUIRE_RESOLVE_WEAK)
      .then(|| {
        eval::evaluate_to_string(
          "function".to_string(),
          expr.span.real_lo(),
          expr.span.real_hi(),
        )
      })
  }

  fn evaluate_identifier(
    &self,
    _parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    match ident {
      expr_name::REQUIRE => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE.to_string(),
        expr_name::REQUIRE.to_string(),
        Some(true),
        start,
        end,
      )),
      expr_name::REQUIRE_RESOLVE => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE.to_string(),
        expr_name::REQUIRE.to_string(),
        Some(true),
        start,
        end,
      )),
      expr_name::REQUIRE_RESOLVE_WEAK => Some(eval::evaluate_to_identifier(
        expr_name::REQUIRE_RESOLVE_WEAK.to_string(),
        expr_name::REQUIRE.to_string(),
        Some(true),
        start,
        end,
      )),
      _ => None,
    }
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    for_name: &str,
  ) -> Option<bool> {
    // same as webpack/tagRequireExpression
    if for_name == expr_name::REQUIRE
      || for_name == expr_name::REQUIRE_RESOLVE
      || for_name == expr_name::REQUIRE_RESOLVE_WEAK
    {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "'function'".into(),
          None,
        )));
      Some(true)
    } else {
      None
    }
  }

  fn call_member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if let Some(dep) = call_expr
      .callee
      .as_expr()
      .and_then(|expr| expr.as_member())
      .and_then(|mem| self.chain_handler(parser, mem, true))
    {
      parser.dependencies.push(Box::new(dep));
      parser.walk_expr_or_spread(&call_expr.args);
      return Some(true);
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if self
      .require_handler(parser, call_expr, for_name)
      .unwrap_or_default()
    {
      Some(true)
    } else if for_name == expr_name::REQUIRE_RESOLVE {
      self.add_require_resolve(parser, call_expr, false);
      Some(true)
    } else if for_name == expr_name::REQUIRE_RESOLVE_WEAK {
      self.add_require_resolve(parser, call_expr, true);
      Some(true)
    } else {
      None
    }
  }

  fn member_chain_of_call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    expr: &MemberExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if let Some(dep) = self.chain_handler(parser, expr, false) {
      parser.dependencies.push(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }

  fn identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &Ident,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == expr_name::REQUIRE {
      return self.require_as_expression_handler(parser, ident);
    }
    None
  }
}
