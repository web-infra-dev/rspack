use itertools::Itertools;
use rspack_core::{
  context_reg_exp, ConstDependency, ContextMode, DependencyCategory, ErrorSpan, SpanExt,
};
use rspack_core::{ContextNameSpaceObject, ContextOptions};
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Ident, Lit, MemberExpr};

use super::JavascriptParserPlugin;
use crate::dependency::RequireHeaderDependency;
use crate::dependency::{CommonJsFullRequireDependency, CommonJsRequireContextDependency};
use crate::dependency::{CommonJsRequireDependency, RequireResolveDependency};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::{expr_matcher, scanner_context_module, JavascriptParser};
use crate::visitors::{extract_require_call_info, is_require_call_start};

pub const COMMONJS_REQUIRE: &str = "require";

fn create_commonjs_require_context_dependency(
  expr: &Expr,
  _param: &BasicEvaluatedExpression,
  callee_start: u32,
  callee_end: u32,
  args_end: u32,
  span: Option<ErrorSpan>,
) -> Option<CommonJsRequireContextDependency> {
  // TODO: enabled it later
  // create_context_dependency(param, expr).map(|result| {
  //   let options = ContextOptions {
  //     chunk_name: None,
  //     mode: ContextMode::Sync,
  //     recursive: true,
  //     reg_exp: context_reg_exp(&result.reg, ""),
  //     reg_str: result.reg,
  //     include: None,
  //     exclude: None,
  //     category: DependencyCategory::CommonJS,
  //     request: format!("{}{}{}", result.context, result.query, result.fragment),
  //     namespace_object: ContextNameSpaceObject::Unset,
  //   };
  //   CommonJsRequireContextDependency::new(callee_start, callee_end, args_end, options, span)
  // });
  scanner_context_module(expr).map(|result| {
    let options = ContextOptions {
      chunk_name: None,
      mode: ContextMode::Sync,
      recursive: true,
      reg_exp: context_reg_exp(&result.reg, ""),
      reg_str: result.reg,
      include: None,
      exclude: None,
      category: DependencyCategory::CommonJS,
      request: format!("{}{}{}", result.context, result.query, result.fragment),
      namespace_object: ContextNameSpaceObject::Unset,
      start: callee_start,
      end: callee_end,
    };
    CommonJsRequireContextDependency::new(callee_start, callee_end, args_end, options, span)
  })
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

  fn replace_require_resolve(
    &self,
    parser: &mut JavascriptParser,
    expr: &Expr,
    value: &'static str,
  ) -> Option<bool> {
    if (expr_matcher::is_require(expr)
      || expr_matcher::is_require_resolve(expr)
      || expr_matcher::is_require_resolve_weak(expr))
      && parser.is_unresolved_require(expr)
    {
      parser
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          value.into(),
          None,
        )));
      Some(true)
    } else {
      None
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
    create_commonjs_require_context_dependency(
      argument_expr,
      param,
      call_expr.callee.span().real_lo(),
      call_expr.callee.span().real_hi(),
      call_expr.span.real_hi(),
      Some(call_expr.span.into()),
    )
    .map(|dep| {
      parser.dependencies.push(Box::new(dep));
      true
    })
  }

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    let is_require_expr = for_name == COMMONJS_REQUIRE
      || call_expr
        .callee
        .as_expr()
        .is_some_and(|expr| expr_matcher::is_module_require(expr)); // FIXME: remove `module.require`

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
}

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn can_rename(&self, parser: &mut JavascriptParser, str: &str) -> Option<bool> {
    if str == COMMONJS_REQUIRE && parser.is_unresolved_ident(str) {
      Some(true)
    } else {
      None
    }
  }

  fn rename(&self, parser: &mut JavascriptParser, expr: &Expr, str: &str) -> Option<bool> {
    if str == COMMONJS_REQUIRE && parser.is_unresolved_ident(str) {
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
    parser: &mut JavascriptParser,
    expression: &Ident,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if expression.sym.as_str() == COMMONJS_REQUIRE && parser.is_unresolved_ident(COMMONJS_REQUIRE) {
      Some(eval::evaluate_to_string("function".to_string(), start, end))
    } else {
      None
    }
  }

  fn evaluate_identifier(
    &self,
    parser: &mut JavascriptParser,
    ident: &str,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if ident == COMMONJS_REQUIRE && parser.is_unresolved_ident(COMMONJS_REQUIRE) {
      Some(eval::evaluate_to_identifier(
        COMMONJS_REQUIRE.to_string(),
        COMMONJS_REQUIRE.to_string(),
        Some(true),
        start,
        end,
      ))
    } else {
      None
    }
  }

  fn expression_logical_operator(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::BinExpr,
  ) -> Option<bool> {
    // TODO: this block can be removed after eval identifier
    let value = if parser.in_if { "true" } else { "undefined" };
    self.replace_require_resolve(parser, &expr.left, value);
    self.replace_require_resolve(parser, &expr.right, value);
    None
  }

  fn statement_if(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::IfStmt,
  ) -> Option<bool> {
    self.replace_require_resolve(parser, &expr.test, "true")
  }

  fn r#typeof(
    &self,
    parser: &mut JavascriptParser,
    expr: &swc_core::ecma::ast::UnaryExpr,
    _for_name: &str,
  ) -> Option<bool> {
    if (expr_matcher::is_require(&expr.arg)
      || expr_matcher::is_require_resolve(&expr.arg)
      || expr_matcher::is_require_resolve_weak(&expr.arg))
      && parser.is_unresolved_require(&expr.arg)
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
    let Callee::Expr(expr) = &call_expr.callee else {
      return Some(false);
    };
    if self
      .require_handler(parser, call_expr, for_name)
      .unwrap_or_default()
    {
      return Some(true);
    }

    if parser.is_unresolved_member_object_ident(expr) {
      if expr_matcher::is_require_resolve(expr) {
        self.add_require_resolve(parser, call_expr, false);
        return Some(true);
      }
      if expr_matcher::is_require_resolve_weak(expr) {
        self.add_require_resolve(parser, call_expr, true);
        return Some(true);
      }
    }
    None
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
}
