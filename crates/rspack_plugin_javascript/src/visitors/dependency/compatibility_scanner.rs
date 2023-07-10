use rspack_core::{CodeGeneratableDependency, ConstDependency, SpanExt};
use rustc_hash::FxHashMap;
use swc_core::common::SyntaxContext;
use swc_core::ecma::ast::{FnDecl, Ident};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

pub struct CompatibilityScanner<'a> {
  unresolved_ctxt: &'a SyntaxContext,
  presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
  count: u8, // flag __webpack_require__ count
  name_map: FxHashMap<SyntaxContext, u8>,
}

impl<'a> CompatibilityScanner<'a> {
  pub fn new(
    presentational_dependencies: &'a mut Vec<Box<dyn CodeGeneratableDependency>>,
    unresolved_ctxt: &'a SyntaxContext,
  ) -> Self {
    Self {
      presentational_dependencies,
      unresolved_ctxt,
      count: 0,
      name_map: Default::default(),
    }
  }
}

impl Visit for CompatibilityScanner<'_> {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    if &ident.sym == "__webpack_require__" && ident.span.ctxt != *self.unresolved_ctxt {
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

  fn visit_fn_decl(&mut self, fn_decl: &FnDecl) {
    if &fn_decl.ident.sym == "__webpack_require__"
      && fn_decl.ident.span.ctxt != *self.unresolved_ctxt
      && !self.name_map.contains_key(&fn_decl.ident.span.ctxt)
    {
      self.count += 1;
      self.name_map.insert(fn_decl.ident.span.ctxt, self.count);
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          fn_decl.ident.span.real_lo(),
          fn_decl.ident.span.real_hi(),
          format!("__nested_webpack_require_{}__", self.count).into(),
          None,
        )));
    }
    fn_decl.function.visit_children_with(self);
  }
}
