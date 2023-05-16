use std::sync::Arc;

use rspack_core::tree_shaking::visitor::SymbolRef;
use rspack_core::{
  CodeGeneratableDeclMappings, CompilerOptions, DependencyCategory, DependencyType, ModuleGraph,
  ModuleIdentifier,
};
use rspack_identifier::{Identifier, IdentifierMap, IdentifierSet};
use rspack_symbol::{BetterId, IndirectTopLevelSymbol, Symbol, SymbolType};
use rustc_hash::FxHashSet as HashSet;
use swc_core::common::{Mark, DUMMY_SP, GLOBALS};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::utils::quote_ident;
use swc_core::ecma::visit::{noop_fold_type, Fold, FoldWith};
#[allow(clippy::too_many_arguments)]
pub fn tree_shaking_visitor<'a>(
  decl_mappings: &'a CodeGeneratableDeclMappings,
  module_graph: &'a ModuleGraph,
  module_id: Identifier,
  used_symbol_set: &'a HashSet<SymbolRef>,
  side_effects_free_modules: &'a IdentifierSet,
  module_item_map: &'a IdentifierMap<Vec<ModuleItem>>,
  extra_mark: MarkInfo,
  include_module_ids: &'a IdentifierSet,
  _options: Arc<CompilerOptions>,
) -> impl Fold + 'a {
  TreeShaker {
    module_graph,
    decl_mappings,
    module_identifier: module_id,
    used_symbol_set,
    module_item_index: 0,
    insert_item_tuple_list: Vec::new(),
    side_effects_free_modules,
    last_module_item_index: 0,
    module_item_map,
    extra_mark,
    include_module_ids,
  }
}

pub struct MarkInfo {
  pub top_level_mark: Mark,
  pub helper_mark: Mark,
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
  module_graph: &'a ModuleGraph,
  decl_mappings: &'a CodeGeneratableDeclMappings,
  module_identifier: Identifier,
  used_symbol_set: &'a HashSet<SymbolRef>,
  /// First element of tuple is the position of body you want to insert with, the second element is the item you want to insert
  insert_item_tuple_list: Vec<(usize, ModuleItem)>,
  module_item_index: usize,
  side_effects_free_modules: &'a IdentifierSet,
  last_module_item_index: usize,
  module_item_map: &'a IdentifierMap<Vec<ModuleItem>>,
  extra_mark: MarkInfo,
  include_module_ids: &'a IdentifierSet,
}

impl<'a> Fold for TreeShaker<'a> {
  noop_fold_type!();
  fn fold_program(&mut self, node: Program) -> Program {
    debug_assert!(GLOBALS.is_set());
    node.fold_children_with(self)
  }

  fn fold_module(&mut self, mut node: Module) -> Module {
    node.body = node
      .body
      .into_iter()
      .enumerate()
      .map(|(index, item)| {
        self.module_item_index = index;
        if !matches!(item, ModuleItem::Stmt(_)) {
          self.last_module_item_index = index;
        }
        item.fold_with(self)
      })
      .collect();
    for (position, module_item) in std::mem::take(&mut self.insert_item_tuple_list)
      .into_iter()
      .rev()
    {
      node.body.insert(position, module_item);
    }

    if let Some(occ) = self.module_item_map.get(&self.module_identifier) {
      for module_item in occ {
        node
          .body
          .insert(self.last_module_item_index, module_item.clone());
      }
    }

    node
  }

  fn fold_module_item(&mut self, node: ModuleItem) -> ModuleItem {
    match node {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::Import(import) => self.custom_fold_import_decl(import),
        ModuleDecl::ExportDecl(decl) => self.custom_fold_export_decl(decl),
        ModuleDecl::ExportNamed(named) => self.custom_fold_named_export(named),
        ModuleDecl::ExportDefaultDecl(decl) => self.custom_fold_export_default_decl(decl),
        ModuleDecl::ExportDefaultExpr(expr) => self.custom_fold_export_default_expr(expr),
        ModuleDecl::ExportAll(export_all) => self.custom_fold_export_all(export_all),
        ModuleDecl::TsImportEquals(_)
        | ModuleDecl::TsExportAssignment(_)
        | ModuleDecl::TsNamespaceExport(_) => ModuleItem::ModuleDecl(module_decl),
      },
      ModuleItem::Stmt(_) => node,
    }
  }
}

impl<'a> TreeShaker<'a> {
  fn crate_virtual_default_symbol(&self) -> Symbol {
    let mut default_ident = quote_ident!("default");
    default_ident.span = default_ident
      .span
      .apply_mark(self.extra_mark.top_level_mark);
    Symbol::new(
      self.module_identifier,
      default_ident.to_id().into(),
      SymbolType::Define,
    )
  }

  fn create_empty_stmt_module_item() -> ModuleItem {
    ModuleItem::Stmt(Stmt::Empty(EmptyStmt { span: DUMMY_SP }))
  }

  fn custom_fold_import_decl(&mut self, mut import: ImportDecl) -> ModuleItem {
    let module_identifier = self
    .resolve_module_identifier(import.src.value.to_string())
    .unwrap_or_else(|| {
      panic!(
        "Failed to resolve dependency where `parent_module_identifier` is {:?}, `request` is {:?} and `dependency_type` is {:?}",
        self.module_identifier,
        import.src.value.to_string(),
        DependencyType::EsmImport
      )
    });
    let mgm = self
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .expect("TODO:");

    if !self.include_module_ids.contains(&mgm.module_identifier) {
      return Self::create_empty_stmt_module_item();
    }
    // return ModuleItem::ModuleDecl(ModuleDecl::Import(import));
    let before_length = import.specifiers.len();
    let specifiers = import
      .specifiers
      .into_iter()
      .filter(|specifier| {
        if !mgm.module_type.is_js_like() {
          return true;
        }
        match specifier {
          ImportSpecifier::Namespace(_) => {
            // import * as xxx  from 'xxx'
            // TODO:
            true
          }
          ImportSpecifier::Default(default) => {
            if default.local.to_id().1.outer() == self.extra_mark.helper_mark {
              return true;
            }
            let symbol = SymbolRef::Indirect(IndirectTopLevelSymbol {
              src: module_identifier,
              ty: rspack_symbol::IndirectType::ImportDefault(default.local.sym.clone()),
              importer: self.module_identifier,
            });
            self.used_symbol_set.contains(&symbol)
            // unreachable!("`export v from ''` is a unrecoverable syntax error")
          }

          ImportSpecifier::Named(named_import) => {
            let local = named_import.local.sym.clone();
            let imported = named_import
              .imported
              .as_ref()
              .map(|exported| match exported {
                ModuleExportName::Ident(ident) => ident.sym.clone(),
                ModuleExportName::Str(str) => str.value.clone(),
              });
            let symbol = SymbolRef::Indirect(IndirectTopLevelSymbol {
              src: module_identifier,
              ty: rspack_symbol::IndirectType::Import(local, imported),
              importer: self.module_identifier,
            });

            self.used_symbol_set.contains(&symbol)
          }
        }
      })
      .collect::<Vec<_>>();

    // try if we could remove this export declaration
    if specifiers.is_empty() && self.side_effects_free_modules.contains(&module_identifier) {
      return Self::create_empty_stmt_module_item();
    }
    let is_all_used = before_length == specifiers.len();
    import.specifiers = specifiers;
    if !is_all_used {
      import.span = DUMMY_SP;
    }
    ModuleItem::ModuleDecl(ModuleDecl::Import(import))
  }

  fn custom_fold_export_decl(&mut self, decl: ExportDecl) -> ModuleItem {
    match decl.decl {
      Decl::Class(mut class) => {
        let id = class.ident.to_id();
        let symbol = SymbolRef::Direct(Symbol::new(
          self.module_identifier,
          id.into(),
          SymbolType::Define,
        ));
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
        let symbol = SymbolRef::Direct(Symbol::new(
          self.module_identifier,
          id.into(),
          SymbolType::Define,
        ));
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
        // assume a is used and b, c is unused
        // Convert
        // ```js
        // export const a = 100, b = 1, c = 3;
        // ```
        // To
        // ```js
        // export const a = 100;
        // const b = 1, c = 3;
        // ```
        // swc dce will drop `b`, and `c`
        let (used, unused): (Vec<_>, Vec<_>) = var
          .decls
          .into_iter()
          .map(|decl| match decl.name {
            Pat::Ident(ident) => {
              let id: BetterId = ident.to_id().into();
              let symbol =
                SymbolRef::Direct(Symbol::new(self.module_identifier, id, SymbolType::Define));
              let used = self.used_symbol_set.contains(&symbol);
              (
                VarDeclarator {
                  span: decl.span,
                  name: Pat::Ident(ident),
                  init: decl.init,
                  definite: decl.definite,
                },
                used,
              )
            }
            Pat::Array(_)
            | Pat::Rest(_)
            | Pat::Object(_)
            | Pat::Assign(_)
            | Pat::Invalid(_)
            | Pat::Expr(_) => (decl, true),
          })
          .partition(|item| item.1);
        if !unused.is_empty() {
          self.insert_item_tuple_list.push((
            self.module_item_index,
            ModuleItem::Stmt(Stmt::Decl(Decl::Var(Box::new(VarDecl {
              span: DUMMY_SP,
              kind: var.kind,
              declare: var.declare,
              decls: unused.into_iter().map(|item| item.0).collect(),
            })))),
          ))
        }
        if used.is_empty() {
          Self::create_empty_stmt_module_item()
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
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!("Javascript don't have these kinds asts")
      }
    }
  }
  /// Resolve module identifier with code generated module id and Esm dependency category.
  fn resolve_module_identifier(&mut self, code_generated_src: String) -> Option<ModuleIdentifier> {
    self
      .decl_mappings
      .get(&(
        self.module_identifier,
        code_generated_src,
        DependencyCategory::Esm,
      ))
      .cloned()
  }

  fn custom_fold_export_all(&mut self, export_all: ExportAll) -> ModuleItem {
    let module_identifier = self
      .resolve_module_identifier(export_all.src.value.to_string())
      .expect("TODO:");
    let mgm = self
      .module_graph
      .module_graph_module_by_identifier(&module_identifier)
      .expect("TODO:");
    if !self.include_module_ids.contains(&mgm.module_identifier) {
      Self::create_empty_stmt_module_item()
    } else {
      ModuleItem::ModuleDecl(ModuleDecl::ExportAll(export_all))
    }
  }

  fn custom_fold_named_export(&mut self, mut named: NamedExport) -> ModuleItem {
    if let Some(ref src) = named.src {
      let before_length = named.specifiers.len();
      let module_identifier = self
        .resolve_module_identifier(src.value.to_string())
        .expect("TODO:");
      let mgm = self
        .module_graph
        .module_graph_module_by_identifier(&module_identifier)
        .expect("TODO:");
      if !self.include_module_ids.contains(&mgm.module_identifier) {
        return Self::create_empty_stmt_module_item();
      }
      let specifiers = named
        .specifiers
        .into_iter()
        .filter(|specifier| match specifier {
          ExportSpecifier::Namespace(_) => {
            // export * from 'xxx'
            true
          }
          ExportSpecifier::Default(_) => {
            unreachable!("`export v from ''` is a unrecoverable syntax error")
          }

          ExportSpecifier::Named(named_spec) => {
            let original = match &named_spec.orig {
              ModuleExportName::Ident(ref ident) => ident.sym.clone(),
              ModuleExportName::Str(str) => str.value.clone(),
            };
            let exported = named_spec.exported.as_ref().map(|exported| match exported {
              ModuleExportName::Ident(ident) => ident.sym.clone(),
              ModuleExportName::Str(str) => str.value.clone(),
            });
            let symbol = SymbolRef::Indirect(IndirectTopLevelSymbol {
              src: self.module_identifier,
              ty: rspack_symbol::IndirectType::ReExport(original, exported),
              importer: self.module_identifier,
            });

            self.used_symbol_set.contains(&symbol)
          }
        })
        .collect::<Vec<_>>();

      // try if we could remove this export declaration
      if specifiers.is_empty() && self.side_effects_free_modules.contains(&module_identifier) {
        return Self::create_empty_stmt_module_item();
      }
      let is_all_used = before_length == specifiers.len();
      named.specifiers = specifiers;
      if !is_all_used {
        named.span = DUMMY_SP;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named))
    } else {
      let before_length = named.specifiers.len();
      let specifiers = named
        .specifiers
        .into_iter()
        .filter(|specifier| match specifier {
          ExportSpecifier::Namespace(_) => {
            unreachable!("`export *` is a syntax error")
          }
          ExportSpecifier::Default(_) => {
            unreachable!("`export v` is a unrecoverable syntax error")
          }

          ExportSpecifier::Named(named_spec) => match named_spec.orig {
            ModuleExportName::Ident(ref ident) => {
              let id: BetterId = ident.to_id().into();
              let symbol =
                SymbolRef::Direct(Symbol::new(self.module_identifier, id, SymbolType::Temp));
              self.used_symbol_set.contains(&symbol)
            }
            ModuleExportName::Str(_) => {
              // named export without src has string lit orig is a syntax error
              // `export { "something" }`
              unreachable!("`export {{ 'something' }}`")
            }
          },
        })
        .collect::<Vec<_>>();
      let is_all_used = before_length == specifiers.len();
      named.specifiers = specifiers;
      if !is_all_used {
        named.span = DUMMY_SP;
      }
      ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(named))
    }
  }

  fn custom_fold_export_default_decl(&mut self, decl: ExportDefaultDecl) -> ModuleItem {
    let default_symbol = self.crate_virtual_default_symbol();

    let ctxt = default_symbol.id().ctxt;
    if self
      .used_symbol_set
      .contains(&SymbolRef::Direct(default_symbol))
    {
      ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(decl))
    } else {
      let decl = match decl.decl {
        DefaultDecl::Class(class) => {
          let ident = class.ident.unwrap_or_else(|| {
            let mut named = quote_ident!("__RSPACK_DEFAULT_EXPORT__");
            named.span = named.span.with_ctxt(ctxt);
            named
          });
          Decl::Class(ClassDecl {
            ident,
            declare: false,
            class: class.class,
          })
        }
        DefaultDecl::Fn(func) => {
          let ident = func.ident.unwrap_or_else(|| {
            let mut named = quote_ident!("__RSPACK_DEFAULT_EXPORT__");
            named.span = named.span.with_ctxt(ctxt);
            named
          });
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

  fn custom_fold_export_default_expr(&mut self, expr: ExportDefaultExpr) -> ModuleItem {
    let default_symbol = SymbolRef::Direct(self.crate_virtual_default_symbol());
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
}
