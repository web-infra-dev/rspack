use linked_hash_set::LinkedHashSet;
use rspack_core::{ModuleDependency, ResolveKind};
use swc_atoms::JsWord;
use swc_common::Span;
use swc_ecma_ast::{CallExpr, Callee, ExportSpecifier, Expr, ExprOrSpread, Lit, ModuleDecl};
use swc_ecma_visit::{noop_visit_type, VisitAll, VisitAllWith};

#[derive(Default)]
pub struct DependencyScanner {
  pub dependencies: LinkedHashSet<ModuleDependency>,
  // pub dyn_dependencies: HashSet<DynImportDesc>,
}

impl DependencyScanner {
  fn add_dependency(&mut self, specifier: JsWord, kind: ResolveKind, span: Span) {
    self.dependencies.insert_if_absent(ModuleDependency {
      specifier: specifier.to_string(),
      kind,
      span: Some(span.into()),
    });
  }

  fn add_import(&mut self, module_decl: &ModuleDecl) {
    if let ModuleDecl::Import(import_decl) = module_decl {
      let source = import_decl.src.value.clone();
      self.add_dependency(source, ResolveKind::Import, import_decl.span);
    }
  }
  fn add_require(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
        // TODO: This might not be correct.
        // Consider what if user overwirte `require` function.
        if "require".eq(&ident.sym) {
          {
            if call_expr.args.len() != 1 {
              return;
            }
            let src = match call_expr.args.first().unwrap() {
              ExprOrSpread { spread: None, expr } => match &**expr {
                Expr::Lit(Lit::Str(s)) => s,
                _ => return,
              },
              _ => return,
            };
            let source = &src.value;
            self.add_dependency(source.clone(), ResolveKind::Require, call_expr.span);
          }
        }
      }
    }
  }
  fn add_dynamic_import(&mut self, node: &CallExpr) {
    if let Callee::Import(_) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          if let Expr::Lit(Lit::Str(imported)) = dyn_imported.expr.as_ref() {
            self.add_dependency(
              imported.value.clone(),
              ResolveKind::DynamicImport,
              node.span,
            );
          }
        }
      }
    }
  }

  fn add_export(&mut self, module_decl: &ModuleDecl) -> Result<(), anyhow::Error> {
    match module_decl {
      ModuleDecl::ExportNamed(node) => {
        node.specifiers.iter().for_each(|specifier| {
          match specifier {
            ExportSpecifier::Named(_s) => {
              if let Some(source_node) = &node.src {
                // export { name } from './other'
                let source = source_node.value.clone();
                self.add_dependency(source, ResolveKind::Import, node.span);
              }
            }
            ExportSpecifier::Namespace(_s) => {
              // export * as name from './other'
              let source = node.src.as_ref().map(|str| str.value.clone()).unwrap();
              self.add_dependency(source, ResolveKind::Import, node.span);
            }
            ExportSpecifier::Default(_) => {
              // export v from 'mod';
              // Rollup doesn't support it.
            }
          };
        });
      }
      ModuleDecl::ExportAll(node) => {
        // export * from './other'
        self.add_dependency(node.src.value.clone(), ResolveKind::Import, node.span);
      }
      _ => {}
    }
    Ok(())
  }
}

impl VisitAll for DependencyScanner {
  noop_visit_type!();

  fn visit_module_decl(&mut self, node: &ModuleDecl) {
    self.add_import(node);
    if let Err(e) = self.add_export(node) {
      eprintln!("{}", e);
    }
    node.visit_all_children_with(self);
  }
  fn visit_call_expr(&mut self, node: &CallExpr) {
    self.add_dynamic_import(node);
    self.add_require(node);
    node.visit_all_children_with(self);
  }
}
