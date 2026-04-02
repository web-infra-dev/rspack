use rayon::prelude::*;
#[cfg(test)]
use rspack_core::ModuleIdentifier;
use rspack_core::{
  DeferredReexportSpec, DependencyExportsAnalysisArtifact, ExportNameOrSpec, ExportSpec,
  ExportsInfo, ExportsInfoArtifact, ExportsInfoData, ExportsOfExportsSpec, ExportsSpec,
  ModuleGraph,
};
use rspack_error::Result;
use rustc_hash::FxHashSet;

use super::process_exports_spec_without_nested;

pub(super) fn propagate_deferred_reexports(
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
) -> Result<()> {
  for wave in dependency_exports_analysis_artifact.topology().waves() {
    let wave_updates = wave
      .par_iter()
      .map(|scc_id| {
        let mut scc_exports_info_artifact =
          snapshot_exports_info_artifact(module_graph, exports_info_artifact);
        resolve_scc_until_fixed_point(
          *scc_id,
          module_graph,
          &mut scc_exports_info_artifact,
          dependency_exports_analysis_artifact,
        )
      })
      .collect::<Vec<_>>();

    for scc_updates in wave_updates {
      for module_update in scc_updates? {
        if module_update.changed {
          exports_info_artifact
            .set_exports_info_by_id(module_update.exports_info.id(), module_update.exports_info);
        }
      }
    }
  }
  dependency_exports_analysis_artifact.clear_all_dirty();

  Ok(())
}

struct PropagationModuleUpdate {
  changed: bool,
  exports_info: ExportsInfoData,
}

fn resolve_scc_until_fixed_point(
  scc_id: usize,
  module_graph: &ModuleGraph,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
) -> Result<Vec<PropagationModuleUpdate>> {
  let scc_modules = dependency_exports_analysis_artifact
    .topology()
    .scc_modules(scc_id)
    .to_vec();

  loop {
    let mut updates = Vec::new();
    let mut changed = false;

    for module_identifier in &scc_modules {
      let Some(module_analysis) = dependency_exports_analysis_artifact.module(module_identifier)
      else {
        continue;
      };
      if module_analysis.deferred_reexports().is_empty() {
        continue;
      }

      let mut module_changed = false;
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
        module_changed |= propagation_changed;
      }

      changed |= module_changed;
      updates.push(PropagationModuleUpdate {
        changed: module_changed,
        exports_info,
      });
    }

    if !changed {
      return Ok(updates);
    }

    for module_update in &updates {
      if module_update.changed {
        exports_info_artifact.set_exports_info_by_id(
          module_update.exports_info.id(),
          module_update.exports_info.clone(),
        );
      }
    }
  }
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

fn snapshot_exports_info_artifact(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> ExportsInfoArtifact {
  let mut snapshot = ExportsInfoArtifact::default();
  let mut copied = FxHashSet::default();

  for module_identifier in module_graph.modules_keys() {
    let exports_info = exports_info_artifact.get_exports_info(module_identifier);
    snapshot.set_exports_info(*module_identifier, exports_info);
    copy_exports_info_recursive(
      exports_info,
      exports_info_artifact,
      &mut snapshot,
      &mut copied,
    );
  }

  snapshot
}

fn copy_exports_info_recursive(
  exports_info: ExportsInfo,
  source: &ExportsInfoArtifact,
  snapshot: &mut ExportsInfoArtifact,
  copied: &mut FxHashSet<ExportsInfo>,
) {
  if !copied.insert(exports_info) {
    return;
  }

  let exports_info_data = source.get_exports_info_by_id(&exports_info).clone();
  let nested_exports_infos = exports_info_data
    .exports()
    .values()
    .filter_map(|export_info| export_info.exports_info())
    .collect::<Vec<_>>();
  snapshot.set_exports_info_by_id(exports_info, exports_info_data);

  for nested_exports_info in nested_exports_infos {
    copy_exports_info_recursive(nested_exports_info, source, snapshot, copied);
  }
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
