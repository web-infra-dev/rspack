use rayon::prelude::*;
#[cfg(test)]
use rspack_core::ModuleIdentifier;
use rspack_core::{
  DeferredReexportSpec, DependencyExportsAnalysisArtifact, ExportNameOrSpec, ExportSpec,
  ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec, ModuleGraph,
};
use rspack_error::Result;

use super::process_exports_spec_without_nested;

pub(super) fn propagate_deferred_reexports(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
) -> Result<()> {
  for wave in dependency_exports_analysis_artifact.topology().waves() {
    loop {
      let wave_iteration = wave
        .par_iter()
        .map(|scc_id| {
          resolve_scc_iteration(
            *scc_id,
            module_graph,
            exports_info_artifact,
            dependency_exports_analysis_artifact,
          )
        })
        .collect::<Vec<_>>();

      let mut changed = false;
      for scc_updates in wave_iteration {
        for module_update in scc_updates? {
          changed |= module_update.changed;
          exports_info_artifact
            .set_exports_info_by_id(module_update.exports_info.id(), module_update.exports_info);
        }
      }

      if !changed {
        break;
      }
    }
  }

  Ok(())
}

struct PropagationModuleUpdate {
  changed: bool,
  exports_info: rspack_core::ExportsInfoData,
}

fn resolve_scc_iteration(
  scc_id: usize,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
) -> Result<Vec<PropagationModuleUpdate>> {
  let mut updates = Vec::new();
  for module_identifier in dependency_exports_analysis_artifact
    .topology()
    .scc_modules(scc_id)
  {
    let Some(module_analysis) = dependency_exports_analysis_artifact.module(module_identifier)
    else {
      continue;
    };
    if module_analysis.deferred_reexports().is_empty() {
      continue;
    }

    let mut changed = false;
    let mut exports_info = exports_info_artifact
      .get_exports_info_data(module_identifier)
      .clone();
    for deferred_reexport in module_analysis.deferred_reexports() {
      let Some(exports_spec) = deferred_reexport_as_exports_spec(module_graph, deferred_reexport)
      else {
        continue;
      };
      let (propagation_changed, _changed_dependencies) = process_exports_spec_without_nested(
        module_graph,
        exports_info_artifact,
        module_identifier,
        deferred_reexport.dep_id,
        &exports_spec,
        &mut exports_info,
      );
      changed |= propagation_changed;
    }

    updates.push(PropagationModuleUpdate {
      changed,
      exports_info,
    });
  }

  Ok(updates)
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
    DeferredReexportItem, DeferredReexportSpec, DependencyExportsAnalysisArtifact, DependencyId,
    ModuleDependencyExportsAnalysis, ModuleIdentifier, Nullable,
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
      ModuleDependencyExportsAnalysis::with_staged_analysis(
        [left, right],
        [],
        [],
        [DeferredReexportSpec::new(
          left,
          DependencyId::from(7),
          vec![DeferredReexportItem {
            exposed_name: "value".into(),
            target_path: Nullable::Value(vec!["value".into()]),
            hidden: false,
          }],
        )],
      ),
    );
    artifact.rebuild_topology();

    let summary = propagation_waves(&artifact);
    assert_eq!(summary[0].len(), 2);
    assert_eq!(summary[1].len(), 1);
    assert_eq!(
      artifact
        .module(&root)
        .expect("root analysis should exist")
        .deferred_reexports()
        .len(),
      1
    );
  }
}
