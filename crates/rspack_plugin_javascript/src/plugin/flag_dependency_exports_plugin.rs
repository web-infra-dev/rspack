use std::collections::VecDeque;

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsyncModulesArtifact, BuildMetaExportsType, Compilation, CompilationFinishModules,
  DependenciesBlock, DependencyId, EvaluatedInlinableValue, ExportInfo, ExportInfoData,
  ExportNameOrSpec, ExportProvided, ExportsInfo, ExportsInfoArtifact, ExportsInfoData,
  ExportsOfExportsSpec, ExportsSpec, GetTargetResult, Logger, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, ModuleIdentifier, Nullable, Plugin,
  PrefetchExportsInfoMode, get_target,
  incremental::{self, IncrementalPasses},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxIndexMap, FxIndexSet};
use swc_core::ecma::atoms::Atom;

use super::flag_dependency_exports_solver::{
  ExportsSolveStats, PlannedModule, build_solve_graph, execute_component_worklist,
};

#[derive(Debug, Default)]
struct SolveModuleOutcome {
  changed: bool,
  changed_dependencies: Vec<(ModuleIdentifier, ModuleIdentifier)>,
}

struct FlagDependencyExportsState<'a> {
  mg: &'a ModuleGraph,
  mg_cache: &'a ModuleGraphCacheArtifact,
  exports_info_artifact: &'a mut ExportsInfoArtifact,
}

impl<'a> FlagDependencyExportsState<'a> {
  pub fn new(
    mg: &'a ModuleGraph,
    mg_cache: &'a ModuleGraphCacheArtifact,
    exports_info_artifact: &'a mut ExportsInfoArtifact,
  ) -> Self {
    Self {
      mg,
      mg_cache,
      exports_info_artifact,
    }
  }

  pub fn apply(&mut self, modules: IdentifierSet) {
    // initialize the exports info data and their provided info for all modules
    for module_id in &modules {
      let exports_type_unset = self
        .mg
        .module_by_identifier(module_id)
        .expect("should have module")
        .build_meta()
        .exports_type
        == BuildMetaExportsType::Unset;
      let exports_info = self
        .exports_info_artifact
        .get_exports_info_data_mut(module_id);
      // Reset exports provide info back to initial
      exports_info.reset_provide_info();
      if exports_type_unset
        && !matches!(
          exports_info.other_exports_info().provided(),
          Some(ExportProvided::Unknown)
        )
      {
        exports_info.set_has_provide_info();
        exports_info.set_unknown_exports_provided(false, None, None, None, None);
        continue;
      }

      exports_info.set_has_provide_info();
    }

    let planned_modules = modules
      .par_iter()
      .map(|module_id| {
        build_planned_module(
          module_id,
          self.mg,
          self.mg_cache,
          self.exports_info_artifact,
        )
      })
      .collect::<Vec<_>>();

    let mut solve_stats = ExportsSolveStats {
      planned_modules: planned_modules.len(),
      provider_edges: planned_modules
        .iter()
        .map(|planned| planned.provider_modules.len())
        .sum(),
      ..Default::default()
    };

    let solve_graph = build_solve_graph(planned_modules);
    solve_stats.scc_count = solve_graph.components.len();
    solve_stats.fallback_components = solve_graph
      .components
      .iter()
      .filter(|component| component.fallback)
      .count();

    let mut batch = modules;
    let mut dynamic_dependencies: IdentifierMap<IdentifierSet> =
      IdentifierMap::with_capacity_and_hasher(solve_stats.planned_modules, Default::default());

    while !batch.is_empty() {
      let scheduled_modules = std::mem::take(&mut batch);
      let mut changed_modules =
        IdentifierSet::with_capacity_and_hasher(scheduled_modules.len(), Default::default());

      for component_id in solve_graph.reverse_topo_order.iter().copied() {
        let component = &solve_graph.components[component_id];
        if component.fallback || component.is_cyclic {
          let mut queue = component
            .modules
            .iter()
            .filter(|module_id| scheduled_modules.contains(module_id))
            .copied()
            .collect::<VecDeque<_>>();
          if queue.is_empty() {
            continue;
          }
          execute_component_worklist(component, &mut queue, |module_id| {
            solve_stats.solve_module_once_calls += 1;
            let outcome = self.solve_module_once(module_id);
            for (provider_module, dependent_module) in outcome.changed_dependencies {
              dynamic_dependencies
                .entry(provider_module)
                .or_default()
                .insert(dependent_module);
            }
            if outcome.changed {
              changed_modules.insert(module_id);
              if let Some(dependents) = component.dependents_within_component.get(&module_id) {
                solve_stats.local_requeues += dependents.len();
              }
            }
            outcome.changed
          });
        } else if let Some(module_id) = component.modules.first().copied()
          && scheduled_modules.contains(&module_id)
        {
          solve_stats.solve_module_once_calls += 1;
          let outcome = self.solve_module_once(module_id);
          for (provider_module, dependent_module) in outcome.changed_dependencies {
            dynamic_dependencies
              .entry(provider_module)
              .or_default()
              .insert(dependent_module);
          }
          if outcome.changed {
            changed_modules.insert(module_id);
          }
        }
      }

      let mut next_batch = IdentifierSet::default();
      for module_id in changed_modules {
        next_batch.extend(
          dynamic_dependencies
            .get(&module_id)
            .into_iter()
            .flat_map(|deps| deps.iter())
            .copied(),
        );
      }
      batch.extend(next_batch);
    }

    tracing::debug!(
      planned_modules = solve_stats.planned_modules,
      provider_edges = solve_stats.provider_edges,
      scc_count = solve_stats.scc_count,
      fallback_components = solve_stats.fallback_components,
      solve_module_once_calls = solve_stats.solve_module_once_calls,
      local_requeues = solve_stats.local_requeues
    );
  }

  fn solve_module_once(&mut self, module_id: ModuleIdentifier) -> SolveModuleOutcome {
    let (exports_specs, has_nested_exports) = collect_module_exports_specs(
      &module_id,
      self.mg,
      self.mg_cache,
      self.exports_info_artifact,
    )
    .unwrap_or_default();

    if !has_nested_exports {
      let mut changed = false;
      let mut changed_dependencies = Vec::with_capacity(exports_specs.len());
      let mut exports_info = self
        .exports_info_artifact
        .get_exports_info_data(&module_id)
        .clone();
      for (dep_id, exports_spec) in exports_specs {
        let (is_changed, resolved_dependencies) = process_exports_spec_without_nested(
          self.mg,
          self.exports_info_artifact,
          &module_id,
          dep_id,
          &exports_spec,
          &mut exports_info,
        );
        changed |= is_changed;
        changed_dependencies.extend(resolved_dependencies);
      }
      self
        .exports_info_artifact
        .set_exports_info_by_id(exports_info.id(), exports_info);

      SolveModuleOutcome {
        changed,
        changed_dependencies,
      }
    } else {
      let mut changed = false;
      let mut changed_dependencies = Vec::with_capacity(exports_specs.len());
      for (dep_id, exports_spec) in exports_specs {
        let (is_changed, resolved_dependencies) = process_exports_spec(
          self.mg,
          self.exports_info_artifact,
          &module_id,
          dep_id,
          &exports_spec,
        );
        changed |= is_changed;
        changed_dependencies.extend(resolved_dependencies);
      }
      SolveModuleOutcome {
        changed,
        changed_dependencies,
      }
    }
  }
}

/// Used for reducing nums of params
#[derive(Debug, Clone)]
pub struct DefaultExportInfo<'a> {
  can_mangle: Option<bool>,
  terminal_binding: bool,
  from: Option<&'a ModuleGraphConnection>,
  priority: Option<u8>,
}

#[plugin]
#[derive(Debug, Default)]
pub struct FlagDependencyExportsPlugin;

#[plugin_hook(CompilationFinishModules for FlagDependencyExportsPlugin)]
async fn finish_modules(
  &self,
  compilation: &Compilation,
  _async_modules_artifact: &mut AsyncModulesArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::FINISH_MODULES)
  {
    let modules = mutations.get_affected_modules_with_module_graph(module_graph);
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::FINISH_MODULES, %mutations, ?modules);
    let logger = compilation.get_logger("rspack.incremental.finishModules");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      module_graph.modules_len()
    ));
    modules
  } else {
    module_graph.modules_keys().copied().collect()
  };
  let module_graph_cache = compilation.module_graph_cache_artifact.clone();

  FlagDependencyExportsState::new(module_graph, &module_graph_cache, exports_info_artifact)
    .apply(modules);

  Ok(())
}

impl Plugin for FlagDependencyExportsPlugin {
  fn name(&self) -> &'static str {
    "FlagDependencyExportsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .finish_modules
      .tap(finish_modules::new(self));
    Ok(())
  }
}

fn collect_module_exports_specs(
  module_id: &ModuleIdentifier,
  mg: &ModuleGraph,
  mg_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Option<(FxIndexMap<DependencyId, ExportsSpec>, bool)> {
  let mut has_nested_exports = false;
  fn walk_block<B: DependenciesBlock + ?Sized>(
    block: &B,
    dep_ids: &mut FxIndexSet<DependencyId>,
    mg: &ModuleGraph,
  ) {
    dep_ids.extend(block.get_dependencies().iter().copied());
    for block_id in block.get_blocks() {
      if let Some(block) = mg.block_by_id(block_id) {
        walk_block(block, dep_ids, mg);
      }
    }
  }

  let block = mg.module_by_identifier(module_id)?.as_ref();
  let mut dep_ids = FxIndexSet::default();
  walk_block(block, &mut dep_ids, mg);

  // There is no need to use the cache here
  // because the `get_exports` of each dependency will only be called once
  // mg_cache.freeze();
  let res = dep_ids
    .into_iter()
    .filter_map(|id| {
      let dep = mg.dependency_by_id(&id);
      let exports_spec = dep.get_exports(mg, mg_cache, exports_info_artifact)?;
      has_nested_exports |= exports_spec.has_nested_exports();
      Some((id, exports_spec))
    })
    .collect::<FxIndexMap<DependencyId, ExportsSpec>>();
  // mg_cache.unfreeze();
  Some((res, has_nested_exports))
}

fn build_planned_module(
  module_id: &ModuleIdentifier,
  mg: &ModuleGraph,
  mg_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> PlannedModule {
  let mut provider_modules = IdentifierSet::default();
  let mut has_unknown_provider = false;
  if let Some((exports_specs, _)) =
    collect_module_exports_specs(module_id, mg, mg_cache, exports_info_artifact)
  {
    for (_, exports_spec) in exports_specs {
      if let Some(dependencies) = exports_spec.dependencies.as_ref() {
        provider_modules.extend(dependencies.iter().copied());
      }
      if let Some(from) = exports_spec.from.as_ref() {
        if !from
          .active_state(mg, None, mg_cache, exports_info_artifact)
          .is_true()
        {
          has_unknown_provider = true;
        }
        provider_modules.insert(*from.module_identifier());
        provider_modules.insert(from.resolved_module);
      }
    }
  } else {
    has_unknown_provider = true;
  }

  PlannedModule {
    module_id: *module_id,
    provider_modules,
    has_unknown_provider,
  }
}

/// Merge exports specs to exports info data
/// and also collect the dependencies
/// which will be used to backtrack when target exports info is changed
pub fn process_exports_spec(
  mg: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  dep_id: DependencyId,
  export_desc: &ExportsSpec,
) -> (bool, Vec<(ModuleIdentifier, ModuleIdentifier)>) {
  let mut changed = false;
  let mut dependencies = vec![];
  let exports = &export_desc.exports;
  let global_can_mangle = &export_desc.can_mangle;
  let global_from = export_desc.from.as_ref();
  let global_priority = &export_desc.priority;
  let global_terminal_binding = export_desc.terminal_binding.unwrap_or(false);
  let export_dependencies = &export_desc.dependencies;
  if let Some(hide_export) = &export_desc.hide_export {
    let exports_info = exports_info_artifact.get_exports_info_data_mut(module_id);
    for name in hide_export.iter() {
      exports_info.ensure_export_info(name);
    }
    for name in hide_export.iter() {
      exports_info
        .named_exports_mut(name)
        .expect("should have named export")
        .unset_target(&dep_id);
    }
  }
  match exports {
    ExportsOfExportsSpec::UnknownExports => {
      changed |= exports_info_artifact
        .get_exports_info_data_mut(module_id)
        .set_unknown_exports_provided(
          global_can_mangle.unwrap_or_default(),
          export_desc.exclude_exports.as_ref(),
          global_from.map(|_| dep_id),
          global_from.map(|_| dep_id),
          *global_priority,
        );
    }
    ExportsOfExportsSpec::NoExports => {}
    ExportsOfExportsSpec::Names(ele) => {
      let (merge_changed, merge_dependencies) = merge_exports(
        mg,
        exports_info_artifact,
        module_id,
        exports_info_artifact.get_exports_info(module_id),
        ele,
        DefaultExportInfo {
          can_mangle: *global_can_mangle,
          terminal_binding: global_terminal_binding,
          from: global_from,
          priority: *global_priority,
        },
        dep_id,
      );
      changed |= merge_changed;
      dependencies.extend(merge_dependencies);
    }
  }

  if let Some(export_dependencies) = export_dependencies {
    for export_dep in export_dependencies {
      dependencies.push((*export_dep, *module_id));
    }
  }

  (changed, dependencies)
}

/// Merge exports specs to exports info data
/// and also collect the dependencies
/// which will be used to backtrack when target exports info is changed
/// This method is used for the case that the exports info data will not be nested modified
/// that means this exports info can be modified parallelly
fn process_exports_spec_without_nested(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  dep_id: DependencyId,
  export_desc: &ExportsSpec,
  exports_info: &mut ExportsInfoData,
) -> (bool, Vec<(ModuleIdentifier, ModuleIdentifier)>) {
  let mut changed = false;
  let mut dependencies = vec![];

  let exports = &export_desc.exports;
  let global_can_mangle = &export_desc.can_mangle;
  let global_from = export_desc.from.as_ref();
  let global_priority = &export_desc.priority;
  let global_terminal_binding = export_desc.terminal_binding.unwrap_or(false);
  let export_dependencies = &export_desc.dependencies;
  if let Some(hide_export) = &export_desc.hide_export {
    for name in hide_export.iter() {
      exports_info
        .ensure_owned_export_info(name)
        .unset_target(&dep_id);
    }
  }
  match exports {
    ExportsOfExportsSpec::UnknownExports => {
      changed |= exports_info.set_unknown_exports_provided(
        global_can_mangle.unwrap_or_default(),
        export_desc.exclude_exports.as_ref(),
        global_from.map(|_| dep_id),
        global_from.map(|_| dep_id),
        *global_priority,
      );
    }
    ExportsOfExportsSpec::NoExports => {}
    ExportsOfExportsSpec::Names(ele) => {
      let (merge_changed, merge_dependencies) = merge_exports_without_nested(
        mg,
        exports_info_artifact,
        module_id,
        exports_info,
        ele,
        DefaultExportInfo {
          can_mangle: *global_can_mangle,
          terminal_binding: global_terminal_binding,
          from: global_from,
          priority: *global_priority,
        },
        dep_id,
      );
      changed |= merge_changed;
      dependencies.extend(merge_dependencies);
    }
  }

  if let Some(export_dependencies) = export_dependencies {
    for export_dep in export_dependencies {
      dependencies.push((*export_dep, *module_id));
    }
  }

  (changed, dependencies)
}

struct ParsedExportSpec<'a> {
  name: &'a Atom,
  can_mangle: Option<bool>,
  terminal_binding: bool,
  exports: Option<&'a Vec<ExportNameOrSpec>>,
  from: Option<&'a ModuleGraphConnection>,
  from_export: Option<&'a Nullable<Vec<Atom>>>,
  priority: Option<u8>,
  hidden: bool,
  inlinable: Option<&'a EvaluatedInlinableValue>,
}

impl<'a> ParsedExportSpec<'a> {
  pub fn new(
    export_name_or_spec: &'a ExportNameOrSpec,
    global_export_info: &'a DefaultExportInfo,
  ) -> Self {
    match export_name_or_spec {
      ExportNameOrSpec::String(name) => Self {
        name,
        can_mangle: global_export_info.can_mangle,
        terminal_binding: global_export_info.terminal_binding,
        exports: None,
        from: global_export_info.from,
        from_export: None,
        priority: global_export_info.priority,
        hidden: false,
        inlinable: None,
      },
      ExportNameOrSpec::ExportSpec(spec) => Self {
        name: &spec.name,
        can_mangle: spec.can_mangle.or(global_export_info.can_mangle),
        terminal_binding: spec
          .terminal_binding
          .unwrap_or(global_export_info.terminal_binding),
        exports: spec.exports.as_ref(),
        from: spec.from.as_ref().or(global_export_info.from),
        from_export: spec.export.as_ref(),
        priority: spec.priority.or(global_export_info.priority),
        hidden: spec.hidden.unwrap_or(false),
        inlinable: spec.inlinable.as_ref(),
      },
    }
  }
}

/// Do merging of exports info and create export infos from export specs
///
/// This method is used for the case that the exports info data will not be nested modified
/// that means this exports info can be modified parallelly
fn merge_exports_without_nested(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  exports_info: &mut ExportsInfoData,
  exports: &[ExportNameOrSpec],
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<(ModuleIdentifier, ModuleIdentifier)>) {
  let mut changed = false;
  let mut dependencies = vec![];
  for export_name_or_spec in exports {
    let ParsedExportSpec {
      name,
      can_mangle,
      terminal_binding,
      from,
      from_export,
      priority,
      hidden,
      inlinable,
      ..
    } = ParsedExportSpec::new(export_name_or_spec, &global_export_info);

    let export_info = exports_info.ensure_owned_export_info(name);
    changed |= set_export_base_info(export_info, can_mangle, terminal_binding, inlinable);

    changed |= set_export_target(
      export_info,
      from,
      from_export,
      priority,
      hidden,
      dep_id,
      name,
    );

    let (target_exports_info, target_module) =
      find_target_exports_info(mg, exports_info_artifact, export_info);
    if let Some(target_module) = target_module {
      dependencies.push((target_module, *module_id));
    }

    if export_info.exports_info() != target_exports_info {
      export_info.set_exports_info(target_exports_info);
      changed = true;
    }
  }
  (changed, dependencies)
}

/// Do merging of exports info and create export infos from export specs
/// This method is used for the case that the exports info data will be nested modified
/// that means this exports info can not be modified parallelly
pub fn merge_exports(
  mg: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  exports_info: ExportsInfo,
  exports: &[ExportNameOrSpec],
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<(ModuleIdentifier, ModuleIdentifier)>) {
  let mut changed = false;
  let mut dependencies = vec![];
  for export_name_or_spec in exports {
    let ParsedExportSpec {
      name,
      can_mangle,
      terminal_binding,
      exports,
      from,
      from_export,
      priority,
      hidden,
      inlinable,
    } = ParsedExportSpec::new(export_name_or_spec, &global_export_info);

    let export_info = exports_info
      .as_data_mut(exports_info_artifact)
      .ensure_export_info(name);
    changed |= set_export_base_info(
      export_info.as_data_mut(exports_info_artifact),
      can_mangle,
      terminal_binding,
      inlinable,
    );

    if let Some(exports) = exports {
      let (merge_changed, merge_dependencies) = merge_nested_exports(
        mg,
        exports_info_artifact,
        module_id,
        export_info.clone(),
        exports,
        global_export_info.clone(),
        dep_id,
      );
      changed |= merge_changed;
      dependencies.extend(merge_dependencies);
    }

    changed |= set_export_target(
      export_info.as_data_mut(exports_info_artifact),
      from,
      from_export,
      priority,
      hidden,
      dep_id,
      name,
    );

    let (target_exports_info, target_module) = find_target_exports_info(
      mg,
      exports_info_artifact,
      export_info.as_data(exports_info_artifact),
    );
    if let Some(target_module) = target_module {
      dependencies.push((target_module, *module_id));
    }

    let export_info_data = export_info.as_data_mut(exports_info_artifact);
    if export_info_data.exports_info_owned()
      && export_info_data.exports_info() != target_exports_info
      && let Some(target_exports_info) = target_exports_info
    {
      export_info_data.set_exports_info(Some(target_exports_info));
      changed = true;
    }
  }
  (changed, dependencies)
}

fn set_export_base_info(
  export_info: &mut ExportInfoData,
  can_mangle: Option<bool>,
  terminal_binding: bool,
  inlinable: Option<&EvaluatedInlinableValue>,
) -> bool {
  let mut changed = false;
  if let Some(provided) = export_info.provided()
    && matches!(
      provided,
      ExportProvided::NotProvided | ExportProvided::Unknown
    )
  {
    export_info.set_provided(Some(ExportProvided::Provided));
    changed = true;
  }

  if Some(false) != export_info.can_mangle_provide() && can_mangle == Some(false) {
    export_info.set_can_mangle_provide(Some(false));
    changed = true;
  }

  if let Some(inlined) = inlinable
    && export_info.can_inline_provide().is_none()
  {
    export_info.set_can_inline_provide(Some(inlined.clone()));
    changed = true;
  }

  if terminal_binding && !export_info.terminal_binding() {
    export_info.set_terminal_binding(true);
    changed = true;
  }
  changed
}

fn merge_nested_exports(
  mg: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  export_info: ExportInfo,
  exports: &[ExportNameOrSpec],
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<(ModuleIdentifier, ModuleIdentifier)>) {
  let mut changed = false;
  let mut dependencies = vec![];
  let nested_exports_info = if export_info
    .as_data(exports_info_artifact)
    .exports_info_owned()
  {
    export_info
      .as_data(exports_info_artifact)
      .exports_info()
      .expect("should have exports_info when exports_info is true")
  } else {
    let export_info = export_info.as_data_mut(exports_info_artifact);
    let new_exports_info = ExportsInfoData::default();
    let new_exports_info_id = new_exports_info.id();
    export_info.set_exports_info(Some(new_exports_info_id));
    export_info.set_exports_info_owned(true);
    exports_info_artifact.set_exports_info_by_id(new_exports_info_id, new_exports_info);

    new_exports_info_id
      .as_data_mut(exports_info_artifact)
      .set_has_provide_info();
    new_exports_info_id
  };

  let (merge_changed, merge_dependencies) = merge_exports(
    mg,
    exports_info_artifact,
    module_id,
    nested_exports_info,
    exports,
    global_export_info.clone(),
    dep_id,
  );
  changed |= merge_changed;
  dependencies.extend(merge_dependencies);

  (changed, dependencies)
}

fn set_export_target(
  export_info: &mut ExportInfoData,
  from: Option<&ModuleGraphConnection>,
  from_export: Option<&Nullable<Vec<Atom>>>,
  priority: Option<u8>,
  hidden: bool,
  dep_id: DependencyId,
  name: &Atom,
) -> bool {
  let mut changed = false;
  // shadowing the previous `export_info_mut` to reduce the mut borrow life time,
  // because `create_nested_exports_info` needs `&mut ModuleGraph`
  if let Some(from) = from {
    changed |= if hidden {
      export_info.unset_target(&dep_id)
    } else {
      let fallback = rspack_core::Nullable::Value(vec![name.clone()]);
      let export_name = if let Some(from) = from_export {
        Some(from)
      } else {
        Some(&fallback)
      };
      export_info.set_target(
        Some(dep_id),
        Some(from.dependency_id),
        export_name,
        priority,
      )
    }
  }
  changed
}

fn find_target_exports_info(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  export_info: &ExportInfoData,
) -> (Option<ExportsInfo>, Option<ModuleIdentifier>) {
  // Recalculate target exportsInfo
  let target = get_target(
    export_info,
    mg,
    exports_info_artifact,
    &|_| true,
    &mut Default::default(),
  );

  let mut target_exports_info = None;
  let mut target_module = None;
  if let Some(GetTargetResult::Target(target)) = target {
    let target_module_exports_info = exports_info_artifact.get_prefetched_exports_info(
      &target.module,
      if let Some(names) = &target.export {
        PrefetchExportsInfoMode::Nested(names)
      } else {
        PrefetchExportsInfoMode::Default
      },
    );
    target_exports_info = target_module_exports_info
      .get_nested_exports_info(target.export.as_deref())
      .map(|data| data.id());
    target_module = Some(target.module);
  }

  (target_exports_info, target_module)
}
