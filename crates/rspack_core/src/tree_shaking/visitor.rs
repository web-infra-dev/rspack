use std::hash::Hash;

use bitflags::bitflags;
use swc_atoms::{js_word, JsWord};
use swc_common::{
  collections::{AHashMap, AHashSet},
  Globals, Mark, SyntaxContext, GLOBALS,
};
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};
use ustr::{ustr, Ustr};

use crate::{tree_shaking::symbol::Symbol, Dependency, ModuleGraph, ResolveKind};

use super::symbol::{BetterId, IndirectTopLevelSymbol};

#[derive(Debug)]
pub(crate) enum SymbolRef {
  Direct(Symbol),
  Indirect(IndirectTopLevelSymbol),
  /// uri
  Star(Ustr),
}

bitflags! {
  struct AnalyzeState: u8 {
    const EXPORT_DEFAULT = 1 << 0;
  }
}
#[derive(Debug)]
pub(crate) struct ModuleRefAnalyze<'a> {
  top_level_mark: Mark,
  unresolved_mark: Mark,
  module_identifier: Ustr,
  module_graph: &'a ModuleGraph,
  pub(crate) export_map: AHashMap<JsWord, SymbolRef>,
  pub(crate) import_map: AHashMap<BetterId, SymbolRef>,
  /// list of uri, each uri represent export all named export from specific uri
  pub export_all_list: Vec<Ustr>,
  current_region: Option<BetterId>,
  pub(crate) reference_map: AHashMap<BetterId, AHashSet<BetterId>>,
  pub(crate) export_import_map: AHashMap<BetterId, AHashSet<BetterId>>,
  state: AnalyzeState,
  // pub(crate) used_set: AHashSet<BetterId>,
}

impl<'a> ModuleRefAnalyze<'a> {
  pub fn new(
    top_level_mark: Mark,
    unresolved_mark: Mark,
    uri: Ustr,
    dep_to_module_uri: &'a ModuleGraph,
  ) -> Self {
    Self {
      top_level_mark,
      unresolved_mark,
      module_identifier: uri,
      module_graph: dep_to_module_uri,
      export_map: AHashMap::default(),
      import_map: AHashMap::default(),
      export_all_list: vec![],
      current_region: None,
      reference_map: AHashMap::default(),
      export_import_map: AHashMap::default(),
      state: AnalyzeState::empty(),
      // used_set: AHashSet::default(),
    }
  }

  pub fn add_reference(&mut self, from: BetterId, to: BetterId) {
    match self.reference_map.entry(from) {
      std::collections::hash_map::Entry::Occupied(mut occ) => {
        occ.get_mut().insert(to);
      }
      std::collections::hash_map::Entry::Vacant(vac) => {
        vac.insert(AHashSet::from_iter([to]));
      }
    }
  }

  pub fn get_all_import_() {}
}

impl<'a> Visit for ModuleRefAnalyze<'a> {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
    // let import_
  }
  fn visit_ident(&mut self, node: &Ident) {
    let id: BetterId = node.to_id().into();
    let marker = id.ctxt.outer();
    if let Some(ref region) = self.current_region && marker == self.top_level_mark && region != &id {
      self.add_reference(region.clone(), id);
    }
  }

  fn visit_module_item(&mut self, node: &ModuleItem) {
    match node {
      ModuleItem::ModuleDecl(decl) => match decl {
        ModuleDecl::Import(import) => {
          let src: String = import.src.value.to_string();
          let resolved_uri = self.resolve_module_identifier(src, ResolveKind::Import);
          let resolved_uri_ukey = ustr(&resolved_uri);
          import
            .specifiers
            .iter()
            .for_each(|specifier| match specifier {
              ImportSpecifier::Named(named) => {
                let local = named.local.sym.clone();
                let imported = match &named.imported {
                  Some(imported) => match imported {
                    ModuleExportName::Ident(ident) => ident.sym.clone(),
                    ModuleExportName::Str(str) => str.value.clone(),
                  },
                  None => local.clone(),
                };
                let symbol_ref =
                  SymbolRef::Indirect(IndirectTopLevelSymbol::new(resolved_uri_ukey, imported));
                self.add_import(named.local.to_id().into(), symbol_ref);
              }
              ImportSpecifier::Default(_) => todo!(),
              ImportSpecifier::Namespace(namespace) => {
                self.add_import(
                  namespace.local.to_id().into(),
                  SymbolRef::Star(resolved_uri_ukey),
                );
              }
            });
        }
        ModuleDecl::ExportDecl(decl) => {
          self.state |= AnalyzeState::EXPORT_DEFAULT;
          match &decl.decl {
            Decl::Class(class) => {
              class.visit_with(self);
              self.add_export(
                class.ident.sym.clone(),
                SymbolRef::Direct(Symbol::from_id_and_uri(
                  class.ident.to_id().into(),
                  self.module_identifier,
                )),
              );
            }
            Decl::Fn(function) => {
              function.visit_with(self);
              self.add_export(
                function.ident.sym.clone(),
                SymbolRef::Direct(Symbol::from_id_and_uri(
                  function.ident.to_id().into(),
                  self.module_identifier,
                )),
              );
            }
            Decl::Var(var) => {
              var.visit_with(self);
            }
            Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
              todo!()
            }
          }
          self.state.remove(AnalyzeState::EXPORT_DEFAULT);
        }
        ModuleDecl::ExportNamed(named_export) => {
          self.analyze_named_export(named_export);
        }
        ModuleDecl::ExportDefaultDecl(_) => todo!(),
        ModuleDecl::ExportDefaultExpr(_) => todo!(),
        ModuleDecl::ExportAll(export_all) => {
          let resolved_uri_key = ustr(
            self.resolve_module_identifier(export_all.src.value.to_string(), ResolveKind::Import),
          );
          self.export_all_list.push(resolved_uri_key);
        }
        ModuleDecl::TsImportEquals(_) => todo!(),
        ModuleDecl::TsExportAssignment(_) => todo!(),
        ModuleDecl::TsNamespaceExport(_) => todo!(),
      },
      ModuleItem::Stmt(_) => {
        node.visit_children_with(self);
      }
    }
  }
  fn visit_class_decl(&mut self, node: &ClassDecl) {
    let id: BetterId = node.ident.to_id().into();
    let mark = id.ctxt.outer();
    let old_region = self.current_region.clone();
    if mark == self.top_level_mark {
      self.current_region = Some(id);
    }
    node.visit_children_with(self);
    self.current_region = old_region;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    let id: BetterId = node.ident.to_id().into();
    let mark = id.ctxt.outer();
    let old_region = self.current_region.clone();
    if mark == self.top_level_mark {
      self.current_region = Some(id);
    }
    node.function.visit_with(self);
    self.current_region = old_region;
  }

  fn visit_var_decl(&mut self, node: &VarDecl) {
    for ele in node.decls.iter() {
      let lhs: BetterId = match &ele.name {
        Pat::Ident(ident) => ident.to_id().into(),
        Pat::Array(_) => todo!(),
        Pat::Rest(_) => todo!(),
        Pat::Object(_) => todo!(),
        Pat::Assign(_) => todo!(),
        Pat::Invalid(_) => todo!(),
        Pat::Expr(_) => todo!(),
      };
      if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
        self.add_export(
          lhs.atom.clone(),
          SymbolRef::Direct(Symbol::from_id_and_uri(lhs.clone(), self.module_identifier)),
        );
      }
      if let Some(ref init) = ele.init && lhs.ctxt.outer() == self.top_level_mark {
        let before_region = self.current_region.clone();
        self.current_region = Some(lhs);
        init.visit_with(self);
        self.current_region = before_region;
      }
    }
    // let id: BetterId = node.ident.to_id().into();
    // let mark = id.ctxt.outer();
    // let old_region = self.current_region.clone();
    // if mark == self.top_level_mark {
    //   self.current_region = Some(id);
    // }
    // node.visit_children_with(self);
    // self.current_region = old_
  }
  fn visit_decl(&mut self, node: &Decl) {
    match node {
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => todo!(),
      Decl::Class(_) | Decl::Fn(_) | Decl::Var(_) => {
        node.visit_children_with(self);
      }
    }
  }
}

impl<'a> ModuleRefAnalyze<'a> {
  fn add_export(&mut self, id: JsWord, symbol: SymbolRef) {
    if self.export_map.contains_key(&id) {
      // TODO: should add some Diagnostic
    } else {
      self.export_map.insert(id, symbol);
    }
  }

  fn add_import(&mut self, id: BetterId, symbol: SymbolRef) {
    if self.import_map.contains_key(&id) {
      // TODO: should add some Diagnostic
    } else {
      self.import_map.insert(id, symbol);
    }
  }
  fn analyze_named_export(&mut self, named_export: &NamedExport) {
    let src: Option<String> = named_export.src.as_ref().map(|src| src.value.to_string());
    if let Some(src) = src {
      let resolved_uri = self.resolve_module_identifier(src, ResolveKind::Import);
      let resolved_uri_ukey = ustr(&resolved_uri);
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(namespace) => {
            // TODO: handle `* as xxx`, do we need a extra binding
            self.export_all_list.push(resolved_uri_ukey);
          }
          ExportSpecifier::Default(_) => {
            // Currently swc does not support syntax like `export v from 'xxx';`
            unreachable!("Module has syntax error should not trigger tree_shaking")
          }
          ExportSpecifier::Named(named) => {
            // TODO: what if the named binding is a unresolved_binding?
            // TODO: handle `as xxx`
            let id = match &named.orig {
              ModuleExportName::Ident(ident) => ident.sym.clone(),
              ModuleExportName::Str(_) => todo!(),
            };
            let symbol_ref =
              SymbolRef::Indirect(IndirectTopLevelSymbol::new(resolved_uri_ukey, id.clone()));
            self.add_export(id, symbol_ref);
          }
        });
    } else {
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(_) => {
            // named_export has namespace specifier but no src will trigger a syntax error and should not reach here. e.g.
            // `export *`;
            unreachable!("")
          }
          ExportSpecifier::Default(_) => {
            // Currently swc does not support syntax like `export v from 'xxx';`
            unreachable!("Module has syntax error should not trigger tree shaking")
          }
          ExportSpecifier::Named(named) => {
            // TODO: what if the named binding is a unresolved_binding?
            // TODO: handle `as xxx`
            let id = match &named.orig {
              ModuleExportName::Ident(ident) => ident.to_id(),
              // export {'a'} is a syntax error;
              // `export {'a'} from 'xxx'` is not.
              // we know here export has no src,  so this branch should not reachable.
              ModuleExportName::Str(_) => unreachable!(),
            };
            let symbol_ref = SymbolRef::Direct(Symbol::from_id_and_uri(
              id.clone().into(),
              self.module_identifier,
            ));
            self.add_export(id.0, symbol_ref);
          }
        });
    };
  }

  /// Try to get the module_identifier from `src`, `resolve_kind`, and `importer`
  /// For simplicity, this function will assume the importer is always `self.module_identifier`
  /// # Panic
  /// This function will panic if can't find
  fn resolve_module_identifier(&mut self, src: String, resolve_kind: ResolveKind) -> &String {
    let resolved_uri = &self
      .module_graph
      .module_by_dependency(&Dependency {
        importer: Some(self.module_identifier.to_string()),
        detail: crate::ModuleDependency {
          specifier: src,
          kind: resolve_kind,
          span: None,
        },
        parent_module_identifier: Some(self.module_identifier.to_string()),
      })
      .unwrap()
      .module_identifier;
    resolved_uri
  }

  // fn try_resolve_uri(&mut self, src: String, resolve_kind: ResolveKind) -> Option<&String> {
  //   self.module_graph.module_uri_by_deppendency(&Dependency {
  //     importer: Some(self.uri.to_string()),
  //     detail: crate::ModuleDependency {
  //       specifier: src,
  //       kind: resolve_kind,
  //       span: None,
  //     },
  //     parent_module_identifier: Some(self.uri.to_string()),
  //   })
  // }
}
