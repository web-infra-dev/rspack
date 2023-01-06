use rspack_core::{Compilation, Dependency, DependencyType, Module, ModuleDependency};

use crate::utils::is_require_literal_expr;

use super::format::SWC_HELPERS_REG;

use {
  swc_core::common::{Mark, SyntaxContext},
  swc_core::ecma::ast::*,
  swc_core::ecma::atoms::{Atom, JsWord},
  swc_core::ecma::visit::{noop_visit_mut_type, VisitMut, VisitMutWith},
};

/// Only rewrite `require('xxx')` and `import _ from 'xxx'` now
pub struct RewriteModuleUrl<'a> {
  module: &'a dyn Module,
  compilation: &'a Compilation,
  unresolved_ctxt: SyntaxContext,
}

impl<'a> RewriteModuleUrl<'a> {
  pub fn new(unresolved_mark: Mark, module: &'a dyn Module, bundle: &'a Compilation) -> Self {
    Self {
      unresolved_ctxt: SyntaxContext::empty().apply_mark(unresolved_mark),
      module,
      compilation: bundle,
    }
  }

  fn get_module(
    &self,
    specifier: String,
    _span: swc_core::common::Span,
    dependency_type: DependencyType,
  ) -> Option<&rspack_core::ModuleGraphModule> {
    self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(&self.module.identifier())
      .and_then(|mgm| {
        mgm.dependencies.iter().find_map(|dep| {
          if dep.request() == specifier && dep.dependency_type() == &dependency_type {
            self.compilation.module_graph.module_by_dependency(dep)
          } else {
            None
          }
        })
      })
  }

  fn get_import_module(
    &self,
    specifier: String,
    span: swc_core::common::Span,
  ) -> Option<&rspack_core::ModuleGraphModule> {
    self.get_module(specifier, span, DependencyType::EsmImport)
  }
}

impl<'a> VisitMut for RewriteModuleUrl<'a> {
  noop_visit_mut_type!();

  fn visit_mut_module_decl(&mut self, n: &mut ModuleDecl) {
    match n {
      ModuleDecl::Import(n) => {
        let specifier = n.src.value.to_string();
        if let Some(module) = self.get_import_module(specifier, n.span) {
          let module_id = module.id(&self.compilation.chunk_graph);
          n.src.value = JsWord::from(module_id);
          n.src.raw = Some(Atom::from(format!("\"{}\"", module_id)));
        }
      }
      ModuleDecl::ExportNamed(n) => {
        if let Some(src) = n.src.as_mut() {
          let specifier = src.value.to_string();
          if let Some(module) = self.get_import_module(specifier, n.span) {
            let module_id = module.id(&self.compilation.chunk_graph);
            src.value = JsWord::from(module_id);
            src.raw = Some(Atom::from(format!("\"{}\"", module_id)));
          }
        }
      }
      ModuleDecl::ExportAll(n) => {
        let specifier = n.src.value.to_string();
        if let Some(module) = self.get_import_module(specifier, n.span) {
          let module_id = module.id(&self.compilation.chunk_graph);
          n.src.value = JsWord::from(module_id);
          n.src.raw = Some(Atom::from(format!("\"{}\"", module_id)));
        }
      }
      _ => (),
    }
    n.visit_mut_children_with(self);
  }

  fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if is_require_literal_expr(n, &self.unresolved_ctxt) {
      if let Callee::Expr(box Expr::Ident(_ident)) = &mut n.callee {
        if let Some(ExprOrSpread {
          spread: None,
          expr: box Expr::Lit(Lit::Str(str)),
        }) = n.args.first_mut()
        {
          // swc will automatically replace @swc/helpers/src/xx.mjs with @swc/helpers/lib/xx.js when it transform code to commonjs
          // so we need replace it to original specifier to find module
          // this is a temporary solution
          let specifier = match SWC_HELPERS_REG.captures(&str.value) {
            Some(cap) => match cap.get(1) {
              Some(cap) => format!(r#"@swc/helpers/src/{}.mjs"#, cap.as_str()),
              None => str.value.to_string(),
            },
            None => str.value.to_string(),
          };

          if let Some(js_module) = self.get_module(specifier, n.span, DependencyType::CjsRequire) {
            let module_id = js_module.id(&self.compilation.chunk_graph);
            str.value = JsWord::from(module_id);
            str.raw = Some(Atom::from(format!("\"{}\"", module_id)));
          }
        };
      }
    }

    n.visit_mut_children_with(self);
  }
}
