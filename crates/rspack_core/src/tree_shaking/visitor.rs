use std::{collections::VecDeque, hash::Hash};

use bitflags::bitflags;
use swc_atoms::JsWord;
use swc_common::{
  collections::{AHashMap, AHashSet},
  util::take::Take,
  Mark, GLOBALS,
};
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};
use ustr::{ustr, Ustr};

use crate::{tree_shaking::symbol::Symbol, Dependency, ModuleGraph, ResolveKind};

use super::symbol::{BetterId, IndirectTopLevelSymbol};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SymbolRef {
  Direct(Symbol),
  Indirect(IndirectTopLevelSymbol),
  /// uri
  Star(Ustr),
}

bitflags! {
  #[derive(Default)]
  struct AnalyzeState: u8 {
    const EXPORT_DECL = 1 << 0;
    const EXPORT_DEFAULT = 1 << 1;
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
  pub(crate) reachable_import_and_export: AHashMap<JsWord, AHashSet<SymbolRef>>,
  state: AnalyzeState,
  pub(crate) used_id_set: AHashSet<BetterId>,
  pub(crate) used_symbol_ref: AHashSet<SymbolRef>,
  pub(crate) export_default_name: Option<JsWord>,
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
      reachable_import_and_export: AHashMap::default(),
      state: AnalyzeState::empty(),
      used_id_set: AHashSet::default(),
      used_symbol_ref: AHashSet::default(),
      export_default_name: None,
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
        let ret =
          self
            .import_map
            .get(id)
            .cloned()
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

  fn generate_default_ident(&self) -> Ident {
    let mut default_ident = Ident::dummy();
    default_ident.sym = "default".into();
    default_ident.span = default_ident.span.apply_mark(self.top_level_mark);
    default_ident
  }
}

impl<'a> Visit for ModuleRefAnalyze<'a> {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
    // calc reachable imports for each export symbol defined in current module
    for (key, symbol) in self.export_map.iter() {
      match symbol {
        // At this time uri of symbol will always equal to `self.module_identifier`
        SymbolRef::Direct(symbol) => {
          let reachable_import = self.get_all_import(symbol.id.clone());
          self
            .reachable_import_and_export
            .insert(key.clone(), reachable_import);
        }
        // ignore any indrect symbol
        SymbolRef::Indirect(_) | SymbolRef::Star(_) => {}
      }
    }
    // dbg!(&self.);
    // all reachable import from used symbol in current module
    for used_id in &self.used_id_set {
      let reachable_import = self.get_all_import(used_id.clone());
      // dbg!(&used_id, &reachable_import);
      self.used_symbol_ref.extend(reachable_import);
    }
  }

  fn visit_ident(&mut self, node: &swc_ecma_ast::Ident) {
    let id: BetterId = node.to_id().into();
    let marker = id.ctxt.outer();
    if marker == self.top_level_mark {
      match self.current_region {
        Some(ref region) if region != &id => {
          self.add_reference(region.clone(), id);
        }
        _ if marker != self.unresolved_mark => {
          self.used_id_set.insert(id);
        }
        _ => {}
      }
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
          let resolved_uri_ukey = ustr(resolved_uri);
          import
            .specifiers
            .iter()
            .for_each(|specifier| match specifier {
              ImportSpecifier::Named(named) => {
                let imported = match &named.imported {
                  Some(imported) => match imported {
                    ModuleExportName::Ident(ident) => ident.sym.clone(),
                    ModuleExportName::Str(str) => str.value.clone(),
                  },
                  None => named.local.sym.clone(),
                };
                let symbol_ref =
                  SymbolRef::Indirect(IndirectTopLevelSymbol::new(resolved_uri_ukey, imported));
                self.add_import(named.local.to_id().into(), symbol_ref);
              }
              ImportSpecifier::Default(default) => {
                self.add_import(
                  default.local.to_id().into(),
                  SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                    resolved_uri_ukey,
                    "default".into(),
                  )),
                );
              }
              ImportSpecifier::Namespace(namespace) => {
                self.add_import(
                  namespace.local.to_id().into(),
                  SymbolRef::Star(resolved_uri_ukey),
                );
              }
            });
        }
        ModuleDecl::ExportDecl(decl) => match &decl.decl {
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
            self.state |= AnalyzeState::EXPORT_DECL;
            var.visit_with(self);
            self.state.remove(AnalyzeState::EXPORT_DECL);
          }
          Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
            todo!()
          }
        },
        ModuleDecl::ExportNamed(named_export) => {
          self.analyze_named_export(named_export);
        }
        ModuleDecl::ExportDefaultDecl(decl) => {
          decl.visit_with(self);
        }
        ModuleDecl::ExportDefaultExpr(expr) => {
          expr.visit_with(self);
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
          let resolved_uri_key = ustr(resolved_uri);
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

  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    let before_region = self.current_region.clone();
    let default_ident: BetterId = self.generate_default_ident().to_id().into();

    self.export_map.insert(
      default_ident.atom.clone(),
      SymbolRef::Direct(Symbol::from_id_and_uri(
        default_ident.clone(),
        self.module_identifier,
      )),
    );
    match self.export_default_name {
      Some(_) => {
        // TODO: Better diagnostic
        panic!("Duplicate export default")
      }
      None => {
        self.export_default_name = Some("".into());
      }
    }
    self.current_region = Some(default_ident);
    node.visit_children_with(self);
    self.current_region = before_region;
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    self.state |= AnalyzeState::EXPORT_DEFAULT;
    match &node.decl {
      DefaultDecl::Class(_) | DefaultDecl::Fn(_) => {
        node.visit_children_with(self);
      }
      DefaultDecl::TsInterfaceDecl(_) => {
        todo!()
      }
    }
    self.state.remove(AnalyzeState::EXPORT_DEFAULT);
  }

  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
      let default_ident = self.generate_default_ident();
      self.export_map.insert(
        default_ident.sym.clone(),
        SymbolRef::Direct(Symbol::from_id_and_uri(
          default_ident.to_id().into(),
          self.module_identifier,
        )),
      );
      let region: BetterId = if let Some(ident) = &node.ident {
        match self.export_default_name {
          Some(_) => {
            // TODO: Better diagnostic
            panic!("Duplicate export default")
          }
          None => {
            self.export_default_name = Some(ident.sym.clone());
            self.add_reference(default_ident.to_id().into(), ident.to_id().into());
            self.add_reference(ident.to_id().into(), default_ident.to_id().into());
          }
        }
        ident.to_id().into()
      } else {
        default_ident.to_id().into()
      };
      let before_region = self.current_region.clone();
      self.current_region = Some(region);
      node.class.visit_with(self);
      self.current_region = before_region;
    } else {
      // if the class expr is not inside a default expr, it will not
      // generate a binding.
      node.class.visit_with(self);
    }
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
      let default_ident = self.generate_default_ident();
      self.export_map.insert(
        default_ident.sym.clone(),
        SymbolRef::Direct(Symbol::from_id_and_uri(
          default_ident.to_id().into(),
          self.module_identifier,
        )),
      );
      let region: BetterId = if let Some(ident) = &node.ident {
        match self.export_default_name {
          Some(_) => {
            // TODO: Better diagnostic
            panic!("Duplicate export default")
          }
          None => {
            self.export_default_name = Some(ident.sym.clone());
            self.add_reference(default_ident.to_id().into(), ident.to_id().into());
            self.add_reference(ident.to_id().into(), default_ident.to_id().into());
          }
        }
        ident.to_id().into()
      } else {
        default_ident.to_id().into()
      };
      let before_region = self.current_region.clone();
      self.current_region = Some(region);
      node.function.visit_with(self);
      self.current_region = before_region;
    } else {
      // if the class expr is not inside a default expr, it will not
      // generate a binding.
      node.function.visit_with(self);
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
      if self.state.contains(AnalyzeState::EXPORT_DECL) {
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
    match self.export_map.entry(id) {
      std::collections::hash_map::Entry::Occupied(_) => {
        // TODO: should add some Diagnostic
      }
      std::collections::hash_map::Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  fn add_import(&mut self, id: BetterId, symbol: SymbolRef) {
    match self.import_map.entry(id) {
      std::collections::hash_map::Entry::Occupied(_) => {
        // TODO: should add some Diagnostic
      }
      std::collections::hash_map::Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
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
      let resolved_uri_ukey = ustr(resolved_uri);
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(_) => {
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
#[derive(Debug)]
#[allow(unused)]
pub struct TreeShakingResult {
  top_level_mark: Mark,
  unresolved_mark: Mark,
  module_identifier: Ustr,
  pub export_map: AHashMap<JsWord, SymbolRef>,
  pub(crate) import_map: AHashMap<BetterId, SymbolRef>,
  /// list of uri, each uri represent export all named export from specific uri
  pub export_all_list: Vec<Ustr>,
  current_region: Option<BetterId>,
  pub(crate) reference_map: AHashMap<BetterId, AHashSet<BetterId>>,
  pub(crate) reachable_import_of_export: AHashMap<JsWord, AHashSet<SymbolRef>>,
  state: AnalyzeState,
  pub(crate) used_symbol_ref: AHashSet<SymbolRef>,
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
      reachable_import_of_export: std::mem::take(&mut analyze.reachable_import_and_export),
      state: std::mem::take(&mut analyze.state),
      used_symbol_ref: std::mem::take(&mut analyze.used_symbol_ref),
    }
  }
}
