use hashbrown::HashSet;
use swc_common::{Mark, DUMMY_SP, GLOBALS};
use swc_ecma_ast::*;
// use swc_ecma_utils::
use rspack_symbol::{BetterId, Symbol};
use swc_atoms::JsWord;
use swc_ecma_utils::quote_ident;
use swc_ecma_visit::{noop_fold_type, Fold, FoldWith};
use ustr::Ustr;
pub fn tree_shaking_visitor(
  module_id: Ustr,
  used_symbol_set: &'_ HashSet<Symbol>,
  top_level_mark: Mark,
) -> impl Fold + '_ {
  TreeShaker {
    module_id,
    used_symbol_set,
    top_level_mark,
  }
}

/// The basic idea of shaking the tree is pretty easy,
/// we visit each export symbol, if the symbol is marked as used in the tree-shaking analysis phase,
/// we keep it as is. Otherwise, we remove the export related reserved word. e.g.
/// ```js
/// export function test() {}
/// ```
/// if the function `test` is never used in other module, remove the `export`, it become :
/// ```js
/// function test() {}
/// ```
/// if function `test` is also unused in local module, then it will be removed in DCE phase of `swc`
struct TreeShaker<'a> {
  module_id: Ustr,
  used_symbol_set: &'a HashSet<Symbol>,
  top_level_mark: Mark,
}

impl<'a> Fold for TreeShaker<'a> {
  noop_fold_type!();
  fn fold_program(&mut self, node: Program) -> Program {
    debug_assert!(GLOBALS.is_set());
    node.fold_with(self)
  }
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
                Pat::Array(_)
                | Pat::Rest(_)
                | Pat::Object(_)
                | Pat::Assign(_)
                | Pat::Invalid(_)
                | Pat::Expr(_) => (decl, true),
              })
              .filter(|item| item.1)
              .collect::<Vec<_>>();
            if used.is_empty() {
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
        ModuleDecl::ExportNamed(mut named) => {
          if named.src.is_some() {
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named))
          } else {
            let before_legnth = named.specifiers.len();
            let specifiers = named
              .specifiers
              .into_iter()
              .filter(|specifier| match specifier {
                ExportSpecifier::Namespace(_) => {
                  // named_export has namespace specifier but no src will trigger a syntax error and should not reach here. e.g.
                  // `export *`;
                  unreachable!("")
                }
                ExportSpecifier::Default(_) => {
                  // `export v`; is a unrecoverable syntax error, code should not reach here.
                  unreachable!("")
                }

                ExportSpecifier::Named(named_spec) => match named_spec.orig {
                  ModuleExportName::Ident(ref ident) => {
                    let id: BetterId = ident.to_id().into();
                    let symbol = Symbol::from_id_and_uri(id, self.module_id);
                    self.used_symbol_set.contains(&symbol)
                  }
                  ModuleExportName::Str(_) => {
                    // named export without src has string lit orig is a syntax error
                    // `export { "something" }`
                    unreachable!("")
                  }
                },
              })
              .collect::<Vec<_>>();
            let is_all_used = before_legnth != specifiers.len();
            named.specifiers = specifiers;
            if !is_all_used {
              named.span = DUMMY_SP;
            }
            ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named))
          }
        }
        ModuleDecl::ExportDefaultDecl(decl) => {
          let default_symbol = self.crate_virtual_default_symbol();
          let ctxt = default_symbol.id().ctxt;
          if self.used_symbol_set.contains(&default_symbol) {
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(decl))
          } else {
            let decl = match decl.decl {
              DefaultDecl::Class(class) => {
                let ident = if let Some(ident) = class.ident {
                  ident
                } else {
                  let mut named = quote_ident!("__RSPACK_DEFAULT_EXPORT__");
                  named.span = named.span.with_ctxt(ctxt);
                  named
                };
                Decl::Class(ClassDecl {
                  ident,
                  declare: false,
                  class: class.class,
                })
              }
              DefaultDecl::Fn(func) => {
                let ident = if let Some(ident) = func.ident {
                  ident
                } else {
                  let mut named = quote_ident!("__RSPACK_DEFAULT_EXPORT__");
                  named.span = named.span.with_ctxt(ctxt);
                  named
                };
                Decl::Fn(FnDecl {
                  ident,
                  declare: false,
                  function: func.function,
                })
              }
              DefaultDecl::TsInterfaceDecl(_) => todo!(),
            };
            ModuleItem::Stmt(Stmt::Decl(decl))
          }
        }
        ModuleDecl::ExportDefaultExpr(expr) => {
          let default_symbol = self.crate_virtual_default_symbol();
          if self.used_symbol_set.contains(&default_symbol) {
            ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(expr))
          } else {
            // convert the original expr to
            // var __RSPACK_DEFAULT_EXPORT__ = ${expr}
            ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
              span: DUMMY_SP,
              kind: VarDeclKind::Let,
              declare: false,
              decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: Pat::Ident(BindingIdent {
                  id: Ident {
                    span: DUMMY_SP,
                    sym: JsWord::from("__RSPACK_DEFAULT_EXPORT__"),
                    optional: false,
                  },
                  type_ann: None,
                }),
                init: Some(expr.expr),
                definite: false,
              }],
            }))))
          }
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

impl<'a> TreeShaker<'a> {
  fn crate_virtual_default_symbol(&self) -> Symbol {
    let mut default_ident = quote_ident!("default");
    default_ident.span = default_ident.span.apply_mark(self.top_level_mark);
    Symbol::from_id_and_uri(default_ident.to_id().into(), self.module_id)
  }
}
