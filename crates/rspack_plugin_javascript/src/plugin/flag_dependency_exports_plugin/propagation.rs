use rayon::prelude::*;
use rspack_collections::IdentifierSet;
use rspack_core::{
  DeferredReexportSpec, DependencyExportsAnalysisArtifact, ExportNameOrSpec, ExportSpec,
  ExportsInfo, ExportsInfoArtifact, ExportsInfoData, ExportsInfoRead, ExportsOfExportsSpec,
  ExportsSpec, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ProvidedExports,
};
use rspack_error::Result;
use rustc_hash::FxHashMap;

use super::{collector, local_apply, process_exports_spec_without_nested};

pub(super) fn propagate_deferred_reexports(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
) -> Result<()> {
  if !dependency_exports_analysis_artifact.has_deferred_reexports() {
    dependency_exports_analysis_artifact.clear_all_dirty();
    return Ok(());
  }
  if dependency_exports_analysis_artifact.topology_dirty() {
    dependency_exports_analysis_artifact.rebuild_topology();
  }
  let mut changed_modules = IdentifierSet::default();
  let waves = dependency_exports_analysis_artifact
    .topology()
    .waves()
    .to_vec();
  for wave in &waves {
    let wave_modules: IdentifierSet = wave
      .iter()
      .flat_map(|scc_id| {
        dependency_exports_analysis_artifact
          .topology()
          .scc_modules(*scc_id)
          .iter()
          .copied()
      })
      .collect();
    let refresh_modules = select_wave_local_refresh_modules(
      dependency_exports_analysis_artifact,
      &wave_modules,
      &changed_modules,
    );
    if !refresh_modules.is_empty() {
      collector::collect_module_analysis_with_reuse(
        module_graph,
        module_graph_cache,
        exports_info_artifact,
        dependency_exports_analysis_artifact,
        &refresh_modules,
      )?;
      changed_modules = local_apply::apply_local_exports_once(
        module_graph,
        exports_info_artifact,
        dependency_exports_analysis_artifact,
        &refresh_modules,
      )?;
    } else {
      changed_modules.clear();
    }
    let wave_updates = wave
      .par_iter()
      .map(|scc_id| {
        resolve_scc_until_fixed_point(
          *scc_id,
          module_graph,
          exports_info_artifact,
          dependency_exports_analysis_artifact,
        )
      })
      .collect::<Result<Vec<_>>>()?;

    for module_update in wave_updates.into_iter().flatten() {
      if module_update.changed {
        changed_modules.insert(module_update.module_identifier);
        exports_info_artifact
          .set_exports_info_by_id(module_update.exports_info.id(), module_update.exports_info);
      }
    }
  }
  dependency_exports_analysis_artifact.clear_all_dirty();

  Ok(())
}

fn select_wave_local_refresh_modules(
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
  wave_modules: &IdentifierSet,
  changed_modules: &IdentifierSet,
) -> IdentifierSet {
  if changed_modules.is_empty() {
    return IdentifierSet::default();
  }

  wave_modules
    .iter()
    .filter(|module_identifier| {
      dependency_exports_analysis_artifact
        .module(module_identifier)
        .is_some_and(|module_analysis| {
          if !module_analysis.deferred_reexports().is_empty() {
            return true;
          }
          module_analysis
            .flat_dependency_targets()
            .iter()
            .any(|dependency| changed_modules.contains(dependency))
        })
    })
    .copied()
    .collect()
}

#[cfg(test)]
fn process_waves_in_parallel<TWaveState, TPatch, TSnapshot, TResolve, TApply>(
  waves: &[Vec<usize>],
  mut build_wave_state: TSnapshot,
  resolve_scc: TResolve,
  mut apply_wave: TApply,
) -> Result<()>
where
  TWaveState: Sync,
  TPatch: Send,
  TSnapshot: FnMut(&[usize]) -> TWaveState,
  TResolve: Fn(&TWaveState, usize) -> Result<Vec<TPatch>> + Sync,
  TApply: FnMut(&[usize], Vec<TPatch>) -> Result<()>,
{
  for wave in waves {
    let wave_state = build_wave_state(wave);
    let wave_patches = wave
      .par_iter()
      .map(|scc_id| resolve_scc(&wave_state, *scc_id))
      .collect::<Result<Vec<_>>>()?
      .into_iter()
      .flatten()
      .collect::<Vec<_>>();
    apply_wave(wave, wave_patches)?;
  }

  Ok(())
}

struct PropagationModuleUpdate {
  changed: bool,
  module_identifier: ModuleIdentifier,
  exports_info: ExportsInfoData,
}

struct PatchedSccExportsInfoRead<'a> {
  source: &'a ExportsInfoArtifact,
  patches: &'a FxHashMap<ExportsInfo, ExportsInfoData>,
}

impl ExportsInfoRead for PatchedSccExportsInfoRead<'_> {
  fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> ExportsInfo {
    self.source.get_exports_info(module_identifier)
  }

  fn get_exports_info_by_id(&self, id: &ExportsInfo) -> &ExportsInfoData {
    self
      .patches
      .get(id)
      .unwrap_or_else(|| self.source.get_exports_info_by_id(id))
  }
}

fn resolve_scc_until_fixed_point(
  scc_id: usize,
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
) -> Result<Vec<PropagationModuleUpdate>> {
  let scc_modules = dependency_exports_analysis_artifact
    .topology()
    .scc_modules(scc_id)
    .to_vec();
  let mut patches = FxHashMap::default();

  loop {
    let read_state = PatchedSccExportsInfoRead {
      source: exports_info_artifact,
      patches: &patches,
    };
    let mut updates = Vec::new();
    let mut changed = false;
    let mut pending_patches = Vec::new();

    for module_identifier in &scc_modules {
      let Some(module_analysis) = dependency_exports_analysis_artifact.module(module_identifier)
      else {
        continue;
      };
      if module_analysis.deferred_reexports().is_empty() {
        continue;
      }

      let mut module_changed = false;
      let exports_info_id = read_state.get_exports_info(module_identifier);
      let mut exports_info = read_state.get_exports_info_by_id(&exports_info_id).clone();
      for deferred_reexport in module_analysis.deferred_reexports() {
        let Some(exports_spec) =
          deferred_reexport_as_exports_spec(module_graph, &read_state, deferred_reexport)
        else {
          continue;
        };
        let (propagation_changed, _changed_dependencies) = process_exports_spec_without_nested(
          module_graph,
          &read_state,
          module_identifier,
          deferred_reexport.dep_id,
          &exports_spec,
          &mut exports_info,
        );
        module_changed |= propagation_changed;
      }

      changed |= module_changed;
      if module_changed {
        pending_patches.push(exports_info.clone());
      }
      updates.push(PropagationModuleUpdate {
        changed: module_changed,
        module_identifier: *module_identifier,
        exports_info,
      });
    }

    if !changed {
      return Ok(
        updates
          .into_iter()
          .filter(|module_update| patches.contains_key(&module_update.exports_info.id()))
          .map(|module_update| PropagationModuleUpdate {
            changed: true,
            ..module_update
          })
          .collect(),
      );
    }
    for exports_info in pending_patches {
      patches.insert(exports_info.id(), exports_info);
    }
  }
}

fn deferred_reexport_as_exports_spec<T: ExportsInfoRead>(
  module_graph: &ModuleGraph,
  exports_info_read: &T,
  deferred_reexport: &DeferredReexportSpec,
) -> Option<ExportsSpec> {
  let from = module_graph
    .connection_by_dependency_id(&deferred_reexport.dep_id)
    .cloned()?;
  if let Some(star_exports) = &deferred_reexport.star_exports {
    let provided_exports = exports_info_read
      .get_prefetched_exports_info(
        &deferred_reexport.target_module,
        rspack_core::PrefetchExportsInfoMode::Default,
      )
      .get_provided_exports();
    let exports = match provided_exports {
      ProvidedExports::Unknown | ProvidedExports::ProvidedAll => {
        ExportsOfExportsSpec::UnknownExports
      }
      ProvidedExports::ProvidedNames(names) => {
        let mut exports = names
          .into_iter()
          .filter(|name| {
            !star_exports.ignored_exports.contains(name)
              && !star_exports.hidden_exports.contains(name)
          })
          .map(|name| {
            let mut export_path = star_exports.export_name_prefix.clone();
            export_path.push(name.clone());
            ExportNameOrSpec::ExportSpec(ExportSpec {
              name,
              from: Some(from.clone()),
              export: Some(rspack_core::Nullable::Value(export_path)),
              hidden: Some(false),
              ..Default::default()
            })
          })
          .collect::<Vec<_>>();

        exports.extend(star_exports.hidden_exports.iter().cloned().map(|name| {
          let mut export_path = star_exports.export_name_prefix.clone();
          export_path.push(name.clone());
          ExportNameOrSpec::ExportSpec(ExportSpec {
            name,
            from: Some(from.clone()),
            export: Some(rspack_core::Nullable::Value(export_path)),
            hidden: Some(true),
            ..Default::default()
          })
        }));
        ExportsOfExportsSpec::Names(exports)
      }
    };

    let hide_export =
      (!star_exports.hidden_exports.is_empty()).then(|| star_exports.hidden_exports.clone());
    let exclude_exports = {
      let mut excluded = star_exports.ignored_exports.clone();
      excluded.extend(star_exports.hidden_exports.iter().cloned());
      (!excluded.is_empty()).then_some(excluded)
    };

    return Some(ExportsSpec {
      exports,
      priority: deferred_reexport.priority,
      can_mangle: deferred_reexport.can_mangle,
      terminal_binding: Some(deferred_reexport.terminal_binding),
      from: Some(from),
      hide_export,
      exclude_exports,
      dependencies: Some(vec![deferred_reexport.target_module]),
      ..Default::default()
    });
  }
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
  use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
  };

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

  #[test]
  fn propagate_builds_one_shared_snapshot_per_wave() -> Result<()> {
    let waves = vec![vec![0, 1], vec![2]];
    let snapshot_builds = AtomicUsize::new(0);
    let resolved = Mutex::new(Vec::new());
    let applied = Mutex::new(Vec::new());

    process_waves_in_parallel(
      &waves,
      |_| snapshot_builds.fetch_add(1, Ordering::SeqCst) + 1,
      |snapshot_id, scc_id| {
        resolved
          .lock()
          .expect("should lock")
          .push((*snapshot_id, scc_id));
        Ok(vec![(scc_id, *snapshot_id)])
      },
      |_wave, patches| {
        applied.lock().expect("should lock").push(patches);
        Ok(())
      },
    )?;

    assert_eq!(
      snapshot_builds.load(Ordering::SeqCst),
      2,
      "each wave should build exactly one shared snapshot"
    );
    let mut resolved = resolved.lock().expect("should lock").clone();
    resolved.sort_unstable();
    assert_eq!(
      resolved,
      vec![(1, 0), (1, 1), (2, 2)],
      "all SCCs in the first wave should resolve from the same shared snapshot"
    );
    assert_eq!(
      applied.lock().expect("should lock").clone(),
      vec![vec![(0, 1), (1, 1)], vec![(2, 2)]],
      "patches should apply only after the wave-wide parallel work completes"
    );

    Ok(())
  }
}
