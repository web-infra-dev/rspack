use itertools::Itertools;
use rspack_core::{context_reg_exp, ConstDependency, ContextMode, SpanExt};
use rspack_core::{ContextNameSpaceObject, ContextOptions, DependencyCategory};
use swc_core::common::Spanned;
use swc_core::ecma::ast::{CallExpr, Callee, Expr, Ident, Lit, MemberExpr};

use super::JavascriptParserPlugin;
use crate::dependency::RequireHeaderDependency;
use crate::dependency::{CommonJsFullRequireDependency, CommonJsRequireContextDependency};
use crate::dependency::{CommonJsRequireDependency, RequireResolveDependency};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::visitors::{expr_matcher, scanner_context_module, JavascriptParser};
use crate::visitors::{extract_require_call_info, is_require_call_start};

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

  fn require_handler(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
  ) -> Option<(Vec<CommonJsRequireDependency>, Vec<RequireHeaderDependency>)> {
    if call_expr.args.len() != 1 {
      return None;
    }

    let is_require_expr = call_expr.callee.as_expr().is_some_and(|expr| {
      (expr_matcher::is_require(expr) && parser.is_unresolved_require(expr))
        || expr_matcher::is_module_require(expr)
    });

    if !is_require_expr {
      return None;
    }

    let Some(argument_expr) = call_expr.args.first().map(|arg| &arg.expr) else {
      return None;
    };

    let in_try = parser.in_try;

    let process_require_item = |p: &BasicEvaluatedExpression| {
      p.is_string().then(|| {
        let dep = CommonJsRequireDependency::new(
          p.string().to_string(),
          Some(call_expr.span.into()),
          p.range().0,
          p.range().1,
          in_try,
        );
        dep
      })
    };
    let param = parser.evaluate_expression(argument_expr);
    let mut commonjs_require_deps = vec![];
    let mut require_header_deps = vec![];
    if param.is_conditional() {
      let mut is_expression = false;
      for p in param.options() {
        if let Some(dep) = process_require_item(p) {
          commonjs_require_deps.push(dep)
        } else {
          is_expression = true;
        }
      }
      if !is_expression {
        require_header_deps.push(RequireHeaderDependency::new(
          call_expr.callee.span().real_lo(),
          call_expr.callee.span().hi().0,
        ));
      }
    }

    if let Some(dep) = process_require_item(&param) {
      commonjs_require_deps.push(dep);
      require_header_deps.push(RequireHeaderDependency::new(
        call_expr.callee.span().real_lo(),
        call_expr.callee.span_hi().0,
      ));
    }

    Some((commonjs_require_deps, require_header_deps))
  }
}

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn evaluate_typeof(
    &self,
    parser: &mut JavascriptParser,
    expression: &Ident,
    start: u32,
    end: u32,
  ) -> Option<BasicEvaluatedExpression> {
    if expression.sym.as_str() == "require" && parser.is_unresolved_ident("require") {
      Some(eval::evaluate_to_string("function".to_string(), start, end))
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

  fn call(&self, parser: &mut JavascriptParser, call_expr: &CallExpr, _name: &str) -> Option<bool> {
    let Callee::Expr(expr) = &call_expr.callee else {
      return Some(false);
    };
    if let Some(dep) = call_expr
      .callee
      .as_expr()
      .and_then(|expr| expr.as_member())
      .and_then(|mem| self.chain_handler(parser, mem, true))
    {
      parser.dependencies.push(Box::new(dep));
      return Some(false);
    }

    let deps = self.require_handler(parser, call_expr);

    if let Some((commonjs_require_deps, require_helper_deps)) = deps {
      for dep in commonjs_require_deps {
        parser.dependencies.push(Box::new(dep))
      }
      for dep in require_helper_deps {
        parser.presentational_dependencies.push(Box::new(dep))
      }
    }

    if let Expr::Ident(ident) = &**expr
      && "require".eq(&ident.sym)
      && parser.is_unresolved_ident("require")
      && let Some(expr) = call_expr.args.first()
      && call_expr.args.len() == 1
      && expr.spread.is_none()
      && let Some((context, reg)) = scanner_context_module(expr.expr.as_ref())
    {
      // `require.resolve`
      parser
        .dependencies
        .push(Box::new(CommonJsRequireContextDependency::new(
          call_expr.callee.span().real_lo(),
          call_expr.callee.span().real_hi(),
          call_expr.span.real_hi(),
          ContextOptions {
            chunk_name: None,
            mode: ContextMode::Sync,
            recursive: true,
            reg_exp: context_reg_exp(&reg, ""),
            reg_str: reg,
            include: None,
            exclude: None,
            category: DependencyCategory::CommonJS,
            request: context,
            namespace_object: ContextNameSpaceObject::Unset,
          },
          Some(call_expr.span.into()),
        )));
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

  fn member(&self, parser: &mut JavascriptParser, expr: &MemberExpr, _name: &str) -> Option<bool> {
    if let Some(dep) = self.chain_handler(parser, expr, false) {
      parser.dependencies.push(Box::new(dep));
      Some(true)
    } else {
      None
    }
  }
}
