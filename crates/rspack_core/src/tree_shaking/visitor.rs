use std::{collections::VecDeque, hash::Hash};

use bitflags::bitflags;
use hashbrown::{hash_map::Entry, HashMap, HashSet};
use indexmap::IndexMap;
use swc_atoms::JsWord;
use swc_common::{util::take::Take, Mark, GLOBALS};
use swc_ecma_ast::*;
use swc_ecma_visit::{noop_visit_type, Visit, VisitWith};
use ustr::{ustr, Ustr};

use crate::{Dependency, ModuleGraph, ModuleSyntax, ResolveKind};
use rspack_symbol::{BetterId, IdOrMemExpr, IndirectTopLevelSymbol, Symbol, SymbolExt, SymbolFlag};

use super::utils::{get_dynamic_import_string_literal, is_require_literal_expr};
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
  pub(crate) export_map: HashMap<JsWord, SymbolRef>,
  pub(crate) import_map: HashMap<BetterId, SymbolRef>,
  /// key is the module identifier, value is the corresponding export map
  /// This data structure is used for collecting reexport * from some module. e.g.
  /// ```js
  /// export * from './test.js'
  /// ```
  /// then inherit_exports_maps become, `{"test.js": {...test_js_export_map} }`
  // Use `IndexMap` to keep the insertion order
  pub inherit_export_maps: IndexMap<Ustr, HashMap<JsWord, SymbolRef>>,
  current_body_owner_symbol_ext: Option<SymbolExt>,
  pub(crate) reference_map: HashMap<SymbolExt, HashSet<IdOrMemExpr>>,
  pub(crate) reachable_import_and_export: HashMap<JsWord, HashSet<SymbolRef>>,
  state: AnalyzeState,
  pub(crate) used_id_set: HashSet<IdOrMemExpr>,
  pub(crate) used_symbol_ref: HashSet<SymbolRef>,
  // This field is used for duplicated export default checking
  pub(crate) export_default_name: Option<JsWord>,
  module_syntax: ModuleSyntax,
}

impl<'a> ModuleRefAnalyze<'a> {
  pub fn new(
    top_level_mark: Mark,
    unresolved_mark: Mark,
    uri: Ustr,
    dep_to_module_identifier: &'a ModuleGraph,
  ) -> Self {
    Self {
      top_level_mark,
      unresolved_mark,
      module_identifier: uri,
      module_graph: dep_to_module_identifier,
      export_map: HashMap::default(),
      import_map: HashMap::default(),
      inherit_export_maps: IndexMap::default(),
      current_body_owner_symbol_ext: None,
      reference_map: HashMap::new(),
      reachable_import_and_export: HashMap::default(),
      state: AnalyzeState::empty(),
      used_id_set: HashSet::default(),
      used_symbol_ref: HashSet::default(),
      export_default_name: None,
      module_syntax: ModuleSyntax::empty(),
    }
  }

  pub fn add_reference(&mut self, from: SymbolExt, to: IdOrMemExpr) {
    if matches!(&to, IdOrMemExpr::Id(to_id) if to_id == from.id()) {
      return;
    }
    match self.reference_map.entry(from) {
      Entry::Occupied(mut occ) => {
        occ.get_mut().insert(to);
      }
      Entry::Vacant(vac) => {
        vac.insert(HashSet::from_iter([to]));
      }
    }
  }

  /// Collecting all reachable import binding from given start binding
  pub fn get_all_import_or_export(&self, start: BetterId) -> HashSet<SymbolRef> {
    let mut seen: HashSet<IdOrMemExpr> = HashSet::default();
    let mut q: VecDeque<IdOrMemExpr> = VecDeque::from_iter([IdOrMemExpr::Id(start)]);
    while let Some(cur) = q.pop_front() {
      if seen.contains(&cur) {
        continue;
      }
      let id = cur.get_id();
      if let Some(ref_list) = self.reference_map.get(&id.clone().into()) {
        q.extend(ref_list.clone());
      }
      seen.insert(cur);
    }
    return seen
      .iter()
      .filter_map(|id_or_mem_expr| match id_or_mem_expr {
        IdOrMemExpr::Id(id) => {
          let ret =
            self
              .import_map
              .get(id)
              .cloned()
              .or_else(|| match self.export_map.get(&id.atom) {
                Some(sym_ref @ SymbolRef::Direct(sym)) => {
                  if sym.id() == id {
                    Some(sym_ref.clone())
                  } else {
                    None
                  }
                }
                _ => None,
              });
          ret
        }
        IdOrMemExpr::MemberExpr { object, property } => {
          self.import_map.get(object).map(|sym_ref| match sym_ref {
            SymbolRef::Direct(_) | SymbolRef::Indirect(_) => sym_ref.clone(),
            SymbolRef::Star(uri) => SymbolRef::Indirect(IndirectTopLevelSymbol::new(
              *uri,
              property.clone(),
              self.module_identifier,
            )),
          })
        }
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
          let reachable_import = self.get_all_import_or_export(symbol.id().clone());
          self
            .reachable_import_and_export
            .insert(key.clone(), reachable_import);
        }
        // ignore any indrect symbol, because it will not generate binding, the reachable exports will
        // be calculated in the module where it is defined
        SymbolRef::Indirect(_) | SymbolRef::Star(_) => {}
      }
    }

    // Any var declaration has reference a symbol from other module, it is marked as used
    // Because the symbol import from other module possibly has side effect
    let side_effect_id_list = self
      .reference_map
      .iter()
      .filter(|(symbol, ref_list)| {
        if !symbol.flag.contains(SymbolFlag::VAR_DECL) {
          false
        } else {
          ref_list.iter().any(|ref_id| {
            self.import_map.contains_key(match ref_id {
              IdOrMemExpr::Id(id) => id,
              IdOrMemExpr::MemberExpr { object, .. } => object,
            })
          })
        }
      })
      .map(|(k, _)| IdOrMemExpr::Id(k.id().clone()));
    self.used_id_set.extend(side_effect_id_list);
    // all reachable export from used symbol in current module
    for used_id in &self.used_id_set {
      match used_id {
        IdOrMemExpr::Id(id) => {
          let reachable_import = self.get_all_import_or_export(id.clone());
          self.used_symbol_ref.extend(reachable_import);
        }
        IdOrMemExpr::MemberExpr { object, property } => match self.import_map.get(object) {
          Some(SymbolRef::Star(uri)) => {
            self
              .used_symbol_ref
              .insert(SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                *uri,
                property.clone(),
                self.module_identifier,
              )));
          }
          _ => {
            let reachable_import = self.get_all_import_or_export(object.clone());
            self.used_symbol_ref.extend(reachable_import);
          }
        },
      }
    }
  }

  fn visit_ident(&mut self, node: &swc_ecma_ast::Ident) {
    let id: BetterId = node.to_id().into();
    let mark = id.ctxt.outer();
    if mark == self.top_level_mark {
      match self.current_body_owner_symbol_ext {
        Some(ref body_owner_symbol_ext) if body_owner_symbol_ext.id() != &id => {
          self.add_reference(body_owner_symbol_ext.clone(), IdOrMemExpr::Id(id));
        }
        None => {
          self.used_id_set.insert(IdOrMemExpr::Id(id));
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
              // TODO: Ignore for now because swc helper interference.
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
                let symbol_ref = SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                  resolved_uri_ukey,
                  imported,
                  self.module_identifier,
                ));
                self.add_import(named.local.to_id().into(), symbol_ref);
              }
              ImportSpecifier::Default(default) => {
                self.add_import(
                  default.local.to_id().into(),
                  SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                    resolved_uri_ukey,
                    "default".into(),
                    self.module_identifier,
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
              // TODO: ignore for now, or three copy js will failed
              return;
            }
          };
          let resolved_uri_key = ustr(resolved_uri);
          self
            .inherit_export_maps
            .insert(resolved_uri_key, HashMap::default());
        }
        ModuleDecl::TsImportEquals(_)
        | ModuleDecl::TsExportAssignment(_)
        | ModuleDecl::TsNamespaceExport(_) => {
          // TODO: ignore ts related syntax visit for now
        }
      },
      ModuleItem::Stmt(_) => {
        node.visit_children_with(self);
      }
    }
  }

  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();
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
        panic!(
          "Duplicate export default (export_default_expr) in {}",
          self.module_identifier
        )
      }
      None => {
        self.export_default_name = Some("default".into());
      }
    }
    let mut symbol_ext: SymbolExt = default_ident.into();
    symbol_ext.flag |= SymbolFlag::EXPORT_DEFAULT;
    self.current_body_owner_symbol_ext = Some(symbol_ext);
    node.visit_children_with(self);
    self.current_body_owner_symbol_ext = before_owner_extend_symbol;
  }

  fn visit_member_expr(&mut self, node: &MemberExpr) {
    match (&*node.obj, &node.prop) {
      // a.b
      (Expr::Ident(obj), MemberProp::Ident(prop)) => {
        let id: BetterId = obj.to_id().into();
        let mark = id.ctxt.outer();
        if mark == self.top_level_mark {
          let member_expr = IdOrMemExpr::MemberExpr {
            object: id.clone(),
            property: prop.sym.clone(),
          };
          match self.current_body_owner_symbol_ext {
            Some(ref body_owner_symbol_ext) if body_owner_symbol_ext.id() != &id => {
              self.add_reference(body_owner_symbol_ext.clone(), member_expr);
            }
            None => {
              self.used_id_set.insert(member_expr);
            }
            _ => {}
          }
        }
      }
      // TODO: Do we need to consider such scenario ?
      // obj[`test`] use a template literal but actually is a pure string lit
      // obj['prop']
      (
        Expr::Ident(obj),
        MemberProp::Computed(ComputedPropName {
          expr: box Expr::Lit(Lit::Str(Str { value, .. })),
          ..
        }),
      ) => {
        let id: BetterId = obj.to_id().into();
        let mark = id.ctxt.outer();
        if mark == self.top_level_mark {
          let member_expr = IdOrMemExpr::MemberExpr {
            object: id.clone(),
            property: value.clone(),
          };
          match self.current_body_owner_symbol_ext {
            Some(ref body_owner_symbol_ext) if body_owner_symbol_ext.id() != &id => {
              self.add_reference(body_owner_symbol_ext.clone(), member_expr);
            }
            None => {
              self.used_id_set.insert(member_expr);
            }
            _ => {}
          }
        }
      }
      _ => {
        node.visit_children_with(self);
      }
    }
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    self.state |= AnalyzeState::EXPORT_DEFAULT;
    match &node.decl {
      DefaultDecl::Class(_) | DefaultDecl::Fn(_) => {
        node.visit_children_with(self);
      }
      DefaultDecl::TsInterfaceDecl(_) => {
        // TODO: Ts syntax related tree-shaking is ignored by now.
        todo!("Ts ")
      }
    }
    self.state.remove(AnalyzeState::EXPORT_DEFAULT);
  }

  fn visit_class_expr(&mut self, node: &ClassExpr) {
    if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
      self.state.remove(AnalyzeState::EXPORT_DEFAULT);
      let default_ident = self.generate_default_ident();
      self.export_map.insert(
        default_ident.sym.clone(),
        SymbolRef::Direct(Symbol::from_id_and_uri(
          default_ident.to_id().into(),
          self.module_identifier,
        )),
      );
      let body_owner_extend_symbol: SymbolExt = match self.export_default_name {
        Some(_) => {
          // TODO: Better diagnostic
          panic!(
            "Duplicate export default(class_expr) in {}",
            self.module_identifier
          )
        }
        None => {
          let symbol_ext: SymbolExt = if let Some(ident) = &node.ident {
            let symbol_ext = SymbolExt::new(ident.to_id().into(), SymbolFlag::EXPORT_DEFAULT);
            self.add_reference(
              BetterId::from(ident.to_id()).into(),
              IdOrMemExpr::Id(symbol_ext.id.clone()),
            );
            self.add_reference(
              symbol_ext.clone(),
              IdOrMemExpr::Id(default_ident.to_id().into()),
            );
            symbol_ext
          } else {
            SymbolExt::new(default_ident.to_id().into(), SymbolFlag::EXPORT_DEFAULT)
          };
          self.export_default_name = Some(symbol_ext.id().atom.clone());
          symbol_ext
        }
      };
      let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();

      self.current_body_owner_symbol_ext = Some(body_owner_extend_symbol);
      node.class.visit_with(self);
      self.current_body_owner_symbol_ext = before_owner_extend_symbol;
    } else {
      // if the class expr is not inside a default expr, it will not
      // generate a binding.
      node.class.visit_with(self);
    }
  }

  fn visit_call_expr(&mut self, node: &CallExpr) {
    // TODO: module.exports, exports.xxxxx
    if is_require_literal_expr(node, self.unresolved_mark) {
      self.module_syntax.insert(ModuleSyntax::COMMONJS);
    } else if let Some(import_str) = get_dynamic_import_string_literal(node) {
    } else {
      node.visit_children_with(self);
    }
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
      self.state.remove(AnalyzeState::EXPORT_DEFAULT);
      let default_ident = self.generate_default_ident();
      self.export_map.insert(
        default_ident.sym.clone(),
        SymbolRef::Direct(Symbol::from_id_and_uri(
          default_ident.to_id().into(),
          self.module_identifier,
        )),
      );
      let body_owner_extend_symbol: SymbolExt = match self.export_default_name {
        Some(_) => {
          // TODO: Better diagnostic
          panic!(
            "Duplicate export default(fn_expr) in {}",
            self.module_identifier
          )
        }
        None => {
          let symbol_ext: SymbolExt = if let Some(ident) = &node.ident {
            let symbol_ext = SymbolExt::new(ident.to_id().into(), SymbolFlag::EXPORT_DEFAULT);
            // considering default export has bind to new symbol e.g.
            // export default function test() {
            // }
            // let result = test();

            self.add_reference(
              symbol_ext.clone(),
              IdOrMemExpr::Id(default_ident.to_id().into()),
            );
            self.add_reference(
              BetterId::from(ident.to_id()).into(),
              IdOrMemExpr::Id(symbol_ext.id.clone()),
            );
            symbol_ext
          } else {
            SymbolExt::new(default_ident.to_id().into(), SymbolFlag::EXPORT_DEFAULT)
          };
          self.export_default_name = Some(symbol_ext.id().atom.clone());
          symbol_ext
        }
      };
      let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();

      self.current_body_owner_symbol_ext = Some(body_owner_extend_symbol);
      node.function.visit_with(self);
      self.current_body_owner_symbol_ext = before_owner_extend_symbol;
    } else {
      // if the function expr is not inside a default expr, it will not
      // generate a binding.
      node.function.visit_with(self);
    }
  }

  fn visit_class_decl(&mut self, node: &ClassDecl) {
    let id: BetterId = node.ident.to_id().into();
    let mark = id.ctxt.outer();
    let old_region = self.current_body_owner_symbol_ext.clone();
    if mark == self.top_level_mark {
      self.current_body_owner_symbol_ext = Some(id.into());
    }
    node.visit_children_with(self);
    self.current_body_owner_symbol_ext = old_region;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    let id: BetterId = node.ident.to_id().into();
    let mark = id.ctxt.outer();
    let before_symbol_ext = self.current_body_owner_symbol_ext.clone();
    if mark == self.top_level_mark {
      self.current_body_owner_symbol_ext = Some(id.into());
    }
    node.function.visit_with(self);
    self.current_body_owner_symbol_ext = before_symbol_ext;
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
      let is_export = self.state.contains(AnalyzeState::EXPORT_DECL);
      if is_export {
        self.add_export(
          lhs.atom.clone(),
          SymbolRef::Direct(Symbol::from_id_and_uri(lhs.clone(), self.module_identifier)),
        );
      }
      if let Some(ref init) = ele.init && lhs.ctxt.outer() == self.top_level_mark {
        let mut symbol_ext = SymbolExt::new(lhs, SymbolFlag::VAR_DECL);
        if is_export {
          symbol_ext.flag.insert(SymbolFlag::EXPORT);
        }
        let before_symbol_ext = self.current_body_owner_symbol_ext.clone();
        self.current_body_owner_symbol_ext = Some(symbol_ext);
        init.visit_with(self);
        self.current_body_owner_symbol_ext = before_symbol_ext;
      }
    }
  }
  fn visit_decl(&mut self, node: &Decl) {
    match node {
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        // TODO: Ignore ts related tree-shaking for now.
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
      Entry::Occupied(_) => {
        // TODO: should add some Diagnostic
      }
      Entry::Vacant(vac) => {
        vac.insert(symbol);
      }
    }
  }

  fn add_import(&mut self, id: BetterId, symbol: SymbolRef) {
    match self.import_map.entry(id) {
      Entry::Occupied(_) => {
        // TODO: should add some Diagnostic
      }
      Entry::Vacant(vac) => {
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
          ExportSpecifier::Namespace(namespace) => {
            let atom = match namespace.name {
              ModuleExportName::Ident(ref ident) => ident.sym.clone(),
              ModuleExportName::Str(ref str) => str.value.clone(),
            };
            self.add_export(atom, SymbolRef::Star(resolved_uri_ukey));
          }
          ExportSpecifier::Default(_) => {
            // Currently swc does not support syntax like `export v from 'xxx';
            // Since this is a syntax error the logic should not reach here.`
            unreachable!("Module has syntax error should not trigger tree_shaking")
          }
          ExportSpecifier::Named(named) => {
            // TODO: what if the named binding is a unresolved_binding?
            // TODO: handle `as xxx`
            let original = match &named.orig {
              ModuleExportName::Ident(ident) => ident.sym.clone(),
              ModuleExportName::Str(_) => {
                todo!()
              }
            };
            let exported = match &named.exported {
              Some(exported) => match exported {
                ModuleExportName::Ident(ident) => ident.sym.clone(),
                ModuleExportName::Str(str) => str.value.clone(),
              },
              None => original.clone(),
            };
            let symbol_ref = SymbolRef::Indirect(IndirectTopLevelSymbol::new(
              resolved_uri_ukey,
              original,
              self.module_identifier,
            ));
            self.add_export(exported, symbol_ref);
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
            unreachable!("Module has syntax error should not reach tree shaking analyze")
          }
          ExportSpecifier::Named(named) => {
            // TODO: what if the named binding is a unresolved_binding?
            // TODO: handle `as xxx`
            let id = match &named.orig {
              ModuleExportName::Ident(ident) => ident.to_id(),
              // `export {'a'}` is a syntax error;
              // we know here export has no src,  so this branch should not reachable.
              ModuleExportName::Str(_) => unreachable!(),
            };

            let exported_atom = match named.exported {
              Some(ref exported) => match exported {
                ModuleExportName::Ident(ident) => ident.sym.clone(),
                ModuleExportName::Str(str) => str.value.clone(),
              },
              None => id.0.clone(),
            };
            let symbol_ref =
              SymbolRef::Direct(Symbol::from_id_and_uri(id.into(), self.module_identifier));
            self.add_export(exported_atom, symbol_ref);
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
}

/// The `allow(unused)` will be removed after the Tree shaking is finished
#[derive(Debug)]
#[allow(unused)]
pub struct TreeShakingResult {
  top_level_mark: Mark,
  unresolved_mark: Mark,
  pub module_identifier: Ustr,
  pub export_map: HashMap<JsWord, SymbolRef>,
  pub(crate) import_map: HashMap<BetterId, SymbolRef>,
  pub inherit_export_maps: IndexMap<Ustr, HashMap<JsWord, SymbolRef>>,
  // current_region: Option<BetterId>,
  // pub(crate) reference_map: HashMap<BetterId, HashSet<BetterId>>,
  pub(crate) reachable_import_of_export: HashMap<JsWord, HashSet<SymbolRef>>,
  state: AnalyzeState,
  pub(crate) used_symbol_ref: HashSet<SymbolRef>,
}

impl From<ModuleRefAnalyze<'_>> for TreeShakingResult {
  fn from(mut analyze: ModuleRefAnalyze<'_>) -> Self {
    Self {
      top_level_mark: std::mem::take(&mut analyze.top_level_mark),
      unresolved_mark: std::mem::take(&mut analyze.unresolved_mark),
      module_identifier: std::mem::take(&mut analyze.module_identifier),
      export_map: std::mem::take(&mut analyze.export_map),
      import_map: std::mem::take(&mut analyze.import_map),
      inherit_export_maps: std::mem::take(&mut analyze.inherit_export_maps),
      // current_region: std::mem::take(&mut analyze.current_body_owner_id),
      // reference_map: std::mem::take(&mut analyze.reference_map),
      reachable_import_of_export: std::mem::take(&mut analyze.reachable_import_and_export),
      state: std::mem::take(&mut analyze.state),
      used_symbol_ref: std::mem::take(&mut analyze.used_symbol_ref),
    }
  }
}
