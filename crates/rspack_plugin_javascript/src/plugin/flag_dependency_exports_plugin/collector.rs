use rayon::prelude::*;
use rspack_collections::IdentifierSet;
use rspack_core::{
  DependenciesBlock, DependencyExportsAnalysisArtifact, DependencyId, ExportsInfoArtifact,
  ExportsProcessing, ExportsSpec, ModuleDependencyExportsAnalysis, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier,
};
use rspack_error::Result;
use rspack_util::fx_hash::FxIndexSet;

use super::types::NormalizedModuleAnalysis;

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
) -> Result<()> {
  collect_module_analysis_inner(
    module_graph,
    module_graph_cache,
    exports_info_artifact,
    dependency_exports_analysis_artifact,
    affected_modules,
    false,
  )
}

pub(super) fn collect_module_analysis_with_reuse(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
  affected_modules: &IdentifierSet,
) -> Result<()> {
  collect_module_analysis_inner(
    module_graph,
    module_graph_cache,
    exports_info_artifact,
    dependency_exports_analysis_artifact,
    affected_modules,
    true,
  )
}

fn collect_module_analysis_inner(
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
  dependency_exports_analysis_artifact: &mut DependencyExportsAnalysisArtifact,
  affected_modules: &IdentifierSet,
  allow_reuse: bool,
) -> Result<()> {
  let reusable_modules = affected_modules
    .iter()
    .filter_map(|module_identifier| {
      dependency_exports_analysis_artifact
        .module(module_identifier)
        .filter(|analysis| {
          allow_reuse && !analysis.dirty() && analysis.can_reuse_without_recollect()
        })
        .map(|_| *module_identifier)
    })
    .collect::<IdentifierSet>();
  let module_analyses = affected_modules
    .par_iter()
    .filter_map(|module_identifier| {
      if reusable_modules.contains(module_identifier) {
        return None;
      }
      let analysis = collect_module_exports_specs(
        module_identifier,
        module_graph,
        module_graph_cache,
        exports_info_artifact,
      );
      Some((*module_identifier, analysis))
    })
    .collect::<Vec<_>>();

  for module_identifier in &reusable_modules {
    if let Some(module_analysis) =
      dependency_exports_analysis_artifact.module_mut(module_identifier)
    {
      module_analysis.set_dirty(true);
    }
  }

  for (module_identifier, analysis) in module_analyses {
    let mut targets = analysis
      .flat_local_apply
      .iter()
      .flat_map(|(_, exports_spec)| exports_spec.dependencies.iter().flatten().copied())
      .collect::<Vec<_>>();
    targets.extend(
      analysis
        .deferred_reexports
        .iter()
        .map(|reexport| reexport.target_module),
    );
    targets.sort_unstable();
    targets.dedup();
    let mut module_analysis = ModuleDependencyExportsAnalysis::with_staged_analysis(
      targets,
      analysis.flat_local_apply,
      analysis.structured_local_apply,
      analysis.deferred_reexports,
    );
    module_analysis.set_dirty(true);
    dependency_exports_analysis_artifact.upsert_module(module_identifier, module_analysis);
  }

  Ok(())
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

#[derive(Debug, Default)]
struct ModuleCollectedAnalysis {
  flat_local_apply: Vec<(DependencyId, ExportsSpec)>,
  structured_local_apply: Vec<(DependencyId, ExportsSpec)>,
  deferred_reexports: Vec<rspack_core::DeferredReexportSpec>,
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
