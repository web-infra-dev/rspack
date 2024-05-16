use std::borrow::Cow;
use std::{collections::hash_map::Entry, collections::VecDeque, hash::Hash, path::PathBuf};

use bitflags::bitflags;
use glob_to_regexp::{glob_to_regexp, GlobOptions};
use hashlink::{LinkedHashMap, LinkedHashSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use serde::Serialize;
use swc_core::common::{util::take::Take, GLOBALS};
use swc_core::common::{Span, Spanned, SyntaxContext};
use swc_core::ecma::ast::*;
use swc_core::ecma::atoms::Atom;
use swc_core::ecma::utils::{ExprCtx, ExprExt};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};
use swc_node_comments::SwcComments;

use super::symbol::{
  BetterId, IndirectTopLevelSymbol, IndirectType, Part, StarSymbol, StarSymbolKind, Symbol,
  SymbolExt, SymbolFlag, SymbolType,
};
use super::SideEffectType;
use super::{
  utils::{get_dynamic_import_string_literal, get_require_literal},
  BailoutFlag,
};
use crate::needs_refactor::WorkerSyntaxList;
use crate::{
  extract_member_expression_chain, BoxDependency, CompilerOptions, DependencyId, DependencyType,
  ErrorSpan, FactoryMeta, ModuleGraph, ModuleIdentifier, ModuleSyntax,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ModuleIdOrDepId {
  ModuleId(ModuleIdentifier),
  DepId(DependencyId),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub enum SymbolRef {
  Declaration(Symbol),
  Indirect(IndirectTopLevelSymbol),
  Star(StarSymbol),
  Usage(Atom, Vec<Atom>, ModuleIdentifier),
  Url {
    importer: ModuleIdentifier,
    src: ModuleIdentifier,
    dep_id: DependencyId,
  },
  Worker {
    importer: ModuleIdentifier,
    src: ModuleIdentifier,
    dep_id: DependencyId,
  },
}

impl SymbolRef {
  pub fn src(&self) -> ModuleIdentifier {
    match self {
      SymbolRef::Declaration(d) => d.src(),
      SymbolRef::Indirect(i) => i.src(),
      SymbolRef::Star(s) => s.src(),
      SymbolRef::Url { src, .. } => *src,
      SymbolRef::Worker { src, .. } => *src,
      SymbolRef::Usage(_, _, src) => *src,
    }
  }

  pub fn update_src_from_dep_id(mut self, mg: &ModuleGraph) -> SymbolRef {
    match self {
      SymbolRef::Declaration(_) => {}
      SymbolRef::Indirect(ref mut i) => {
        if i.src.is_empty()
          && let Some(module_id) = mg.module_identifier_by_dependency_id(&i.dep_id)
        {
          i.src = *module_id;
        }
      }
      SymbolRef::Star(ref mut s) => {
        if let Some(module_id) = mg.module_identifier_by_dependency_id(&s.dep_id) {
          s.src = *module_id;
        }
      }
      SymbolRef::Url {
        dep_id,
        ref mut src,
        ..
      } => {
        if let Some(module_id) = mg.module_identifier_by_dependency_id(&dep_id) {
          *src = *module_id;
        }
      }
      SymbolRef::Worker {
        ref mut src,
        dep_id,
        ..
      } => {
        if let Some(module_id) = mg.module_identifier_by_dependency_id(&dep_id) {
          *src = *module_id;
        }
      }
      SymbolRef::Usage(_, _, _) => {}
    }
    self
  }

  pub fn importer(&self) -> ModuleIdentifier {
    match self {
      SymbolRef::Declaration(d) => d.src(),
      SymbolRef::Indirect(i) => i.importer,
      SymbolRef::Star(s) => s.module_ident(),
      SymbolRef::Url { importer, .. } => *importer,
      SymbolRef::Worker { importer, .. } => *importer,
      SymbolRef::Usage(_, _, src) => *src,
    }
  }
  /// Returns `true` if the symbol ref is [`Direct`].
  ///
  /// [`Direct`]: SymbolRef::Direct
  #[must_use]
  pub fn is_direct(&self) -> bool {
    matches!(self, Self::Declaration(..))
  }

  /// Returns `true` if the symbol ref is [`Indirect`].
  ///
  /// [`Indirect`]: SymbolRef::Indirect
  #[must_use]
  pub fn is_indirect(&self) -> bool {
    matches!(self, Self::Indirect(..))
  }

  /// Returns `true` if the symbol ref is [`Star`].
  ///
  /// [`Star`]: SymbolRef::Star
  #[must_use]
  pub fn is_star(&self) -> bool {
    matches!(self, Self::Star(..))
  }

  pub fn is_skipable_symbol(&self) -> bool {
    match self {
      SymbolRef::Indirect(indirect) => !indirect.is_import(),
      SymbolRef::Star(StarSymbol {
        ty: StarSymbolKind::ReExportAll,
        ..
      }) => true,
      _ => false,
    }
  }
}

bitflags! {
  #[derive(Debug, Default, Clone, Copy)]
  struct AnalyzeState: u8 {
    const EXPORT_DECL = 1 << 0;
    const EXPORT_DEFAULT = 1 << 1;
    const ASSIGNMENT_LHS = 1 << 2;
    const ASSIGNMENT_RHS = 1 << 3;
    const STATIC_VAR_DECL = 1 << 4;
  }
}
pub(crate) struct ModuleRefAnalyze<'a> {
  top_level_ctxt: SyntaxContext,
  unresolved_ctxt: SyntaxContext,
  module_identifier: ModuleIdentifier,
  dependencies: &'a Vec<BoxDependency>,
  /// Value of `export_map` must have type [SymbolRef::Direct]
  pub(crate) export_map: HashMap<Atom, SymbolRef>,
  pub(crate) import_map: HashMap<Atom, SymbolRef>,
  /// key is the module identifier, value is the corresponding export map
  /// This data structure is used for collecting reexport * from some module. e.g.
  /// ```js
  /// export * from './test.js'
  /// ```
  /// then inherit_exports_maps become, `{"test.js": {...test_js_export_map} }`
  // Use `IndexMap` to keep the insertion order
  pub export_all_dep_id: LinkedHashSet<DependencyId>,
  current_body_owner_symbol_ext: Option<SymbolExt>,
  pub(crate) maybe_lazy_reference_map: HashMap<SymbolExt, HashSet<Part>>,
  pub(crate) immediate_evaluate_reference_map: HashMap<SymbolExt, HashSet<Part>>,
  pub(crate) reachable_import_and_export: HashMap<Atom, HashSet<SymbolRef>>,
  state: AnalyzeState,
  pub(crate) used_id_set: HashSet<Part>,
  pub(crate) used_symbol_ref: HashSet<SymbolRef>,
  // This field is used for duplicated export default checking
  pub(crate) export_default_name: Option<Atom>,
  /// only care about the related export semantic.
  /// # Examples
  /// 1. `require()` -> CommonJs
  /// 2. `export ` -> ESM
  module_syntax: ModuleSyntax,
  pub(crate) bail_out_module_identifiers: HashMap<ModuleIdOrDepId, BailoutFlag>,
  pub(crate) side_effects: SideEffectType,
  pub(crate) options: &'a CompilerOptions,
  pub(crate) has_side_effects_stmt: bool,
  pub(crate) potential_top_level_ctxt: HashSet<SyntaxContext>,
  worker_syntax_list: &'a WorkerSyntaxList,
  /// record harmony_import_specifier_dependency_id -> related SymbolRef
  pub harmony_import_specifier_dependency_alias_map: HashMap<Span, SymbolRef>,
}

impl<'a> std::fmt::Debug for ModuleRefAnalyze<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ModuleRefAnalyze")
      .field("top_level_mark", &self.top_level_ctxt)
      .field("unresolved_mark", &self.unresolved_ctxt)
      .field("module_identifier", &self.module_identifier)
      .field("dependencies", &"..".to_string())
      .field("export_map", &self.export_map)
      .field("import_map", &self.import_map)
      .field("export_all_dep_id", &self.export_all_dep_id)
      .field(
        "current_body_owner_symbol_ext",
        &self.current_body_owner_symbol_ext,
      )
      .field("maybe_lazy_reference_map", &self.maybe_lazy_reference_map)
      .field(
        "immediate_evaluate_reference_map",
        &self.immediate_evaluate_reference_map,
      )
      .field(
        "reachable_import_and_export",
        &self.reachable_import_and_export,
      )
      .field("state", &self.state)
      .field("used_id_set", &self.used_id_set)
      .field("used_symbol_ref", &self.used_symbol_ref)
      .field("export_default_name", &self.export_default_name)
      .field("module_syntax", &self.module_syntax)
      .field(
        "bail_out_module_identifiers",
        &self.bail_out_module_identifiers,
      )
      .field("side_effects", &self.side_effects)
      .field("options", &self.options)
      .field("has_side_effects_stmt", &self.has_side_effects_stmt)
      .field("unresolved_ctxt", &self.unresolved_ctxt)
      .field("potential_top_mark", &self.potential_top_level_ctxt)
      .field("comments", &"...")
      .field("worker_syntax_list", self.worker_syntax_list)
      .finish()
  }
}

pub struct SyntaxContextInfo {
  top_level_ctxt: SyntaxContext,
  unresolved_ctxt: SyntaxContext,
}

impl SyntaxContextInfo {
  pub fn new(top_level_ctxt: SyntaxContext, unresolved_ctxt: SyntaxContext) -> Self {
    Self {
      top_level_ctxt,
      unresolved_ctxt,
    }
  }
}

impl<'a> ModuleRefAnalyze<'a> {
  pub fn new(
    mark_info: SyntaxContextInfo,
    module_identifier: ModuleIdentifier,
    dependencies: &'a Vec<BoxDependency>,
    options: &'a CompilerOptions,
    _comments: Option<&'a SwcComments>,
    worker_syntax_list: &'a WorkerSyntaxList,
  ) -> Self {
    Self {
      top_level_ctxt: mark_info.top_level_ctxt,
      unresolved_ctxt: mark_info.unresolved_ctxt,
      module_identifier,
      dependencies,
      export_map: HashMap::default(),
      import_map: HashMap::default(),
      current_body_owner_symbol_ext: None,
      maybe_lazy_reference_map: HashMap::default(),
      reachable_import_and_export: HashMap::default(),
      state: AnalyzeState::empty(),
      used_id_set: HashSet::default(),
      used_symbol_ref: HashSet::default(),
      export_default_name: None,
      module_syntax: ModuleSyntax::empty(),
      bail_out_module_identifiers: HashMap::default(),
      side_effects: SideEffectType::Analyze(true),
      immediate_evaluate_reference_map: HashMap::default(),
      options,
      has_side_effects_stmt: false,
      potential_top_level_ctxt: HashSet::from_iter([mark_info.top_level_ctxt]),
      worker_syntax_list,
      export_all_dep_id: LinkedHashSet::default(),
      harmony_import_specifier_dependency_alias_map: Default::default(),
    }
  }

  /// Some times we want to insert the reference anyway, even if `from == to`
  /// e.g.
  /// ```js
  /// import xxx from 'xxx'
  /// xxx.test = aaa;
  /// ```
  pub fn add_reference(&mut self, from: SymbolExt, to: Part, force_insert: bool) {
    if matches!(&to, Part::TopLevelId(to_id) if to_id == from.id()) && !force_insert {
      return;
    }
    // TODO: refactor this to use intersects
    if self
      .state
      .intersection(
        AnalyzeState::ASSIGNMENT_RHS | AnalyzeState::ASSIGNMENT_LHS | AnalyzeState::STATIC_VAR_DECL,
      )
      .bits()
      .count_ones()
      >= 1
    {
      match self.immediate_evaluate_reference_map.entry(from) {
        Entry::Occupied(mut occ) => {
          occ.get_mut().insert(to);
        }
        Entry::Vacant(vac) => {
          vac.insert(HashSet::from_iter([to]));
        }
      }
    } else {
      match self.maybe_lazy_reference_map.entry(from) {
        Entry::Occupied(mut occ) => {
          occ.get_mut().insert(to);
        }
        Entry::Vacant(vac) => {
          vac.insert(HashSet::from_iter([to]));
        }
      }
    }
  }

  /// Collecting all reachable import binding from given start binding
  /// when a export has been used from other module, we need to get all
  /// reachable import and export(defined in the same module)
  /// in rest of scenario we only count binding imported from other module.
  pub fn get_all_import_or_export(&self, start: Atom, only_import: bool) -> HashSet<SymbolRef> {
    let mut visited: HashSet<Part> = HashSet::default();
    let mut q: VecDeque<Part> = VecDeque::from_iter([Part::TopLevelId(start)]);
    while let Some(cur) = q.pop_front() {
      if visited.contains(&cur) {
        continue;
      }
      let id = match cur.get_id() {
        Some(id) => id,
        None => {
          visited.insert(cur);
          continue;
        }
      };
      if let Some(ref_list) = self.maybe_lazy_reference_map.get(&id.clone().into()) {
        q.extend(ref_list.clone());
      }
      visited.insert(cur);
    }
    // dbg!(&start, &seen, &self.reference_map);
    return visited
      .iter()
      .filter_map(|part| match part {
        Part::TopLevelId(id) => {
          let ret = self.import_map.get(id).cloned().or_else(|| {
            if only_import {
              None
            } else {
              match self.export_map.get(id) {
                Some(sym_ref @ SymbolRef::Declaration(sym)) => {
                  if &sym.id().atom == id {
                    Some(sym_ref.clone())
                  } else {
                    None
                  }
                }
                _ => None,
              }
            }
          });
          ret
        }
        Part::MemberExpr {
          first: object,
          rest: property,
        } => self.import_map.get(object).map(|sym_ref| match sym_ref {
          SymbolRef::Indirect(_) => {
            SymbolRef::Usage(object.clone(), property.clone(), self.module_identifier)
          }
          SymbolRef::Star(_) => {
            SymbolRef::Usage(object.clone(), property.clone(), self.module_identifier)
          }
          SymbolRef::Url { .. }
          | SymbolRef::Worker { .. }
          | SymbolRef::Declaration(_)
          | SymbolRef::Usage(..) => unreachable!(),
        }),
        Part::Url(src) => {
          let dep_id = self
            .resolve_module_identifier(src, &[DependencyType::NewUrl])
            .unwrap_or_else(|| panic!("Can't resolve {} in {}", src, self.module_identifier));

          Some(SymbolRef::Url {
            importer: self.module_identifier,
            src: "".into(),
            dep_id,
          })
        }
        Part::Worker(src) => {
          let dep_id = self
            .resolve_module_identifier(src, &[DependencyType::NewWorker])
            .unwrap_or_else(|| panic!("Can't resolve {} in {}", src, self.module_identifier));

          Some(SymbolRef::Url {
            importer: self.module_identifier,
            src: "".into(),
            dep_id,
          })
        }
      })
      .collect();
  }

  fn generate_default_ident(&self) -> Ident {
    let mut default_ident = Ident::dummy();
    default_ident.sym = "default".into();
    default_ident.span = default_ident.span.with_ctxt(self.top_level_ctxt);
    default_ident
  }

  fn check_commonjs_feature(&mut self, member_chain: &[Cow<(Atom, SyntaxContext)>]) {
    if self.state.contains(AnalyzeState::ASSIGNMENT_LHS) {
      let member_chain = member_chain.iter().map(|m| &**m).collect::<Vec<_>>();
      match &*member_chain {
        [(first, first_ctxt), (second, _), ..]
          if first == "module" && second == "exports" && first_ctxt == &self.unresolved_ctxt => {}
        [(first, first_ctxt), ..] if first == "exports" && &self.unresolved_ctxt == first_ctxt => {}
        _ => return,
      }
      self.module_syntax.insert(ModuleSyntax::COMMONJS);
      match self
        .bail_out_module_identifiers
        .entry(ModuleIdOrDepId::ModuleId(self.module_identifier))
      {
        Entry::Occupied(mut occ) => {
          *occ.get_mut() |= BailoutFlag::COMMONJS_EXPORTS;
        }
        Entry::Vacant(vac) => {
          vac.insert(BailoutFlag::COMMONJS_EXPORTS);
        }
      }
    }
  }
}

impl<'a> Visit for ModuleRefAnalyze<'a> {
  noop_visit_type!();
  fn visit_program(&mut self, node: &Program) {
    assert!(GLOBALS.is_set());
    node.visit_children_with(self);
    // calc reachable imports for each export symbol defined in current module
    for (_, symbol) in self.export_map.iter() {
      match symbol {
        // At this time uri of symbol will always equal to `self.module_identifier`
        SymbolRef::Declaration(symbol) => {
          let reachable_import_and_export =
            self.get_all_import_or_export(symbol.id().atom.clone(), true);
          self
            .reachable_import_and_export
            .insert(symbol.exported().clone(), reachable_import_and_export);
        }
        // ignore any indirect symbol, because it will not generate binding, the reachable exports will
        // be calculated in the module where it is defined
        SymbolRef::Indirect(_) | SymbolRef::Star(_) => {}
        SymbolRef::Url { .. } | SymbolRef::Worker { .. } => {}
        SymbolRef::Usage(..) => {}
      }
    }
    // Any var declaration has reference a symbol from other module, it is marked as used
    // Because the symbol import from other module possibly has side effect
    let side_effect_symbol_list = self
      .maybe_lazy_reference_map
      .iter()
      .flat_map(|(symbol, ref_list)| {
        // it class decl, fn decl is lazy they don't immediately generate side effects unless they are called,
        // or constructed. The initializer of var decl will evaluate except rhs is function expr or arrow expr.
        if !symbol.flag.contains(SymbolFlag::VAR_DECL)
          && !symbol.flag.contains(SymbolFlag::EXPORT_DEFAULT)
        {
          vec![]
        } else {
          // like
          // ```js
          // export var a = function () {}
          // ```
          // a is still lazy
          if symbol
            .flag
            .intersection(
              SymbolFlag::FUNCTION_EXPR
                | SymbolFlag::ARROW_EXPR
                | SymbolFlag::CLASS_EXPR
                | SymbolFlag::ALIAS,
            )
            .bits()
            .count_ones()
            >= 1
          {
            return vec![];
          }
          ref_list
            .iter()
            .flat_map(|ref_part| {
              // Only used id imported from other module would generate a side effects.
              let id = match ref_part {
                Part::TopLevelId(ref id) => id,
                Part::MemberExpr {
                  first: object,
                  rest: property,
                } => match self.import_map.get(object) {
                  Some(_) => {
                    return HashSet::from_iter([SymbolRef::Usage(
                      object.clone(),
                      property.clone(),
                      self.module_identifier,
                    )]);
                  }
                  _ => object,
                },
                Part::Url(src) => {
                  let dep_id = self
                    .resolve_module_identifier(src, &[DependencyType::NewUrl])
                    .unwrap_or_else(|| {
                      panic!("Can't resolve {} in {}", src, self.module_identifier)
                    });
                  return HashSet::from_iter([SymbolRef::Url {
                    importer: self.module_identifier,
                    src: "".into(),
                    dep_id,
                  }]);
                }
                Part::Worker(src) => {
                  let dep_id = self
                    .resolve_module_identifier(src, &[DependencyType::NewWorker])
                    .unwrap_or_else(|| {
                      panic!("Can't resolve {} in {}", src, self.module_identifier)
                    });
                  return HashSet::from_iter([SymbolRef::Url {
                    importer: self.module_identifier,
                    src: "".into(),
                    dep_id,
                  }]);
                }
              };
              let ret = self.import_map.get(id);
              match ret {
                Some(ret) => HashSet::from_iter([ret.clone()]),
                None => self.get_all_import_or_export(id.clone(), true),
              }
            })
            .collect::<Vec<_>>()
        }
      })
      .collect::<Vec<_>>();
    self.used_symbol_ref.extend(side_effect_symbol_list);
    let side_effect_symbol_list = self
      .immediate_evaluate_reference_map
      .iter()
      .flat_map(|(_, ref_list)| {
        // it class decl, fn decl is lazy they don't immediately generate side effects unless they are called,
        // Or constructed. The init of var decl will evaluate except rhs is function expr or arrow expr.
        ref_list
          .iter()
          .flat_map(|ref_part| {
            // Only used id imported from other module would generate a side effects.
            let id = match ref_part {
              Part::TopLevelId(ref id) => id,
              Part::MemberExpr {
                first: object,
                rest: property,
              } => match self.import_map.get(object) {
                Some(_) => {
                  return HashSet::from_iter([SymbolRef::Usage(
                    object.clone(),
                    property.clone(),
                    self.module_identifier,
                  )]);
                }
                _ => object,
              },
              Part::Url(src) => {
                let dep_id = self
                  .resolve_module_identifier(src, &[DependencyType::NewUrl])
                  .unwrap_or_else(|| panic!("Can't resolve {} in {}", src, self.module_identifier));

                return HashSet::from_iter([SymbolRef::Url {
                  importer: self.module_identifier,
                  src: "".into(),
                  dep_id,
                }]);
              }
              Part::Worker(src) => {
                let dep_id = self
                  .resolve_module_identifier(src, &[DependencyType::NewWorker])
                  .unwrap_or_else(|| panic!("Can't resolve {} in {}", src, self.module_identifier));

                return HashSet::from_iter([SymbolRef::Url {
                  importer: self.module_identifier,
                  src: "".into(),
                  dep_id,
                }]);
              }
            };
            let ret = self.import_map.get(id);
            match ret {
              Some(ret) => HashSet::from_iter([ret.clone()]),
              None => self.get_all_import_or_export(id.clone(), true),
            }
          })
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();
    self.used_symbol_ref.extend(side_effect_symbol_list);
    // all reachable export from used symbol in current module
    for used_id in &self.used_id_set {
      match used_id {
        Part::TopLevelId(id) => {
          let reachable_import = self.get_all_import_or_export(id.clone(), true);
          self.used_symbol_ref.extend(reachable_import);
        }
        Part::MemberExpr {
          first: object,
          rest: property,
        } => match self.import_map.get(object) {
          Some(_) => {
            self.used_symbol_ref.insert(SymbolRef::Usage(
              object.clone(),
              property.clone(),
              self.module_identifier,
            ));
          }
          _ => {
            let reachable_import = self.get_all_import_or_export(object.clone(), true);
            self.used_symbol_ref.extend(reachable_import);
          }
        },
        Part::Url(src) => {
          let dep_id = self
            .resolve_module_identifier(src, &[DependencyType::NewUrl])
            .unwrap_or_else(|| panic!("Can't resolve {} in {}", src, self.module_identifier));

          let url = SymbolRef::Url {
            importer: self.module_identifier,
            src: "".into(),
            dep_id,
          };
          self.used_symbol_ref.insert(url);
        }
        Part::Worker(src) => {
          let dep_id = self
            .resolve_module_identifier(src, &[DependencyType::NewWorker])
            .unwrap_or_else(|| panic!("Can't resolve {} in {}", src, self.module_identifier));

          let url = SymbolRef::Url {
            importer: self.module_identifier,
            src: "".into(),
            dep_id,
          };
          self.used_symbol_ref.insert(url);
        }
      }
    }

    let side_effects_option = self.options.optimization.side_effects;
    if side_effects_option.is_enable() {
      self.side_effects = if side_effects_option.is_true() {
        SideEffectType::Analyze(self.has_side_effects_stmt)
      } else {
        // side_effects_option must be `flag` here
        SideEffectType::Configuration(true)
      };
    }
  }

  fn visit_module(&mut self, node: &Module) {
    // prescan import decl
    for module_item in &node.body {
      if is_import_decl(module_item) {
        module_item.visit_with(self);
      }
    }
    for module_item in &node.body {
      if !is_import_decl(module_item) {
        self.analyze_module_item_side_effects(module_item);
        module_item.visit_with(self);
      }
    }
  }

  fn visit_script(&mut self, node: &Script) {
    for stmt in &node.body {
      self.analyze_stmt_side_effects(stmt);
      stmt.visit_with(self);
    }
  }

  fn visit_ident(&mut self, node: &Ident) {
    let id: BetterId = node.to_id().into();
    let ctxt = id.ctxt;

    if self.potential_top_level_ctxt.contains(&ctxt) {
      match self.current_body_owner_symbol_ext {
        Some(ref body_owner_symbol_ext) if body_owner_symbol_ext.id() != &id.atom => {
          self.add_reference(
            body_owner_symbol_ext.clone(),
            Part::TopLevelId(id.atom),
            false,
          );
        }
        None => {
          self.used_id_set.insert(Part::TopLevelId(id.atom));
        }
        _ => {}
      }
    }
  }

  fn visit_new_expr(&mut self, new_expr: &NewExpr) {
    if self.worker_syntax_list.match_new_worker(new_expr)
      && let Some(args) = &new_expr.args
    {
      new_expr.callee.visit_with(self);
      if let Some(ExprOrSpread {
        expr: box Expr::New(new_expr),
        ..
      }) = args.first()
      {
        if let Some((_, _, request)) = crate::needs_refactor::match_new_url(new_expr) {
          let src = Part::Worker(request.into());
          match self.current_body_owner_symbol_ext {
            Some(ref body_owner_symbol_ext) => {
              self.add_reference(body_owner_symbol_ext.clone(), src, false);
            }
            None => {
              self.used_id_set.insert(src);
            }
          }
          if let Some(args) = &new_expr.args {
            for ele in args.iter().skip(1) {
              ele.visit_with(self);
            }
          }
          return;
        }
      }
    }
    if let Some((_, _, request)) = crate::needs_refactor::match_new_url(new_expr) {
      let src = Part::Url(request.into());
      match self.current_body_owner_symbol_ext {
        Some(ref body_owner_symbol_ext) => {
          self.add_reference(body_owner_symbol_ext.clone(), src, false);
        }
        None => {
          self.used_id_set.insert(src);
        }
      }
    }
    new_expr.callee.visit_with(self);
    if let Some(args) = &new_expr.args {
      for ele in args {
        ele.visit_with(self);
      }
    }
  }

  fn visit_module_item(&mut self, node: &ModuleItem) {
    match node {
      ModuleItem::ModuleDecl(decl) => {
        self.module_syntax.insert(ModuleSyntax::ESM);
        match decl {
          ModuleDecl::Import(import) => {
            let src = &import.src.value;
            let dep_id = match self.resolve_module_identifier(
              src,
              &[DependencyType::EsmImport(ErrorSpan::from(import.span))],
            ) {
              Some(module_identifier) => module_identifier,
              None => {
                // TODO: Ignore for now because swc helper interference.
                return;
              }
            };
            import
              .specifiers
              .iter()
              .for_each(|specifier| match specifier {
                ImportSpecifier::Named(named) => {
                  let imported = named.imported.as_ref().map(|imported| match imported {
                    ModuleExportName::Ident(ident) => ident.sym.clone(),
                    ModuleExportName::Str(str) => str.value.clone(),
                  });

                  let local = named.local.sym.clone();

                  let symbol_ref = SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                    "".into(),
                    self.module_identifier,
                    IndirectType::Import(local, imported),
                    dep_id,
                  ));

                  self.add_import(named.local.to_id().into(), symbol_ref);
                }
                ImportSpecifier::Default(default) => {
                  self.add_import(
                    default.local.to_id().into(),
                    SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                      "".into(),
                      self.module_identifier,
                      IndirectType::ImportDefault(default.local.sym.clone()),
                      dep_id,
                    )),
                  );
                }
                ImportSpecifier::Namespace(namespace) => {
                  self.add_import(
                    namespace.local.to_id().into(),
                    SymbolRef::Star(StarSymbol::new(
                      "".into(),
                      namespace.local.sym.clone(),
                      self.module_identifier,
                      StarSymbolKind::ImportAllAs,
                      dep_id,
                    )),
                  );
                }
              });
          }
          ModuleDecl::ExportDecl(decl) => match &decl.decl {
            Decl::Class(class) => {
              class.visit_with(self);
              self.add_export(
                class.ident.sym.clone(),
                SymbolRef::Declaration(Symbol::new(
                  self.module_identifier,
                  class.ident.to_id().into(),
                  SymbolType::Define,
                  None,
                )),
              );
            }
            Decl::Fn(function) => {
              function.visit_with(self);
              self.add_export(
                function.ident.sym.clone(),
                SymbolRef::Declaration(Symbol::new(
                  self.module_identifier,
                  function.ident.to_id().into(),
                  SymbolType::Define,
                  None,
                )),
              );
            }
            Decl::Using(_) => {
              // TODO(hyf0): swc bump
              unimplemented!()
            }
            Decl::Var(var) => {
              let pre_state = self.state;
              self.state.insert(AnalyzeState::EXPORT_DECL);
              var.visit_with(self);
              self.state = pre_state;
            }
            Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
              unreachable!("We have been converted Typescript to javascript already")
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
            let dep_id = match self.resolve_module_identifier(
              &export_all.src.value,
              &[DependencyType::EsmExport(export_all.span.into())],
            ) {
              Some(module_identifier) => module_identifier,
              None => {
                // TODO: ignore for now, or three copy js will failed
                return;
              }
            };
            self.export_all_dep_id.insert(dep_id);
          }
          ModuleDecl::TsImportEquals(_)
          | ModuleDecl::TsExportAssignment(_)
          | ModuleDecl::TsNamespaceExport(_) => {
            unreachable!("We have been converted Typescript to javascript already")
          }
        }
      }
      ModuleItem::Stmt(stmt) => {
        stmt.visit_children_with(self);
      }
    }
  }

  fn visit_export_default_expr(&mut self, node: &ExportDefaultExpr) {
    let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();
    let default_ident: BetterId = self.generate_default_ident().to_id().into();
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
    let mut symbol_ext: SymbolExt = default_ident.atom.clone().into();
    symbol_ext.flag.insert(SymbolFlag::EXPORT_DEFAULT);
    match node.expr {
      box Expr::Fn(_) => symbol_ext.flag.insert(SymbolFlag::FUNCTION_EXPR),
      box Expr::Arrow(_) => symbol_ext.flag.insert(SymbolFlag::ARROW_EXPR),
      box Expr::Ident(_) => symbol_ext.flag.insert(SymbolFlag::ALIAS),
      _ => {}
    };

    let export_symbol = SymbolRef::Declaration(Symbol::new(
      self.module_identifier,
      default_ident.clone(),
      SymbolType::Define,
      None,
    ));
    if symbol_ext.flag.contains(SymbolFlag::ALIAS) {
      self
        .harmony_import_specifier_dependency_alias_map
        .insert(node.expr.span(), export_symbol.clone());
    };

    self.add_export(default_ident.atom.clone(), export_symbol);
    self.current_body_owner_symbol_ext = Some(symbol_ext);
    node.visit_children_with(self);
    self.current_body_owner_symbol_ext = before_owner_extend_symbol;
  }

  fn visit_assign_expr(&mut self, node: &AssignExpr) {
    let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();
    let target = if before_owner_extend_symbol.is_none() {
      let target = first_ident_of_assign_lhs(node);
      target.and_then(|target| {
        if target.1 == self.top_level_ctxt {
          Some(target)
        } else {
          None
        }
      })
    } else {
      None
    };
    let valid_assign_target = target.is_some();
    if let Some(target) = target {
      self.current_body_owner_symbol_ext = Some(SymbolExt {
        id: target.0,
        flag: SymbolFlag::empty(),
      });
    }

    let mut pre_state = self.state;
    self.state.insert(AnalyzeState::ASSIGNMENT_LHS);
    node.left.visit_with(self);
    // cargo clippy told me to do this..
    std::mem::swap(&mut self.state, &mut pre_state);

    let pre_state = self.state;
    if valid_assign_target {
      self.state.insert(AnalyzeState::ASSIGNMENT_RHS);
    }
    node.right.visit_with(self);
    self.state = pre_state;
    self.current_body_owner_symbol_ext = before_owner_extend_symbol;
  }

  fn visit_class_prop(&mut self, node: &ClassProp) {
    node.key.visit_with(self);
    if let Some(ref expr) = node.value {
      match expr {
        box Expr::Fn(_) | box Expr::Arrow(_) => {
          expr.visit_with(self);
        }
        _ => {
          let pre_state = self.state;
          self.state.insert(AnalyzeState::STATIC_VAR_DECL);
          expr.visit_with(self);
          self.state = pre_state;
        }
      }
    }
  }

  fn visit_member_expr(&mut self, node: &MemberExpr) {
    let expression_info = extract_member_expression_chain(node);
    let member_chain = expression_info.members().into_iter().collect::<Vec<_>>();
    self.check_commonjs_feature(&member_chain);
    if !member_chain.is_empty() {
      let (first, first_ctxt) = member_chain[0].clone().into_owned();
      if self.potential_top_level_ctxt.contains(&first_ctxt) {
        let member_expr = Part::MemberExpr {
          first: first.clone(),
          rest: member_chain
            .into_iter()
            .skip(1)
            // .take(1)
            .map(|m| m.0.clone())
            .collect::<Vec<_>>(),
        };
        match self.current_body_owner_symbol_ext {
          Some(ref body_owner_symbol_ext) => {
            if body_owner_symbol_ext.id() != &first {
              self.add_reference(body_owner_symbol_ext.clone(), member_expr, false);
            } else if self.state.contains(AnalyzeState::ASSIGNMENT_LHS) {
              self.add_reference(body_owner_symbol_ext.clone(), member_expr, true);
            }
          }
          None => {
            self.used_id_set.insert(member_expr);
          }
        }
        return;
      }
    }
    node.obj.visit_with(self);
    node.prop.visit_with(self);
  }

  fn visit_export_default_decl(&mut self, node: &ExportDefaultDecl) {
    let pre_state = self.state;
    self.state.insert(AnalyzeState::EXPORT_DEFAULT);
    match &node.decl {
      DefaultDecl::Class(_) | DefaultDecl::Fn(_) => {
        node.visit_children_with(self);
      }
      DefaultDecl::TsInterfaceDecl(_) => {
        unreachable!("We have been converted Typescript to javascript already")
      }
    }
    self.state = pre_state;
  }

  fn visit_class_expr(&mut self, node: &ClassExpr) {
    // TODO: handle
    if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
      let pre_state = self.state;
      self.state.remove(AnalyzeState::EXPORT_DEFAULT);
      let default_ident = self.generate_default_ident();
      self.add_export(
        default_ident.sym.clone(),
        SymbolRef::Declaration(Symbol::new(
          self.module_identifier,
          default_ident.to_id().into(),
          SymbolType::Define,
          None,
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
          let symbol_flag = SymbolFlag::EXPORT_DEFAULT | SymbolFlag::CLASS_EXPR;
          let symbol_ext: SymbolExt = if let Some(ident) = &node.ident {
            let renamed_symbol_ext = SymbolExt::new(ident.sym.clone(), symbol_flag);
            let default_ident_ext: SymbolExt = default_ident.sym.into();
            self.add_reference(
              default_ident_ext.clone(),
              Part::TopLevelId(renamed_symbol_ext.id.clone()),
              false,
            );
            self.add_reference(
              renamed_symbol_ext.clone(),
              Part::TopLevelId(default_ident_ext.id),
              false,
            );
            renamed_symbol_ext
          } else {
            SymbolExt::new(default_ident.sym, symbol_flag)
          };
          self.export_default_name = Some(symbol_ext.id().clone());
          symbol_ext
        }
      };
      let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();
      self.current_body_owner_symbol_ext = Some(body_owner_extend_symbol);
      node.class.visit_with(self);
      self.current_body_owner_symbol_ext = before_owner_extend_symbol;
      self.state = pre_state;
    } else {
      // if the class expr is not inside a default expr, it will not
      // generate a binding.
      node.class.visit_with(self);
    }
  }
  fn visit_call_expr(&mut self, node: &CallExpr) {
    if let Some(require_lit) = get_require_literal(node, self.unresolved_ctxt) {
      self.module_syntax.insert(ModuleSyntax::COMMONJS);
      match self.resolve_module_identifier(
        &require_lit,
        &[DependencyType::CjsRequire, DependencyType::CjsExportRequire],
      ) {
        Some(dep_id) => match self
          .bail_out_module_identifiers
          .entry(ModuleIdOrDepId::DepId(dep_id))
        {
          Entry::Occupied(mut occ) => {
            *occ.get_mut() |= BailoutFlag::COMMONJS_REQUIRE;
          }
          Entry::Vacant(vac) => {
            vac.insert(BailoutFlag::COMMONJS_REQUIRE);
          }
        },
        None => {
          eprintln!(
            "Can't resolve require {} in {}",
            require_lit, self.module_identifier
          );
        }
      };
    } else if let Some(import_str) = get_dynamic_import_string_literal(node) {
      match self.resolve_module_identifier(&import_str, &[DependencyType::DynamicImport]) {
        Some(dep_id) => match self
          .bail_out_module_identifiers
          .entry(ModuleIdOrDepId::DepId(dep_id))
        {
          Entry::Occupied(mut occ) => {
            *occ.get_mut() |= BailoutFlag::DYNAMIC_IMPORT;
          }
          Entry::Vacant(vac) => {
            vac.insert(BailoutFlag::DYNAMIC_IMPORT);
          }
        },
        None => {
          eprintln!(
            "Can't resolve dynamic import {} in {}",
            import_str, self.module_identifier
          );
        }
      };
    } else {
      node.visit_children_with(self);
    }
  }

  fn visit_fn_expr(&mut self, node: &FnExpr) {
    if self.state.contains(AnalyzeState::EXPORT_DEFAULT) {
      let pre_state = self.state;
      self.state.remove(AnalyzeState::EXPORT_DEFAULT);
      let default_ident = self.generate_default_ident();
      self.add_export(
        default_ident.sym.clone(),
        SymbolRef::Declaration(Symbol::new(
          self.module_identifier,
          default_ident.to_id().into(),
          SymbolType::Define,
          None,
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
          let symbol_flag = SymbolFlag::EXPORT_DEFAULT | SymbolFlag::FUNCTION_EXPR;
          let symbol_ext: SymbolExt = if let Some(ident) = &node.ident {
            let symbol_ext = SymbolExt::new(ident.sym.clone(), symbol_flag);
            // considering default export has bind to new symbol e.g.
            // ```js
            // export default function test() {
            // }
            // let result = test();
            // ``

            self.add_reference(
              symbol_ext.clone(),
              Part::TopLevelId(default_ident.sym.clone()),
              false,
            );
            self.add_reference(
              default_ident.sym.into(),
              Part::TopLevelId(symbol_ext.id.clone()),
              false,
            );
            symbol_ext
          } else {
            SymbolExt::new(default_ident.sym, symbol_flag)
          };
          self.export_default_name = Some(symbol_ext.id().clone());
          symbol_ext
        }
      };
      let before_owner_extend_symbol = self.current_body_owner_symbol_ext.clone();

      self.current_body_owner_symbol_ext = Some(body_owner_extend_symbol);
      node.function.visit_with(self);
      self.current_body_owner_symbol_ext = before_owner_extend_symbol;
      self.state = pre_state;
    } else {
      // if the function expr is not inside a default expr, it will not
      // generate a binding.
      node.function.visit_with(self);
    }
  }

  fn visit_class_decl(&mut self, node: &ClassDecl) {
    let id: BetterId = node.ident.to_id().into();
    let ctxt = id.ctxt;
    let old_region = self.current_body_owner_symbol_ext.clone();
    if ctxt == self.top_level_ctxt {
      self.current_body_owner_symbol_ext = Some(id.atom.into());
    }
    node.visit_children_with(self);
    self.current_body_owner_symbol_ext = old_region;
  }

  fn visit_fn_decl(&mut self, node: &FnDecl) {
    let id: BetterId = node.ident.to_id().into();
    let ctxt = id.ctxt;
    let before_symbol_ext = self.current_body_owner_symbol_ext.clone();
    if ctxt == self.top_level_ctxt {
      self.current_body_owner_symbol_ext = Some(id.atom.into());
    }
    node.function.visit_with(self);
    self.current_body_owner_symbol_ext = before_symbol_ext;
  }

  fn visit_var_decl(&mut self, node: &VarDecl) {
    for ele in node.decls.iter() {
      // TODO: I think it is safe to move is_export out of loop.
      let is_export = self.state.contains(AnalyzeState::EXPORT_DECL);
      let Some(lhs) = self.visit_var_decl_pattern(&ele.name, is_export) else {
        ele.init.visit_with(self);
        continue;
      };
      if let Some(ref init) = ele.init
        && self.potential_top_level_ctxt.contains(&lhs.ctxt)
      {
        let mut symbol_ext = SymbolExt::new(lhs.atom, SymbolFlag::VAR_DECL);
        match init {
          box Expr::Fn(_) => symbol_ext.flag.insert(SymbolFlag::FUNCTION_EXPR),
          box Expr::Arrow(_) => symbol_ext.flag.insert(SymbolFlag::ARROW_EXPR),
          _ => {}
        };
        if is_export {
          symbol_ext.flag.insert(SymbolFlag::EXPORT);
        }
        let before_symbol_ext = self.current_body_owner_symbol_ext.clone();
        self.current_body_owner_symbol_ext = Some(symbol_ext);
        init.visit_with(self);
        self.current_body_owner_symbol_ext = before_symbol_ext;
      } else {
        ele.init.visit_with(self);
      }
    }
  }
  fn visit_decl(&mut self, node: &Decl) {
    match node {
      Decl::TsInterface(_) | Decl::TsTypeAlias(_) | Decl::TsEnum(_) | Decl::TsModule(_) => {
        unreachable!("We have been transformed typescript to javascript before.")
      }
      Decl::Class(_) | Decl::Fn(_) | Decl::Var(_) | Decl::Using(_) => {
        node.visit_children_with(self);
      }
    }
  }
}

impl<'a> ModuleRefAnalyze<'a> {
  // returns BetterId if the decl pattern only has one identifier binding, e.g. var binding = xxx
  // other binding patterns like let [state, setState] = useState() will return None
  fn visit_var_decl_pattern(&mut self, pattern: &Pat, is_export: bool) -> Option<BetterId> {
    let mut add_export = |lhs: &BetterId| {
      if is_export && lhs.ctxt == self.top_level_ctxt {
        self.add_export(
          lhs.atom.clone(),
          SymbolRef::Declaration(Symbol::new(
            self.module_identifier,
            lhs.clone(),
            SymbolType::Define,
            None,
          )),
        );
      }
    };
    match pattern {
      // var ident = xxx
      Pat::Ident(ident) => {
        let id = BetterId::from(ident.to_id());
        add_export(&id);
        Some(id)
      }
      // var [ident, ...idents] = xxx
      Pat::Array(bindings) => {
        for binding in bindings.elems.iter().flatten() {
          self.visit_var_decl_pattern(binding, is_export);
        }
        None
      }
      // var { ident } = xxx
      Pat::Object(obj) => {
        for prop in &obj.props {
          match prop {
            ObjectPatProp::KeyValue(pair) => {
              pair.key.visit_with(self);
              self.visit_var_decl_pattern(&pair.value, is_export);
            }
            ObjectPatProp::Assign(assign) => {
              assign.value.visit_with(self);
              let lhs = BetterId::from(assign.key.to_id());
              // inline code here to avoid compiler complaints
              if is_export && lhs.ctxt == self.top_level_ctxt {
                self.add_export(
                  lhs.atom.clone(),
                  SymbolRef::Declaration(Symbol::new(
                    self.module_identifier,
                    lhs.clone(),
                    SymbolType::Define,
                    None,
                  )),
                );
              }
            }
            ObjectPatProp::Rest(rest) => {
              self.visit_var_decl_pattern(&rest.arg, is_export);
            }
          }
        }
        None
      }
      Pat::Assign(assign) => {
        self.visit_var_decl_pattern(&assign.left, is_export);
        assign.right.visit_with(self);
        None
      }
      Pat::Rest(rest) => {
        self.visit_var_decl_pattern(&rest.arg, is_export);
        None
      }
      Pat::Invalid(_) | Pat::Expr(_) => {
        // TODO: confirm if these pattern occurs only in for loop or is invalid
        pattern.visit_with(self);
        None
      }
    }
  }
  pub fn get_side_effects_from_config(
    factory_meta: Option<&FactoryMeta>,
  ) -> Option<SideEffectType> {
    // sideEffects in module.rule has higher priority,
    // we could early return if we match a rule.
    if let Some(FactoryMeta {
      side_effect_free_old: Some(side_effect_free),
      ..
    }) = factory_meta
    {
      return Some(SideEffectType::Configuration(!*side_effect_free));
    }
    None
  }
}

pub fn get_side_effects_from_package_json(
  side_effects: SideEffects,
  relative_path: PathBuf,
) -> bool {
  match side_effects {
    SideEffects::Bool(s) => s,
    SideEffects::String(s) => {
      glob_match_with_normalized_pattern(&s, &relative_path.to_string_lossy())
    }
    SideEffects::Array(patterns) => patterns
      .iter()
      .any(|pattern| glob_match_with_normalized_pattern(pattern, &relative_path.to_string_lossy())),
  }
}

fn glob_match_with_normalized_pattern(pattern: &str, string: &str) -> bool {
  let new_glob = if !pattern.contains('/') {
    format!("**/{}", pattern)
  } else {
    pattern.to_string()
  };
  // TODO: cache result
  let base_regexp = glob_to_regexp(
    &new_glob,
    &GlobOptions {
      globstar: true,
      extended: true,
      ..GlobOptions::default()
    },
  );
  let regexp_source = base_regexp.as_str();
  let regexp_pattern = format!("^(\\./)?{}", &regexp_source[1..]);
  let regexp = regex::Regex::new(&regexp_pattern).unwrap_or_else(|err| {
    panic!(
      "Failed to compile regex pattern '{}': {}",
      regexp_pattern, err
    );
  });
  regexp.is_match(string)
}

impl<'a> ModuleRefAnalyze<'a> {
  fn analyze_module_item_side_effects(&mut self, ele: &ModuleItem) {
    match ele {
      ModuleItem::ModuleDecl(module_decl) => match module_decl {
        ModuleDecl::ExportDecl(decl) => {
          if !is_pure_decl(&decl.decl, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        ModuleDecl::ExportDefaultDecl(decl) => {
          match decl.decl {
            DefaultDecl::Class(ref class) => {
              if !is_pure_class(&class.class, self.unresolved_ctxt) {
                self.has_side_effects_stmt = true;
              }
            }
            DefaultDecl::Fn(_) => {}
            DefaultDecl::TsInterfaceDecl(_) => unreachable!(),
          };
        }
        ModuleDecl::ExportDefaultExpr(expr) => {
          if !is_pure_expression(&expr.expr, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        ModuleDecl::ExportAll(_)
        | ModuleDecl::Import(_)
        | ModuleDecl::ExportNamed(_)
        | ModuleDecl::TsImportEquals(_)
        | ModuleDecl::TsExportAssignment(_)
        | ModuleDecl::TsNamespaceExport(_) => {}
      },
      ModuleItem::Stmt(stmt) => self.analyze_stmt_side_effects(stmt),
    }
  }

  /// If we find a stmt that has side effects, we will skip the rest of the stmts.
  /// And mark the module as having side effects.
  fn analyze_stmt_side_effects(&mut self, ele: &Stmt) {
    if !self.has_side_effects_stmt {
      match ele {
        Stmt::If(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::While(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::DoWhile(stmt) => {
          if !is_pure_expression(&stmt.test, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::For(stmt) => {
          let pure_init = match stmt.init {
            Some(ref init) => match init {
              VarDeclOrExpr::VarDecl(decl) => is_pure_var_decl(decl, self.unresolved_ctxt),
              VarDeclOrExpr::Expr(expr) => is_pure_expression(expr, self.unresolved_ctxt),
            },
            None => true,
          };

          if !pure_init {
            self.has_side_effects_stmt = true;
            return;
          }

          let pure_test = match stmt.test {
            Some(box ref test) => is_pure_expression(test, self.unresolved_ctxt),
            None => true,
          };

          if !pure_test {
            self.has_side_effects_stmt = true;
            return;
          }

          let pure_update = match stmt.update {
            Some(ref expr) => is_pure_expression(expr, self.unresolved_ctxt),
            None => true,
          };

          if !pure_update {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Expr(stmt) => {
          if !is_pure_expression(&stmt.expr, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Switch(stmt) => {
          if !is_pure_expression(&stmt.discriminant, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Decl(stmt) => {
          if !is_pure_decl(stmt, self.unresolved_ctxt) {
            self.has_side_effects_stmt = true;
          }
        }
        Stmt::Empty(_) => {}
        Stmt::Labeled(_) => {}
        Stmt::Block(_) => {}
        _ => self.has_side_effects_stmt = true,
      };
    }
  }
  fn add_export(&mut self, id: Atom, symbol: SymbolRef) {
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
    match self.import_map.entry(id.atom.clone()) {
      Entry::Occupied(_) => {
        // TODO: should add some Diagnostic
      }
      Entry::Vacant(vac) => {
        self.potential_top_level_ctxt.insert(id.ctxt);
        vac.insert(symbol);
      }
    }
  }
  fn analyze_named_export(&mut self, named_export: &NamedExport) {
    let src = named_export.src.as_ref().map(|src| &src.value);
    if let Some(src) = src {
      let dep_id = match self
        .resolve_module_identifier(src, &[DependencyType::EsmExport(named_export.span.into())])
      {
        Some(module_identifier) => module_identifier,
        None => {
          eprintln!(
            "Can't resolve import {} in module {} ",
            // SAFETY: we already know that src is not empty
            named_export.src.as_ref().expect("TODO:").value,
            self.module_identifier
          );
          // TODO: Ignore because helper interference.
          return;
        }
      };
      named_export
        .specifiers
        .iter()
        .for_each(|specifier| match specifier {
          ExportSpecifier::Namespace(namespace) => {
            let atom = match namespace.name {
              ModuleExportName::Ident(ref ident) => ident.sym.clone(),
              ModuleExportName::Str(ref str) => str.value.clone(),
            };
            self.add_export(
              atom.clone(),
              SymbolRef::Star(StarSymbol::new(
                "".into(),
                atom,
                self.module_identifier,
                StarSymbolKind::ReExportAllAs,
                dep_id,
              )),
            );
          }
          ExportSpecifier::Default(_) => {
            // Currently swc does not support syntax like `export v from 'xxx';
            // Since this is a syntax error the logic should not reach here.`
            unreachable!("Module has syntax error should not trigger tree_shaking")
          }
          ExportSpecifier::Named(named) => {
            let original = match &named.orig {
              ModuleExportName::Ident(ident) => ident.sym.clone(),
              ModuleExportName::Str(str) => str.value.clone(),
            };
            let exported = named.exported.clone().map(|exported| match exported {
              ModuleExportName::Ident(ident) => ident.sym,
              ModuleExportName::Str(str) => str.value,
            });

            let exported_atom = exported.clone().unwrap_or_else(|| original.clone());

            self.reachable_import_and_export.insert(
              exported_atom.clone(),
              HashSet::from_iter([SymbolRef::Indirect(IndirectTopLevelSymbol::new(
                "".into(),
                self.module_identifier,
                IndirectType::Temp(original.clone()),
                dep_id,
              ))]),
            );

            let exported_symbol = SymbolRef::Indirect(IndirectTopLevelSymbol::new(
              self.module_identifier,
              self.module_identifier,
              IndirectType::ReExport(original, exported),
              dep_id,
            ));
            self.add_export(exported_atom, exported_symbol);
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
            let id: BetterId = match &named.orig {
              ModuleExportName::Ident(ident) => ident.to_id().into(),
              // `export {'a'}` is a syntax error;
              // we know here export has no src,  so this branch should not reachable.
              ModuleExportName::Str(_) => unreachable!(),
            };

            let exported_atom = named.exported.as_ref().map(|exported| match exported {
              ModuleExportName::Ident(ident) => ident.sym.clone(),
              ModuleExportName::Str(str) => str.value.clone(),
            });

            let export_name = exported_atom.clone().unwrap_or_else(|| id.atom.clone());
            let symbol_ref = SymbolRef::Declaration(Symbol::new(
              self.module_identifier,
              id,
              SymbolType::Temp,
              exported_atom,
            ));

            self.add_export(export_name, symbol_ref);
          }
        });
    };
  }

  /// Try to get the module_identifier from `src`, `dependency_type`, and `importer`
  /// For simplicity, this function will assume the importer is always `self.module_identifier`
  /// # Panic
  /// This function will panic if can't find
  fn resolve_module_identifier(
    &self,
    src: &str,
    dependency_types: &[DependencyType],
  ) -> Option<DependencyId> {
    self.dependencies.iter().find_map(|dep| {
      if let Some(dep) = dep.as_module_dependency()
        && dep.request() == src
        && dependency_types.contains(dep.dependency_type())
      {
        Some(*dep.id())
      } else if let Some(dep) = dep.as_context_dependency()
        && dep.request() == src
        && dependency_types.contains(dep.dependency_type())
      {
        Some(*dep.id())
      } else {
        None
      }
    })
  }
}

/// The `allow(unused)` will be removed after the Tree shaking is finished
#[derive(Debug, Default, Clone)]
#[allow(unused)]
pub struct OptimizeAnalyzeResult {
  pub top_level_ctxt: SyntaxContext,
  unresolved_ctxt: SyntaxContext,
  pub module_identifier: ModuleIdentifier,
  pub export_map: HashMap<Atom, SymbolRef>,
  pub(crate) import_map: HashMap<Atom, SymbolRef>,
  pub inherit_export_maps: LinkedHashMap<ModuleIdentifier, HashMap<Atom, SymbolRef>>,
  pub export_all_dep_id: LinkedHashSet<DependencyId>,
  // current_region: Option<BetterId>,
  // pub(crate) reference_map: HashMap<BetterId, HashSet<BetterId>>,
  pub(crate) reachable_import_of_export: HashMap<Atom, HashSet<SymbolRef>>,
  state: AnalyzeState,
  pub(crate) used_symbol_refs: HashSet<SymbolRef>,
  pub(crate) bail_out_module_identifiers: HashMap<ModuleIdOrDepId, BailoutFlag>,
  pub(crate) side_effects: SideEffectType,
  pub(crate) module_syntax: ModuleSyntax,
  pub harmony_import_specifier_dependency_alias_map: HashMap<Span, SymbolRef>,
}

impl From<ModuleRefAnalyze<'_>> for OptimizeAnalyzeResult {
  fn from(analyze: ModuleRefAnalyze<'_>) -> Self {
    Self {
      top_level_ctxt: analyze.top_level_ctxt,
      unresolved_ctxt: analyze.unresolved_ctxt,
      module_identifier: analyze.module_identifier,
      export_map: analyze.export_map,
      import_map: analyze.import_map,
      inherit_export_maps: LinkedHashMap::default(),
      // current_region: analyze.current_body_owner_id),
      // reference_map: analyze.reference_map),
      reachable_import_of_export: analyze.reachable_import_and_export,
      state: analyze.state,
      used_symbol_refs: analyze.used_symbol_ref,
      bail_out_module_identifiers: analyze.bail_out_module_identifiers,
      side_effects: analyze.side_effects,
      module_syntax: analyze.module_syntax,
      export_all_dep_id: analyze.export_all_dep_id,
      harmony_import_specifier_dependency_alias_map: analyze
        .harmony_import_specifier_dependency_alias_map,
    }
  }
}

fn first_ident_of_assign_lhs(node: &AssignExpr) -> Option<Id> {
  let mut visitor = FirstIdentVisitor::default();
  node.left.visit_with(&mut visitor);
  visitor.id
}

#[derive(Default)]
struct FirstIdentVisitor {
  id: Option<Id>,
}

impl Visit for FirstIdentVisitor {
  noop_visit_type!();

  fn visit_ident(&mut self, node: &Ident) {
    if self.id.is_none() {
      self.id = Some(node.to_id());
    }
  }
}

fn is_pure_expression(expr: &Expr, unresolved_ctxt: SyntaxContext) -> bool {
  match expr {
    // Mark `module.exports = require('xxx')` as pure
    Expr::Assign(AssignExpr {
      left: AssignTarget::Simple(SimpleAssignTarget::Member(left_expr)),
      right: box Expr::Call(call_expr_right),
      op: op!("="),
      ..
    }) if is_module_exports_member_expr(left_expr, unresolved_ctxt)
      && get_require_literal(call_expr_right, unresolved_ctxt).is_some() =>
    {
      true
    }
    _ => !expr.may_have_side_effects(&ExprCtx {
      unresolved_ctxt,
      is_unresolved_ref_safe: false,
    }),
  }
}

/// Check if the expression is `module.exports`
fn is_module_exports_member_expr(expr: &MemberExpr, unresolved_ctxt: SyntaxContext) -> bool {
  matches!(expr, MemberExpr {
    obj:
      box Expr::Ident(Ident {
        sym: obj_sym,
        span: obj_span,
        ..
      }),
    prop: MemberProp::Ident(Ident { sym: prop_sym, .. }),
    ..
  } if obj_sym == "module" && obj_span.ctxt == unresolved_ctxt && prop_sym == "exports")
}

fn is_pure_decl(stmt: &Decl, unresolved_ctxt: SyntaxContext) -> bool {
  match stmt {
    Decl::Class(class) => is_pure_class(&class.class, unresolved_ctxt),
    Decl::Fn(_) => true,
    Decl::Var(var) => is_pure_var_decl(var, unresolved_ctxt),
    Decl::Using(_) => false,
    Decl::TsInterface(_) => unreachable!(),
    Decl::TsTypeAlias(_) => unreachable!(),

    Decl::TsEnum(_) => unreachable!(),
    Decl::TsModule(_) => unreachable!(),
  }
}

fn is_pure_class(class: &Class, unresolved_ctxt: SyntaxContext) -> bool {
  if let Some(ref super_class) = class.super_class {
    if !is_pure_expression(super_class, unresolved_ctxt) {
      return false;
    }
  }
  let is_pure_key = |key: &PropName| -> bool {
    match key {
      PropName::BigInt(_) | PropName::Ident(_) | PropName::Str(_) | PropName::Num(_) => true,
      PropName::Computed(ref computed) => is_pure_expression(&computed.expr, unresolved_ctxt),
    }
  };

  class.body.iter().all(|item| -> bool {
    match item {
      ClassMember::Constructor(cons) => is_pure_key(&cons.key),
      ClassMember::Method(method) => is_pure_key(&method.key),
      ClassMember::PrivateMethod(method) => {
        is_pure_expression(&Expr::PrivateName(method.key.clone()), unresolved_ctxt)
      }
      ClassMember::ClassProp(prop) => {
        is_pure_key(&prop.key)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt)
            } else {
              true
            })
      }
      ClassMember::PrivateProp(prop) => {
        is_pure_expression(&Expr::PrivateName(prop.key.clone()), unresolved_ctxt)
          && (!prop.is_static
            || if let Some(ref value) = prop.value {
              is_pure_expression(value, unresolved_ctxt)
            } else {
              true
            })
      }
      ClassMember::TsIndexSignature(_) => unreachable!(),
      ClassMember::Empty(_) => true,
      ClassMember::StaticBlock(_) => true,
      ClassMember::AutoAccessor(_) => true,
    }
  })
}

fn is_pure_var_decl(var: &VarDecl, unresolved_ctxt: SyntaxContext) -> bool {
  var.decls.iter().all(|decl| {
    if let Some(ref init) = decl.init {
      is_pure_expression(init, unresolved_ctxt)
    } else {
      true
    }
  })
}

fn is_import_decl(module_item: &ModuleItem) -> bool {
  matches!(module_item, ModuleItem::ModuleDecl(ModuleDecl::Import(_)))
}

#[derive(Clone, Debug)]
pub enum SideEffects {
  Bool(bool),
  String(String),
  Array(Vec<String>),
}

impl SideEffects {
  pub fn from_description(description: &serde_json::Value) -> Option<Self> {
    description.get("sideEffects").and_then(|value| {
      if let Some(b) = value.as_bool() {
        Some(SideEffects::Bool(b))
      } else if let Some(s) = value.as_str() {
        Some(SideEffects::String(s.to_owned()))
      } else if let Some(vec) = value.as_array() {
        let mut side_effects = vec![];
        for value in vec {
          if let Some(str) = value.as_str() {
            side_effects.push(str.to_string());
          } else {
            return None;
          }
        }
        Some(SideEffects::Array(side_effects))
      } else {
        None
      }
    })
  }
}
