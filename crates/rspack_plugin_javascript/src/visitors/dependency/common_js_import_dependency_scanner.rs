use rspack_core::{
  CodeGeneratableDependency, ConstDependency, ContextMode, ContextOptions, DependencyCategory,
  ModuleDependency, RuntimeGlobals, SpanExt,
};
use rspack_regex::RspackRegex;
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::{
    ast::{BinExpr, CallExpr, Callee, Expr, IfStmt, Lit, TryStmt, UnaryExpr, UnaryOp},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{
  context_helper::scanner_context_module, expr_matcher, is_unresolved_member_object_ident,
};
use crate::dependency::{
  CommonJsRequireContextDependency, CommonJsRequireDependency, RequireResolveDependency,
};

pub struct CommonJsImportDependencyScanner<'a> {
  dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
  unresolved_ctxt: &'a SyntaxContext,
  in_try: bool,
  in_if: bool,
}

impl<'a> CommonJsImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
    unresolved_ctxt: &'a SyntaxContext,
  ) -> Self {
    Self {
      dependencies,
      presentational_dependencies,
      unresolved_ctxt,
      in_try: false,
      in_if: false,
    }
  }

  fn add_require_resolve(&mut self, node: &CallExpr, weak: bool) {
    if !node.args.is_empty() {
      if let Some(Lit::Str(str)) = node.args.get(0).and_then(|x| x.expr.as_lit()) {
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
    if expr_matcher::is_require(expr)
      || expr_matcher::is_require_resolve(expr)
      || expr_matcher::is_require_resolve_weak(expr)
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
}

impl Visit for CommonJsImportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_try_stmt(&mut self, node: &TryStmt) {
    self.in_try = true;
    node.visit_children_with(self);
    self.in_try = false;
  }

  fn visit_call_expr(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        if "require".eq(&ident.sym) && ident.span.ctxt == *self.unresolved_ctxt {
          {
            if let Some(expr) = call_expr.args.get(0) && call_expr.args.len() == 1 && expr.spread.is_none() {
              // TemplateLiteral String
              if let Expr::Tpl(tpl) = expr.expr.as_ref()  && tpl.exprs.is_empty(){
                let s = tpl.quasis.first().expect("should have one quasis").raw.as_ref();
                let request = JsWord::from(s);
                  self.dependencies.push(Box::new(CommonJsRequireDependency::new(
                  request,
                  Some(call_expr.span.into()),
                  call_expr.span.real_lo(),
                  call_expr.span.real_hi(),
                  self.in_try
                )));
                return;
              }
              if let Expr::Lit(Lit::Str(s)) = expr.expr.as_ref() {
                self
                  .dependencies
                  .push(Box::new(CommonJsRequireDependency::new(
                    s.value.clone(),
                    Some(call_expr.span.into()),
                    call_expr.span.real_lo(),
                    call_expr.span.real_hi(),
                    self.in_try,
                  )));
                return;
              }
              if let Some((context, reg)) = scanner_context_module(expr.expr.as_ref()) {
                self
                  .dependencies
                  .push(Box::new(CommonJsRequireContextDependency::new(
                    call_expr.callee.span().real_lo(),
                    call_expr.callee.span().real_hi(),
                    call_expr.span.real_hi(),
                    ContextOptions {
                      mode: ContextMode::Sync,
                      recursive: true,
                      reg_exp: RspackRegex::new(&reg).expect("reg failed"),
                      reg_str: reg,
                      include: None,
                      exclude: None,
                      category: DependencyCategory::CommonJS,
                      request: context,
                    },
                    Some(call_expr.span.into()),
                  )));
                return;
              }
            }
            self
              .presentational_dependencies
              .push(Box::new(ConstDependency::new(
                ident.span().real_lo(),
                ident.span().real_hi(),
                RuntimeGlobals::REQUIRE.name().into(),
                None,
              )));
          }
        }
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
    if_stmt.visit_children_with(self);
    self.in_if = false;
  }

  fn visit_bin_expr(&mut self, bin_expr: &BinExpr) {
    let value = if self.in_if { "true" } else { "undefined" };
    self.replace_require_resolve(&bin_expr.left, value);
    self.replace_require_resolve(&bin_expr.right, value);
    bin_expr.visit_children_with(self);
  }
}
