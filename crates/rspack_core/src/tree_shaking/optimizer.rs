use std::{
  borrow::BorrowMut,
  collections::{hash_map::Entry, VecDeque},
};

use petgraph::{
  algo,
  prelude::{DiGraphMap, GraphMap},
  stable_graph::NodeIndex,
  visit::{Bfs, Dfs, EdgeRef},
  Directed,
};
use rspack_error::{
  errors_to_diagnostics, Error, InternalError, IntoTWithDiagnosticArray, Result, Severity,
  TWithDiagnosticArray,
};
use rspack_identifier::{Identifier, IdentifierLinkedSet, IdentifierMap, IdentifierSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::{common::SyntaxContext, ecma::atoms::Atom};

use super::{
  symbol::{
    BetterId, IndirectTopLevelSymbol, IndirectType, SerdeSymbol, StarSymbol, StarSymbolKind,
    Symbol, SymbolType,
  },
  visitor::ModuleIdOrDepId,
};
use super::{
  symbol_graph::SymbolGraph,
  visitor::{OptimizeAnalyzeResult, SymbolRef},
  BailoutFlag, ModuleUsedType, OptimizeDependencyResult, SideEffectType,
};
use crate::{
  contextify, join_string_component,
  tree_shaking::{utils::ConvertModulePath, visitor::ModuleRefAnalyze},
  Compilation, DependencyId, DependencyType, ModuleGraph, ModuleIdentifier, ModuleType,
  NormalModuleSource,
};

pub struct CodeSizeOptimizer<'a> {
  compilation: &'a mut Compilation,
  bailout_modules: IdentifierMap<BailoutFlag>,
  side_effects_free_modules: IdentifierSet,
  symbol_graph: SymbolGraph,
}

enum ReExportConnectionStatus {
  Vacant(Vec<Vec<SymbolRef>>),
  Occupied(Vec<SymbolRef>),
}

enum EntryLikeType {
  Entry,
  Bailout,
}

#[derive(Debug)]
struct ModuleEliminator {
  export_used: bool,
  is_bailout: bool,
  side_effects_free: bool,
  is_entry: bool,
  /// used for debugging
  #[allow(unused)]
  module_identifier: ModuleIdentifier,
}

type SymbolRefWithMemberChain = (SymbolRef, Vec<Atom>);

impl ModuleEliminator {
  fn could_be_skipped(&self) -> bool {
    !self.export_used && !self.is_bailout && self.side_effects_free && !self.is_entry
  }
}

impl<'a> CodeSizeOptimizer<'a> {
  pub fn new(compilation: &'a mut Compilation) -> Self {
    Self {
      bailout_modules: compilation.bailout_module_identifiers.clone(),
      symbol_graph: SymbolGraph::default(),
      side_effects_free_modules: IdentifierSet::default(),
      compilation,
    }
  }

  pub async fn run(&mut self) -> Result<TWithDiagnosticArray<OptimizeDependencyResult>> {
    // let is_incremental_rebuild = self
    //   .compilation
    //   .options
    //   .is_incremental_rebuild_make_enabled();
    // let mut analyze_result_map = par_analyze_module(self.compilation).await;
    // let mut finalized_result_map = if is_incremental_rebuild {
    //   if is_first_time_analyze {
    //     analyze_result_map
    //   } else {
    //     for (ident, result) in analyze_result_map.into_iter() {
    //       self
    //         .compilation
    //         .optimize_analyze_result_map
    //         .insert(ident, result);
    //     }
    //     // Merge new analyze result with previous in incremental_rebuild mode
    //     std::mem::take(&mut self.compilation.optimize_analyze_result_map)
    //   }
    // } else {
    //   analyze_result_map
    // };
    let mut optimize_analyze_result_map =
      std::mem::take(self.compilation.optimize_analyze_result_map_mut());
    let module_graph = self.compilation.get_module_graph();
    optimize_analyze_result_map.iter_mut().for_each(
      |(module_identifier, optimize_analyze_result)| {
        if let Some(factory_meta_side_effects) = module_graph
          .module_by_identifier(module_identifier)
          .and_then(|m| ModuleRefAnalyze::get_side_effects_from_config(m.factory_meta()))
        {
          optimize_analyze_result.side_effects = factory_meta_side_effects;
        }
      },
    );
    std::mem::swap(
      &mut optimize_analyze_result_map,
      self.compilation.optimize_analyze_result_map_mut(),
    );
    // We will set it back and return the analyze result, take is safe here.
    let mut finalized_result_map =
      std::mem::take(self.compilation.optimize_analyze_result_map_mut());
    let mut evaluated_used_symbol_ref: HashSet<SymbolRef> = HashSet::default();
    let mut evaluated_module_identifiers = IdentifierSet::default();
    let side_effects_options = self
      .compilation
      .options
      .optimization
      .side_effects
      .is_enable();
    let mut side_effect_map: IdentifierMap<SideEffectType> = IdentifierMap::default();
    for analyze_result in finalized_result_map.values() {
      side_effect_map.insert(
        analyze_result.module_identifier,
        analyze_result.side_effects,
      );
      // if `side_effects` is disabled, then force every module has side_effects
      let forced_side_effects = !side_effects_options
        || self
          .compilation
          .entry_module_identifiers
          .contains(&analyze_result.module_identifier);
      // side_effects: true
      if forced_side_effects
        || !matches!(
          analyze_result.side_effects,
          SideEffectType::Configuration(false)
        )
      {
        evaluated_module_identifiers.insert(analyze_result.module_identifier);
        evaluated_used_symbol_ref.extend(analyze_result.used_symbol_refs.iter().cloned());
      }
      // merge bailout module identifier
      for (module_identifier, &reason) in analyze_result.bail_out_module_identifiers.iter() {
        self.merge_bailout_modules_reason(module_identifier, reason);
      }
    }
    tracing::debug!(side_effect_map = format!("{:#?}", side_effect_map));

    self.side_effects_free_modules = self.get_side_effects_free_modules(side_effect_map);

    let inherit_export_ref_graph = get_inherit_export_ref_graph(
      &mut finalized_result_map,
      &self.compilation.get_module_graph(),
    );
    let mut errors = vec![];
    let mut used_symbol_ref = HashSet::default();
    let mut used_export_module_identifiers: IdentifierMap<ModuleUsedType> =
      IdentifierMap::default();
    let mut traced_tuple = HashMap::default();
    // Marking used symbol and all reachable export symbol from the used symbol for each module

    let mut visited_symbol_ref: HashSet<SymbolRefWithMemberChain> = HashSet::default();

    self.mark_used_symbol_with(
      &finalized_result_map,
      VecDeque::from_iter(
        evaluated_used_symbol_ref
          .into_iter()
          .map(|item| (item, vec![])),
      ),
      &mut evaluated_module_identifiers,
      &mut used_export_module_identifiers,
      &inherit_export_ref_graph,
      &mut traced_tuple,
      &mut visited_symbol_ref,
      &mut errors,
    );

    // We considering all export symbol in each entry module as used for now
    self.mark_entry_symbol(
      &finalized_result_map,
      &mut evaluated_module_identifiers,
      &mut used_export_module_identifiers,
      &inherit_export_ref_graph,
      &mut traced_tuple,
      &mut visited_symbol_ref,
      &mut errors,
    );

    // All lazy imported module will be treadted as entry module, which means
    // Its export symbol will be marked as used
    // let mut bailout_entry_module_identifiers = IdentifierSet::default();
    self.mark_bailout_module(
      &finalized_result_map,
      evaluated_module_identifiers,
      &mut used_export_module_identifiers,
      inherit_export_ref_graph,
      traced_tuple,
      &mut visited_symbol_ref,
      &mut errors,
    );
    // let debug_graph = generate_debug_symbol_graph(
    //   &self.symbol_graph,
    //   &self.compilation.get_module_graph(),
    //   &self.compilation.options.context.as_str().to_owned(),
    // );
    // let res = serde_json::to_string(&debug_graph).unwrap();
    // println!("{}", res);
    self.check_symbol_query();

    let dead_nodes_index = HashSet::default();
    // dependency_replacement();
    let include_module_ids = self.finalize_symbol(
      side_effects_options,
      used_export_module_identifiers,
      &mut used_symbol_ref,
      &dead_nodes_index,
    );
    Ok(
      OptimizeDependencyResult {
        used_symbol_ref,
        analyze_results: finalized_result_map,
        bail_out_module_identifiers: std::mem::take(&mut self.bailout_modules),
        side_effects_free_modules: std::mem::take(&mut self.side_effects_free_modules),
        module_item_map: IdentifierMap::default(),
        include_module_ids,
      }
      .with_diagnostic(errors_to_diagnostics(errors)),
    )
  }

  fn merge_bailout_modules_reason(&mut self, k: &ModuleIdOrDepId, v: BailoutFlag) {
    let mg = &self.compilation.get_module_graph();
    let normalized_module_id = match k {
      ModuleIdOrDepId::ModuleId(module_id) => *module_id,
      ModuleIdOrDepId::DepId(dep_id) => {
        if let Some(module_id) = mg.module_identifier_by_dependency_id(dep_id) {
          *module_id
        } else {
          return;
        }
      }
    };
    match self.bailout_modules.entry(normalized_module_id) {
      Entry::Occupied(mut occ) => {
        *occ.get_mut() |= v;
      }
      Entry::Vacant(vac) => {
        vac.insert(v);
      }
    }
  }

  fn check_symbol_query(&self) {
    let symbol_list = match &std::env::var("SYMBOL_QUERY_PATH") {
      Ok(relative_path) => {
        let log = std::env::current_dir().expect("Can't get cwd");
        let ab_path = log.join(relative_path);
        let file =
          std::fs::read_to_string(ab_path).expect("Failed to read target file into string");
        serde_json::from_str::<Vec<SerdeSymbol>>(&file)
          .expect("Can't convert to symbol from sourcefile")
      }
      Err(_) => {
        vec![]
      }
    };

    let get_node_index_from_serde_symbol = |serde_symbol: &SerdeSymbol| {
      for symbol_ref in self.symbol_graph.symbol_refs() {
        match symbol_ref {
          SymbolRef::Declaration(direct) => {
            if direct.id().atom != serde_symbol.id || !direct.src().contains(&serde_symbol.uri) {
              continue;
            }
          }
          SymbolRef::Indirect(_) | SymbolRef::Star(_) => {
            continue;
          }
          SymbolRef::Url { .. } | SymbolRef::Worker { .. } => continue,
          SymbolRef::Usage(_, _, _) => continue,
        }
        let index = self
          .symbol_graph
          .get_node_index(symbol_ref)
          .unwrap_or_else(|| panic!("Can't find NodeIndex for symbol {symbol_ref:?}",));
        return Some(*index);
      }
      None
    };

    let symbol_graph = self.symbol_graph.clone().reverse_graph();

    for symbol in symbol_list {
      // get_symbol_path()
      match get_node_index_from_serde_symbol(&symbol) {
        Some(node_index) => {
          let paths = get_symbol_path(&symbol_graph, node_index);
          println!("Reason of Included Symbol symbol{symbol:?}",);
          for p in paths {
            let normalized_symbols = p
              .into_iter()
              .map(|symbol| {
                symbol
                  .convert_module_identifier_to_module_path(&self.compilation.get_module_graph())
              })
              .collect::<Vec<_>>();
            println!("{normalized_symbols:#?}",);
          }
        }
        None => {
          eprintln!("Can't find symbol {symbol:?} in symbolGraph",);
        }
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn mark_bailout_module(
    &mut self,
    analyze_result_map: &IdentifierMap<OptimizeAnalyzeResult>,
    mut evaluated_module_identifiers: IdentifierSet,
    used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
    inherit_export_ref_graph: GraphMap<Identifier, (), Directed>,
    mut traced_tuple: HashMap<(Identifier, Identifier), Vec<SymbolRef>>,
    visited_symbol_ref: &mut HashSet<SymbolRefWithMemberChain>,
    errors: &mut Vec<Error>,
  ) {
    let bailout_entry_modules = self.bailout_modules.keys().copied().collect::<Vec<_>>();
    for module_id in bailout_entry_modules {
      self.collect_from_entry_like(
        analyze_result_map,
        module_id,
        &mut evaluated_module_identifiers,
        used_export_module_identifiers,
        &inherit_export_ref_graph,
        &mut traced_tuple,
        EntryLikeType::Bailout,
        visited_symbol_ref,
        errors,
      );
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn mark_entry_symbol(
    &mut self,
    analyze_result_map: &IdentifierMap<OptimizeAnalyzeResult>,
    evaluated_module_identifiers: &mut IdentifierSet,
    used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
    inherit_export_ref_graph: &GraphMap<Identifier, (), Directed>,
    traced_tuple: &mut HashMap<(Identifier, Identifier), Vec<SymbolRef>>,
    visited_symbol_ref: &mut HashSet<SymbolRefWithMemberChain>,
    errors: &mut Vec<Error>,
  ) {
    for entry in self.compilation.entry_modules() {
      self.collect_from_entry_like(
        analyze_result_map,
        entry,
        evaluated_module_identifiers,
        used_export_module_identifiers,
        inherit_export_ref_graph,
        traced_tuple,
        EntryLikeType::Entry,
        visited_symbol_ref,
        errors,
      );
    }
  }

  fn finalize_symbol(
    &mut self,
    side_effects_analyze: bool,
    used_export_module_identifiers: IdentifierMap<ModuleUsedType>,
    used_symbol_ref: &mut HashSet<SymbolRef>,
    dead_node_index: &HashSet<NodeIndex>,
  ) -> IdentifierSet {
    let symbol_graph = &self.symbol_graph;
    let visited_symbol_ref = symbol_graph
      .symbol_to_index
      .keys()
      .cloned()
      .collect::<HashSet<SymbolRef>>();
    let mut include_module_ids = IdentifierSet::default();

    if side_effects_analyze {
      let mut module_visited_symbol_ref: IdentifierMap<Vec<SymbolRef>> = IdentifierMap::default();
      for symbol in visited_symbol_ref {
        let module_identifier = symbol.importer();
        match module_visited_symbol_ref.entry(module_identifier) {
          Entry::Occupied(mut occ) => {
            occ.get_mut().push(symbol);
          }
          Entry::Vacant(vac) => {
            vac.insert(vec![symbol]);
          }
        }
      }
      // pruning
      let mut visited_symbol_node_index: HashSet<NodeIndex> = HashSet::default();
      let mut visited = IdentifierSet::default();
      let mut q = VecDeque::from_iter(
        self.compilation.entry_modules(), //
      );
      while let Some(module_identifier) = q.pop_front() {
        if visited.contains(&module_identifier) {
          continue;
        } else {
          visited.insert(module_identifier);
        }
        let eliminator = ModuleEliminator {
          export_used: used_export_module_identifiers.contains_key(&module_identifier),
          is_bailout: self.bailout_modules.contains_key(&module_identifier),
          side_effects_free: self.side_effects_free_modules.contains(&module_identifier),
          is_entry: self
            .compilation
            .entry_module_identifiers
            .contains(&module_identifier),
          module_identifier,
        };

        if eliminator.could_be_skipped() {
          continue;
        }

        let mut reachable_dependency_identifier = IdentifierSet::default();

        //        let mgm = self
        //          .compilation
        //          .get_module_graph()
        //          .module_graph_module_by_identifier(&module_identifier)
        //          .unwrap_or_else(|| panic!("Failed to get mgm by module identifier {module_identifier}"));
        include_module_ids.insert(module_identifier);
        if let Some(symbol_ref_list) = module_visited_symbol_ref.get(&module_identifier) {
          for symbol_ref in symbol_ref_list {
            update_reachable_dependency(
              symbol_ref,
              &mut reachable_dependency_identifier,
              symbol_graph,
            );
            let node_index = symbol_graph
              .get_node_index(symbol_ref)
              .expect("Can't get NodeIndex of SymbolRef");
            if !visited_symbol_node_index.contains(node_index) {
              let mut bfs = Bfs::new(&symbol_graph.graph, *node_index);
              while let Some(node_index) = bfs.next(&symbol_graph.graph) {
                update_reachable_symbol(dead_node_index, node_index, &mut visited_symbol_node_index)
              }
            }
          }
        }

        // reachable_dependency_identifier.extend(analyze_result.inherit_export_maps.keys());
        let module_graph = self.compilation.get_module_graph();
        for dependency_id in module_graph
          .get_module_all_dependencies(&module_identifier)
          .unwrap_or_else(|| {
            panic!("Failed to get ModuleGraphModule by module identifier {module_identifier}")
          })
          .iter()
        {
          let dep = module_graph
            .dependency_by_id(dependency_id)
            .expect("should have dependency");

          if dep.as_module_dependency().is_none() && dep.as_context_dependency().is_none() {
            continue;
          }
          let module_id_by_dep_id =
            match module_graph.module_identifier_by_dependency_id(dependency_id) {
              Some(module_identifier) => module_identifier,
              None => {
                let module = module_graph.module_by_identifier(&module_identifier);

                if module
                  .and_then(|module| module.as_context_module())
                  .is_some()
                {
                  // If the referenced module of context dependency is not found, then it might be failed to factorize in the first place. So let's skip it.
                  continue;
                }
                match module
                  .and_then(|module| module.as_normal_module())
                  .map(|normal_module| normal_module.source())
                {
                  Some(NormalModuleSource::Unbuild) => {
                    panic!("Failed to build module {module_identifier}");
                  }
                  Some(_) => {
                    // If module is failed to build, we know that the build output can't run, so it is alright to generate a wrong tree-shaking result.
                    // Also, if the referenced module of the dependency is not found, then it might failed to factorize in the first place. So let's skip it.
                    continue;
                  }
                  None => {
                    panic!("Failed to get normal module of {module_identifier}");
                  }
                };
              }
            };
          let dependency = match module_graph.dependency_by_id(dependency_id) {
            Some(dep) => dep,
            None => {
              // It means this dependency has been removed before
              continue;
            }
          };
          let need_bailout = matches!(
            dependency.dependency_type(),
            DependencyType::CommonJSRequireContext
              | DependencyType::RequireContext
              | DependencyType::DynamicImport
              | DependencyType::CjsRequire
              | DependencyType::CjsExportRequire
              | DependencyType::ImportContext
              | DependencyType::ContainerExposed
              | DependencyType::ProvideModuleForShared
          );

          if self.side_effects_free_modules.contains(module_id_by_dep_id)
            && !reachable_dependency_identifier.contains(module_id_by_dep_id)
            && !need_bailout
          {
            continue;
          }

          // we need to push either dependencies of context module instead of only its self, context module doesn't have ast,
          // which imply it will be treated as a external module in analyze phase
          if matches!(
            dependency.dependency_type(),
            DependencyType::CommonJSRequireContext
              | DependencyType::RequireContext
              | DependencyType::ImportContext
          ) {
            let deps_module_id_of_context_module = self
              .compilation
              .get_module_graph()
              .get_module_all_dependencies(module_id_by_dep_id)
              .map(|deps| {
                deps
                  .iter()
                  .filter_map(|dep| {
                    self
                      .compilation
                      .get_module_graph()
                      .module_identifier_by_dependency_id(dep)
                      .cloned()
                  })
                  .collect::<Vec<_>>()
              })
              .unwrap_or_default();
            q.extend(deps_module_id_of_context_module);
          }
          q.push_back(*module_id_by_dep_id);
        }
      }

      for node_index in visited_symbol_node_index {
        used_symbol_ref.insert(
          self
            .symbol_graph
            .get_symbol(&node_index)
            .expect("Can't get SymbolRef of NodeIndex")
            .clone(),
        );
      }
    } else {
      *used_symbol_ref = visited_symbol_ref;
    }
    include_module_ids
  }

  fn get_side_effects_free_modules(
    &self,
    mut side_effect_map: IdentifierMap<SideEffectType>,
  ) -> IdentifierSet {
    // normalize side_effects, there are two kinds of `side_effects` one from configuration and another from analyze ast
    for entry_module_ident in self.compilation.entry_module_identifiers.iter() {
      Self::normalize_side_effects(
        *entry_module_ident,
        &self.compilation.get_module_graph(),
        &mut IdentifierSet::default(),
        &mut side_effect_map,
      );
    }

    let side_effects_free_modules = side_effect_map
      .iter()
      .filter_map(|(k, v)| {
        let side_effect = match v {
          SideEffectType::Configuration(value) => value,
          SideEffectType::Analyze(value) => value,
        };
        if !side_effect {
          Some(*k)
        } else {
          None
        }
      })
      .collect::<IdentifierSet>();
    side_effects_free_modules
  }

  fn normalize_side_effects(
    cur: Identifier,
    module_graph: &ModuleGraph,
    visited_module: &mut IdentifierSet,
    side_effects_map: &mut IdentifierMap<SideEffectType>,
  ) {
    if visited_module.contains(&cur) {
      return;
    }
    visited_module.insert(cur);
    let mut module_ident_list = vec![];
    for dep in module_graph
      .get_module_all_dependencies(&cur)
      .unwrap_or_else(|| panic!("Failed to get mgm by module identifier {cur}"))
      .iter()
    {
      let dependency = module_graph
        .dependency_by_id(dep)
        .expect("should have dependency");
      if dependency.as_module_dependency().is_none() && dependency.as_context_dependency().is_none()
      {
        continue;
      }
      let Some(&module_ident) = module_graph.module_identifier_by_dependency_id(dep) else {
        let module = module_graph.module_by_identifier(&cur);

        if module
          .and_then(|module| module.as_context_module())
          .is_some()
        {
          // If the referenced module of context dependency is not found, then it might be failed to factorize in the first place. So let's skip it.
          continue;
        }
        let ast_or_source = module
          .and_then(|module| module.as_normal_module())
          .map(|normal_module| normal_module.source())
          .unwrap_or_else(|| panic!("Failed to get normal module of {}", cur));

        if matches!(ast_or_source, NormalModuleSource::Unbuild) {
          panic!("Failed to build module {cur}");
        } else {
          // If module is failed to build, we know that the build output can't run, so it is alright to generate a wrong tree-shaking result.
          // Also, if the referenced module of the dependency is not found, then it might failed to factorize in the first place. So let's skip it.
          continue;
        }
      };
      module_ident_list.push(module_ident);
      Self::normalize_side_effects(module_ident, module_graph, visited_module, side_effects_map);
    }
    // visited_module.remove(&cur);

    let need_change_to_side_effects_true = match side_effects_map.get(&cur) {
      // skip no deps or user already specified side effect in package.json
      Some(SideEffectType::Configuration(_)) | None => false,
      // already marked as side-effectful
      Some(SideEffectType::Analyze(true)) => false,
      Some(SideEffectType::Analyze(false)) => {
        let mut side_effect_list = module_ident_list.into_iter().filter(|ident| {
          matches!(
            side_effects_map.get(ident),
            Some(SideEffectType::Analyze(true)) | Some(SideEffectType::Configuration(true))
          )
        });
        side_effect_list.next().is_some()
        // uncomment below for debugging side_effect_list
        // let side_effect_list = side_effect_list.collect::<Vec<_>>();
        // side_effect_list.is_empty()
      }
    };

    if need_change_to_side_effects_true {
      if let Some(cur) = side_effects_map.get_mut(&cur) {
        *cur = SideEffectType::Analyze(true);
      }
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn mark_used_symbol_with(
    &mut self,
    analyze_map: &IdentifierMap<OptimizeAnalyzeResult>,
    mut init_queue: VecDeque<SymbolRefWithMemberChain>,
    evaluated_module_identifiers: &mut IdentifierSet,
    used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
    inherit_extend_graph: &GraphMap<ModuleIdentifier, (), Directed>,
    traced_tuple: &mut HashMap<(ModuleIdentifier, ModuleIdentifier), Vec<SymbolRef>>,
    visited_symbol_ref: &mut HashSet<SymbolRefWithMemberChain>,
    errors: &mut Vec<Error>,
  ) {
    while let Some(sym_ref) = init_queue.pop_front() {
      self.mark_symbol(
        sym_ref,
        analyze_map,
        &mut init_queue,
        evaluated_module_identifiers,
        used_export_module_identifiers,
        inherit_extend_graph,
        traced_tuple,
        visited_symbol_ref,
        errors,
      );
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn mark_symbol(
    &mut self,
    mut current_symbol_ref_with_member_chain: SymbolRefWithMemberChain,
    analyze_map: &IdentifierMap<OptimizeAnalyzeResult>,
    symbol_queue: &mut VecDeque<SymbolRefWithMemberChain>,
    evaluated_module_identifiers: &mut IdentifierSet,
    used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
    inherit_extend_graph: &GraphMap<ModuleIdentifier, (), Directed>,
    traced_tuple: &mut HashMap<(ModuleIdentifier, ModuleIdentifier), Vec<SymbolRef>>,
    visited_symbol_ref: &mut HashSet<SymbolRefWithMemberChain>,
    errors: &mut Vec<Error>,
  ) {
    current_symbol_ref_with_member_chain.0 = current_symbol_ref_with_member_chain
      .0
      .update_src_from_dep_id(&self.compilation.get_module_graph());
    if visited_symbol_ref.contains(&current_symbol_ref_with_member_chain) {
      return;
    } else {
      visited_symbol_ref.insert(current_symbol_ref_with_member_chain.clone());
    }
    let (current_symbol_ref, member_chain) = current_symbol_ref_with_member_chain;

    if !evaluated_module_identifiers.contains(&current_symbol_ref.importer()) {
      evaluated_module_identifiers.insert(current_symbol_ref.importer());
      if let Some(module_result) = analyze_map.get(&current_symbol_ref.importer()) {
        for used_symbol in module_result.used_symbol_refs.iter() {
          // graph.add_edge(&current_symbol_ref, used_symbol);
          symbol_queue.push_back((used_symbol.clone(), vec![]));
        }
      };
    }
    self.symbol_graph.add_node(&current_symbol_ref);
    // We don't need mark the symbol usage if it is from a bailout module because
    // bailout module will skipping tree-shaking anyway
    // let is_bailout_module_identifier = self.bailout_modules.contains_key(&current_symbol_ref.src());
    match &current_symbol_ref {
      SymbolRef::Declaration(symbol) => {
        merge_used_export_type(
          used_export_module_identifiers,
          symbol.src(),
          ModuleUsedType::DIRECT,
        );
      }
      SymbolRef::Indirect(IndirectTopLevelSymbol {
        ty: IndirectType::ReExport(_, _),
        src,
        ..
      }) => {
        merge_used_export_type(
          used_export_module_identifiers,
          *src,
          ModuleUsedType::REEXPORT,
        );
      }
      SymbolRef::Indirect(IndirectTopLevelSymbol {
        ty: IndirectType::Temp(_) | IndirectType::Import(_, _) | IndirectType::ImportDefault(_),
        src,
        ..
      }) => {
        merge_used_export_type(
          used_export_module_identifiers,
          *src,
          ModuleUsedType::INDIRECT,
        );
      }
      SymbolRef::Star(StarSymbol {
        ty: StarSymbolKind::ReExportAll,
        module_ident,
        ..
      })
      | SymbolRef::Star(StarSymbol {
        ty: StarSymbolKind::ReExportAllAs,
        module_ident,
        ..
      }) => {
        merge_used_export_type(
          used_export_module_identifiers,
          *module_ident,
          ModuleUsedType::EXPORT_ALL,
        );
      }
      SymbolRef::Url { src, .. } => {
        merge_used_export_type(used_export_module_identifiers, *src, ModuleUsedType::DIRECT);
      }
      _ => {}
    };
    match current_symbol_ref {
      SymbolRef::Declaration(ref symbol) => {
        let module_result = analyze_map.get(&symbol.src()).expect("TODO:");
        if let Some(set) = module_result
          .reachable_import_of_export
          .get(symbol.exported())
        {
          for symbol_ref_ele in set.iter() {
            self
              .symbol_graph
              .add_edge(&current_symbol_ref, symbol_ref_ele);
            let is_imported = module_result.import_map.get(&symbol.id().atom).is_some();
            let next_member_chain = if is_imported {
              // Considering following scenario, so we need to replace first name with local id
              // instead of exported name
              // import * as _Lib from "./lib";
              // export { _Lib as Lib };
              let mut normalized_member_chain = vec![symbol.id().atom.clone()];
              normalized_member_chain.extend(member_chain.iter().skip(1).cloned());
              normalized_member_chain
            } else {
              vec![]
            };

            symbol_queue.push_back((symbol_ref_ele.clone(), next_member_chain));
          }
        };

        // Assume the module name is app.js
        // ```js
        // import {myanswer, secret} from './lib'
        // export {myanswer as m, secret as s}
        // ```
        // In such scenario there are two `myanswer` binding would create
        // one for `app.js`, one for `lib.js`
        // the binding in `app.js` used for shake the `export {xxx}`
        // In other words, we need two binding for supporting indirect redirect.
        // if let Some(import_symbol_ref) = module_result.import_map.get(&symbol.id().atom) {
        //   dbg!(&symbol);
        //   dbg!(&member_chain);
        //   dbg!(&import_symbol_ref);
        //   self
        //     .symbol_graph
        //     .add_edge(&current_symbol_ref, import_symbol_ref);
        //
        //   symbol_queue.push_back((import_symbol_ref.clone(), member_chain));
        //
        //   dbg!(&symbol_queue);
        // }
      }
      SymbolRef::Indirect(ref indirect_symbol) => {
        let _importer = indirect_symbol.importer();
        let module_result = match analyze_map.get(&indirect_symbol.src) {
          Some(module_result) => module_result,
          None => {
            // eprintln!(
            //   "Can't get optimize dep result for module {}",
            //   indirect_symbol.uri,
            // );
            return;
          }
        };

        match module_result.export_map.get(indirect_symbol.indirect_id()) {
          Some(symbol) => match symbol {
            SymbolRef::Indirect(IndirectTopLevelSymbol {
              ty: IndirectType::ReExport(_, _),
              ..
            }) => {
              // This only happen when a bailout module have reexport statement, e.g. crates/rspack/tests/tree-shaking/ts-target-es5
              let is_same_symbol = &current_symbol_ref == symbol;
              if !is_same_symbol {
                self.symbol_graph.add_edge(&current_symbol_ref, symbol);
              }
              symbol_queue.push_back((symbol.clone(), member_chain.clone()));
              // if a bailout module has reexport symbol
              if let Some(set) = module_result
                .reachable_import_of_export
                .get(indirect_symbol.indirect_id())
              {
                for symbol_ref_ele in set.iter() {
                  self.symbol_graph.add_edge(symbol, symbol_ref_ele);
                  symbol_queue.push_back((symbol_ref_ele.clone(), member_chain.clone()));
                }
              };
            }
            _ => {
              self.symbol_graph.add_edge(&current_symbol_ref, symbol);
              symbol_queue.push_back((symbol.clone(), member_chain));
            }
          },

          None => {
            // TODO: better diagnostic and handle if multiple extends_map has export same symbol
            let mut ret = vec![];
            // Checking if any inherit export map is belong to a bailout module
            let mut bailout_module_identifiers = HashSet::default();
            let mut is_first_result = true;
            for (inherit_module_identifier, extends_export_map) in
              module_result.inherit_export_maps.iter()
            {
              //
              // ```js
              // // index.js
              // import {a} from './a.js'
              // a
              // // a.js
              // export * from './b.js'
              // export * from './c.js'
              // //b.js
              // export * from './d.js'
              // //c.js
              // export const c = 10;
              // //d.js
              // export const a = 3;
              // ```
              // the path is a.js -> b.js -> d.js
              if let Some(value) = extends_export_map.get(indirect_symbol.indirect_id()) {
                ret.push((inherit_module_identifier, value));
                if is_first_result {
                  let tuple = (indirect_symbol.src, *inherit_module_identifier);
                  let connection_stats = match traced_tuple.entry(tuple) {
                    Entry::Occupied(occ) => ReExportConnectionStatus::Occupied(occ.get().clone()),
                    Entry::Vacant(vac) => {
                      let mut reexport_paths = vec![];
                      for path in algo::all_simple_paths::<Vec<_>, _>(
                        &inherit_extend_graph,
                        indirect_symbol.src,
                        *inherit_module_identifier,
                        0,
                        None,
                      ) {
                        let mut reexport_path = vec![];
                        // TODO: use real dependencyID
                        for i in 0..path.len() - 1 {
                          let star_symbol = StarSymbol::new(
                            path[i + 1],
                            Default::default(),
                            path[i],
                            StarSymbolKind::ReExportAll,
                            DependencyId::default(),
                          );

                          let reexport_ref = SymbolRef::Star(star_symbol);
                          reexport_path.push(reexport_ref);
                        }
                        reexport_paths.push(reexport_path);
                      }
                      let first_reexport_of_each_path = reexport_paths
                        .iter()
                        .filter_map(|path| path.first().cloned())
                        .collect::<Vec<_>>();
                      vac.insert(first_reexport_of_each_path);
                      ReExportConnectionStatus::Vacant(reexport_paths)
                    }
                  };
                  match connection_stats {
                    ReExportConnectionStatus::Vacant(reexport_paths) => {
                      for reexport_path in reexport_paths {
                        let mut pre = &current_symbol_ref;
                        for reexport_ref in reexport_path.iter() {
                          self.symbol_graph.add_edge(pre, reexport_ref);
                          pre = reexport_ref;
                          if !evaluated_module_identifiers.contains(&reexport_ref.importer()) {
                            evaluated_module_identifiers.insert(reexport_ref.importer());
                            if let Some(module_result) = analyze_map.get(&reexport_ref.importer()) {
                              for used_symbol in module_result.used_symbol_refs.iter() {
                                symbol_queue.push_back((used_symbol.clone(), vec![]));
                              }
                            };
                          }
                          merge_used_export_type(
                            used_export_module_identifiers,
                            reexport_ref.importer(),
                            ModuleUsedType::EXPORT_ALL,
                          );
                        }
                        self.symbol_graph.add_edge(pre, value);
                      }
                    }
                    ReExportConnectionStatus::Occupied(ref first_reexport_of_each_path) => {
                      for reexport in first_reexport_of_each_path {
                        self.symbol_graph.add_edge(&current_symbol_ref, reexport);
                      }
                    }
                  }
                  is_first_result = false;
                }
              }
              if self.bailout_modules.contains_key(inherit_module_identifier) {
                bailout_module_identifiers.insert(*inherit_module_identifier);
              }
            }

            let selected_symbol = match ret.len() {
              0 if !bailout_module_identifiers.is_empty() => {
                for mi in bailout_module_identifiers {
                  let mid = SymbolRef::Star(StarSymbol {
                    src: mi,
                    binding: Default::default(),
                    module_ident: indirect_symbol.src(),
                    ty: StarSymbolKind::ReExportAll,
                    dep_id: DependencyId::default(),
                  });

                  self.symbol_graph.add_edge(&current_symbol_ref, &mid);
                  self.symbol_graph.add_edge(
                    &mid,
                    &SymbolRef::Declaration(Symbol::new(
                      mi,
                      BetterId {
                        ctxt: SyntaxContext::empty(),
                        atom: indirect_symbol.indirect_id().clone(),
                      },
                      SymbolType::Temp,
                      None,
                    )),
                  );

                  merge_used_export_type(
                    used_export_module_identifiers,
                    mi,
                    ModuleUsedType::DIRECT,
                  );
                }
                return;
              }
              0 => {
                self.symbol_graph.add_edge(
                  &current_symbol_ref,
                  &SymbolRef::Declaration(Symbol::new(
                    module_result.module_identifier,
                    BetterId {
                      ctxt: SyntaxContext::empty(),
                      atom: indirect_symbol.indirect_id().clone(),
                    },
                    SymbolType::Temp,
                    None,
                  )),
                );
                merge_used_export_type(
                  used_export_module_identifiers,
                  current_symbol_ref.src(),
                  ModuleUsedType::INDIRECT,
                );
                return;

                // Only report diagnostic when following conditions are satisfied:
                // 1. src module is not a bailout module and src module using ESM syntax to export some symbols.
                // 2. src module has no reexport or any reexport src module is not bailouted
                // let should_diagnostic = !is_bailout_module_identifier
                //   && module_result.module_syntax == ModuleSyntax::ESM
                //   && (module_result.inherit_export_maps.is_empty()
                //     || !has_bailout_module_identifiers);
                // if should_diagnostic {
                //   let module_path = self
                //     .compilation
                //     .get_module_graph()
                //     .normal_module_source_path_by_identifier(&module_result.module_identifier);
                //   let importer_module_path = self
                //     .compilation
                //     .get_module_graph()
                //     .normal_module_source_path_by_identifier(&indirect_symbol.importer());
                //   if let (Some(module_path), Some(importer_module_path)) =
                //     (module_path, importer_module_path)
                //   {
                //     let error_message = format!(
                //       "{} did not export `{}`, imported by {}",
                //       contextify(&self.compilation.options.context, &module_path),
                //       indirect_symbol.indirect_id(),
                //       contextify(&self.compilation.options.context, &importer_module_path),
                //     );
                //     errors.push(Error::InternalError(InternalError {
                //       error_message,
                //       severity: Severity::Warn,
                //     }));
                //   }
                //   return;
                // } else {
                //   // TODO: This branch should be remove after we analyze module.exports
                //   // If one of inherit module is a bailout module, that most probably means that module has some common js export
                //   // which we don't analyze yet, we just pass it. It is alright because we don't modified the ast of bailout module
                //   return;
                // }
              }
              1 => ret[0].1.clone(),
              // multiple export candidate in reexport
              // mark the first symbol_ref as used, align to webpack
              _ => {
                let mut error_message = format!(
                  "Conflicting star exports for the name '{}' in ",
                  indirect_symbol.indirect_id()
                );
                // let cwd = std::env::current_dir();
                let module_graph = self.compilation.get_module_graph();
                let module_identifier_list = ret
                  .iter()
                  .filter_map(|(module_identifier, _)| {
                    module_graph.normal_module_source_path_by_identifier(module_identifier)
                  })
                  .map(|identifier| {
                    contextify(self.compilation.options.context.clone(), &identifier)
                  })
                  .collect::<Vec<_>>();
                error_message += &join_string_component(module_identifier_list);
                errors.push(InternalError::new(error_message, Severity::Warn).into());
                ret[0].1.clone()
              }
            };
            symbol_queue.push_back((selected_symbol, member_chain));
          }
        };
        // graph.add_edge(&current_symbol_ref, &symbol);

        // symbol_queue.push_back(symbol);
      }
      SymbolRef::Star(ref star_symbol) => {
        // If a star ref is used. e.g.
        // ```js
        // import * as all from './test.js'
        // all
        // ```
        // then, all the exports in `test.js` including
        // export defined in `test.js` and all related
        // reexport should be marked as used
        let src_module_identifier: Identifier = star_symbol.src();
        let analyze_result = match analyze_map.get(&src_module_identifier) {
          Some(analyze_result) => analyze_result,
          None => {
            let module_graph = self.compilation.get_module_graph();
            if is_js_like_uri(&src_module_identifier) {
              let module_path =
                module_graph.normal_module_source_path_by_identifier(&star_symbol.src());
              if let Some(module_path) = module_path {
                let error_message = format!("Can't get analyze result of {module_path}");
                errors.push(InternalError::new(error_message, Severity::Warn).into());
              }
            }
            return;
          }
        };

        let (include_default_export, next_member_chain) = match star_symbol.ty() {
          StarSymbolKind::ReExportAllAs => {
            let next_member_chain = if let Some(name) = member_chain.first()
              && name == star_symbol.binding()
            {
              member_chain[1..].to_vec()
            } else {
              vec![]
            };
            (true, next_member_chain)
          }
          StarSymbolKind::ImportAllAs => {
            let next_member_chain = if let Some(name) = member_chain.first()
              && name == star_symbol.binding()
            {
              member_chain[1..].to_vec()
            } else {
              vec![]
            };
            (true, next_member_chain)
          }
          StarSymbolKind::ReExportAll => (false, vec![]),
        };

        // try to access first member expr element
        if let Some(name) = next_member_chain.first() {
          if let Some(export_symbol_ref) = analyze_result.export_map.get(name) {
            self
              .symbol_graph
              .add_edge(&current_symbol_ref, export_symbol_ref);
            symbol_queue.push_back((export_symbol_ref.clone(), next_member_chain.to_vec()));
            return;
          }

          for (inherit_module_identifier, extends_export_map) in
            analyze_result.inherit_export_maps.iter()
          {
            if let Some(value) = extends_export_map.get(name) {
              let tuple = (star_symbol.src, *inherit_module_identifier);
              let connection_stats = match traced_tuple.entry(tuple) {
                Entry::Occupied(occ) => {
                  // self.symbol_graph.add_edge(&current_symbol_ref, to);
                  ReExportConnectionStatus::Occupied(occ.get().clone())
                }
                Entry::Vacant(vac) => {
                  let mut reexport_paths = vec![];
                  for path in algo::all_simple_paths::<Vec<_>, _>(
                    &inherit_extend_graph,
                    star_symbol.src,
                    *inherit_module_identifier,
                    0,
                    None,
                  ) {
                    let mut reexport_path = vec![];
                    for i in 0..path.len() - 1 {
                      // TODO: use real dependency id, currently we don't have
                      let star_symbol = StarSymbol::new(
                        path[i + 1],
                        Default::default(),
                        path[i],
                        StarSymbolKind::ReExportAll,
                        DependencyId::default(),
                      );

                      let reexport_ref = SymbolRef::Star(star_symbol);
                      reexport_path.push(reexport_ref);
                    }
                    reexport_paths.push(reexport_path);
                  }
                  let first_reexport_of_each_path = reexport_paths
                    .iter()
                    .filter_map(|path| path.first().cloned())
                    .collect::<Vec<_>>();
                  vac.insert(first_reexport_of_each_path);
                  ReExportConnectionStatus::Vacant(reexport_paths)
                }
              };
              match connection_stats {
                ReExportConnectionStatus::Vacant(reexport_paths) => {
                  for reexport_path in reexport_paths {
                    let mut pre = &current_symbol_ref;
                    for reexport_ref in reexport_path.iter() {
                      self.symbol_graph.add_edge(pre, reexport_ref);
                      pre = reexport_ref;
                      if !evaluated_module_identifiers.contains(&reexport_ref.importer()) {
                        evaluated_module_identifiers.insert(reexport_ref.importer());
                        if let Some(module_result) = analyze_map.get(&reexport_ref.importer()) {
                          for used_symbol in module_result.used_symbol_refs.iter() {
                            symbol_queue.push_back((used_symbol.clone(), vec![]));
                          }
                        };
                      }
                      merge_used_export_type(
                        used_export_module_identifiers,
                        reexport_ref.importer(),
                        ModuleUsedType::EXPORT_ALL,
                      );
                    }
                    self.symbol_graph.add_edge(pre, value);
                  }
                }
                ReExportConnectionStatus::Occupied(ref first_reexport_of_each_path) => {
                  for reexport in first_reexport_of_each_path {
                    self.symbol_graph.add_edge(&current_symbol_ref, reexport);
                  }
                }
              }

              symbol_queue.push_back((value.clone(), next_member_chain));
              return;
            }
            // has_bailout_module_identifiers = has_bailout_module_identifiers
            //   || self.bailout_modules.contains_key(module_identifier);
          }
        }

        if !matches!(star_symbol.ty(), StarSymbolKind::ReExportAll) {
          // It means the module has not export or reexport target reference, maybe the src module
          // is a empty module, we should avoid to eliminate the module even it is a sideEffects
          // free module
          merge_used_export_type(
            used_export_module_identifiers,
            src_module_identifier,
            ModuleUsedType::EXPORT_ALL,
          );
        }
        // Failed to look up a specific element, connect all
        for (key, export_symbol_ref) in analyze_result.export_map.iter() {
          if !include_default_export && key == "default" {
          } else {
            self
              .symbol_graph
              .add_edge(&current_symbol_ref, export_symbol_ref);
            symbol_queue.push_back((export_symbol_ref.clone(), vec![]));
          }
        }

        for (key, _) in analyze_result.inherit_export_maps.iter() {
          let export_all = SymbolRef::Star(StarSymbol::new(
            *key,
            Default::default(),
            src_module_identifier,
            StarSymbolKind::ReExportAll,
            DependencyId::default(),
          ));
          self.symbol_graph.add_edge(&current_symbol_ref, &export_all);
          symbol_queue.push_back((export_all.clone(), vec![]));
        }
      }
      SymbolRef::Url { .. } | SymbolRef::Worker { .. } => {}
      SymbolRef::Usage(ref binding, ref member_chain, ref src) => {
        let analyze_result = analyze_map.get(src).expect("Should have analyze result");
        if let Some(import_symbol_ref) = analyze_result.import_map.get(binding) {
          self
            .symbol_graph
            .add_edge(&current_symbol_ref, import_symbol_ref);
          let mut next_member_chain = vec![binding.clone()];
          next_member_chain.extend(member_chain.iter().cloned());
          symbol_queue.push_back((import_symbol_ref.clone(), next_member_chain));
        }
      }
    }
  }
  #[allow(clippy::too_many_arguments)]
  fn collect_from_entry_like(
    &mut self,
    analyze_map: &IdentifierMap<OptimizeAnalyzeResult>,
    entry_identifier: ModuleIdentifier,
    evaluated_module_identifiers: &mut IdentifierSet,
    used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
    inherit_extend_graph: &GraphMap<ModuleIdentifier, (), Directed>,
    traced_tuple: &mut HashMap<(ModuleIdentifier, ModuleIdentifier), Vec<SymbolRef>>,
    entry_type: EntryLikeType,
    visited_symbol_ref: &mut HashSet<SymbolRefWithMemberChain>,
    errors: &mut Vec<Error>,
  ) {
    let mut q = VecDeque::new();
    let entry_module_result = match analyze_map.get(&entry_identifier) {
      Some(result) => result,
      None => {
        // TODO: checking if it is none js type
        return;
        // panic!("Can't get analyze result from entry_identifier {}", entry_identifier);
      }
    };

    // by default webpack will not mark the `export *` as used in entry module
    if matches!(entry_type, EntryLikeType::Bailout) {
      let inherit_export_symbols = get_inherit_export_symbol_ref(entry_module_result);
      if !inherit_export_symbols.is_empty() {
        // transitive bailout
        inherit_export_symbols.iter().for_each(|item| {
          q.push_back((
            SymbolRef::Star(StarSymbol::new(
              item.src(),
              Default::default(),
              entry_identifier,
              StarSymbolKind::ReExportAll,
              DependencyId::default(),
            )),
            vec![],
          ));
        });
      }
      q.extend(
        inherit_export_symbols
          .into_iter()
          .map(|item| (item, vec![])),
      );
      q.extend(
        entry_module_result
          .used_symbol_refs
          .iter()
          .cloned()
          .map(|item| (item, vec![])),
      );
    }

    for item in entry_module_result.export_map.values() {
      self.mark_symbol(
        (item.clone(), vec![]),
        analyze_map,
        &mut q,
        evaluated_module_identifiers,
        used_export_module_identifiers,
        inherit_extend_graph,
        traced_tuple,
        visited_symbol_ref,
        errors,
      );
    }

    while let Some(symbol_ref) = q.pop_front() {
      self.mark_symbol(
        symbol_ref,
        analyze_map,
        &mut q,
        evaluated_module_identifiers,
        used_export_module_identifiers,
        inherit_extend_graph,
        traced_tuple,
        visited_symbol_ref,
        errors,
      );
    }
  }
}

// TODO: dep replacement
// fn dependency_replacement() {
// let module_item_map = if side_effects_options {
//   // let start = Instant::now();
//   // let dependency_replacement = update_dependency(
//   //   &symbol_graph,
//   //   &used_export_module_identifiers,
//   //   &bailout_module_identifiers,
//   //   &side_effects_free_modules,
//   //   &compilation.entry_module_identifiers,
//   // );

//   // // dbg!(&dependency_replacement);

//   // // apply replacement start
//   // // let mut module_item_map = IdentifierMap::default();
//   // let module_item_map = compilation.apply_dependency_replacement(
//   //   dependency_replacement,
//   //   &mut dead_nodes_index,
//   //   &mut symbol_graph,
//   // );

//   // // dbg!(&module_item_map.keys().collect::<Vec<_>>());
//   // dbg!(&start.elapsed());
//   // // module_item_map
//   IdentifierMap::default()
// } else {
//   IdentifierMap::default()
// };
// }

fn get_inherit_export_ref_graph(
  analyze_result_map: &mut IdentifierMap<OptimizeAnalyzeResult>,
  mg: &ModuleGraph,
) -> GraphMap<Identifier, (), Directed> {
  // calculate relation of module that has `export * from 'xxxx'`
  let inherit_export_ref_graph = create_inherit_graph(&*analyze_result_map, mg);
  // key is the module_id of module that potential have reexport all symbol from other module
  // value is the set which contains several module_id the key related module need to inherit
  let map_of_inherit_map = get_extends_map(&inherit_export_ref_graph);

  for (module_id, inherit_export_module_id) in map_of_inherit_map.iter() {
    // This is just a work around for rustc checker, because we have immutable and mutable borrow at the same time.
    let mut inherit_export_maps = {
      let main_module = if let Some(result) = analyze_result_map.get_mut(module_id) {
        result
      } else {
        tracing::warn!("Can't get analyze result of {}", module_id);
        continue;
      };
      std::mem::take(&mut main_module.inherit_export_maps)
    };
    for inherit_export_module_identifier in inherit_export_module_id {
      let inherit_export_map = if let Some(inherit_export_map) = analyze_result_map
        .get(inherit_export_module_identifier)
        .map(|analyze_result| {
          analyze_result
            .export_map
            .iter()
            .filter_map(|(k, v)| {
              // export * should not reexport default export
              if k == "default" {
                None
              } else {
                Some((k.clone(), v.clone()))
              }
            })
            .collect::<HashMap<Atom, SymbolRef>>()
        }) {
        inherit_export_map
      } else {
        tracing::warn!(
          "Can't get analyze result of {}",
          inherit_export_module_identifier
        );
        HashMap::default()
      };

      inherit_export_maps.insert(*inherit_export_module_identifier, inherit_export_map);
    }
    analyze_result_map
      .get_mut(module_id)
      .unwrap_or_else(|| panic!("Module({module_id:?}) not found"))
      .inherit_export_maps = inherit_export_maps;
  }
  inherit_export_ref_graph
}

// async fn par_analyze_module(compilation: &mut Compilation) -> IdentifierMap<OptimizeAnalyzeResult> {
//   let analyze_results = {
//     compilation
//       .get_module_graph()
//       .get_module_graph()_modules()
//       .par_iter()
//       .filter_map(|(module_identifier, mgm)| {
//         let optimize_analyze_result = if mgm.module_type.is_js_like() {
//           match compilation
//             .get_module_graph()
//             .module_by_identifier(&mgm.module_identifier)
//             .and_then(|module| module.as_normal_module().and_then(|m| m.ast()))
//             // A module can missing its AST if the module is failed to build
//             .and_then(|ast| ast.as_javascript())
//           {
//             Some(ast) => JsModule::new(
//               ast,
//               &mgm
//                 .dependencies
//                 .iter()
//                 .filter_map(|id| compilation.get_module_graph().dependency_by_id(id).cloned())
//                 .collect::<Vec<_>>(),
//               *module_identifier,
//               &compilation.options,
//             )
//             .analyze(),
//             None => {
//               return None;
//             }
//           }
//         } else {
//           AssetModule::new(*module_identifier).analyze()
//         };

//         // dbg_matches!(
//         //   &module_identifier.as_str(),
//         //   &optimize_analyze_result.reachable_import_of_export,
//         //   &optimize_analyze_result.used_symbol_refs,
//         //   &optimize_analyze_result.export_map,
//         //   &optimize_analyze_result.import_map,
//         //   &optimize_analyze_result.side_effects
//         // );

//         Some((*module_identifier, optimize_analyze_result))
//       })
//       .collect::<IdentifierMap<OptimizeAnalyzeResult>>()
//   };
//   analyze_results
// }

fn update_reachable_dependency(
  symbol_ref: &SymbolRef,
  reachable_dependency_identifier: &mut IdentifierSet,
  symbol_graph: &SymbolGraph,
) {
  let root_module_identifier = symbol_ref.importer();
  let node_index = *symbol_graph
    .get_node_index(symbol_ref)
    .unwrap_or_else(|| panic!("Can't get NodeIndex of {symbol_ref:?}"));
  let mut q = VecDeque::from_iter([node_index]);
  let mut visited = HashSet::default();
  while let Some(cur) = q.pop_front() {
    if visited.contains(&cur) {
      continue;
    } else {
      visited.insert(cur);
    }
    let symbol = symbol_graph
      .get_symbol(&cur)
      .expect("Can't get Symbol of NodeIndex");
    let module_identifier = symbol.src();
    if module_identifier == root_module_identifier {
      for ele in symbol_graph
        .graph
        .edges_directed(node_index, petgraph::Direction::Outgoing)
      {
        let target_node_index = ele.target();
        q.push_back(target_node_index);
      }
    } else {
      reachable_dependency_identifier.insert(module_identifier);
    }
  }
}

fn update_reachable_symbol(
  dead_node_index: &HashSet<NodeIndex>,
  symbol_node_index: NodeIndex,
  visited_symbol_node_index: &mut HashSet<NodeIndex>,
) {
  if dead_node_index.contains(&symbol_node_index) {
    return;
  }
  visited_symbol_node_index.insert(symbol_node_index);
}

fn is_js_like_uri(uri: &str) -> bool {
  fn resolve_module_type_by_uri(uri: &str) -> Option<ModuleType> {
    let uri = std::path::Path::new(uri);
    let ext = uri.extension()?.to_str()?;
    Some(ModuleType::from(ext))
  }
  match resolve_module_type_by_uri(uri) {
    Some(module_type) => matches!(
      module_type,
      crate::ModuleType::Js | crate::ModuleType::JsDynamic | crate::ModuleType::JsEsm
    ),
    None => false,
  }
}

fn get_extends_map(
  export_all_ref_graph: &GraphMap<ModuleIdentifier, (), petgraph::Directed>,
) -> IdentifierMap<IdentifierLinkedSet> {
  let mut map = IdentifierMap::default();
  for node in export_all_ref_graph.nodes() {
    let reachable_set = get_reachable(node, export_all_ref_graph);
    map.insert(node, reachable_set);
  }
  map
}

fn get_reachable(
  start: ModuleIdentifier,
  g: &GraphMap<ModuleIdentifier, (), petgraph::Directed>,
) -> IdentifierLinkedSet {
  let mut dfs = Dfs::new(&g, start);

  let mut reachable_module_id = IdentifierLinkedSet::default();
  while let Some(next) = dfs.next(g) {
    // reachable inherit export map should not include self.
    if reachable_module_id.contains(&next) || next == start {
      continue;
    } else {
      reachable_module_id.insert(next);
    }
  }
  reachable_module_id
}

fn create_inherit_graph(
  analyze_map: &IdentifierMap<OptimizeAnalyzeResult>,
  mg: &ModuleGraph,
) -> GraphMap<ModuleIdentifier, (), petgraph::Directed> {
  let mut g = DiGraphMap::new();
  for (module_id, result) in analyze_map.iter() {
    for export_all_dep_id in result.export_all_dep_id.iter().rev() {
      if let Some(export_all_module_id) = mg.module_identifier_by_dependency_id(export_all_dep_id) {
        g.add_edge(*module_id, *export_all_module_id, ());
      }
    }
  }
  g
}

pub fn merge_used_export_type(
  used_export_map: &mut IdentifierMap<ModuleUsedType>,
  module_id: ModuleIdentifier,
  ty: ModuleUsedType,
) {
  match used_export_map.entry(module_id) {
    Entry::Occupied(mut occ) => {
      occ.borrow_mut().get_mut().insert(ty);
    }
    Entry::Vacant(vac) => {
      vac.insert(ty);
    }
  }
}

fn get_inherit_export_symbol_ref(entry_module_result: &OptimizeAnalyzeResult) -> Vec<SymbolRef> {
  let mut export_atom = HashSet::default();
  let mut inherit_export_symbols = vec![];
  // All the reexport star symbol should be included in the bundle
  // TODO: webpack will emit an warning, we should align to them
  for inherit_map in entry_module_result.inherit_export_maps.values() {
    for (atom, symbol_ref) in inherit_map.iter() {
      if export_atom.contains(atom) {
        continue;
      } else {
        export_atom.insert(atom.clone());
        inherit_export_symbols.push(symbol_ref.clone());
      }
    }
  }
  inherit_export_symbols
}

fn get_symbol_path(symbol_graph: &SymbolGraph, cur: NodeIndex) -> Vec<Vec<SymbolRef>> {
  fn recursive(
    symbol_graph: &SymbolGraph,
    cur_path: &mut Vec<NodeIndex>,
    paths: &mut Vec<Vec<NodeIndex>>,
    visited_node: &mut HashSet<NodeIndex>,
    cur: NodeIndex,
  ) {
    if visited_node.contains(&cur) {
      return;
    }
    cur_path.push(cur);
    visited_node.insert(cur);
    let mut has_outgoing = false;
    for edge in symbol_graph
      .graph
      .edges_directed(cur, petgraph::Direction::Outgoing)
    {
      has_outgoing = true;
      let to = edge.target();
      recursive(symbol_graph, cur_path, paths, visited_node, to);
    }
    visited_node.remove(&cur);
    if !has_outgoing {
      paths.push(cur_path.clone());
    }
  }
  let mut cur_path = vec![];
  let mut paths = vec![];
  let mut visited_node = HashSet::default();
  recursive(
    symbol_graph,
    &mut cur_path,
    &mut paths,
    &mut visited_node,
    cur,
  );
  paths
    .into_iter()
    .map(|path| {
      path
        .into_iter()
        .map(|node_index| {
          symbol_graph
            .get_symbol(&node_index)
            .cloned()
            .expect("Can't get nodeIndex of SymbolRef")
        })
        .collect::<Vec<_>>()
    })
    .collect::<Vec<_>>()
}
// TODO: dep replacement
// fn update_dependency(
//   symbol_graph: &SymbolGraph,
//   used_export_module_identifiers: &IdentifierMap<ModuleUsedType>,
//   bail_out_module_identifiers: &IdentifierMap<BailoutFlog>,
//   side_effects_free_modules: &IdentifierSet,
//   entry_modules_identifier: &IdentifierSet,
// ) -> Vec<DependencyReplacement> {
//   // dbg!(&used_export_module_identifiers);
//   let mut exported_symbol_set = HashSet::default();
//   let mut dependency_replacement_list = vec![];
//   let directed_symbol_node_set = symbol_graph
//     .symbol_to_index
//     .iter()
//     .filter_map(|(k, v)| {
//       if matches!(k, SymbolRef::Direct(_)) {
//         Some(*v)
//       } else {
//         None
//       }
//     })
//     .collect::<HashSet<NodeIndex>>();
//   for (symbol_ref, node_index) in symbol_graph.symbol_to_index.iter() {
//     // println!("----------------");
//     if !matches!(symbol_ref, SymbolRef::Direct(_)) {
//       continue;
//     }

//     let mut paths = Vec::new();
//     recursive_visited(
//       symbol_graph,
//       &mut vec![],
//       &mut paths,
//       &mut HashSet::default(),
//       *node_index,
//       &directed_symbol_node_set,
//     );
//     let symbol_paths = paths
//       .into_par_iter()
//       .map(|path| {
//         path
//           .iter()
//           .map(|node_index| symbol_graph.get_symbol(node_index).unwrap().clone())
//           .collect::<Vec<_>>()
//       })
//       .collect::<Vec<_>>();
//     // dbg!(&symbol_paths);
//     // sliding window
//     for symbol_path in symbol_paths {
//       // dbg!(&symbol_path);
//       let mut start = 0;
//       let mut end = 0;
//       init_sliding_window(&mut start, &mut end, &symbol_path);

//       while end < symbol_path.len() {
//         let end_symbol = &symbol_path[end];
//         // let is_reexport = end_symbol.is_star()
//         let owner_module_identifier = end_symbol.module_identifier();
//         // let is_owner_module_export_used = used_export_module_identifiers
//         //   .get(&owner_module_identifier)
//         //   .map(|flag| flag.contains(ModuleUsedType::DIRECT))
//         //   .unwrap_or(false);

//         // TODO: optimize export *
//         // safe to process
//         // if is_owner_module_export_used
//         //   || bail_out_module_identifiers.contains_key(&owner_module_identifier)
//         //   || !side_effects_free_modules.contains(&owner_module_identifier)
//         //   || entry_modules_identifier.contains(&owner_module_identifier)
//         // {
//         //   if end - start > 1 {
//         //     println!("cant removed: {start}, {end}");
//         //     validate_and_insert_replacement(
//         //       false,
//         //       &mut dependency_replacement_list,
//         //       &symbol_path,
//         //       end - 1,
//         //       start,
//         //       used_export_module_identifiers,
//         //     );
//         //     // dependency_replacement_list.push(DependencyReplacement {
//         //     //   from: symbol_path[end].clone(),
//         //     //   replacement: symbol_path[start].clone(),
//         //     // })
//         //   }
//         //   init_sliding_window(&mut start, &mut end, &symbol_path);
//         //   continue;
//         // }

//         if !end_symbol.is_skipable_symbol() && end != symbol_path.len() - 1 {
//           if end - start > 1 {
//             // println!("none reexport: {start}, {end}");
//             validate_and_insert_replacement(
//               false,
//               &mut dependency_replacement_list,
//               &symbol_path,
//               end - if end_symbol.is_indirect() { 0 } else { 1 },
//               start,
//               used_export_module_identifiers,
//               &mut exported_symbol_set,
//             );
//           }

//           init_sliding_window(&mut start, &mut end, &symbol_path);

//           continue;
//         }
//         end += 1;
//       }
//       // because last window range is [start, end - 1]
//       if end - start > 1 {
//         // println!("end check: {start}, {end}");
//         validate_and_insert_replacement(
//           true,
//           &mut dependency_replacement_list,
//           &symbol_path,
//           end - 1,
//           start,
//           used_export_module_identifiers,
//           &mut exported_symbol_set,
//         );
//       }
//     }
//     // println!("end ----------------");
//   }
//   // dbg!(&exported_symbol_set);
//   dependency_replacement_list
// }

// TODO: dep replacement
// fn validate_and_insert_replacement(
//   end_check: bool,
//   dependency_replacement_list: &mut Vec<DependencyReplacement>,
//   symbol_path: &Vec<SymbolRef>,
//   end: usize,
//   start: usize,
//   used_export_module_identifiers: &IdentifierMap<ModuleUsedType>,
//   used_export_symbol_set: &mut HashSet<Identifier>,
// ) {
//   enum CheckResult {
//     Valid,
//     Invalid,
//     Wrong,
//   }
//   // println!(
//   //   "{:#?}, \n{:#?}, {}",
//   //   &symbol_path[start], &symbol_path[end], end_check
//   // );

//   let unused_export_symbol = symbol_path[start..=end]
//     .iter()
//     .filter(|symbol| {
//       used_export_module_identifiers
//         .get(&symbol.module_identifier())
//         .map(|ty| !ty.contains(ModuleUsedType::DIRECT))
//         .unwrap_or(false)
//     })
//     .map(|s| s.module_identifier())
//     .collect::<HashSet<_>>();

//   used_export_symbol_set.extend(unused_export_symbol.iter().cloned());

//   // dbg!(&unused_export_symbol);
//   let is_valid_path = match (&symbol_path[start], &symbol_path[end]) {
//     (SymbolRef::Direct(_), SymbolRef::Direct(_)) => false,
//     (SymbolRef::Direct(replace), SymbolRef::Indirect(original)) => {
//       // validate if we this path point to same symbol
//       // we know that start must be has `SymbolType == Define`
//       if end - start > 1 {
//         // dbg!(&&symbol_path[start..=end]);
//       }
//       is_same_symbol(original, end, start, symbol_path, replace)
//     }
//     (SymbolRef::Direct(_), SymbolRef::Star(_)) => false,
//     (SymbolRef::Indirect(_), SymbolRef::Direct(_)) => false,
//     (SymbolRef::Indirect(replace), SymbolRef::Indirect(_)) => {
//       matches!(replace.ty, IndirectType::ReExport(_, _))
//     }
//     (SymbolRef::Indirect(_), SymbolRef::Star(_)) => {
//       // todo!()
//       // TODO:
//       false
//     }
//     (SymbolRef::Star(_), SymbolRef::Direct(_)) => todo!(),
//     (SymbolRef::Star(replace), SymbolRef::Indirect(_)) => {
//       replace.ty == StarSymbolKind::ReExportAll
//       // dbg!(&symbol_path[start]);
//       // dbg!(&symbol_path[end]);
//       // todo!()
//     }
//     (SymbolRef::Star(_), SymbolRef::Star(_)) => todo!(),
//   };
//   if unused_export_symbol.len() > 0 && is_valid_path && end - start > 1 {
//     if symbol_path[start..=end].len() > 3 {
//       // dbg!(&symbol_path[start..=end]);
//     }
//     dependency_replacement_list.push(DependencyReplacement {
//       original: symbol_path[end].clone(),
//       to: symbol_path[end - 1].clone(),
//       replacement: symbol_path[start].clone(),
//       unused_export_symbol_count: unused_export_symbol.len(),
//     })
//   } else {
//     if !is_valid_path && !end_check {
//       println!(
//         "{:#?}, \n{:#?}, {}",
//         &symbol_path[start], &symbol_path[end], end_check
//       );
//     }
//   }
//   // if has_unused_export_symbol {
//   // }
// }

// TODO: dep replacement
// fn is_same_symbol(
//   original: &IndirectTopLevelSymbol,
//   end: usize,
//   start: usize,
//   symbol_path: &Vec<SymbolRef>,
//   replace: &Symbol,
// ) -> bool {
//   // dbg!(&symbol_path);
//   let mut pre = match original.ty {
//     IndirectType::ReExport(ref original, ref exported) => original.clone(),
//     _ => original.indirect_id().clone(),
//   };
//   let mut i = end - 1;
//   while i > start {
//     let cur = &symbol_path[i];
//     let next_id = match cur {
//       SymbolRef::Direct(_) => unreachable!(),
//       SymbolRef::Indirect(indirect) => match &indirect.ty {
//         IndirectType::Temp(ref id) => {
//           if id != &pre {
//             return false;
//           }
//           id.clone()
//         }
//         IndirectType::ReExport(original, _) => {
//           // let exported = indirect.id();
//           if &pre != indirect.id() {
//             return false;
//           }
//           original.clone()
//         }
//         IndirectType::Import(..) => {
//           unreachable!()
//         }
//         IndirectType::ImportDefault(_) => {
//           unreachable!()
//         }
//       },
//       SymbolRef::Star(_) => pre,
//     };
//     pre = next_id;
//     i -= 1;
//   }
//   pre == replace.id().atom
// }

// fn init_sliding_window(start: &mut usize, end: &mut usize, symbol_path: &Vec<SymbolRef>) {
//   // println!("{start}, {end}");
//   *start = *end;
//   while *start < symbol_path.len() && !could_be_start_of_path(&symbol_path[*start]) {
//     *start += 1;
//   }
//   *end = *start + 1;
//   loop {
//     if *end >= symbol_path.len() {
//       break;
//     }
//     if symbol_path[*end].module_identifier() != symbol_path[*start].module_identifier() {
//       break;
//     }
//     *end += 1;
//   }
// }

// #[inline]
// pub fn could_be_start_of_path(symbol: &SymbolRef) -> bool {
//   match symbol {
//     SymbolRef::Direct(direct) => direct.ty() == &SymbolType::Define,
//     SymbolRef::Indirect(indirect) => matches!(indirect.ty, IndirectType::ReExport(_, _)),
//     SymbolRef::Star(star) => star.ty == StarSymbolKind::ReExportAll,
//   }
// }

// TODO: dep replacement
// fn apply_dependency_replacement(
//   &mut self,
//   dependency_replacement: Vec<DependencyReplacement>,
//   dead_nodes_index: &mut HashSet<NodeIndex>,
//   symbol_graph: &mut SymbolGraph,
// ) -> IdentifierMap<Vec<ModuleItem>> {
//   let mut module_item_map: IdentifierMap<Vec<ModuleItem>> = IdentifierMap::default();
//   // let mut dead_nodes: HashSet<NodeIndex> = HashSet::new();
//   let temp_global = Default::default();
//   GLOBALS.set(&temp_global, || {
//     let top_level_mark = Mark::new();
//     for replace in dependency_replacement {
//       let DependencyReplacement {
//         original,
//         to,
//         replacement,
//         ..
//       } = replace;
//       // dbg!(&t);
//       symbol_graph.remove_edge(&original, &to);
//       symbol_graph.add_edge(&original, &replacement);
//       let original_node_index = symbol_graph.get_node_index(&original).cloned().unwrap();
//       dead_nodes_index.insert(original_node_index);

//       let replace_src_module_id = replacement.module_identifier();
//       let contextify_src = contextify(&self.options.context, &replace_src_module_id);
//       // TODO: Consider multiple replacement points to same original [SymbolRef]
//       let (module_decl, module_ident) = match (original, to) {
//         (SymbolRef::Indirect(ref indirect), to) => {
//           let importer = indirect.importer();
//           let local_binding = match &indirect.ty {
//             IndirectType::Temp(_) => todo!(),
//             IndirectType::ReExport(original, exported) => match exported {
//               Some(exported) => exported,
//               None => original,
//             },
//             IndirectType::Import(local, imported) => local,
//             IndirectType::ImportDefault(binding) => binding,
//           };
//           let import_binding = match replacement {
//             SymbolRef::Direct(direct) => Some(direct.id().atom.clone()),
//             SymbolRef::Indirect(indirect) => Some(indirect.indirect_id().clone()),
//             SymbolRef::Star(_) => None,
//           };
//           let module_decl = match (import_binding, local_binding) {
//             (Some(import_binding), local_binding) => {
//               let is_reexport_all = matches!(indirect.ty, IndirectType::ReExport(_, _));
//               if is_reexport_all {
//                 let specifier = ExportSpecifier::Named(ExportNamedSpecifier {
//                   span: DUMMY_SP,
//                   exported: if local_binding == &import_binding {
//                     None
//                   } else {
//                     // TODO: Considering another export name type
//                     Some(ModuleExportName::Ident(Ident::new(
//                       local_binding.clone(),
//                       DUMMY_SP,
//                     )))
//                   },
//                   orig: ModuleExportName::Ident(Ident::new(import_binding.clone(), DUMMY_SP)),
//                   is_type_only: false,
//                 });
//                 let export = NamedExport {
//                   span: DUMMY_SP,
//                   specifiers: vec![specifier],
//                   src: Some(Box::new(Str {
//                     span: DUMMY_SP,
//                     value: contextify_src.into(),
//                     raw: None,
//                   })),
//                   type_only: false,
//                   asserts: None,
//                 };
//                 ModuleDecl::ExportNamed(export)
//               } else if &import_binding == "default" {
//                 let specifier = ImportSpecifier::Default(ImportDefaultSpecifier {
//                   span: DUMMY_SP,
//                   local: Ident::new(
//                     local_binding.clone(),
//                     DUMMY_SP.with_ctxt(SyntaxContext::empty().apply_mark(top_level_mark)),
//                   ),
//                 });
//                 let import = ImportDecl {
//                   span: DUMMY_SP,
//                   specifiers: vec![specifier],
//                   src: Box::new(Str {
//                     span: DUMMY_SP,
//                     value: contextify_src.into(),
//                     raw: None,
//                   }),
//                   type_only: false,
//                   asserts: None,
//                 };
//                 ModuleDecl::Import(import)
//               } else {
//                 let specifier = ImportSpecifier::Named(ImportNamedSpecifier {
//                   span: DUMMY_SP,
//                   local: Ident::new(
//                     local_binding.clone(),
//                     DUMMY_SP.with_ctxt(SyntaxContext::empty().apply_mark(top_level_mark)),
//                   ),
//                   imported: if &import_binding == local_binding {
//                     None
//                   } else {
//                     // TODO: Consider ModuleExportName is `Str`
//                     Some(ModuleExportName::Ident(Ident::new(
//                       import_binding,
//                       DUMMY_SP,
//                     )))
//                   },
//                   is_type_only: false,
//                 });
//                 let import = ImportDecl {
//                   span: DUMMY_SP,
//                   specifiers: vec![specifier],
//                   src: Box::new(Str {
//                     span: DUMMY_SP,
//                     value: contextify_src.into(),
//                     raw: None,
//                   }),
//                   type_only: false,
//                   asserts: None,
//                 };
//                 ModuleDecl::Import(import)
//               }
//             }
//             (None, _) => {
//               match &indirect.ty {
//                 IndirectType::Temp(_) => todo!(),
//                 IndirectType::ReExport(_, _) => todo!(),
//                 IndirectType::Import(local, imported) => {
//                   let specifier = ImportSpecifier::Named(ImportNamedSpecifier {
//                     span: DUMMY_SP,
//                     local: Ident::new(
//                       local.clone(),
//                       DUMMY_SP.with_ctxt(SyntaxContext::empty().apply_mark(top_level_mark)),
//                     ),
//                     imported: imported.as_ref().map(|imported| {
//                       // TODO: Consider ModuleExportName is `Str`
//                       ModuleExportName::Ident(Ident::new(imported.clone(), DUMMY_SP))
//                     }),
//                     is_type_only: false,
//                   });
//                   let import = ImportDecl {
//                     span: DUMMY_SP,
//                     specifiers: vec![specifier],
//                     src: Box::new(Str {
//                       span: DUMMY_SP,
//                       value: contextify_src.into(),
//                       raw: None,
//                     }),
//                     type_only: false,
//                     asserts: None,
//                   };
//                   ModuleDecl::Import(import)
//                 }
//                 IndirectType::ImportDefault(binding) => {
//                   let specifier = ImportSpecifier::Default(ImportDefaultSpecifier {
//                     span: DUMMY_SP,
//                     local: Ident::new(
//                       binding.clone(),
//                       DUMMY_SP.with_ctxt(SyntaxContext::empty().apply_mark(top_level_mark)),
//                     ),
//                   });
//                   let import = ImportDecl {
//                     span: DUMMY_SP,
//                     specifiers: vec![specifier],
//                     src: Box::new(Str {
//                       span: DUMMY_SP,
//                       value: contextify_src.into(),
//                       raw: None,
//                     }),
//                     type_only: false,
//                     asserts: None,
//                   };
//                   ModuleDecl::Import(import)
//                 }
//               }
//             }
//           };
//           (module_decl, importer)
//         }
//         _ => todo!(),
//       };
//       match module_item_map.entry(module_ident.into()) {
//         Entry::Occupied(mut occ) => {
//           let module_item = ModuleItem::ModuleDecl(module_decl);
//           occ.borrow_mut().get_mut().push(module_item);
//         }
//         Entry::Vacant(occ) => {
//           let module_item = ModuleItem::ModuleDecl(module_decl);
//           occ.insert(vec![module_item]);
//         }
//       };
//     }
//     module_item_map
//   })
// }

// TODO: dep replacement
// #[derive(Debug, Clone)]
// struct DependencyReplacement {
//   original: SymbolRef,
//   to: SymbolRef,
//   replacement: SymbolRef,
//   unused_export_symbol_count: usize,
// }

// impl DependencyReplacement {
//   fn new(
//     from: SymbolRef,
//     to: SymbolRef,
//     replacement: SymbolRef,
//     unused_export_symbol_count: usize,
//   ) -> Self {
//     Self {
//       original: from,
//       to,
//       unused_export_symbol_count,
//       replacement,
//     }
//   }
// }

// TODO: dep replacement
// fn recursive_visited(
//   symbol_graph: &SymbolGraph,
//   cur_path: &mut Vec<NodeIndex>,
//   paths: &mut Vec<Vec<NodeIndex>>,
//   visited_node: &mut HashSet<NodeIndex>,
//   cur: NodeIndex,
//   directed_symbol_node_index: &HashSet<NodeIndex>,
// ) {
//   if visited_node.contains(&cur) {
//     return;
//   }
//   let is_directed = directed_symbol_node_index.contains(&cur) && !cur_path.is_empty();
//   if is_directed {
//     paths.push(cur_path.clone());
//     return;
//   }
//   visited_node.insert(cur);
//   cur_path.push(cur);
//   let mut has_neighbor = false;
//   for ele in symbol_graph
//     .graph
//     .neighbors_directed(cur, petgraph::Direction::Incoming)
//   {
//     has_neighbor = true;
//     recursive_visited(
//       symbol_graph,
//       cur_path,
//       paths,
//       visited_node,
//       ele,
//       directed_symbol_node_index,
//     );
//   }
//   if !has_neighbor {
//     paths.push(cur_path.clone());
//   }
//   cur_path.pop();
//   visited_node.remove(&cur);
// }
