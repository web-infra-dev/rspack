use rspack_core::{
  BuildMeta, BuildMetaDefaultObject, BuildMetaExportsType, DependencyTemplate, ModuleType,
  RuntimeGlobals,
};
use swc_core::{
  common::SyntaxContext,
  ecma::{
    ast::{Expr, Ident, ModuleItem, Program},
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::expr_matcher;
use crate::dependency::ModuleDecoratorDependency;

pub struct CommonJsExportDependencyScanner<'a> {
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  unresolved_ctxt: &'a SyntaxContext,
  build_meta: &'a mut BuildMeta,
  module_type: ModuleType,
  is_harmony: bool,
}

impl<'a> CommonJsExportDependencyScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: &'a SyntaxContext,
    build_meta: &'a mut BuildMeta,
    module_type: ModuleType,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      build_meta,
      module_type,
      is_harmony: false,
    }
  }
}

impl Visit for CommonJsExportDependencyScanner<'_> {
  noop_visit_type!();

  fn visit_program(&mut self, program: &Program) {
    self.is_harmony = matches!(self.module_type, ModuleType::JsEsm | ModuleType::JsxEsm)
      || matches!(program, Program::Module(module) if module.body.iter().any(|s| matches!(s, ModuleItem::ModuleDecl(_))));
    program.visit_children_with(self);
  }

  fn visit_ident(&mut self, ident: &Ident) {
    if &ident.sym == "module" && ident.span.ctxt == *self.unresolved_ctxt {
      // here should use, but scanner is not one pass, so here use extra `visit_program` to calculate is_harmony
      // matches!( self.build_meta.exports_type, BuildMetaExportsType::Namespace)
      let decorator = if self.is_harmony {
        RuntimeGlobals::HARMONY_MODULE_DECORATOR
      } else {
        RuntimeGlobals::NODE_MODULE_DECORATOR
      };
      self
        .presentational_dependencies
        .push(Box::new(ModuleDecoratorDependency::new(decorator)));
      bailout(self.build_meta);
    }
  }

  fn visit_expr(&mut self, expr: &Expr) {
    if expr_matcher::is_module_id(expr)
      || expr_matcher::is_module_loaded(expr)
      || expr_matcher::is_module_hot(expr)
      || expr_matcher::is_module_hot_accept(expr)
      || expr_matcher::is_module_hot_decline(expr)
      || (!self.is_harmony && expr_matcher::is_module_exports(expr))
    {
      return;
    }
    expr.visit_children_with(self);
  }
}

fn bailout(build_meta: &mut BuildMeta) {
  build_meta.exports_type = BuildMetaExportsType::Unset;
  build_meta.default_object = BuildMetaDefaultObject::False;
}
