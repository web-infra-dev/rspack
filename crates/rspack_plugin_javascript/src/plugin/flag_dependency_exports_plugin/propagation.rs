#[cfg(test)]
use rspack_core::ModuleIdentifier;
use rspack_core::{
  DeferredReexportSpec, DependencyExportsAnalysisArtifact, ExportNameOrSpec, ExportSpec,
  ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
};
use rspack_error::Result;

use super::{process_exports_spec, types::CollectedDependencyExportsAnalysis};

pub(super) fn propagate_deferred_reexports(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  analysis: &CollectedDependencyExportsAnalysis,
) -> Result<()> {
  for wave in dependency_exports_analysis_artifact.topology().waves() {
    for scc_id in wave {
      resolve_scc_until_fixed_point(
        *scc_id,
        module_graph,
        exports_info_artifact,
        dependency_exports_analysis_artifact,
        analysis,
      )?;
    }
  }

  Ok(())
}

fn resolve_scc_until_fixed_point(
  scc_id: usize,
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  analysis: &CollectedDependencyExportsAnalysis,
) -> Result<()> {
  loop {
    let mut changed = false;
    for module_identifier in dependency_exports_analysis_artifact
      .topology()
      .scc_modules(scc_id)
    {
      let Some(module_analysis) = analysis.get(module_identifier) else {
        continue;
      };
      for deferred_reexport in &module_analysis.deferred_reexports {
        let Some(exports_spec) = deferred_reexport_as_exports_spec(module_graph, deferred_reexport)
        else {
          continue;
        };
        let (propagation_changed, _changed_dependencies) = process_exports_spec(
          module_graph,
          exports_info_artifact,
          module_identifier,
          deferred_reexport.dep_id,
          &exports_spec,
        );
        changed |= propagation_changed;
      }
    }

    if !changed {
      break;
    }
  }

  Ok(())
}

fn deferred_reexport_as_exports_spec(
  module_graph: &ModuleGraph,
  deferred_reexport: &DeferredReexportSpec,
) -> Option<ExportsSpec> {
  let from = module_graph
    .connection_by_dependency_id(&deferred_reexport.dep_id)
    .cloned()?;
  let exports = deferred_reexport
    .items
    .iter()
    .map(|item| {
      ExportNameOrSpec::ExportSpec(ExportSpec {
        name: item.exposed_name.clone(),
        from: Some(from.clone()),
        export: match &item.target_path {
          rspack_core::Nullable::Null => None,
          rspack_core::Nullable::Value(path) => Some(rspack_core::Nullable::Value(path.clone())),
        },
        hidden: Some(item.hidden),
        ..Default::default()
      })
    })
    .collect::<Vec<_>>();

  Some(ExportsSpec {
    exports: ExportsOfExportsSpec::Names(exports),
    priority: deferred_reexport.priority,
    can_mangle: deferred_reexport.can_mangle,
    terminal_binding: Some(deferred_reexport.terminal_binding),
    dependencies: Some(vec![deferred_reexport.target_module]),
    ..Default::default()
  })
}

#[cfg(test)]
fn propagation_waves(
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
) -> Vec<Vec<Vec<ModuleIdentifier>>> {
  dependency_exports_analysis_artifact
    .topology()
    .waves()
    .iter()
    .map(|wave| {
      wave
        .iter()
        .map(|scc_id| {
          dependency_exports_analysis_artifact
            .topology()
            .scc_modules(*scc_id)
            .to_vec()
        })
        .collect()
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use rspack_core::{
    DependencyExportsAnalysisArtifact, ModuleDependencyExportsAnalysis, ModuleIdentifier,
  };

  use super::*;

  #[test]
  fn propagate_runs_independent_sccs_in_the_same_wave_and_converges_per_scc() {
    let mut artifact = DependencyExportsAnalysisArtifact::default();
    let left = ModuleIdentifier::from("left");
    let right = ModuleIdentifier::from("right");
    let root = ModuleIdentifier::from("root");

    artifact.replace_module(left, ModuleDependencyExportsAnalysis::with_targets([]));
    artifact.replace_module(right, ModuleDependencyExportsAnalysis::with_targets([]));
    artifact.replace_module(
      root,
      ModuleDependencyExportsAnalysis::with_targets([left, right]),
    );
    artifact.rebuild_topology();

    let summary = propagation_waves(&artifact);
    assert_eq!(summary[0].len(), 2);
    assert_eq!(summary[1].len(), 1);
  }
}
