use std::collections::HashSet;

use linked_hash_map::LinkedHashMap;
use swc_atoms::JsWord;
use swc_ecma_visit::{VisitMut, VisitMutWith};

use crate::structs::DynImportDesc;

#[derive(Default)]
pub struct DependencyScanner {
  pub dependencies: LinkedHashMap<JsWord, ()>,
  pub dyn_dependencies: HashSet<DynImportDesc>,
}

impl DependencyScanner {}

use swc_ecma_ast::{
  CallExpr, Callee, ExportSpecifier, Expr, ExprOrSpread, Ident, ImportDecl, Lit, ModuleDecl,
};

impl DependencyScanner {
  pub fn add_import(&mut self, module_decl: &mut ModuleDecl) {
    if let ModuleDecl::Import(import_decl) = module_decl {
      let source = &import_decl.src.value;
      self.dependencies.entry(source.clone()).or_insert(());
    }
  }
  pub fn add_require(&mut self, call_expr: &CallExpr) {
    if let Callee::Expr(expr) = &call_expr.callee {
      if let Expr::Ident(ident) = &**expr {
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
            self.dependencies.entry(source.clone()).or_insert(());
          }
        }
      }
    }
  }
  pub fn add_dynamic_import(&mut self, node: &CallExpr) {
    if let Callee::Import(_) = node.callee {
      if let Some(dyn_imported) = node.args.get(0) {
        if dyn_imported.spread.is_none() {
          if let Expr::Lit(Lit::Str(imported)) = dyn_imported.expr.as_ref() {
            self.dyn_dependencies.insert(DynImportDesc {
              argument: imported.value.clone(),
            });
          }
        }
      }
    }
  }

  pub fn add_export(&mut self, module_decl: &ModuleDecl) -> Result<(), anyhow::Error> {
    match module_decl {
      ModuleDecl::ExportNamed(node) => {
        node.specifiers.iter().for_each(|specifier| {
          match specifier {
            ExportSpecifier::Named(_s) => {
              if let Some(source_node) = &node.src {
                // export { name } from './other'
                let source = source_node.value.clone();
                self.dependencies.entry(source).or_insert(());
              }
            }
            ExportSpecifier::Namespace(_s) => {
              // export * as name from './other'
              let source = node.src.as_ref().map(|str| str.value.clone()).unwrap();
              self.dependencies.entry(source).or_insert(());
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
        self
          .dependencies
          .entry(node.src.value.clone())
          .or_insert(());
      }
      _ => {}
    }
    Ok(())
  }
}

impl VisitMut for DependencyScanner {
  fn visit_mut_module_decl(&mut self, node: &mut ModuleDecl) {
    self.add_import(node);
    if let Err(e) = self.add_export(node) {
      eprintln!("{}", e);
    }
    node.visit_mut_children_with(self);
  }
  fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
    self.add_dynamic_import(node);
    self.add_require(node);
    node.visit_mut_children_with(self);
  }
}
