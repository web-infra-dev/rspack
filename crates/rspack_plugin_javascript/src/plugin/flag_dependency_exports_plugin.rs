mod collector;
mod local_apply;
mod propagation;
mod types;

use rspack_collections::IdentifierSet;
use rspack_core::{
  AsyncModulesArtifact, BuildMetaExportsType, Compilation, CompilationFinishModules,
  DependencyExportsAnalysisArtifact, DependencyId, EvaluatedInlinableValue, ExportInfo,
  ExportInfoData, ExportNameOrSpec, ExportProvided, ExportsInfo, ExportsInfoArtifact,
  ExportsInfoData, ExportsOfExportsSpec, ExportsSpec, GetTargetResult, Logger, ModuleGraph,
  ModuleGraphConnection, ModuleIdentifier, Nullable, Plugin, PrefetchExportsInfoMode, get_target,
  incremental::{self, IncrementalPasses, Mutation},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use swc_core::ecma::atoms::Atom;

fn collect_affected_modules(compilation: &Compilation) -> IdentifierSet {
  let module_graph = compilation.get_module_graph();
  if let Some(mutations) = compilation
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
  }
}

fn prepare_provide_info(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  affected_modules: &IdentifierSet,
) {
  for module_identifier in affected_modules {
    let exports_type_unset = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module")
      .build_meta()
      .exports_type
      == BuildMetaExportsType::Unset;
    let exports_info = exports_info_artifact.get_exports_info_data_mut(module_identifier);
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
}

fn prune_removed_modules(
  compilation: &Compilation,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
) {
  let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::FINISH_MODULES)
  else {
    return;
  };

  for mutation in mutations.iter() {
    if let Mutation::ModuleRemove { module } = mutation {
      dependency_exports_analysis_artifact.remove_module(module);
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
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
) -> Result<()> {
  let module_graph = compilation.get_module_graph();
  let affected_modules = collect_affected_modules(compilation);
  let module_graph_cache = compilation.module_graph_cache_artifact.clone();
  prune_removed_modules(compilation, dependency_exports_analysis_artifact);
  prepare_provide_info(module_graph, exports_info_artifact, &affected_modules);
  let analysis = collector::collect_module_analysis(
    module_graph,
    &module_graph_cache,
    exports_info_artifact,
    dependency_exports_analysis_artifact,
    &affected_modules,
  )?;
  local_apply::apply_local_exports(
    module_graph,
    exports_info_artifact,
    &analysis,
    &affected_modules,
  )?;
  propagation::propagate_deferred_reexports(
    module_graph,
    exports_info_artifact,
    dependency_exports_analysis_artifact,
    &analysis,
  )?;

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

/// Merge exports specs to exports info data
/// and also collect the dependencies
/// which will be used to backtrack when target exports info is changed
pub(super) fn process_exports_spec(
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
pub(super) fn process_exports_spec_without_nested(
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
