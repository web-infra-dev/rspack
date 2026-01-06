use std::rc::Rc;

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsyncModulesArtifact, BuildMetaExportsType, Compilation, CompilationFinishModules,
  DependenciesBlock, DependencyId, EvaluatedInlinableValue, ExportInfo, ExportInfoData,
  ExportNameOrSpec, ExportProvided, ExportSpecExports, ExportsInfo, ExportsInfoData,
  ExportsOfExportsSpec, ExportsSpec, GetTargetResult, Logger, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, ModuleIdentifier, Nullable, Plugin,
  PrefetchExportsInfoMode, get_target,
  incremental::{self, IncrementalPasses},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxIndexMap, FxIndexSet};
use rustc_hash::FxHashSet;
use swc_core::ecma::atoms::Atom;

struct FlagDependencyExportsState<'a> {
  mg: &'a mut ModuleGraph,
  mg_cache: &'a ModuleGraphCacheArtifact,
}

impl<'a> FlagDependencyExportsState<'a> {
  pub fn new(mg: &'a mut ModuleGraph, mg_cache: &'a ModuleGraphCacheArtifact) -> Self {
    Self { mg, mg_cache }
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
      // for module_id in modules {
      let exports_info = self.mg.get_exports_info_data_mut(module_id);
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

    // collect the exports specs from all modules and their dependencies
    // and then merge the exports specs to exports info data
    // and collect the dependencies which will be used to backtrack when target exports info is changed
    let mut batch = modules;
    let mut dependencies: IdentifierMap<IdentifierSet> = IdentifierMap::default();
    while !batch.is_empty() {
      let modules = std::mem::take(&mut batch);

      // collect the exports specs from modules by calling `dependency.get_exports`
      let module_exports_specs = modules
        .into_par_iter()
        .map(|module_id| {
          let exports_specs =
            collect_module_exports_specs(&module_id, self.mg, self.mg_cache).unwrap_or_default();
          (module_id, exports_specs)
        })
        .collect::<Vec<_>>();

      let mut changed_modules = FxHashSet::default();

      // partition the exports specs into two parts:
      // 1. if the exports info data do not have `redirect_to` and exports specs do not have nested `exports`,
      // then the merging only affect the exports info data itself and can be done parallelly
      // 2. if the exports info data have `redirect_to` or exports specs have nested `exports`,
      // then the merging will affect the redirected exports info data or create a new exports info data
      // and this merging can not be done parallelly
      //
      // There are two cases that the `redirect_to` or nested `exports` exist:
      // 1. exports from json dependency which has nested json object data
      // 2. exports from an esm reexport and the target is a commonjs module which should create a interop `default` export
      let (non_nested_specs, has_nested_specs): (Vec<_>, Vec<_>) = module_exports_specs
        .into_iter()
        .partition(|(_mid, (_, has_nested_exports))| {
          if *has_nested_exports {
            return false;
          }
          true
        });

      // parallelize the merging of exports specs to exports info data
      let non_nested_tasks = non_nested_specs
        .into_iter()
        .map(|(module_id, (exports_specs, _))| {
          let exports_info = self.mg.get_exports_info_data(&module_id).clone();
          (module_id, exports_info, exports_specs)
        })
        .par_bridge()
        .map(|(module_id, mut exports_info, exports_specs)| {
          let mut changed = false;
          let mut dependencies = vec![];
          for (dep_id, exports_spec) in exports_specs.into_iter() {
            let (is_changed, changed_dependencies) = process_exports_spec_without_nested(
              self.mg,
              &module_id,
              dep_id,
              &exports_spec,
              &mut exports_info,
            );
            changed |= is_changed;
            dependencies.extend(changed_dependencies);
          }
          (module_id, changed, dependencies, exports_info)
        })
        .collect::<Vec<_>>();

      // handle collected side effects and apply the merged exports info data to module graph
      for (module_id, changed, changed_dependencies, exports_info) in non_nested_tasks {
        if changed {
          changed_modules.insert(module_id);
        }
        for (module_id, dep_id) in changed_dependencies {
          dependencies.entry(module_id).or_default().insert(dep_id);
        }
        self.mg.set_exports_info(exports_info.id(), exports_info);
      }

      // serializing the merging of exports specs to nested exports info data
      for (module_id, (exports_specs, _)) in has_nested_specs {
        let mut changed = false;
        for (dep_id, exports_spec) in exports_specs.into_iter() {
          let (is_changed, changed_dependencies) =
            process_exports_spec(self.mg, &module_id, dep_id, &exports_spec);
          changed |= is_changed;
          for (module_id, dep_id) in changed_dependencies {
            dependencies.entry(module_id).or_default().insert(dep_id);
          }
        }
        if changed {
          changed_modules.insert(module_id);
        }
      }

      // collect the dependencies which will be used to backtrack when target exports info is changed
      batch.extend(changed_modules.into_iter().flat_map(|m| {
        dependencies
          .get(&m)
          .into_iter()
          .flat_map(|d| d.iter())
          .copied()
      }));
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
  compilation: &mut Compilation,
  _async_modules_artifact: &mut AsyncModulesArtifact,
) -> Result<()> {
  let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::PROVIDED_EXPORTS)
  {
    let modules = mutations.get_affected_modules_with_module_graph(compilation.get_module_graph());
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::PROVIDED_EXPORTS, %mutations, ?modules);
    let logger = compilation.get_logger("rspack.incremental.providedExports");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      compilation.get_module_graph().modules().len()
    ));
    modules
  } else {
    compilation
      .get_module_graph()
      .modules()
      .keys()
      .copied()
      .collect()
  };
  let module_graph_cache = compilation.module_graph_cache_artifact.clone();

  compilation
    .build_module_graph_artifact
    .with_mut(|artifact| {
      let module_graph = artifact.get_module_graph_mut();
      FlagDependencyExportsState::new(module_graph, &module_graph_cache).apply(modules);
    });
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

/**
 * Collect all exports specs from a module and its dependencies
 * by calling `dependency.get_exports` for each dependency.
 */
fn collect_module_exports_specs(
  module_id: &ModuleIdentifier,
  mg: &ModuleGraph,
  mg_cache: &ModuleGraphCacheArtifact,
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
      let exports_spec = dep.get_exports(mg, mg_cache)?;
      has_nested_exports |= exports_spec.has_nested_exports();
      Some((id, exports_spec))
    })
    .collect::<FxIndexMap<DependencyId, ExportsSpec>>();
  // mg_cache.unfreeze();
  Some((res, has_nested_exports))
}

/// Merge exports specs to exports info data
/// and also collect the dependencies
/// which will be used to backtrack when target exports info is changed
pub fn process_exports_spec(
  mg: &mut ModuleGraph,
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
    let exports_info = mg.get_exports_info_data_mut(module_id);
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
      changed |= mg
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
        module_id,
        mg.get_exports_info(module_id),
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
pub fn process_exports_spec_without_nested(
  mg: &ModuleGraph,
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
  exports: Option<&'a ExportSpecExports>,
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
pub fn merge_exports_without_nested(
  mg: &ModuleGraph,
  module_id: &ModuleIdentifier,
  exports_info: &mut ExportsInfoData,
  exports: &Vec<ExportNameOrSpec>,
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

    let (target_exports_info, target_dependencies) =
      find_target_exports_info(mg, export_info, module_id);
    dependencies.extend(target_dependencies);

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
  mg: &mut ModuleGraph,
  module_id: &ModuleIdentifier,
  exports_info: ExportsInfo,
  exports: &Vec<ExportNameOrSpec>,
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

    let export_info = exports_info.as_data_mut(mg).ensure_export_info(name);
    changed |= set_export_base_info(
      export_info.as_data_mut(mg),
      can_mangle,
      terminal_binding,
      inlinable,
    );

    if let Some(exports) = exports {
      let (merge_changed, merge_dependencies) = merge_nested_exports(
        mg,
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
      export_info.as_data_mut(mg),
      from,
      from_export,
      priority,
      hidden,
      dep_id,
      name,
    );

    let (target_exports_info, target_dependencies) =
      find_target_exports_info(mg, export_info.as_data(mg), module_id);
    dependencies.extend(target_dependencies);

    let export_info_data = export_info.as_data_mut(mg);
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
  mg: &mut ModuleGraph,
  module_id: &ModuleIdentifier,
  export_info: ExportInfo,
  exports: &ExportSpecExports,
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<(ModuleIdentifier, ModuleIdentifier)>) {
  let mut changed = false;
  let mut dependencies = vec![];
  let nested_exports_info = if export_info.as_data(mg).exports_info_owned() {
    export_info
      .as_data(mg)
      .exports_info()
      .expect("should have exports_info when exports_info is true")
  } else {
    let export_info = export_info.as_data_mut(mg);
    let new_exports_info = ExportsInfoData::default();
    let new_exports_info_id = new_exports_info.id();
    export_info.set_exports_info(Some(new_exports_info_id));
    export_info.set_exports_info_owned(true);
    mg.set_exports_info(new_exports_info_id, new_exports_info);

    new_exports_info_id.as_data_mut(mg).set_has_provide_info();
    new_exports_info_id
  };

  if exports.unknown_provided {
    nested_exports_info
      .as_data_mut(mg)
      .set_unknown_exports_provided(false, None, None, None, None);
  }

  let (merge_changed, merge_dependencies) = merge_exports(
    mg,
    module_id,
    nested_exports_info,
    &exports.exports,
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
  export_info: &ExportInfoData,
  module_id: &ModuleIdentifier,
) -> (
  Option<ExportsInfo>,
  Vec<(ModuleIdentifier, ModuleIdentifier)>,
) {
  let mut dependencies = vec![];
  // Recalculate target exportsInfo
  let target = get_target(export_info, mg, Rc::new(|_| true), &mut Default::default());

  let mut target_exports_info = None;
  if let Some(GetTargetResult::Target(target)) = target {
    let target_module_exports_info = mg.get_prefetched_exports_info(
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

    dependencies.push((target.module, *module_id));
  }

  (target_exports_info, dependencies)
}
