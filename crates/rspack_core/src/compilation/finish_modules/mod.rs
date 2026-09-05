use async_trait::async_trait;
use rspack_error::Result;

use super::*;
use crate::{
  DependencyDiagnosticsContext, OptimizationBailoutItem, SideEffectsStateArtifact, logger::Logger,
  pass::PassExt,
};

pub struct FinishModulesPhasePass;

#[async_trait]
impl PassExt for FinishModulesPhasePass {
  fn name(&self) -> &'static str {
    "finish modules"
  }

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::FINISH_MODULES
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    finish_modules_pass(compilation).await?;

    use crate::incremental::IncrementalPasses;
    if compilation
      .incremental
      .passes_enabled(IncrementalPasses::BUILD_MODULE_GRAPH)
    {
      compilation.exports_info_artifact.checkpoint();
    }
    Ok(())
  }
}

#[tracing::instrument("Compilation:finish_modules", skip_all)]
pub async fn finish_modules_pass(compilation: &mut Compilation) -> Result<()> {
  if let Some(mut mutations) = compilation.incremental.mutations_write() {
    let build_module_graph_artifact = &compilation.build_module_graph_artifact;
    mutations.extend(
      build_module_graph_artifact
        .affected_dependencies
        .updated()
        .iter()
        .map(|&dependency| Mutation::DependencyUpdate { dependency }),
    );
    mutations.extend(
      build_module_graph_artifact
        .affected_modules
        .removed()
        .iter()
        .map(|&module| Mutation::ModuleRemove { module }),
    );
    mutations.extend(
      build_module_graph_artifact
        .affected_modules
        .updated()
        .iter()
        .map(|&module| Mutation::ModuleUpdate { module }),
    );
    mutations.extend(
      build_module_graph_artifact
        .affected_modules
        .added()
        .iter()
        .map(|&module| Mutation::ModuleAdd { module }),
    );
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::BUILD_MODULE_GRAPH, %mutations);
  }

  // finish_modules means the module graph (modules, connections, dependencies) are
  // frozen and start to optimize (provided exports, infer async, etc.) based on the
  // module graph, so any kind of change that affect these should be done before the
  // finish_modules
  // Keep artifacts in Compilation across hook calls. JavaScript taps can access
  // ModuleGraph and ExportsInfo, including while an asynchronous tap is pending.
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .finish_modules
    .call(compilation)
    .await?;

  // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2988
  compilation.module_graph_cache_artifact.freeze();
  // Collect dependencies diagnostics at here to make sure:
  // 1. after finish_modules: has provide exports info
  // 2. before optimize dependencies: side effects free module hasn't been skipped
  let mut all_diagnostics = collect_dependencies_diagnostics(compilation);
  compilation.module_graph_cache_artifact.unfreeze();

  // take make diagnostics
  let diagnostics = compilation.build_module_graph_artifact.diagnostics();
  all_diagnostics.extend(diagnostics);

  let build_module_graph_artifact = &mut *compilation.build_module_graph_artifact;
  apply_side_effects_state_artifact(
    &mut build_module_graph_artifact.module_graph,
    &build_module_graph_artifact.side_effects_state_artifact,
  );
  compilation.extend_diagnostics(all_diagnostics);
  Ok(())
}

#[tracing::instrument("Compilation:collect_dependencies_diagnostics", skip_all)]
fn collect_dependencies_diagnostics(compilation: &mut Compilation) -> Vec<Diagnostic> {
  let logger = compilation.get_logger("rspack.incremental.finishModules");
  let build_module_graph_artifact = &compilation.build_module_graph_artifact;
  let dependencies_diagnostics_artifact = &mut *compilation.dependencies_diagnostics_artifact;
  let exports_info_artifact = &compilation.exports_info_artifact;
  // Compute modules while holding the lock, then release it
  let (modules, has_mutations) = {
    let mutations = compilation
      .incremental
      .mutations_read(IncrementalPasses::FINISH_MODULES);

    // TODO move diagnostic collect to make
    if let Some(mutations) = mutations {
      if !dependencies_diagnostics_artifact.is_empty() {
        let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
          Mutation::ModuleRemove { module } => Some(*module),
          _ => None,
        });
        for revoked_module in revoked_modules {
          dependencies_diagnostics_artifact.remove(&revoked_module);
        }
        let modules = mutations
          .get_affected_modules_with_module_graph(build_module_graph_artifact.get_module_graph());
        logger.log(format!(
          "{} modules are affected, {} in total",
          modules.len(),
          build_module_graph_artifact.get_module_graph().modules_len()
        ));
        (modules, true)
      } else {
        (
          build_module_graph_artifact
            .get_module_graph()
            .modules_keys()
            .copied()
            .collect(),
          true,
        )
      }
    } else {
      (
        build_module_graph_artifact
          .get_module_graph()
          .modules_keys()
          .copied()
          .collect(),
        false,
      )
    }
  };

  let module_graph = build_module_graph_artifact.get_module_graph();
  let module_graph_cache = &compilation.module_graph_cache_artifact;
  let dependencies_diagnostics: DependenciesDiagnosticsArtifact = modules
    .par_iter()
    .map(|module_identifier| {
      let mgm = module_graph
        .module_graph_module_by_identifier(module_identifier)
        .expect("should have mgm");
      let diagnostics_context = DependencyDiagnosticsContext::default();
      let diagnostics = mgm
        .all_dependencies()
        .iter()
        .filter_map(|dependency_id| {
          let dependency = module_graph.dependency_by_id(dependency_id);
          dependency
            .get_diagnostics_with_context(
              module_graph,
              module_graph_cache,
              exports_info_artifact,
              &diagnostics_context,
            )
            .map(|diagnostics| {
              diagnostics.into_iter().map(|mut diagnostic| {
                diagnostic.module_identifier = Some(*module_identifier);
                diagnostic.loc = dependency.loc();
                diagnostic
              })
            })
        })
        .flatten()
        .collect::<Vec<_>>();
      (*module_identifier, diagnostics)
    })
    .collect::<rspack_collections::IdentifierMap<Vec<Diagnostic>>>()
    .into();
  let all_modules_diagnostics = if has_mutations {
    dependencies_diagnostics_artifact.extend(dependencies_diagnostics);
    dependencies_diagnostics_artifact.clone()
  } else {
    dependencies_diagnostics
  };
  all_modules_diagnostics.into_values().flatten().collect()
}

fn apply_side_effects_state_artifact(
  module_graph: &mut ModuleGraph,
  side_effects_state_artifact: &SideEffectsStateArtifact,
) {
  if side_effects_state_artifact.is_empty() {
    return;
  }

  for (module_id, state) in side_effects_state_artifact.iter() {
    if module_graph.module_by_identifier(module_id).is_none() {
      continue;
    }

    let bailouts = module_graph.get_optimization_bailout_mut(module_id);
    bailouts.retain(|item| {
      !state
        .optimization_bailouts_to_remove
        .iter()
        .any(|target| optimization_bailout_item_eq(item, target))
    });
    for item in &state.optimization_bailouts_to_add {
      if bailouts
        .iter()
        .any(|existing| optimization_bailout_item_eq(existing, item))
      {
        continue;
      }
      bailouts.push(item.clone());
    }
  }
}

fn optimization_bailout_item_eq(
  left: &OptimizationBailoutItem,
  right: &OptimizationBailoutItem,
) -> bool {
  match (left, right) {
    (OptimizationBailoutItem::Message(left), OptimizationBailoutItem::Message(right)) => {
      left == right
    }
    (
      OptimizationBailoutItem::SideEffects {
        node_type: left_node_type,
        loc: left_loc,
        short_id: left_short_id,
      },
      OptimizationBailoutItem::SideEffects {
        node_type: right_node_type,
        loc: right_loc,
        short_id: right_short_id,
      },
    ) => {
      left_node_type == right_node_type && left_loc == right_loc && left_short_id == right_short_id
    }
    _ => false,
  }
}
