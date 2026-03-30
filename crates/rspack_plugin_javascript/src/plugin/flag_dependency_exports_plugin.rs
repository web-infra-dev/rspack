use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
};

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  AsyncModulesArtifact, BuildMetaExportsType, Compilation, CompilationFinishModules,
  DependenciesBlock, DependencyId, EvaluatedInlinableValue, ExportInfo, ExportInfoData,
  ExportInfoTargetValue, ExportNameOrSpec, ExportProvided, ExportsInfo, ExportsInfoArtifact,
  ExportsInfoData, ExportsOfExportsSpec, ExportsSpec, GetTargetResult, Logger, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, ModuleIdentifier, Nullable, Plugin,
  PrefetchExportsInfoMode, get_target,
  incremental::{self, IncrementalPasses},
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxHashMap, FxHashSet, FxIndexMap, FxIndexSet};
use rustc_hash::FxHasher;
use swc_core::ecma::atoms::Atom;

struct FlagDependencyExportsState<'a> {
  mg: &'a ModuleGraph,
  mg_cache: &'a ModuleGraphCacheArtifact,
  exports_info_artifact: &'a mut ExportsInfoArtifact,
}

type TargetExportsInfoCache = FxHashMap<ExportTargetCacheKey, CachedTargetExportsInfo>;
type ChangedDependency = (ModuleIdentifier, ModuleIdentifier);
type TargetExportsInfoResult = (Option<ExportsInfo>, Option<ModuleIdentifier>);

struct NonNestedMergeContext<'a> {
  mg: &'a ModuleGraph,
  exports_info_artifact: &'a ExportsInfoArtifact,
  module_id: &'a ModuleIdentifier,
  target_exports_info_cache: &'a mut TargetExportsInfoCache,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct NormalizedExportTargetValue {
  dependency: Option<DependencyId>,
  export: Option<Vec<Atom>>,
  priority: u8,
}

impl From<&ExportInfoTargetValue> for NormalizedExportTargetValue {
  fn from(value: &ExportInfoTargetValue) -> Self {
    Self {
      dependency: value.dependency,
      export: value.export.clone(),
      priority: value.priority,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExportTargetCacheKey {
  Single {
    target_key: Option<DependencyId>,
    target: NormalizedExportTargetValue,
  },
  Multi(FxHashMap<Option<DependencyId>, NormalizedExportTargetValue>),
}

impl Hash for ExportTargetCacheKey {
  fn hash<H: Hasher>(&self, state: &mut H) {
    std::mem::discriminant(self).hash(state);
    match self {
      Self::Single { target_key, target } => {
        target_key.hash(state);
        target.hash(state);
      }
      Self::Multi(targets) => {
        state.write_usize(targets.len());
        let mut hash_sum = 0_u64;
        let mut hash_xor = 0_u64;
        for (target_key, target) in targets {
          let mut hasher = FxHasher::default();
          target_key.hash(&mut hasher);
          target.hash(&mut hasher);
          let entry_hash = hasher.finish();
          hash_sum = hash_sum.wrapping_add(entry_hash);
          hash_xor ^= entry_hash.rotate_left(1);
        }
        state.write_u64(hash_sum);
        state.write_u64(hash_xor);
      }
    }
  }
}

impl ExportTargetCacheKey {
  fn from_export_info(export_info: &ExportInfoData) -> Option<Self> {
    if !export_info.target_is_set() || export_info.target().is_empty() {
      return None;
    }

    let max_target = export_info.get_max_target();
    match max_target {
      Cow::Borrowed(targets) if targets.len() == 1 => {
        targets
          .iter()
          .next()
          .map(|(target_key, target)| Self::Single {
            target_key: *target_key,
            target: target.into(),
          })
      }
      _ => Some(Self::Multi(
        max_target
          .iter()
          .map(|(target_key, target)| (*target_key, target.into()))
          .collect(),
      )),
    }
  }
}

#[derive(Debug, Clone)]
struct CachedTargetExportsInfo {
  exports_info: Option<ExportsInfo>,
  target_module: Option<ModuleIdentifier>,
}

/// Build a per-module merge context so repeated target resolutions can share
/// a small cache without crossing task boundaries.
fn with_non_nested_merge_context<T>(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  f: impl FnOnce(&mut NonNestedMergeContext<'_>) -> T,
) -> T {
  let mut target_exports_info_cache = TargetExportsInfoCache::default();
  let mut ctx = NonNestedMergeContext {
    mg,
    exports_info_artifact,
    module_id,
    target_exports_info_cache: &mut target_exports_info_cache,
  };
  f(&mut ctx)
}

/// Backtracking only needs the target module that should be revisited later.
fn push_target_dependency(
  dependencies: &mut Vec<ChangedDependency>,
  module_id: ModuleIdentifier,
  target_module: Option<ModuleIdentifier>,
) {
  if let Some(target_module) = target_module {
    dependencies.push((target_module, module_id));
  }
}

/// Export specs can declare extra module dependencies that also need to join
/// the backtracking queue.
fn extend_export_dependencies(
  dependencies: &mut Vec<ChangedDependency>,
  export_dependencies: Option<&[ModuleIdentifier]>,
  module_id: ModuleIdentifier,
) {
  if let Some(export_dependencies) = export_dependencies {
    dependencies.extend(
      export_dependencies
        .iter()
        .map(|export_dep| (*export_dep, module_id)),
    );
  }
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

    // collect the exports specs from all modules and their dependencies
    // and then merge the exports specs to exports info data
    // and collect the dependencies which will be used to backtrack when target exports info is changed
    let mut batch = modules;
    let mut dependencies: IdentifierMap<IdentifierSet> =
      IdentifierMap::with_capacity_and_hasher(batch.len(), Default::default());
    while !batch.is_empty() {
      let modules = std::mem::take(&mut batch);

      // collect the exports specs from modules by calling `dependency.get_exports`
      let module_exports_specs = modules
        .into_par_iter()
        .map(|module_id| {
          let exports_specs = collect_module_exports_specs(
            &module_id,
            self.mg,
            self.mg_cache,
            self.exports_info_artifact,
          )
          .unwrap_or_default();
          (module_id, exports_specs)
        })
        .collect::<Vec<_>>();

      let mut changed_modules =
        IdentifierSet::with_capacity_and_hasher(module_exports_specs.len(), Default::default());

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
        .into_par_iter()
        .map(|(module_id, (exports_specs, _))| {
          let mut changed = false;
          let mut exports_info = self
            .exports_info_artifact
            .get_exports_info_data(&module_id)
            .clone();
          let mut dependencies = Vec::with_capacity(exports_specs.len());
          with_non_nested_merge_context(
            self.mg,
            self.exports_info_artifact,
            &module_id,
            |merge_ctx| {
              for (dep_id, exports_spec) in exports_specs {
                let (is_changed, changed_dependencies) = process_exports_spec_without_nested_inner(
                  merge_ctx,
                  dep_id,
                  &exports_spec,
                  &mut exports_info,
                );
                changed |= is_changed;
                dependencies.extend(changed_dependencies);
              }
            },
          );
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
        self
          .exports_info_artifact
          .set_exports_info_by_id(exports_info.id(), exports_info);
      }

      // serializing the merging of exports specs to nested exports info data
      for (module_id, (exports_specs, _)) in has_nested_specs {
        let mut changed = false;
        for (dep_id, exports_spec) in exports_specs {
          let (is_changed, changed_dependencies) = process_exports_spec(
            self.mg,
            self.exports_info_artifact,
            &module_id,
            dep_id,
            &exports_spec,
          );
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

impl<'a> DefaultExportInfo<'a> {
  /// Capture the shared defaults once so nested and non-nested merge paths can
  /// reuse the same parameters.
  fn from_exports_spec(export_desc: &'a ExportsSpec) -> Self {
    Self {
      can_mangle: export_desc.can_mangle,
      terminal_binding: export_desc.terminal_binding.unwrap_or(false),
      from: export_desc.from.as_ref(),
      priority: export_desc.priority,
    }
  }
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

/**
 * Collect all exports specs from a module and its dependencies
 * by calling `dependency.get_exports` for each dependency.
 */
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

/// Merge exports specs to exports info data
/// and also collect the dependencies
/// which will be used to backtrack when target exports info is changed
pub fn process_exports_spec(
  mg: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  dep_id: DependencyId,
  export_desc: &ExportsSpec,
) -> (bool, Vec<ChangedDependency>) {
  let mut changed = false;
  let mut dependencies = vec![];
  let default_export_info = DefaultExportInfo::from_exports_spec(export_desc);
  let from_dependency = default_export_info.from.map(|_| dep_id);
  let exports = &export_desc.exports;
  let export_dependencies = &export_desc.dependencies;
  unset_hidden_exports(
    exports_info_artifact,
    module_id,
    export_desc.hide_export.as_ref(),
    dep_id,
  );
  match exports {
    ExportsOfExportsSpec::UnknownExports => {
      changed |= exports_info_artifact
        .get_exports_info_data_mut(module_id)
        .set_unknown_exports_provided(
          default_export_info.can_mangle.unwrap_or_default(),
          export_desc.exclude_exports.as_ref(),
          from_dependency,
          from_dependency,
          default_export_info.priority,
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
        default_export_info,
        dep_id,
      );
      changed |= merge_changed;
      dependencies.extend(merge_dependencies);
    }
  }

  extend_export_dependencies(
    &mut dependencies,
    export_dependencies.as_deref(),
    *module_id,
  );

  (changed, dependencies)
}

/// Merge exports specs to exports info data
/// and also collect the dependencies
/// which will be used to backtrack when target exports info is changed
/// This method is used for the case that the exports info data will not be nested modified
/// that means this exports info can be modified parallelly
pub fn process_exports_spec_without_nested(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  dep_id: DependencyId,
  export_desc: &ExportsSpec,
  exports_info: &mut ExportsInfoData,
) -> (bool, Vec<ChangedDependency>) {
  with_non_nested_merge_context(mg, exports_info_artifact, module_id, |ctx| {
    process_exports_spec_without_nested_inner(ctx, dep_id, export_desc, exports_info)
  })
}

fn process_exports_spec_without_nested_inner(
  ctx: &mut NonNestedMergeContext<'_>,
  dep_id: DependencyId,
  export_desc: &ExportsSpec,
  exports_info: &mut ExportsInfoData,
) -> (bool, Vec<ChangedDependency>) {
  let mut changed = false;
  let mut dependencies = vec![];
  let default_export_info = DefaultExportInfo::from_exports_spec(export_desc);
  let from_dependency = default_export_info.from.map(|_| dep_id);

  let exports = &export_desc.exports;
  let export_dependencies = &export_desc.dependencies;
  unset_owned_hidden_exports(exports_info, export_desc.hide_export.as_ref(), dep_id);
  match exports {
    ExportsOfExportsSpec::UnknownExports => {
      changed |= exports_info.set_unknown_exports_provided(
        default_export_info.can_mangle.unwrap_or_default(),
        export_desc.exclude_exports.as_ref(),
        from_dependency,
        from_dependency,
        default_export_info.priority,
      );
    }
    ExportsOfExportsSpec::NoExports => {}
    ExportsOfExportsSpec::Names(ele) => {
      let (merge_changed, merge_dependencies) =
        merge_exports_without_nested_inner(ctx, exports_info, ele, default_export_info, dep_id);
      changed |= merge_changed;
      dependencies.extend(merge_dependencies);
    }
  }

  extend_export_dependencies(
    &mut dependencies,
    export_dependencies.as_deref(),
    *ctx.module_id,
  );

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
pub fn merge_exports_without_nested(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  exports_info: &mut ExportsInfoData,
  exports: &[ExportNameOrSpec],
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<ChangedDependency>) {
  with_non_nested_merge_context(mg, exports_info_artifact, module_id, |ctx| {
    merge_exports_without_nested_inner(ctx, exports_info, exports, global_export_info, dep_id)
  })
}

fn merge_exports_without_nested_inner(
  ctx: &mut NonNestedMergeContext<'_>,
  exports_info: &mut ExportsInfoData,
  exports: &[ExportNameOrSpec],
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<ChangedDependency>) {
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

    let (target_exports_info, target_module) = find_target_exports_info_cached(
      ctx.mg,
      ctx.exports_info_artifact,
      export_info,
      ctx.target_exports_info_cache,
    );
    push_target_dependency(&mut dependencies, *ctx.module_id, target_module);

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
) -> (bool, Vec<ChangedDependency>) {
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
    push_target_dependency(&mut dependencies, *module_id, target_module);

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

fn unset_hidden_exports(
  exports_info_artifact: &mut ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  hide_export: Option<&FxHashSet<Atom>>,
  dep_id: DependencyId,
) {
  // The artifact-backed path has to materialize hidden exports before clearing
  // their targets.
  let Some(hide_export) = hide_export else {
    return;
  };
  let exports_info = exports_info_artifact.get_exports_info_data_mut(module_id);
  for name in hide_export {
    exports_info.ensure_export_info(name);
  }
  for name in hide_export {
    exports_info
      .named_exports_mut(name)
      .expect("should have named export")
      .unset_target(&dep_id);
  }
}

fn unset_owned_hidden_exports(
  exports_info: &mut ExportsInfoData,
  hide_export: Option<&FxHashSet<Atom>>,
  dep_id: DependencyId,
) {
  // The cloned `ExportsInfoData` path can clear targets directly in place.
  if let Some(hide_export) = hide_export {
    for name in hide_export {
      exports_info
        .ensure_owned_export_info(name)
        .unset_target(&dep_id);
    }
  }
}

fn merge_nested_exports(
  mg: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  module_id: &ModuleIdentifier,
  export_info: ExportInfo,
  exports: &[ExportNameOrSpec],
  global_export_info: DefaultExportInfo,
  dep_id: DependencyId,
) -> (bool, Vec<ChangedDependency>) {
  let mut changed = false;
  let mut dependencies = vec![];
  // Reuse an existing nested exports info when possible; otherwise create it
  // lazily so flat exports do not pay the allocation cost.
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
) -> TargetExportsInfoResult {
  // Resolve the effective target module/export pair, then translate it into
  // the nested exports info id this export should point at.
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

fn find_target_exports_info_cached(
  mg: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  export_info: &ExportInfoData,
  target_exports_info_cache: &mut TargetExportsInfoCache,
) -> TargetExportsInfoResult {
  let Some(cache_key) = ExportTargetCacheKey::from_export_info(export_info) else {
    return (None, None);
  };

  if let Some(cached) = target_exports_info_cache.get(&cache_key) {
    return (cached.exports_info, cached.target_module);
  }

  // Cache entries are scoped to one module merge task, so there is no cross-task
  // invalidation or synchronization to worry about.
  let (target_exports_info, target_module) =
    find_target_exports_info(mg, exports_info_artifact, export_info);
  target_exports_info_cache.insert(
    cache_key,
    CachedTargetExportsInfo {
      exports_info: target_exports_info,
      target_module,
    },
  );
  (target_exports_info, target_module)
}
