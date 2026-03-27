use async_trait::async_trait;
use rspack_error::Result;

use super::*;
use crate::{OptimizationBailoutItem, cache::Cache, logger::Logger, pass::PassExt};

pub struct FinishModulesPhasePass;

#[async_trait]
impl PassExt for FinishModulesPhasePass {
  fn name(&self) -> &'static str {
    "finish modules"
  }

  async fn before_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.before_finish_modules(compilation).await;
  }

  async fn after_pass(&self, compilation: &mut Compilation, cache: &mut dyn Cache) {
    cache.after_finish_modules(compilation).await;
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

pub async fn finish_modules_pass(compilation: &mut Compilation) -> Result<()> {
  let mut dependencies_diagnostics_artifact = compilation.dependencies_diagnostics_artifact.steal();
  let mut async_modules_artifact = compilation.async_modules_artifact.steal();
  let mut exports_info_artifact = compilation.exports_info_artifact.steal();
  let diagnostics = finish_modules_inner(
    compilation,
    &mut dependencies_diagnostics_artifact,
    &mut async_modules_artifact,
    &mut exports_info_artifact,
  )
  .await;
  compilation.dependencies_diagnostics_artifact = dependencies_diagnostics_artifact.into();
  compilation.async_modules_artifact = async_modules_artifact.into();
  compilation.exports_info_artifact = exports_info_artifact.into();
  let diagnostics = diagnostics?;
  compilation.extend_diagnostics(diagnostics);

  Ok(())
}

#[tracing::instrument("Compilation:finish_modules_inner", skip_all)]
pub async fn finish_modules_inner(
  compilation: &mut Compilation,
  dependencies_diagnostics_artifact: &mut DependenciesDiagnosticsArtifact,
  async_modules_artifact: &mut AsyncModulesArtifact,
  exports_info_artifact: &mut ExportsInfoArtifact,
) -> Result<Vec<Diagnostic>> {
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
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .finish_modules
    .call(compilation, async_modules_artifact, exports_info_artifact)
    .await?;

  apply_side_effects_state_artifact(compilation);

  // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2988
  compilation.module_graph_cache_artifact.freeze();
  // Collect dependencies diagnostics at here to make sure:
  // 1. after finish_modules: has provide exports info
  // 2. before optimize dependencies: side effects free module hasn't been skipped
  let mut all_diagnostics = collect_dependencies_diagnostics(
    compilation,
    dependencies_diagnostics_artifact,
    exports_info_artifact,
  );
  compilation.module_graph_cache_artifact.unfreeze();

  // take make diagnostics
  let diagnostics = compilation.build_module_graph_artifact.diagnostics();
  all_diagnostics.extend(diagnostics);
  Ok(all_diagnostics)
}

#[tracing::instrument("Compilation:collect_dependencies_diagnostics", skip_all)]
fn collect_dependencies_diagnostics(
  compilation: &Compilation,
  dependencies_diagnostics_artifact: &mut DependenciesDiagnosticsArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> Vec<Diagnostic> {
  let build_module_graph_artifact = &compilation.build_module_graph_artifact;
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
        let logger = compilation.get_logger("rspack.incremental.finishModules");
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
      let diagnostics = mgm
        .all_dependencies
        .iter()
        .filter_map(|dependency_id| {
          let dependency = module_graph.dependency_by_id(dependency_id);
          dependency
            .get_diagnostics(module_graph, module_graph_cache, exports_info_artifact)
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

fn apply_side_effects_state_artifact(compilation: &mut Compilation) {
  let side_effects_states: Vec<_> = {
    let side_effects_state_artifact = compilation
      .build_module_graph_artifact
      .side_effects_state_artifact
      .read()
      .expect("should lock side effects state artifact");
    side_effects_state_artifact
      .iter()
      .map(|(module_id, state)| (*module_id, state.clone()))
      .collect()
  };

  if side_effects_states.is_empty() {
    return;
  }

  let module_graph = compilation.get_module_graph_mut();
  for (module_id, state) in side_effects_states {
    if module_graph.module_by_identifier(&module_id).is_none() {
      continue;
    }

    let bailouts = module_graph.get_optimization_bailout_mut(&module_id);
    bailouts.retain(|item| {
      !state
        .optimization_bailouts_to_remove
        .iter()
        .any(|target| optimization_bailout_item_eq(item, target))
    });
    for item in state.optimization_bailouts_to_add {
      if bailouts
        .iter()
        .any(|existing| optimization_bailout_item_eq(existing, &item))
      {
        continue;
      }
      bailouts.push(item);
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
