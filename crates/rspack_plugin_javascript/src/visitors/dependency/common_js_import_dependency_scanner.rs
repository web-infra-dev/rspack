use rspack_core::{ContextMode, ContextOptions, DependencyCategory, ModuleDependency, SpanExt};
use rspack_regex::RspackRegex;
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::{
    ast::{CallExpr, Callee, Expr, Lit},
    atoms::JsWord,
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::scanner::scanner_context_module;
use crate::dependency::{CommonJsRequireContextDependency, CommonJsRequireDependency};

pub struct CommonJsImportDependencyScanner<'a> {
  pub dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
  pub unresolved_ctxt: &'a SyntaxContext,
}

impl<'a> CommonJsImportDependencyScanner<'a> {
  pub fn new(
    dependencies: &'a mut Vec<Box<dyn ModuleDependency>>,
    unresolved_ctxt: &'a SyntaxContext,
  ) -> Self {
    Self {
      dependencies,
      unresolved_ctxt,
    }
  }
}

impl Visit for CommonJsImportDependencyScanner<'_> {
  noop_visit_type!();

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
                    false
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
                      false,
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
                }
              }
            }
          }
        }
      }
    }
    call_expr.visit_children_with(self);
  }
}
