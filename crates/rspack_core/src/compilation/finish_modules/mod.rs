use rspack_error::Result;

use super::*;
use crate::logger::Logger;

pub async fn finish_modules_pass(compilation: &mut Compilation) -> Result<()> {
  let mut dependencies_diagnostics_artifact = compilation.dependencies_diagnostics_artifact.steal();
  let mut async_modules_artifact = compilation.async_modules_artifact.steal();
  let diagnostics = compilation
    .finish_modules_inner(
      &mut dependencies_diagnostics_artifact,
      &mut async_modules_artifact,
    )
    .await;
  compilation.dependencies_diagnostics_artifact = dependencies_diagnostics_artifact.into();
  compilation.async_modules_artifact = async_modules_artifact.into();
  let diagnostics = diagnostics?;
  compilation.extend_diagnostics(diagnostics);

  Ok(())
}

impl Compilation {
  #[tracing::instrument("Compilation:finish_modules_inner", skip_all)]
  pub async fn finish_modules_inner(
    &mut self,
    dependencies_diagnostics_artifact: &mut DependenciesDiagnosticsArtifact,
    async_modules_artifact: &mut AsyncModulesArtifact,
  ) -> Result<Vec<Diagnostic>> {
    let logger = self.get_logger("rspack.Compilation");
    if let Some(mut mutations) = self.incremental.mutations_write() {
      mutations.extend(
        self
          .build_module_graph_artifact
          .affected_dependencies
          .updated()
          .iter()
          .map(|&dependency| Mutation::DependencyUpdate { dependency }),
      );
      mutations.extend(
        self
          .build_module_graph_artifact
          .affected_modules
          .removed()
          .iter()
          .map(|&module| Mutation::ModuleRemove { module }),
      );
      mutations.extend(
        self
          .build_module_graph_artifact
          .affected_modules
          .updated()
          .iter()
          .map(|&module| Mutation::ModuleUpdate { module }),
      );
      mutations.extend(
        self
          .build_module_graph_artifact
          .affected_modules
          .added()
          .iter()
          .map(|&module| Mutation::ModuleAdd { module }),
      );
      tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::BUILD_MODULE_GRAPH, %mutations);
    }

    let start = logger.time("finish modules");
    // finish_modules means the module graph (modules, connections, dependencies) are
    // frozen and start to optimize (provided exports, infer async, etc.) based on the
    // module graph, so any kind of change that affect these should be done before the
    // finish_modules
    self
      .plugin_driver
      .clone()
      .compilation_hooks
      .finish_modules
      .call(self, async_modules_artifact)
      .await?;

    logger.time_end(start);

    // https://github.com/webpack/webpack/blob/19ca74127f7668aaf60d59f4af8fcaee7924541a/lib/Compilation.js#L2988
    self.module_graph_cache_artifact.freeze();
    // Collect dependencies diagnostics at here to make sure:
    // 1. after finish_modules: has provide exports info
    // 2. before optimize dependencies: side effects free module hasn't been skipped
    let mut all_diagnostics =
      self.collect_dependencies_diagnostics(dependencies_diagnostics_artifact);
    self.module_graph_cache_artifact.unfreeze();

    // take make diagnostics
    let diagnostics = self.build_module_graph_artifact.diagnostics();
    all_diagnostics.extend(diagnostics);
    Ok(all_diagnostics)
  }

  #[tracing::instrument("Compilation:collect_dependencies_diagnostics", skip_all)]
  fn collect_dependencies_diagnostics(
    &self,
    dependencies_diagnostics_artifact: &mut DependenciesDiagnosticsArtifact,
  ) -> Vec<Diagnostic> {
    // Compute modules while holding the lock, then release it
    let (modules, has_mutations) = {
      let mutations = self
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
          let modules = mutations.get_affected_modules_with_module_graph(self.get_module_graph());
          let logger = self.get_logger("rspack.incremental.finishModules");
          logger.log(format!(
            "{} modules are affected, {} in total",
            modules.len(),
            self.get_module_graph().module_count()
          ));
          (modules, true)
        } else {
          (
            self
              .get_module_graph()
              .modules()
              .map(|(module_identifier, _)| module_identifier)
              .collect(),
            true,
          )
        }
      } else {
        (
          self
            .get_module_graph()
            .modules()
            .map(|(module_identifier, _)| module_identifier)
            .collect(),
          false,
        )
      }
    };

    let module_graph = self.get_module_graph();
    let module_graph_cache = &self.module_graph_cache_artifact;
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
              .get_diagnostics(module_graph, module_graph_cache)
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
}
