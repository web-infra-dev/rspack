use rayon::prelude::*;
use rspack_collections::IdentifierSet;
use rspack_core::{ExportsInfoArtifact, ModuleGraph};
use rspack_error::Result;

use super::{
  process_exports_spec, process_exports_spec_without_nested,
  types::CollectedDependencyExportsAnalysis,
};

pub(super) fn apply_local_exports(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  analysis: &CollectedDependencyExportsAnalysis,
  affected_modules: &IdentifierSet,
) -> Result<()> {
  apply_flat_local_exports_in_parallel(
    module_graph,
    exports_info_artifact,
    analysis,
    affected_modules,
  )?;
  apply_structured_local_exports_sequentially(
    module_graph,
    exports_info_artifact,
    analysis,
    affected_modules,
  )?;
  Ok(())
}

fn apply_flat_local_exports_in_parallel(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  analysis: &CollectedDependencyExportsAnalysis,
  affected_modules: &IdentifierSet,
) -> Result<()> {
  let flat_tasks = affected_modules
    .par_iter()
    .filter_map(|module_identifier| {
      let module_analysis = analysis.get(module_identifier)?;
      if module_analysis.flat_local_apply.is_empty() {
        return None;
      }

      let mut changed = false;
      let mut exports_info = exports_info_artifact
        .get_exports_info_data(module_identifier)
        .clone();
      for (dep_id, exports_spec) in &module_analysis.flat_local_apply {
        let (task_changed, _changed_dependencies) = process_exports_spec_without_nested(
          module_graph,
          exports_info_artifact,
          module_identifier,
          *dep_id,
          exports_spec,
          &mut exports_info,
        );
        changed |= task_changed;
      }

      Some((*module_identifier, changed, exports_info))
    })
    .collect::<Vec<_>>();

  for (_module_identifier, _changed, exports_info) in flat_tasks {
    exports_info_artifact.set_exports_info_by_id(exports_info.id(), exports_info);
  }

  Ok(())
}

fn apply_structured_local_exports_sequentially(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  analysis: &CollectedDependencyExportsAnalysis,
  affected_modules: &IdentifierSet,
) -> Result<()> {
  for module_identifier in affected_modules {
    let Some(module_analysis) = analysis.get(module_identifier) else {
      continue;
    };
    for (dep_id, exports_spec) in &module_analysis.structured_local_apply {
      let (_changed, _changed_dependencies) = process_exports_spec(
        module_graph,
        exports_info_artifact,
        module_identifier,
        *dep_id,
        exports_spec,
      );
    }
  }

  Ok(())
}
