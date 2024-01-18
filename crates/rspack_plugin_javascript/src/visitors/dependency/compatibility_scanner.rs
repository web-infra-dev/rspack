use rspack_core::{ConstDependency, DependencyLocation, DependencyTemplate, SpanExt};
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{FnDecl, Ident, Program};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use crate::{no_visit_ignored_expr, no_visit_ignored_stmt};

pub struct CompatibilityScanner<'a> {
  unresolved_ctxt: SyntaxContext,
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  count: u8, // flag __webpack_require__ count
  name_map: FxHashMap<SyntaxContext, u8>,
  ignored: &'a mut FxHashSet<DependencyLocation>,
}

impl<'a> CompatibilityScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
    unresolved_ctxt: SyntaxContext,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      count: 0,
      name_map: Default::default(),
      ignored,
    }
  }
}

impl Visit for CompatibilityScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();
  no_visit_ignored_expr!();

  fn visit_program(&mut self, program: &Program) {
    program.visit_children_with(self);

    program.visit_children_with(&mut ReplaceNestWebpackRequireVisitor {
      unresolved_ctxt: self.unresolved_ctxt,
      name_map: &self.name_map,
      presentational_dependencies: self.presentational_dependencies,
      ignored: self.ignored,
    });
  }

  fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
    if &fn_decl.ident.sym == "__webpack_require__"
      && fn_decl.ident.span.ctxt != self.unresolved_ctxt
      && !self.name_map.contains_key(&fn_decl.ident.span.ctxt)
    {
      self.count += 1;
      self.name_map.insert(fn_decl.ident.span.ctxt, self.count);
    }
  }
}

pub struct ReplaceNestWebpackRequireVisitor<'a> {
  unresolved_ctxt: SyntaxContext,
  name_map: &'a FxHashMap<SyntaxContext, u8>,
  presentational_dependencies: &'a mut Vec<Box<dyn DependencyTemplate>>,
  ignored: &'a mut FxHashSet<DependencyLocation>,
}

impl Visit for ReplaceNestWebpackRequireVisitor<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();
  no_visit_ignored_expr!();

  fn visit_ident(&mut self, ident: &Ident) {
    if &ident.sym == "__webpack_require__" && ident.span.ctxt != self.unresolved_ctxt {
      if let Some(count) = self.name_map.get(&ident.span.ctxt) {
        self
          .presentational_dependencies
          .push(Box::new(ConstDependency::new(
            ident.span.real_lo(),
            ident.span.real_hi(),
            format!("__nested_webpack_require_{count}__").into(),
            None,
          )));
      }
    }
  }
}
