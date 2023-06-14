use rspack_core::{
  CodeReplaceSourceDependency, ContextMode, ContextOptions, DependencyCategory, ModuleDependency,
  ReplaceConstDependency, RuntimeGlobals, SpanExt,
};
use rspack_regex::RspackRegex;
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::{
    ast::{CallExpr, Callee, Expr, Ident, IfStmt, Lit, TryStmt, UnaryExpr, UnaryOp},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{expr_matcher, scanner::scanner_context_module};
use crate::dependency::{
  CommonJsRequireContextDependency, CommonJsRequireDependency, RequireResolveDependency,
};

pub struct CommonJsImportDependencyScanner<'a> {
  dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
  unresolved_ctxt: &'a SyntaxContext,
  in_try: bool,
}

impl<'a> CommonJsImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    code_generable_dependencies: &'a mut Vec<Box<dyn CodeReplaceSourceDependency>>,
    unresolved_ctxt: &'a SyntaxContext,
  ) -> Self {
    Self {
      dependencies,
      code_generable_dependencies,
      unresolved_ctxt,
      in_try: false,
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

  fn replace_require_resolve(&mut self, expr: &Expr) {
    if expr_matcher::is_require(expr)
      || expr_matcher::is_require_resolve(expr)
      || expr_matcher::is_require_resolve_weak(expr)
    {
      self
        .code_generable_dependencies
        .push(Box::new(ReplaceConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "true".into(),
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
            if call_expr.args.len() != 1 {
              return;
            }
            if let Some(expr) = call_expr.args.get(0) {
              if expr.spread.is_none() {
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
            }
          }
        }
      }
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
          .code_generable_dependencies
          .push(Box::new(ReplaceConstDependency::new(
            unary_expr.span().real_lo(),
            unary_expr.span().real_hi(),
            "'function'".into(),
            None,
          )));
        return;
      }
    }
    unary_expr.visit_children_with(self);
  }

  fn visit_if_stmt(&mut self, if_stmt: &IfStmt) {
    self.replace_require_resolve(&if_stmt.test);
    if let Expr::Bin(bin) = &*if_stmt.test {
      self.replace_require_resolve(&bin.left);
      self.replace_require_resolve(&bin.right);
    }
    if_stmt.visit_children_with(self);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    // TODO: webpack will replace it to undefined
    if "require".eq(&ident.sym) && ident.span.ctxt == *self.unresolved_ctxt {
      self
        .code_generable_dependencies
        .push(Box::new(ReplaceConstDependency::new(
          ident.span().real_lo(),
          ident.span().real_hi(),
          RuntimeGlobals::REQUIRE.name().into(),
          None,
        )));
    }
  }
}
