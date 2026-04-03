use rayon::prelude::*;
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  DeferredReexportSpec, DependencyExportsAnalysisArtifact, ExportNameOrSpec, ExportSpec,
  ExportsInfo, ExportsInfoArtifact, ExportsInfoData, ExportsInfoRead, ExportsOfExportsSpec,
  ExportsSpec, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, ProvidedExports,
};
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{collector, local_apply, process_exports_spec_without_nested};

pub(super) fn propagate_deferred_reexports(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
) -> Result<()> {
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
    collector::collect_module_analysis(
      module_graph,
      module_graph_cache,
      exports_info_artifact,
      dependency_exports_analysis_artifact,
      &wave_modules,
    )?;
    local_apply::apply_local_exports_once(
      module_graph,
      exports_info_artifact,
      dependency_exports_analysis_artifact,
      &wave_modules,
    )?;
    let wave_snapshot = snapshot_exports_info_artifact(module_graph, exports_info_artifact);
    let wave_updates = wave
      .par_iter()
      .map(|scc_id| {
        resolve_scc_until_fixed_point(
          *scc_id,
          module_graph,
          &wave_snapshot,
          dependency_exports_analysis_artifact,
        )
      })
      .collect::<Result<Vec<_>>>()?;

    for module_update in wave_updates.into_iter().flatten() {
      if module_update.changed {
        exports_info_artifact
          .set_exports_info_by_id(module_update.exports_info.id(), module_update.exports_info);
      }
    }
  }
  dependency_exports_analysis_artifact.clear_all_dirty();

  Ok(())
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
  exports_info: ExportsInfoData,
}

struct WaveExportsInfoSnapshot {
  module_exports_info: IdentifierMap<ExportsInfo>,
  exports_info_map: FxHashMap<ExportsInfo, ExportsInfoData>,
}

impl ExportsInfoRead for WaveExportsInfoSnapshot {
  fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> ExportsInfo {
    *self
      .module_exports_info
      .get(module_identifier)
      .expect("should have module exports info")
  }

  fn get_exports_info_by_id(&self, id: &ExportsInfo) -> &ExportsInfoData {
    self
      .exports_info_map
      .get(id)
      .expect("should have exports info snapshot")
  }
}

struct PatchedSccExportsInfoRead<'a> {
  snapshot: &'a WaveExportsInfoSnapshot,
  patches: &'a FxHashMap<ExportsInfo, ExportsInfoData>,
}

impl ExportsInfoRead for PatchedSccExportsInfoRead<'_> {
  fn get_exports_info(&self, module_identifier: &ModuleIdentifier) -> ExportsInfo {
    self.snapshot.get_exports_info(module_identifier)
  }

  fn get_exports_info_by_id(&self, id: &ExportsInfo) -> &ExportsInfoData {
    self
      .patches
      .get(id)
      .unwrap_or_else(|| self.snapshot.get_exports_info_by_id(id))
  }
}

fn resolve_scc_until_fixed_point(
  scc_id: usize,
  module_graph: &ModuleGraph,
  wave_snapshot: &WaveExportsInfoSnapshot,
  dependency_exports_analysis_artifact: &DependencyExportsAnalysisArtifact,
) -> Result<Vec<PropagationModuleUpdate>> {
  let scc_modules = dependency_exports_analysis_artifact
    .topology()
    .scc_modules(scc_id)
    .to_vec();
  let mut patches = FxHashMap::default();

  loop {
    let read_state = PatchedSccExportsInfoRead {
      snapshot: wave_snapshot,
      patches: &patches,
    };
    let mut updates = Vec::new();
    let mut changed = false;
    let mut next_patches = patches.clone();

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
        next_patches.insert(exports_info.id(), exports_info.clone());
      }
      updates.push(PropagationModuleUpdate {
        changed: module_changed,
        exports_info,
      });
    }

    if !changed {
      return Ok(
        patches
          .into_values()
          .map(|exports_info| PropagationModuleUpdate {
            changed: true,
            exports_info,
          })
          .collect(),
      );
    }
    patches = next_patches;
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

fn snapshot_exports_info_artifact(
  module_graph: &ModuleGraph,
  exports_info_artifact: &ExportsInfoArtifact,
) -> WaveExportsInfoSnapshot {
  let mut snapshot = WaveExportsInfoSnapshot {
    module_exports_info: IdentifierMap::default(),
    exports_info_map: FxHashMap::default(),
  };
  let mut copied = FxHashSet::default();

  for module_identifier in module_graph.modules_keys() {
    let exports_info = exports_info_artifact.get_exports_info(module_identifier);
    snapshot
      .module_exports_info
      .insert(*module_identifier, exports_info);
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
  snapshot: &mut WaveExportsInfoSnapshot,
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
  snapshot
    .exports_info_map
    .insert(exports_info, exports_info_data);

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
