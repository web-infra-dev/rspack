use super::*;
use crate::logger::Logger;

pub async fn code_generation_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("code generation");
  let code_generation_modules = if let Some(mutations) = compilation
    .incremental
    .mutations_read(IncrementalPasses::MODULES_CODEGEN)
    && !compilation.code_generation_results.is_empty()
  {
    let revoked_modules = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    for revoked_module in revoked_modules {
      compilation.code_generation_results.remove(&revoked_module);
    }
    let modules: IdentifierSet = mutations
      .iter()
      .filter_map(|mutation| match mutation {
        Mutation::ModuleSetHashes { module } => Some(*module),
        _ => None,
      })
      .collect();
    // also cleanup for updated modules, for `insert(); insert();` the second insert() won't override the first insert() on code_generation_results
    for module in &modules {
      compilation.code_generation_results.remove(module);
    }
    tracing::debug!(target: incremental::TRACING_TARGET, passes = %IncrementalPasses::MODULES_CODEGEN, %mutations);
    let logger = compilation.get_logger("rspack.incremental.modulesCodegen");
    logger.log(format!(
      "{} modules are affected, {} in total",
      modules.len(),
      compilation.get_module_graph().modules().len()
    ));
    modules
  } else {
    compilation.code_generation_results = Default::default();
    compilation
      .get_module_graph()
      .modules()
      .keys()
      .copied()
      .collect()
  };
  compilation.code_generation(code_generation_modules).await?;

  let mut diagnostics = vec![];
  plugin_driver
    .compilation_hooks
    .after_code_generation
    .call(compilation, &mut diagnostics)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterCodeGeneration"))?;
  compilation.extend_diagnostics(diagnostics);

  logger.time_end(start);
  Ok(())
}

impl Compilation {
  #[instrument("Compilation:code_generation",target=TRACING_BENCH_TARGET, skip_all)]
  async fn code_generation(&mut self, modules: IdentifierSet) -> Result<()> {
    let logger = self.get_logger("rspack.Compilation");
    let mut codegen_cache_counter = match self.options.cache {
      CacheOptions::Disabled => None,
      _ => Some(logger.cache("module code generation cache")),
    };

    let module_graph = self.get_module_graph();
    let mut no_codegen_dependencies_modules = IdentifierSet::default();
    let mut has_codegen_dependencies_modules = IdentifierSet::default();
    for module_identifier in modules {
      let module = module_graph
        .module_by_identifier(&module_identifier)
        .expect("should have module");
      if module.get_code_generation_dependencies().is_none() {
        no_codegen_dependencies_modules.insert(module_identifier);
      } else {
        has_codegen_dependencies_modules.insert(module_identifier);
      }
    }

    self
      .code_generation_modules(&mut codegen_cache_counter, no_codegen_dependencies_modules)
      .await?;
    self
      .code_generation_modules(&mut codegen_cache_counter, has_codegen_dependencies_modules)
      .await?;

    if let Some(counter) = codegen_cache_counter {
      logger.cache_end(counter);
    }

    Ok(())
  }

  pub(crate) async fn code_generation_modules(
    &mut self,
    cache_counter: &mut Option<CacheCount>,
    modules: IdentifierSet,
  ) -> Result<()> {
    let chunk_graph = &self.chunk_graph;
    let module_graph = self.get_module_graph();
    let mut jobs = Vec::new();
    for module in modules {
      let mut map: HashMap<RspackHashDigest, CodeGenerationJob> = HashMap::default();
      for runtime in chunk_graph.get_module_runtimes_iter(module, &self.chunk_by_ukey) {
        let hash = ChunkGraph::get_module_hash(self, module, runtime)
          .expect("should have cgm.hash in code generation");
        let scope = self
          .plugin_driver
          .compilation_hooks
          .concatenation_scope
          .call(self, module)
          .await?;
        if let Some(job) = map.get_mut(hash) {
          job.runtimes.push(runtime.clone());
        } else {
          map.insert(
            hash.clone(),
            CodeGenerationJob {
              module,
              hash: hash.clone(),
              runtime: runtime.clone(),
              runtimes: vec![runtime.clone()],
              scope,
            },
          );
        }
      }
      jobs.extend(map.into_values());
    }

    let results = rspack_futures::scope::<_, _>(|token| {
      jobs.into_iter().for_each(|job| {
        // SAFETY: await immediately and trust caller to poll future entirely
        let s = unsafe { token.used((&self, &module_graph, job)) };

        s.spawn(|(this, module_graph, job)| async {
          let options = &this.options;

          let module = module_graph
            .module_by_identifier(&job.module)
            .expect("should have module");
          let codegen_res = this
            .code_generate_cache_artifact
            .use_cache(&job, || async {
              module
                .code_generation(this, Some(&job.runtime), job.scope.clone())
                .await
                .map(|mut codegen_res| {
                  codegen_res.set_hash(
                    &options.output.hash_function,
                    &options.output.hash_digest,
                    &options.output.hash_salt,
                  );
                  codegen_res
                })
            })
            .await;

          (job.module, job.runtimes, codegen_res)
        })
      })
    })
    .await;
    let results = results
      .into_iter()
      .map(|res| res.to_rspack_result())
      .collect::<Result<Vec<_>>>()?;

    for (module, runtimes, (codegen_res, from_cache)) in results {
      if let Some(counter) = cache_counter {
        if from_cache {
          counter.hit();
        } else {
          counter.miss();
        }
      }
      let codegen_res = match codegen_res {
        Ok(codegen_res) => codegen_res,
        Err(err) => {
          let mut diagnostic = Diagnostic::from(err);
          diagnostic.module_identifier = Some(module);
          self.push_diagnostic(diagnostic);
          let mut codegen_res = CodeGenerationResult::default();
          codegen_res.set_hash(
            &self.options.output.hash_function,
            &self.options.output.hash_digest,
            &self.options.output.hash_salt,
          );
          codegen_res
        }
      };
      self
        .code_generation_results
        .insert(module, codegen_res, runtimes);
      self.code_generated_modules.insert(module);
    }
    Ok(())
  }
}
