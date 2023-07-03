use rspack_core::{
  CodeGeneratableDependency, ConstDependency, ResourceData, RuntimeGlobals, SpanExt,
};
use rspack_plugin_javascript_shared::JsAstVisitorHook;
use swc_core::{
  common::{Spanned, SyntaxContext},
  ecma::ast::{Expr, Ident},
};

use crate::visitors::expr_matcher;

pub const WEBPACK_HASH: &str = "__webpack_hash__";
pub const WEBPACK_PUBLIC_PATH: &str = "__webpack_public_path__";
pub const WEBPACK_MODULES: &str = "__webpack_modules__";
pub const WEBPACK_RESOURCE_QUERY: &str = "__resourceQuery";
pub const WEBPACK_CHUNK_LOAD: &str = "__webpack_chunk_load__";

pub struct ApiScanner<'a> {
  pub unresolved_ctxt: &'a SyntaxContext,
  pub resource_data: &'a ResourceData,
  pub presentational_dependencies: Vec<Box<dyn CodeGeneratableDependency>>,
}

impl<'a> ApiScanner<'a> {
  pub fn new(unresolved_ctxt: &'a SyntaxContext, resource_data: &'a ResourceData) -> Self {
    Self {
      unresolved_ctxt,
      resource_data,
      presentational_dependencies: Default::default(),
    }
  }

  fn scan_ident(&mut self, ident: &Ident) {
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
}

impl JsAstVisitorHook for ApiScanner<'_> {
  fn visit_expr(&mut self, expr: &Expr) -> bool {
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
    } else if let Expr::Ident(node) = expr {
      self.scan_ident(node)
    }

    false
  }
}
