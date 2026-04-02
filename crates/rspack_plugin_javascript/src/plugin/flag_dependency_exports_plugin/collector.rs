use rayon::prelude::*;
use rspack_collections::IdentifierSet;
use rspack_core::{
  DependenciesBlock, DependencyExportsAnalysisArtifact, DependencyId, ExportsInfoArtifact,
  ExportsProcessing, ExportsSpec, ModuleDependencyExportsAnalysis, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier,
};
use rspack_error::Result;
use rspack_util::fx_hash::FxIndexSet;

use super::types::{
  CollectedDependencyExportsAnalysis, ModuleCollectedAnalysis, NormalizedModuleAnalysis,
};

pub(super) fn normalize_exports_spec(mut spec: ExportsSpec) -> NormalizedModuleAnalysis {
  match std::mem::take(&mut spec.processing) {
    ExportsProcessing::Immediate => NormalizedModuleAnalysis::from_local(spec),
    ExportsProcessing::DeferredReexport(deferred_reexports) => {
      NormalizedModuleAnalysis::from_deferred(spec, deferred_reexports)
    }
  }
}

pub(super) fn collect_module_analysis(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
  affected_modules: &IdentifierSet,
) -> Result<CollectedDependencyExportsAnalysis> {
  let module_analyses = affected_modules
    .par_iter()
    .map(|module_identifier| {
      let analysis = collect_module_exports_specs(
        module_identifier,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      );
      (*module_identifier, analysis)
    })
    .collect::<Vec<_>>();

  let mut collected = CollectedDependencyExportsAnalysis::default();
  for (module_identifier, analysis) in module_analyses {
    let targets = analysis
      .deferred_reexports
      .iter()
      .map(|reexport| reexport.target_module);
    dependency_exports_analysis_artifact.replace_module(
      module_identifier,
      ModuleDependencyExportsAnalysis::with_targets(targets),
    );
    collected.insert(module_identifier, analysis);
  }

  if dependency_exports_analysis_artifact.topology_dirty() {
    dependency_exports_analysis_artifact.rebuild_topology();
  }

  Ok(collected)
}

fn collect_module_exports_specs(
  module_id: &ModuleIdentifier,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> ModuleCollectedAnalysis {
  fn walk_block<B: DependenciesBlock + ?Sized>(
    block: &B,
    dep_ids: &mut FxIndexSet<DependencyId>,
    module_graph: &ModuleGraph,
  ) {
    dep_ids.extend(block.get_dependencies().iter().copied());
    for block_id in block.get_blocks() {
      if let Some(block) = module_graph.block_by_id(block_id) {
        walk_block(block, dep_ids, module_graph);
      }
    }
  }

  let Some(block) = module_graph
    .module_by_identifier(module_id)
    .map(AsRef::as_ref)
  else {
    return ModuleCollectedAnalysis::default();
  };

  let mut dep_ids = FxIndexSet::default();
  walk_block(block, &mut dep_ids, module_graph);

  let mut analysis = ModuleCollectedAnalysis::default();
  for dep_id in dep_ids {
    let dep = module_graph.dependency_by_id(&dep_id);
    let Some(exports_spec) =
      dep.get_exports(module_graph, module_graph_cache, exports_info_artifact)
    else {
      continue;
    };
    let normalized = normalize_exports_spec(exports_spec);
    let NormalizedModuleAnalysis {
      local_apply,
      deferred_reexports,
    } = normalized;
    for (bound_dep_id, exports_spec) in
      NormalizedModuleAnalysis::bind_local_apply_with_dep_id(dep_id, local_apply)
    {
      if exports_spec.has_nested_exports() {
        analysis
          .structured_local_apply
          .push((bound_dep_id, exports_spec));
      } else {
        analysis.flat_local_apply.push((bound_dep_id, exports_spec));
      }
    }
    analysis.deferred_reexports.extend(deferred_reexports);
  }

  analysis
}

#[cfg(test)]
mod tests {
  use rspack_core::{
    DeferredReexportItem, DeferredReexportSpec, DependencyId, ExportsOfExportsSpec,
    ExportsProcessing, ExportsSpec, ModuleIdentifier, Nullable,
  };

  use super::*;
  use crate::plugin::flag_dependency_exports_plugin::types::NormalizedModuleAnalysis;

  #[test]
  fn normalize_exports_spec_keeps_deferred_reexports_out_of_local_apply() {
    let target = ModuleIdentifier::from("leaf");
    let spec = ExportsSpec {
      exports: ExportsOfExportsSpec::Names(vec![]),
      processing: ExportsProcessing::DeferredReexport(vec![DeferredReexportSpec::new(
        target,
        DependencyId::from(7),
        vec![DeferredReexportItem {
          exposed_name: "value".into(),
          target_path: Nullable::Value(vec!["value".into()]),
          hidden: false,
        }],
      )]),
      ..Default::default()
    };

    let normalized = normalize_exports_spec(spec);
    assert!(normalized.local_apply.is_empty());
    assert_eq!(normalized.deferred_reexports.len(), 1);
  }

  #[test]
  fn bind_local_apply_preserves_fragment_multiplicity_for_one_dependency() {
    let dep_id = DependencyId::from(9);
    let analysis = NormalizedModuleAnalysis {
      local_apply: vec![
        ExportsSpec {
          exports: ExportsOfExportsSpec::UnknownExports,
          ..Default::default()
        },
        ExportsSpec {
          hide_export: Some(["value".into()].into_iter().collect()),
          ..Default::default()
        },
      ],
      deferred_reexports: Vec::new(),
    };

    let bound =
      NormalizedModuleAnalysis::bind_local_apply_with_dep_id(dep_id, analysis.local_apply);
    assert_eq!(bound.len(), 2);
    assert!(
      bound
        .iter()
        .all(|(bound_dep_id, _)| *bound_dep_id == dep_id)
    );
  }
}
