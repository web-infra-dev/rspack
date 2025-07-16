use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  get_target,
  incremental::{self, IncrementalPasses},
  ApplyContext, BuildMetaExportsType, Compilation, CompilationFinishModules, CompilerOptions,
  DependenciesBlock, DependencyId, ExportNameOrSpec, ExportProvided, ExportsInfo, ExportsInfoData,
  ExportsOfExportsSpec, ExportsSpec, Inlinable, Logger, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ModuleIdentifier, Plugin, PluginContext, PrefetchExportsInfoMode,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxIndexMap, FxIndexSet};
use rustc_hash::FxHashSet;
use swc_core::ecma::atoms::Atom;

struct FlagDependencyExportsState<'a> {
  mg: &'a mut ModuleGraph<'a>,
  mg_cache: &'a ModuleGraphCacheArtifact,
}

impl<'a> FlagDependencyExportsState<'a> {
  pub fn new(mg: &'a mut ModuleGraph<'a>, mg_cache: &'a ModuleGraphCacheArtifact) -> Self {
    Self { mg, mg_cache }
  }

  pub fn apply(&mut self, modules: IdentifierSet) {
    for module_id in &modules {
      let exports_type = self
        .mg
        .module_by_identifier(module_id)
        .expect("should have module")
        .build_meta()
        .exports_type;
      // for module_id in modules {
      let exports_info = self.mg.get_exports_info(module_id).as_data_mut(self.mg);

      // Reset exports provide info back to initial
      exports_info.reset_provide_info();

      if exports_type == BuildMetaExportsType::Unset
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

    let mut batch = modules;
    let mut dependencies: IdentifierMap<IdentifierSet> = IdentifierMap::default();
    while !batch.is_empty() {
      let modules = std::mem::take(&mut batch);
      let module_exports_specs = modules
        .into_par_iter()
        .map(|module_id| {
          let exports_specs =
            collect_module_exports_specs(&module_id, self.mg, self.mg_cache).unwrap_or_default();
          (module_id, exports_specs)
        })
        .collect::<Vec<_>>();

      let mut changed_modules = FxHashSet::default();
      for (module_id, exports_specs) in module_exports_specs {
        let exports_info = self.mg.get_exports_info(&module_id);
        let mut changed = false;
        for (dep_id, exports_spec) in exports_specs.into_iter() {
          let (is_changed, changed_dependencies) =
            self.process_exports_spec(&module_id, dep_id, &exports_spec, exports_info);
          changed |= is_changed;
          for (module_id, dep_id) in changed_dependencies {
            dependencies.entry(module_id).or_default().insert(dep_id);
          }
        }
        if changed {
          changed_modules.insert(module_id);
        }
      }
      batch.extend(changed_modules.into_iter().flat_map(|m| {
        dependencies
          .get(&m)
          .into_iter()
          .flat_map(|d| d.iter())
          .copied()
      }));
    }
  }

  pub fn process_exports_spec(
    &mut self,
    module_id: &ModuleIdentifier,
    dep_id: DependencyId,
    export_desc: &ExportsSpec,
    exports_info: ExportsInfo,
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
      let exports_info = exports_info.as_data_mut(self.mg);
      for name in hide_export.iter() {
        exports_info.get_export_info(name).unset_target(&dep_id);
      }
    }
    match exports {
      ExportsOfExportsSpec::UnknownExports => {
        changed |= exports_info
          .as_data_mut(self.mg)
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
          self.mg,
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
async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
  let modules: IdentifierSet = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::PROVIDED_EXPORTS)
  {
    let modules = mutations.get_affected_modules_with_module_graph(&compilation.get_module_graph());
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
  let mut module_graph = compilation.get_module_graph_mut();
  FlagDependencyExportsState::new(&mut module_graph, &module_graph_cache).apply(modules);
  Ok(())
}

impl Plugin for FlagDependencyExportsPlugin {
  fn name(&self) -> &'static str {
    "FlagDependencyExportsPlugin"
  }

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
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
) -> Option<FxIndexMap<DependencyId, ExportsSpec>> {
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

  let block = &**mg.module_by_identifier(module_id)?;
  let mut dep_ids = FxIndexSet::default();
  walk_block(block, &mut dep_ids, mg);

  // There is no need to use the cache here
  // because the `get_exports` of each dependency will only be called once
  // mg_cache.freeze();
  let res = dep_ids
    .into_iter()
    .filter_map(|id| {
      let dep = mg.dependency_by_id(&id)?;
      let exports_spec = dep.get_exports(mg, mg_cache)?;
      Some((id, exports_spec))
    })
    .collect::<FxIndexMap<DependencyId, ExportsSpec>>();
  // mg_cache.unfreeze();
  Some(res)
}

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
    let (
      name,
      can_mangle,
      terminal_binding,
      exports,
      from,
      from_export,
      priority,
      hidden,
      inlinable,
    ) = match export_name_or_spec {
      ExportNameOrSpec::String(name) => (
        name.clone(),
        global_export_info.can_mangle,
        global_export_info.terminal_binding,
        None::<&Vec<ExportNameOrSpec>>,
        global_export_info.from.cloned(),
        None::<&rspack_core::Nullable<Vec<Atom>>>,
        global_export_info.priority,
        false,
        None,
      ),
      ExportNameOrSpec::ExportSpec(spec) => (
        spec.name.clone(),
        match spec.can_mangle {
          Some(v) => Some(v),
          None => global_export_info.can_mangle,
        },
        spec
          .terminal_binding
          .unwrap_or(global_export_info.terminal_binding),
        spec.exports.as_ref(),
        if spec.from.is_some() {
          spec.from.clone()
        } else {
          global_export_info.from.cloned()
        },
        spec.export.as_ref(),
        match spec.priority {
          Some(v) => Some(v),
          None => global_export_info.priority,
        },
        spec.hidden.unwrap_or(false),
        spec.inlinable.as_ref(),
      ),
    };
    let export_info = exports_info.as_data_mut(mg).get_export_info(&name);
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
      && !export_info.inlinable().can_inline()
    {
      export_info.set_inlinable(Inlinable::Inlined(inlined.clone()));
      changed = true;
    }

    if terminal_binding && !export_info.terminal_binding() {
      export_info.set_terminal_binding(true);
      changed = true;
    }

    if let Some(exports) = exports {
      // create nested exports info
      let nested_exports_info = if export_info.exports_info_owned() {
        export_info
          .exports_info()
          .expect("should have exports_info when exports_info is true")
      } else {
        let mut new_exports_info = ExportsInfoData::default();
        new_exports_info.set_has_provide_info();
        let new_exports_info_id = new_exports_info.id();
        export_info.set_exports_info(Some(new_exports_info_id));
        export_info.set_exports_info_owned(true);
        mg.set_exports_info(new_exports_info_id, new_exports_info);

        new_exports_info_id
      };

      let (merge_changed, merge_dependencies) = merge_exports(
        mg,
        module_id,
        nested_exports_info,
        exports,
        global_export_info.clone(),
        dep_id,
      );
      changed |= merge_changed;
      dependencies.extend(merge_dependencies);
    }

    // shadowing the previous `export_info_mut` to reduce the mut borrow life time,
    // because `create_nested_exports_info` needs `&mut ModuleGraph`
    let export_info = exports_info.as_data_mut(mg).get_export_info(&name);
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

    // Recalculate target exportsInfo
    let export_info = exports_info
      .as_data(mg)
      .named_exports(&name)
      .expect("should have export info");

    let target = get_target(export_info, mg);

    let mut target_exports_info = None;
    if let Some(target) = target {
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

    let export_info = exports_info.as_data_mut(mg).get_export_info(&name);
    if export_info.exports_info_owned() {
      if export_info.exports_info() != target_exports_info
        && let Some(target_exports_info) = target_exports_info
      {
        export_info.set_exports_info(Some(target_exports_info));
        changed = true;
      }
    } else if export_info.exports_info() != target_exports_info {
      export_info.set_exports_info(target_exports_info);
      changed = true;
    }
  }
  (changed, dependencies)
}
