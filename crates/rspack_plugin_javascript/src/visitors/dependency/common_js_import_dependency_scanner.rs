use std::rc::Rc;

use itertools::Itertools;
use rspack_core::{context_reg_exp, ContextOptions, DependencyCategory, DependencyLocation};
use rspack_core::{BoxDependency, ConstDependency, ContextMode, ContextNameSpaceObject};
use rspack_core::{DependencyTemplate, SpanExt};
use swc_core::common::{Spanned, SyntaxContext};
use swc_core::ecma::ast::{BinExpr, BlockStmt, CallExpr, Callee, Expr, IfStmt, MemberExpr};
use swc_core::ecma::ast::{Lit, TryStmt, UnaryExpr, UnaryOp};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use super::api_scanner::ApiParserPlugin;
use super::common_js_export_scanner::CommonJsExportParserPlugin;
use super::context_helper::scanner_context_module;
use super::expr_matcher::{is_module_require, is_require};
use super::{
  expr_matcher, extract_require_call_info, is_require_call_start,
  is_unresolved_member_object_ident, is_unresolved_require,
};
use crate::dependency::{
  CommonJsFullRequireDependency, CommonJsRequireContextDependency, RequireHeaderDependency,
};
use crate::dependency::{CommonJsRequireDependency, RequireResolveDependency};
use crate::parser_plugin::{
  BoxJavascriptParserPlugin, JavaScriptParserPluginDrive, JavascriptParserPlugin,
  RequireContextDependencyParserPlugin,
};
use crate::utils::eval::{self, BasicEvaluatedExpression};
use crate::utils::{expression_logic_operator, statement_if};

struct CommonJsImportsParserPlugin;

impl JavascriptParserPlugin for CommonJsImportsParserPlugin {
  fn evaluate_typeof(
    &self,
    expression: &swc_core::ecma::ast::Ident,
    start: u32,
    end: u32,
    unresolved_mark: swc_core::common::SyntaxContext,
  ) -> Option<BasicEvaluatedExpression> {
    if expression.sym.as_str() == "require" && expression.span.ctxt == unresolved_mark {
      Some(eval::evaluate_to_string("function".to_string(), start, end))
    } else {
      None
    }
  }
}

pub struct CommonJsImportDependencyScanner<'a> {
  pub(crate) dependencies: &'a mut Vec<BoxDependency>,
  pub(crate) presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  pub(crate) unresolved_ctxt: SyntaxContext,
  pub(crate) in_try: bool,
  pub(crate) in_if: bool,
  pub(crate) is_strict: bool,
  pub(crate) plugin_drive: Rc<JavaScriptParserPluginDrive>,
  pub(crate) ignored: &'a mut Vec<DependencyLocation>,
}

#[derive(Debug)]
enum Mode {
  Strict,
  Nothing,
}

fn detect_mode(stmt: &BlockStmt) -> Mode {
  let Some(Lit::Str(str)) = stmt
    .stmts
    .first()
    .and_then(|stmt| stmt.as_expr())
    .and_then(|expr_stmt| expr_stmt.expr.as_lit())
  else {
    return Mode::Nothing;
  };

  if str.value.as_str() == "use strict" {
    Mode::Strict
  } else {
    Mode::Nothing
  }
}

fn is_strict(stmt: &BlockStmt) -> bool {
  matches!(detect_mode(stmt), Mode::Strict)
}

impl<'a> CommonJsImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<BoxDependency>,
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    ignored: &'a mut Vec<DependencyLocation>,
  ) -> Self {
    let plugins: Vec<BoxJavascriptParserPlugin> = vec![
      Box::new(CommonJsImportsParserPlugin),
      Box::new(RequireContextDependencyParserPlugin),
      Box::new(ApiParserPlugin),
      Box::new(CommonJsExportParserPlugin),
    ];
    let plugin_drive = JavaScriptParserPluginDrive::new(plugins);
    Self {
      dependencies,
      presentational_dependencies,
      unresolved_ctxt,
      in_try: false,
      in_if: false,
      is_strict: false,
      plugin_drive: Rc::new(plugin_drive),
      ignored,
    }
  }

  fn add_require_resolve(&mut self, node: &CallExpr, weak: bool) {
    if !node.args.is_empty() {
      if let Some(Lit::Str(str)) = node.args.first().and_then(|x| x.expr.as_lit()) {
        self
          .dependencies
          .push(Box::new(RequireResolveDependency::new(
            node.span.real_lo(),
            node.span.real_hi(),
            str.value.to_string(),
            weak,
            node.span.into(),
            self.in_try,
          )));
      }
    }
  }

  fn replace_require_resolve(&mut self, expr: &Expr, value: &'static str) {
    if (expr_matcher::is_require(expr)
      || expr_matcher::is_require_resolve(expr)
      || expr_matcher::is_require_resolve_weak(expr))
      && is_unresolved_require(expr, self.unresolved_ctxt)
    {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          value.into(),
          None,
        )));
    }
  }

  fn chain_handler(
    &mut self,
    mem_expr: &MemberExpr,
    is_call: bool,
  ) -> Option<CommonJsFullRequireDependency> {
    let expr = Expr::Member(mem_expr.to_owned());
    let is_require_member_chain =
      is_require_call_start(&expr) && !is_require(&expr) && !is_module_require(&expr);
    if !is_require_member_chain {
      return None;
    }

    let Some((members, first_arg, loc)) = extract_require_call_info(&expr) else {
      return None;
    };

    let param = self.evaluate_expression(&first_arg.expr);
    if param.is_string() {
      Some(CommonJsFullRequireDependency::new(
        param.string().to_string(),
        members.iter().map(|i| i.to_owned()).collect_vec(),
        loc,
        Some(mem_expr.span.into()),
        is_call,
      ))
    } else {
      None
    }
  }

  fn require_handler(
    &mut self,
    call_expr: &CallExpr,
  ) -> Option<(Vec<CommonJsRequireDependency>, Vec<RequireHeaderDependency>)> {
    if call_expr.args.len() != 1 {
      return None;
    }

    let is_require_expr = call_expr.callee.as_expr().is_some_and(|expr| {
      (is_require(expr) && expr.span().ctxt == self.unresolved_ctxt) || is_module_require(expr)
    });
    if !is_require_expr {
      return None;
    }

    let Some(argument_expr) = call_expr.args.first().map(|arg| &arg.expr) else {
      return None;
    };

    let in_try = self.in_try;

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
    let param = self.evaluate_expression(argument_expr);
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

  pub fn walk_left_right_expression(&mut self, expr: &BinExpr) {
    self.walk_expression(&expr.left);
    self.walk_expression(&expr.right);
  }

  pub fn walk_expression(&mut self, expr: &Expr) {
    expr.visit_children_with(self);
  }
}

impl<'a> Visit for CommonJsImportDependencyScanner<'a> {
  noop_visit_type!();

  fn visit_try_stmt(&mut self, node: &TryStmt) {
    self.in_try = true;
    node.visit_children_with(self);
    self.in_try = false;
  }

  fn visit_member_expr(&mut self, mem_expr: &MemberExpr) {
    if let Some(dep) = self.chain_handler(mem_expr, false) {
      self.dependencies.push(Box::new(dep));
      return;
    }
    mem_expr.visit_children_with(self);
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    let Callee::Expr(expr) = &call_expr.callee else {
      call_expr.visit_children_with(self);
      return;
    };

    if self
      .plugin_drive
      .clone()
      .call(self, call_expr)
      .unwrap_or_default()
    {
      return;
    };

    if let Some(dep) = call_expr
      .callee
      .as_expr()
      .and_then(|expr| expr.as_member())
      .and_then(|mem| self.chain_handler(mem, true))
    {
      self.dependencies.push(Box::new(dep));
      return;
    }

    let deps = self.require_handler(call_expr);

    if let Some((commonjs_require_deps, require_helper_deps)) = deps {
      for dep in commonjs_require_deps {
        self.dependencies.push(Box::new(dep))
      }
      for dep in require_helper_deps {
        self.presentational_dependencies.push(Box::new(dep))
      }
    }

    if let Expr::Ident(ident) = &**expr
      && "require".eq(&ident.sym)
      && ident.span.ctxt == self.unresolved_ctxt
      && let Some(expr) = call_expr.args.first()
      && call_expr.args.len() == 1
      && expr.spread.is_none()
      && let Some((context, reg)) = scanner_context_module(expr.expr.as_ref())
    {
      // `require.resolve`
      self
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
      return;
    }

    if is_unresolved_member_object_ident(expr, self.unresolved_ctxt) {
      if expr_matcher::is_require_resolve(expr) {
        self.add_require_resolve(call_expr, false);
        return;
      }
      if expr_matcher::is_require_resolve_weak(expr) {
        self.add_require_resolve(call_expr, true);
        return;
      }
    }
    call_expr.visit_children_with(self);
  }

  fn visit_unary_expr(&mut self, unary_expr: &UnaryExpr) {
    if let UnaryExpr {
      op: UnaryOp::TypeOf,
      arg: box expr,
      ..
    } = unary_expr
    {
      if expr_matcher::is_require(expr)
        || expr_matcher::is_require_resolve(expr)
        || expr_matcher::is_require_resolve_weak(expr)
      {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            unary_expr.span().real_lo(),
            unary_expr.span().real_hi(),
            "'function'".into(),
            None,
          )));
      }
    }
    unary_expr.visit_children_with(self);
  }

  fn visit_if_stmt(&mut self, if_stmt: &IfStmt) {
    self.replace_require_resolve(&if_stmt.test, "true");

    self.in_if = true;
    if let Some(result) = statement_if(self, if_stmt) {
      if result {
        if_stmt.cons.visit_children_with(self);
      } else if let Some(alt) = &if_stmt.alt {
        alt.visit_children_with(self)
      }
    } else {
      self.walk_expression(&if_stmt.test);
      if_stmt.cons.visit_children_with(self);
      if let Some(alt) = &if_stmt.alt {
        alt.visit_children_with(self)
      }
    }
    self.in_if = false;
  }

  fn visit_bin_expr(&mut self, bin_expr: &BinExpr) {
    let value = if self.in_if { "true" } else { "undefined" };
    self.replace_require_resolve(&bin_expr.left, value);
    self.replace_require_resolve(&bin_expr.right, value);

    if let Some(keep_right) = expression_logic_operator(self, bin_expr) {
      if keep_right {
        self.walk_expression(&bin_expr.right);
      }
    } else {
      self.walk_left_right_expression(bin_expr);
    }
  }

  fn visit_block_stmt(&mut self, n: &BlockStmt) {
    let old_in_strict = self.is_strict;

    self.is_strict = is_strict(n);
    n.visit_children_with(self);
    self.is_strict = old_in_strict;
  }
}

impl CommonJsImportDependencyScanner<'_> {
  pub fn evaluate_expression(&mut self, expr: &Expr) -> BasicEvaluatedExpression {
    match self.evaluating(expr) {
      Some(evaluated) => {
        if evaluated.is_compile_time_value() {
          self.ignored.push(DependencyLocation::new(
            expr.span().real_lo(),
            expr.span().real_hi(),
          ));
        }
        evaluated
      }
      None => BasicEvaluatedExpression::with_range(expr.span().real_lo(), expr.span_hi().0),
    }
  }

  // same as `JavascriptParser._initializeEvaluating` in webpack
  // FIXME: should mv it to plugin(for example `parse.hooks.evaluate for`)
  fn evaluating(&mut self, expr: &Expr) -> Option<BasicEvaluatedExpression> {
    match expr {
      Expr::Tpl(tpl) => eval::eval_tpl_expression(self, tpl),
      Expr::Lit(lit) => eval::eval_lit_expr(lit),
      Expr::Cond(cond) => eval::eval_cond_expression(self, cond),
      Expr::Unary(unary) => eval::eval_unary_expression(self, unary),
      Expr::Bin(binary) => eval::eval_binary_expression(self, binary),
      Expr::Array(array) => eval::eval_array_expression(self, array),
      Expr::New(new) => eval::eval_new_expression(self, new),
      _ => None,
    }
  }
}
