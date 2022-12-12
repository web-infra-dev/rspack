use rspack_core::{Compilation, Module};
use {
  swc_core::common::{Mark, SyntaxContext, DUMMY_SP},
  swc_core::ecma::ast::{self, *},
  swc_core::ecma::utils::quote_str,
  swc_core::ecma::visit::{noop_fold_type, Fold, FoldWith},
};

pub struct ModuleVariables<'a> {
  pub module: &'a dyn Module,
  pub unresolved_ctxt: SyntaxContext,
  pub compilation: &'a Compilation,
  // store module variables flag
  pub has_resource_query: bool,
}

impl<'a> ModuleVariables<'a> {
  pub fn new(module: &'a dyn Module, unresolved_mark: Mark, compilation: &'a Compilation) -> Self {
    Self {
      module,
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      compilation,
      has_resource_query: false,
    }
  }
}

impl<'a> Fold for ModuleVariables<'a> {
  noop_fold_type!();

  fn fold_module(&mut self, mut module: ast::Module) -> ast::Module {
    module.body = module.body.fold_children_with(self);

    if self.has_resource_query {
      if let Some(normal_module) = self.module.as_normal_module() {
        if let Some(resource_query) = &normal_module.resource_resolved_data().resource_query {
          module.body.insert(
            0,
            new_var_decl(
              "__resourceQuery",
              resource_query.clone(),
              self.unresolved_ctxt,
            )
            .into(),
          );
        }
      }
    }

    module
  }

  fn fold_ident(&mut self, ident: Ident) -> Ident {
    if "__resourceQuery".eq(&ident.sym) && ident.span.ctxt == self.unresolved_ctxt {
      self.has_resource_query = true;
    }
    ident
  }
}

#[inline]
fn new_var_decl(name: &str, value: String, unresolved_ctxt: SyntaxContext) -> VarDecl {
  VarDecl {
    span: DUMMY_SP,
    kind: VarDeclKind::Var,
    decls: vec![VarDeclarator {
      span: DUMMY_SP,
      name: Pat::Ident(BindingIdent::from(Ident::new(
        name.into(),
        DUMMY_SP.with_ctxt(unresolved_ctxt),
      ))),
      init: Some(Box::new(Expr::Lit(Lit::Str(quote_str!(value))))),
      definite: false,
    }],
    declare: false,
  }
}
