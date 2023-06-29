use rspack_core::{
  CodeGeneratableDependency, ConstDependency, ResourceData, RuntimeGlobals, SpanExt,
};
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::{
    ast::{AssignExpr, AssignOp, Expr, Ident, Pat, PatOrExpr, VarDeclarator},
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::expr_matcher;
pub const WEBPACK_HASH: &str = "__webpack_hash__";
pub const WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
pub const WEBPACK_MODULES: &str = "__webpack_modules__";
pub const WEBPACK_RESOURCE_QUERY: &str = "__resourceQuery";
pub const WEBPACK_CHUNK_LOAD: &str = "__webpack_chunk_load__";

pub struct ApiScanner<'a> {
  pub unresolved_ctxt: &'a SyntaxContext,
  pub enter_assign: bool,
  pub resource_data: &'a ResourceData,
  pub presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
}

impl<'a> ApiScanner<'a> {
  pub fn new(
    unresolved_ctxt: &'a SyntaxContext,
    resource_data: &'a ResourceData,
    presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
  ) -> Self {
    Self {
      unresolved_ctxt,
      enter_assign: false,
      resource_data,
      presentational_dependencies,
    }
  }
}

impl Visit for ApiScanner<'_> {
  noop_visit_type!();

  fn visit_var_declarator(&mut self, var_declarator: &VarDeclarator) {
    match &var_declarator.name {
      Pat::Ident(ident) => {
        self.enter_assign = true;
        ident.visit_children_with(self);
        self.enter_assign = false;
      }
      _ => var_declarator.name.visit_children_with(self),
    }
    var_declarator.init.visit_children_with(self);
  }

  fn visit_assign_expr(&mut self, assign_expr: &AssignExpr) {
    if matches!(assign_expr.op, AssignOp::Assign) {
      match &assign_expr.left {
        PatOrExpr::Pat(box Pat::Ident(ident)) => {
          self.enter_assign = true;
          ident.visit_children_with(self);
          self.enter_assign = false;
        }
        _ => assign_expr.left.visit_children_with(self),
      }
    }
    assign_expr.right.visit_children_with(self);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if ident.span.ctxt != *self.unresolved_ctxt {
      return;
    }
    match ident.sym.as_ref() as &str {
      WEBPACK_HASH => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            format!("{}()", RuntimeGlobals::GET_FULL_HASH).into(),
            Some(RuntimeGlobals::GET_FULL_HASH),
          )));
      }
      WEBPACK_PUBLIC_PATH => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::PUBLIC_PATH.name().into(),
            Some(RuntimeGlobals::PUBLIC_PATH),
          )));
      }
      WEBPACK_MODULES => {
        if self.enter_assign {
          return;
        }
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::MODULE_FACTORIES.name().into(),
            Some(RuntimeGlobals::MODULE_FACTORIES),
          )));
      }
      WEBPACK_RESOURCE_QUERY => {
        if let Some(resource_query) = &self.resource_data.resource_query {
          self
            .presentational_dependencies
            .push(Box::new(ConstDependency::new(
              ident.span.real_lo(),
              ident.span.real_hi(),
              serde_json::to_string(resource_query)
                .expect("should render module id")
                .into(),
              None,
            )));
        }
      }
      WEBPACK_CHUNK_LOAD => {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            RuntimeGlobals::ENSURE_CHUNK.name().into(),
            Some(RuntimeGlobals::ENSURE_CHUNK),
          )));
      }
      _ => {}
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_require_cache(expr) {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          RuntimeGlobals::MODULE_CACHE.name().into(),
          Some(RuntimeGlobals::MODULE_CACHE),
        )));
    } else if expr_matcher::is_webpack_module_id(expr) {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          expr.span().real_lo(),
          expr.span().real_hi(),
          "module.id".into(), // todo module_arguments
          Some(RuntimeGlobals::MODULE_ID),
        )));
    }
    expr.visit_children_with(self);
  }
}
