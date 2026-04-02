use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{DependencyExportsAnalysisArtifact, ExportsInfoArtifact, ModuleGraph};
use rspack_error::Result;

use super::{process_exports_spec, process_exports_spec_without_nested};

pub(super) fn apply_local_exports(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  initial_modules: &IdentifierSet,
) -> Result<()> {
  let mut batch = initial_modules.clone();
  let mut dependencies: IdentifierMap<IdentifierSet> =
    IdentifierMap::with_capacity_and_hasher(batch.len(), Default::default());

  while !batch.is_empty() {
    let modules = std::mem::take(&mut batch);
    let mut changed_modules =
      IdentifierSet::with_capacity_and_hasher(modules.len(), Default::default());

    apply_flat_local_exports_in_parallel(
      module_graph,
      exports_info_artifact,
      dependency_exports_analysis_artifact,
      &modules,
      &mut changed_modules,
      &mut dependencies,
    )?;
    apply_structured_local_exports_sequentially(
      module_graph,
      exports_info_artifact,
      dependency_exports_analysis_artifact,
      &modules,
      &mut changed_modules,
      &mut dependencies,
    )?;

    batch.extend(changed_modules.into_iter().flat_map(|module_identifier| {
      dependencies
        .get(&module_identifier)
        .into_iter()
        .flat_map(|dependents| dependents.iter())
        .copied()
    }));
  }

  Ok(())
}

fn apply_flat_local_exports_in_parallel(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  modules: &IdentifierSet,
  changed_modules: &mut IdentifierSet,
  dependencies: &mut IdentifierMap<IdentifierSet>,
) -> Result<()> {
  let flat_tasks = modules
    .par_iter()
    .filter_map(|module_identifier| {
      let module_analysis = dependency_exports_analysis_artifact.module(module_identifier)?;
      if module_analysis.flat_local_apply().is_empty() {
        return None;
      }

      let mut changed = false;
      let mut changed_dependencies = Vec::new();
      let mut exports_info = exports_info_artifact
        .get_exports_info_data(module_identifier)
        .clone();
      for (dep_id, exports_spec) in module_analysis.flat_local_apply() {
        let (task_changed, task_changed_dependencies) = process_exports_spec_without_nested(
          module_graph,
          exports_info_artifact,
          module_identifier,
          *dep_id,
          exports_spec,
          &mut exports_info,
        );
        changed |= task_changed;
        changed_dependencies.extend(task_changed_dependencies);
      }

      Some((
        *module_identifier,
        changed,
        changed_dependencies,
        exports_info,
      ))
    })
    .collect::<Vec<_>>();

  for (module_identifier, changed, changed_dependencies, exports_info) in flat_tasks {
    if changed {
      changed_modules.insert(module_identifier);
    }
    for (target_module, dependent_module) in changed_dependencies {
      dependencies
        .entry(target_module)
        .or_default()
        .insert(dependent_module);
    }
    exports_info_artifact.set_exports_info_by_id(exports_info.id(), exports_info);
  }

  Ok(())
}

fn apply_structured_local_exports_sequentially(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  modules: &IdentifierSet,
  changed_modules: &mut IdentifierSet,
  dependencies: &mut IdentifierMap<IdentifierSet>,
) -> Result<()> {
  for module_identifier in modules {
    let Some(module_analysis) = dependency_exports_analysis_artifact.module(module_identifier)
    else {
      continue;
    };
    let mut changed = false;
    for (dep_id, exports_spec) in module_analysis.structured_local_apply() {
      let (task_changed, changed_dependencies) = process_exports_spec(
        module_graph,
        exports_info_artifact,
        module_identifier,
        *dep_id,
        exports_spec,
      );
      changed |= task_changed;
      for (target_module, dependent_module) in changed_dependencies {
        dependencies
          .entry(target_module)
          .or_default()
          .insert(dependent_module);
      }
    }
    if changed {
      changed_modules.insert(*module_identifier);
    }
  }

  Ok(())
}
