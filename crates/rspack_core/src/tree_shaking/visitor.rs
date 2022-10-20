use std::{collections::VecDeque, hash::Hash};

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum SymbolRef {
  Direct(Symbol),
  Indirect(IndirectTopLevelSymbol),
  /// uri
  Star(Ustr),
}

bitflags! {
  #[derive(Default)]
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
  pub(crate) reachable_import_of_export: AHashMap<JsWord, AHashSet<SymbolRef>>,
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
      reachable_import_of_export: AHashMap::default(),
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

  /// Collecting all reachable import binding from given start binding
  pub fn get_all_import(&self, start: BetterId) -> AHashSet<SymbolRef> {
    let mut seen: AHashSet<BetterId> = AHashSet::default();
    let mut q: VecDeque<BetterId> = VecDeque::from_iter([start]);
    while let Some(cur) = q.pop_front() {
      if seen.contains(&cur) {
        continue;
      }
      if let Some(ref_list) = self.reference_map.get(&cur) {
        q.extend(ref_list.clone());
      }
      seen.insert(cur);
    }
    return seen
      .iter()
      .filter_map(|id| {
        let ret = self
          .import_map
          .get(&id)
          .map(|item| item.clone())
          .or_else(|| match self.export_map.get(&id.atom) {
            Some(sym_ref @ SymbolRef::Direct(sym)) => {
              if &sym.id == id {
                Some(sym_ref.clone())
              } else {
                None
              }
            }
            _ => None,
          });
        ret
      })
      .collect();
  }
}

impl<'a> Visit for ModuleRefAnalyze<'a> {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
    for (key, symbol) in self.export_map.iter() {
      match symbol {
        // At this time uri of symbol will always equal to `self.module_identifier`
        SymbolRef::Direct(symbol) => {
          let reachable_import = self.get_all_import(symbol.id.clone());
          self
            .reachable_import_of_export
            .insert(key.clone(), reachable_import);
        }
        // ignore any indrect symbol
        SymbolRef::Indirect(_) | SymbolRef::Star(_) => {}
      }
    }
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
          let resolved_uri = match self.resolve_module_identifier(src, ResolveKind::Import) {
            Some(module_identifier) => module_identifier,
            None => {
              // TODO: Ignore because helper interference.
              return;
            }
          };
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
              ImportSpecifier::Default(_) => {
                // TODO:
              }
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
        ModuleDecl::ExportDefaultDecl(_) => {
          // TODO:
        }
        ModuleDecl::ExportDefaultExpr(_) => {
          // TODO:
        }
        ModuleDecl::ExportAll(export_all) => {
          let resolved_uri = match self
            .resolve_module_identifier(export_all.src.value.to_string(), ResolveKind::Import)
          {
            Some(module_identifier) => module_identifier,
            None => {
              // TODO: ignore for now, or  three copy js will failed
              return;
            }
          };
          let resolved_uri_key = ustr(&resolved_uri);
          self.export_all_list.push(resolved_uri_key);
        }
        ModuleDecl::TsImportEquals(_)
        | ModuleDecl::TsExportAssignment(_)
        | ModuleDecl::TsNamespaceExport(_) => {
          // TODO:
        }
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
        Pat::Array(_)
        | Pat::Rest(_)
        | Pat::Object(_)
        | Pat::Assign(_)
        | Pat::Invalid(_)
        | Pat::Expr(_) => {
          // TODO:
          continue;
        }
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
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        // TODO:
      }
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
      let resolved_uri = match self.resolve_module_identifier(src, ResolveKind::Import) {
        Some(module_identifier) => module_identifier,
        None => {
          // TODO: Ignore because helper interference.
          return;
        }
      };
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
              ModuleExportName::Str(_) => {
                todo!()
              }
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
  fn resolve_module_identifier(
    &mut self,
    src: String,
    resolve_kind: ResolveKind,
  ) -> Option<&String> {
    let dep = Dependency {
      importer: Some(self.module_identifier.to_string()),
      detail: crate::ModuleDependency {
        specifier: src,
        kind: resolve_kind,
        span: None,
      },
      parent_module_identifier: Some(self.module_identifier.to_string()),
    };
    self
      .module_graph
      .module_by_dependency(&dep)
      .map(|module| &module.module_identifier)
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

/// The `allow(unused)` will be removed after the Tree shaking is finished
#[allow(unused)]
pub(crate) struct TreeShakingResult {
  top_level_mark: Mark,
  unresolved_mark: Mark,
  module_identifier: Ustr,
  pub(crate) export_map: AHashMap<JsWord, SymbolRef>,
  pub(crate) import_map: AHashMap<BetterId, SymbolRef>,
  /// list of uri, each uri represent export all named export from specific uri
  pub export_all_list: Vec<Ustr>,
  current_region: Option<BetterId>,
  pub(crate) reference_map: AHashMap<BetterId, AHashSet<BetterId>>,
  pub(crate) reachable_import_of_export: AHashMap<JsWord, AHashSet<SymbolRef>>,
  state: AnalyzeState,
}

impl From<ModuleRefAnalyze<'_>> for TreeShakingResult {
  fn from(mut analyze: ModuleRefAnalyze<'_>) -> Self {
    Self {
      top_level_mark: std::mem::take(&mut analyze.top_level_mark),
      unresolved_mark: std::mem::take(&mut analyze.unresolved_mark),
      module_identifier: std::mem::take(&mut analyze.module_identifier),
      export_map: std::mem::take(&mut analyze.export_map),
      import_map: std::mem::take(&mut analyze.import_map),
      export_all_list: std::mem::take(&mut analyze.export_all_list),
      current_region: std::mem::take(&mut analyze.current_region),
      reference_map: std::mem::take(&mut analyze.reference_map),
      reachable_import_of_export: std::mem::take(&mut analyze.reachable_import_of_export),
      state: std::mem::take(&mut analyze.state),
    }
  }
}
