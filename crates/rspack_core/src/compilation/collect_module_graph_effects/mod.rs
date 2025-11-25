pub mod artifact;
use std::mem;

use rayon::iter::{IntoParallelRefIterator as _, ParallelIterator};
use rspack_error::Result;

use crate::{
  Compilation, Logger as _,
  collect_module_graph_effects::artifact::{
    CollectModuleGraphEffectsArtifact, DependenciesDiagnostics,
  },
  incremental::{self, IncrementalPasses, Mutation},
};
pub async fn collect_build_module_graph_effects(compilation: &mut Compilation) -> Result<()> {
  let mut artifact = mem::take(&mut compilation.collect_build_module_graph_effects_artifact);
  collect_build_module_graph_effects_inner(compilation, &mut artifact).await?;
  compilation.diagnostics.extend(
    artifact
      .dependencies_diagnostics
      .clone()
      .into_values()
      .flatten(),
  );
  if let Some(mutations) = artifact.incremental.mutations_take() {
    for mutation in mutations {
      if let Some(mutations) = compilation.incremental.mutations_write() {
        mutations.add(mutation);
      }
    }
  }

  compilation.collect_build_module_graph_effects_artifact = artifact;
  Ok(())
}
// collect build module graph effects for incremental compilation
#[tracing::instrument("Compilation:collect_build_module_graph_effects", skip_all)]
pub async fn collect_build_module_graph_effects_inner(
  ctx: &mut Compilation,
  artifact: &mut CollectModuleGraphEffectsArtifact,
) -> Result<()> {
  let logger = ctx.get_logger("rspack.Compilation");
  if let Some(mutations) = artifact.incremental.mutations_write() {
    mutations.extend(
      ctx
        .build_module_graph_artifact
        .affected_dependencies
        .updated()
        .iter()
        .map(|&dependency| Mutation::DependencyUpdate { dependency }),
    );

    mutations.extend(
      ctx
        .build_module_graph_artifact
        .affected_modules
        .removed()
        .iter()
        .map(|&module| Mutation::ModuleRemove { module }),
    );
    mutations.extend(
      ctx
        .build_module_graph_artifact
        .affected_modules
        .updated()
        .iter()
        .map(|&module| Mutation::ModuleUpdate { module }),
    );
    mutations.extend(
      ctx
        .build_module_graph_artifact
        .affected_modules
        .added()
        .iter()
        .map(|&module| Mutation::ModuleAdd { module }),
    );
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MAKE, %mutations);
  }

  let start = logger.time("finish modules");
  // finish_modules means the module graph (modules, connections, dependencies) are
  // frozen and start to optimize (provided exports, infer async, etc.) based on the
  // module graph, so any kind of change that affect these should be done before the
  // finish_modules

  ctx
    .plugin_driver
    .clone()
    .compilation_hooks
    .finish_modules
    .call(ctx, artifact)
    .await?;

  logger.time_end(start);

  // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2988
  ctx.module_graph_cache_artifact.freeze();
  // Collect dependencies diagnostics at here to make sure:
  // 1. after finish_modules: has provide exports info
  // 2. before optimize dependencies: side effects free module hasn't been skipped
  collect_dependencies_diagnostics(ctx, artifact);
  ctx.module_graph_cache_artifact.unfreeze();
  Ok(())
}
#[tracing::instrument("Compilation:collect_dependencies_diagnostics", skip_all)]
fn collect_dependencies_diagnostics(
  ctx: &Compilation,
  artifact: &mut CollectModuleGraphEffectsArtifact,
) {
  let mutations = ctx
    .incremental
    .mutations_read(IncrementalPasses::DEPENDENCIES_DIAGNOSTICS);
  // TODO move diagnostic collect to make
  let modules = if let Some(mutations) = mutations
    && !artifact.dependencies_diagnostics.is_empty()
  {
    let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    for revoked_module in revoked_modules {
      artifact.dependencies_diagnostics.remove(&revoked_module);
    }
    let modules = mutations.get_affected_modules_with_module_graph(&ctx.get_module_graph());
    let logger = ctx.get_logger("rspack.incremental.dependenciesDiagnostics");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      ctx.get_module_graph().modules().len()
    ));
    modules
  } else {
    ctx.get_module_graph().modules().keys().copied().collect()
  };
  let module_graph = ctx.get_module_graph();
  let module_graph_cache = &ctx.module_graph_cache_artifact;
  let dependencies_diagnostics: DependenciesDiagnostics = modules
    .par_iter()
    .map(|module_identifier| {
      let mgm = module_graph
        .module_graph_module_by_identifier(module_identifier)
        .expect("should have mgm");
      let diagnostics = mgm
        .all_dependencies
        .iter()
        .filter_map(|dependency_id| module_graph.dependency_by_id(dependency_id))
        .filter_map(|dependency| {
          dependency
            .get_diagnostics(&module_graph, module_graph_cache)
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
    .collect();

  artifact
    .dependencies_diagnostics
    .extend(dependencies_diagnostics);
}
