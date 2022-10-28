use crate::runtime::RSPACK_RUNTIME;
use hashbrown::HashSet;
use swc_common::DUMMY_SP;
use swc_ecma_ast::*;
// use swc_ecma_utils::
use rspack_symbol::{BetterId, Symbol};
use swc_ecma_visit::{as_folder, noop_fold_type, noop_visit_mut_type, Fold, VisitMut};
use ustr::Ustr;

pub fn tree_shaker<'a>(module_id: Ustr, used_symbol_set: &'a HashSet<Symbol>) -> impl Fold + 'a {
  TreeShaker {
    module_id,
    used_symbol_set,
  }
}

struct TreeShaker<'a> {
  pub(crate) module_id: Ustr,
  used_symbol_set: &'a HashSet<Symbol>,
}

impl<'a> Fold for TreeShaker<'a> {
  noop_fold_type!();
  fn fold_module_item(&mut self, node: ModuleItem) -> ModuleItem {
    match node {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::Import(_) => ModuleItem::ModuleDecl(module_decl),
        ModuleDecl::ExportDecl(decl) => match decl.decl {
          Decl::Class(mut class) => {
            let id = class.ident.to_id();
            let symbol = Symbol::from_id_and_uri(id.into(), self.module_id);
            if !self.used_symbol_set.contains(&symbol) {
              class.class.span = DUMMY_SP;
              ModuleItem::Stmt(Stmt::Decl(Decl::Class(class)))
            } else {
              ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                span: decl.span,
                decl: Decl::Class(class),
              }))
            }
          }
          Decl::Fn(mut func) => {
            let id = func.ident.to_id();
            let symbol = Symbol::from_id_and_uri(id.into(), self.module_id);
            if !self.used_symbol_set.contains(&symbol) {
              func.function.span = DUMMY_SP;
              ModuleItem::Stmt(Stmt::Decl(Decl::Fn(func)))
            } else {
              ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                span: decl.span,
                decl: Decl::Fn(func),
              }))
            }
          }
          Decl::Var(var) => {
            // TODO: for simplicity don't concern var
            let used = var
              .decls
              .into_iter()
              .map(|decl| match decl.name {
                Pat::Ident(ident) => {
                  let id: BetterId = ident.to_id().into();
                  let symbol = Symbol::from_id_and_uri(id, self.module_id);
                  (
                    VarDeclarator {
                      span: decl.span,
                      name: Pat::Ident(ident),
                      init: decl.init,
                      definite: decl.definite,
                    },
                    self.used_symbol_set.contains(&symbol),
                  )
                }
                Pat::Array(_) => (decl, true),
                Pat::Rest(_) => (decl, true),
                Pat::Object(_) => (decl, true),
                Pat::Assign(_) => (decl, true),
                Pat::Invalid(_) => (decl, true),
                Pat::Expr(_) => (decl, true),
              })
              .filter(|item| item.1)
              .collect::<Vec<_>>();
            if used.len() == 0 {
              ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
            } else {
              ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(ExportDecl {
                span: DUMMY_SP,
                decl: Decl::Var(Box::new(VarDecl {
                  span: var.span,
                  kind: var.kind,
                  declare: var.declare,
                  decls: used.into_iter().map(|item| item.0).collect(),
                })),
              }))
            }
          }
          Decl::TsInterface(_) => todo!(),
          Decl::TsTypeAlias(_) => todo!(),
          Decl::TsEnum(_) => todo!(),
          Decl::TsModule(_) => todo!(),
        },
        ModuleDecl::ExportNamed(_) => {
          // TODO: TODO!
          ModuleItem::ModuleDecl(module_decl)
        }
        ModuleDecl::ExportDefaultDecl(_) => {
          // TODO: TODO!
          ModuleItem::ModuleDecl(module_decl)
        }
        ModuleDecl::ExportDefaultExpr(_) => {
          // TODO: TODO!
          ModuleItem::ModuleDecl(module_decl)
        }
        ModuleDecl::ExportAll(_) => ModuleItem::ModuleDecl(module_decl),
        ModuleDecl::TsImportEquals(_) => ModuleItem::ModuleDecl(module_decl),
        ModuleDecl::TsExportAssignment(_) => ModuleItem::ModuleDecl(module_decl),
        ModuleDecl::TsNamespaceExport(_) => ModuleItem::ModuleDecl(module_decl),
      },
      ModuleItem::Stmt(_) => node,
    }
  }
}
