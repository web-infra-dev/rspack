use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  DependencyExportsAnalysisArtifact, ExportsInfoArtifact, ModuleGraph, ModuleGraphCacheArtifact,
};
use rspack_error::Result;

use super::{collector, process_exports_spec, process_exports_spec_without_nested};

pub(super) fn apply_local_exports(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
  initial_modules: &IdentifierSet,
) -> Result<()> {
  let mut driver = ArtifactLocalApplyDriver {
    module_graph,
    module_graph_cache,
    dependency_exports_analysis_artifact,
  };

  apply_local_exports_with_driver(&mut driver, exports_info_artifact, initial_modules)
}

pub(super) fn apply_local_exports_once(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  modules: &IdentifierSet,
) -> Result<IdentifierSet> {
  let mut changed_modules =
    IdentifierSet::with_capacity_and_hasher(modules.len(), Default::default());
  let mut dependencies = IdentifierMap::default();

  apply_flat_local_exports_in_parallel(
    module_graph,
    exports_info_artifact,
    dependency_exports_analysis_artifact,
    modules,
    &mut changed_modules,
    &mut dependencies,
  )?;

  Ok(changed_modules)
}

trait LocalApplyDriver {
  fn recollect(
    &mut self,
    exports_info_artifact: &ExportsInfoArtifact,
    modules: &IdentifierSet,
  ) -> Result<()>;

  fn apply_flat(
    &mut self,
    exports_info_artifact: &mut ExportsInfoArtifact,
    modules: &IdentifierSet,
    changed_modules: &mut IdentifierSet,
    dependencies: &mut IdentifierMap<IdentifierSet>,
  ) -> Result<()>;

  fn apply_structured(
    &mut self,
    exports_info_artifact: &mut ExportsInfoArtifact,
    modules: &IdentifierSet,
    changed_modules: &mut IdentifierSet,
    dependencies: &mut IdentifierMap<IdentifierSet>,
  ) -> Result<()>;
}

struct ArtifactLocalApplyDriver<'a> {
  module_graph: &'a ModuleGraph,
  module_graph_cache: &'a ModuleGraphCacheArtifact,
  dependency_exports_analysis_artifact: &'a mut DependencyExportsAnalysisArtifact,
}

impl LocalApplyDriver for ArtifactLocalApplyDriver<'_> {
  fn recollect(
    &mut self,
    exports_info_artifact: &ExportsInfoArtifact,
    modules: &IdentifierSet,
  ) -> Result<()> {
    collector::collect_module_analysis(
      self.module_graph,
      self.module_graph_cache,
      exports_info_artifact,
      self.dependency_exports_analysis_artifact,
      modules,
    )
  }

  fn apply_flat(
    &mut self,
    exports_info_artifact: &mut ExportsInfoArtifact,
    modules: &IdentifierSet,
    changed_modules: &mut IdentifierSet,
    dependencies: &mut IdentifierMap<IdentifierSet>,
  ) -> Result<()> {
    apply_flat_local_exports_in_parallel(
      self.module_graph,
      exports_info_artifact,
      self.dependency_exports_analysis_artifact,
      modules,
      changed_modules,
      dependencies,
    )
  }

  fn apply_structured(
    &mut self,
    exports_info_artifact: &mut ExportsInfoArtifact,
    modules: &IdentifierSet,
    changed_modules: &mut IdentifierSet,
    dependencies: &mut IdentifierMap<IdentifierSet>,
  ) -> Result<()> {
    apply_structured_local_exports_sequentially(
      self.module_graph,
      exports_info_artifact,
      self.dependency_exports_analysis_artifact,
      modules,
      changed_modules,
      dependencies,
    )
  }
}

fn apply_local_exports_with_driver(
  driver: &mut impl LocalApplyDriver,
  exports_info_artifact: &mut ExportsInfoArtifact,
  initial_modules: &IdentifierSet,
) -> Result<()> {
  let mut batch = initial_modules.clone();
  let mut dependencies: IdentifierMap<IdentifierSet> =
    IdentifierMap::with_capacity_and_hasher(batch.len(), Default::default());

  while !batch.is_empty() {
    let modules = std::mem::take(&mut batch);
    let mut changed_modules =
      IdentifierSet::with_capacity_and_hasher(modules.len(), Default::default());

    driver.recollect(exports_info_artifact, &modules)?;
    driver.apply_flat(
      exports_info_artifact,
      &modules,
      &mut changed_modules,
      &mut dependencies,
    )?;
    driver.apply_structured(
      exports_info_artifact,
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

#[cfg(test)]
fn apply_local_exports_with<TRecollect, TFlat, TStructured>(
  exports_info_artifact: &mut ExportsInfoArtifact,
  initial_modules: &IdentifierSet,
  recollect: TRecollect,
  apply_flat: TFlat,
  apply_structured: TStructured,
) -> Result<()>
where
  TRecollect: FnMut(&ExportsInfoArtifact, &IdentifierSet) -> Result<()>,
  TFlat: FnMut(
    &mut ExportsInfoArtifact,
    &IdentifierSet,
    &mut IdentifierSet,
    &mut IdentifierMap<IdentifierSet>,
  ) -> Result<()>,
  TStructured: FnMut(
    &mut ExportsInfoArtifact,
    &IdentifierSet,
    &mut IdentifierSet,
    &mut IdentifierMap<IdentifierSet>,
  ) -> Result<()>,
{
  struct ClosureLocalApplyDriver<TRecollect, TFlat, TStructured> {
    recollect: TRecollect,
    apply_flat: TFlat,
    apply_structured: TStructured,
  }

  impl<TRecollect, TFlat, TStructured> LocalApplyDriver
    for ClosureLocalApplyDriver<TRecollect, TFlat, TStructured>
  where
    TRecollect: FnMut(&ExportsInfoArtifact, &IdentifierSet) -> Result<()>,
    TFlat: FnMut(
      &mut ExportsInfoArtifact,
      &IdentifierSet,
      &mut IdentifierSet,
      &mut IdentifierMap<IdentifierSet>,
    ) -> Result<()>,
    TStructured: FnMut(
      &mut ExportsInfoArtifact,
      &IdentifierSet,
      &mut IdentifierSet,
      &mut IdentifierMap<IdentifierSet>,
    ) -> Result<()>,
  {
    fn recollect(
      &mut self,
      exports_info_artifact: &ExportsInfoArtifact,
      modules: &IdentifierSet,
    ) -> Result<()> {
      (self.recollect)(exports_info_artifact, modules)
    }

    fn apply_flat(
      &mut self,
      exports_info_artifact: &mut ExportsInfoArtifact,
      modules: &IdentifierSet,
      changed_modules: &mut IdentifierSet,
      dependencies: &mut IdentifierMap<IdentifierSet>,
    ) -> Result<()> {
      (self.apply_flat)(
        exports_info_artifact,
        modules,
        changed_modules,
        dependencies,
      )
    }

    fn apply_structured(
      &mut self,
      exports_info_artifact: &mut ExportsInfoArtifact,
      modules: &IdentifierSet,
      changed_modules: &mut IdentifierSet,
      dependencies: &mut IdentifierMap<IdentifierSet>,
    ) -> Result<()> {
      (self.apply_structured)(
        exports_info_artifact,
        modules,
        changed_modules,
        dependencies,
      )
    }
  }

  let mut driver = ClosureLocalApplyDriver {
    recollect,
    apply_flat,
    apply_structured,
  };
  apply_local_exports_with_driver(&mut driver, exports_info_artifact, initial_modules)
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

#[cfg(test)]
mod tests {
  use rspack_core::{ExportProvided, ModuleIdentifier};
  use swc_core::ecma::atoms::Atom;

  use super::*;

  fn sorted_modules(modules: &IdentifierSet) -> Vec<ModuleIdentifier> {
    let mut modules = modules.iter().copied().collect::<Vec<_>>();
    modules.sort_unstable();
    modules
  }

  #[test]
  fn apply_local_exports_recollects_requeued_modules_against_latest_exports_info() -> Result<()> {
    let root = ModuleIdentifier::from("root");
    let dependent = ModuleIdentifier::from("dependent");
    let export_name = Atom::from("value");
    let mut exports_info_artifact = ExportsInfoArtifact::default();
    exports_info_artifact.new_exports_info(root);
    exports_info_artifact.new_exports_info(dependent);

    let mut initial_modules = IdentifierSet::default();
    initial_modules.insert(root);

    let mut recollected_batches = Vec::new();
    let mut dependent_saw_latest_exports = false;

    apply_local_exports_with(
      &mut exports_info_artifact,
      &initial_modules,
      |exports_info_artifact, modules| {
        recollected_batches.push(sorted_modules(modules));
        if modules.contains(&dependent) {
          dependent_saw_latest_exports = matches!(
            exports_info_artifact
              .get_exports_info_data(&root)
              .named_exports(&export_name)
              .and_then(|export_info| export_info.provided()),
            Some(ExportProvided::Provided)
          );
        }
        Ok(())
      },
      |exports_info_artifact, modules, changed_modules, dependencies| {
        if modules.contains(&root) {
          exports_info_artifact
            .get_exports_info_data_mut(&root)
            .ensure_owned_export_info(&export_name)
            .set_provided(Some(ExportProvided::Provided));
          changed_modules.insert(root);
          dependencies.entry(root).or_default().insert(dependent);
        }
        Ok(())
      },
      |_, _, _, _| Ok(()),
    )?;

    assert_eq!(
      recollected_batches,
      vec![vec![root], vec![dependent]],
      "the requeued dependent should be recollected in its own follow-up batch"
    );
    assert!(
      dependent_saw_latest_exports,
      "the dependent recollection should observe the root export after the first batch applies"
    );

    Ok(())
  }
}
