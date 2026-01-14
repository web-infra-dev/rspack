use rspack_collections::IdentifierSet;
use rspack_error::{Diagnostic, Result, ToStringResultToRspackResultExt};
use rspack_hash::RspackHashDigest;
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  CacheCount, CacheOptions, ChunkGraph, CodeGenerationJob, CodeGenerationResult, Compilation,
  Logger,
};

#[instrument("Compilation:code_generation", target = TRACING_BENCH_TARGET, skip_all)]
pub async fn code_generation(compilation: &mut Compilation, modules: IdentifierSet) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let mut codegen_cache_counter = match compilation.options.cache {
    CacheOptions::Disabled => None,
    _ => Some(logger.cache("module code generation cache")),
  };

  let module_graph = compilation.get_module_graph();
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

  code_generation_modules(
    compilation,
    &mut codegen_cache_counter,
    no_codegen_dependencies_modules,
  )
  .await?;
  code_generation_modules(
    compilation,
    &mut codegen_cache_counter,
    has_codegen_dependencies_modules,
  )
  .await?;

  if let Some(counter) = codegen_cache_counter {
    logger.cache_end(counter);
  }

  Ok(())
}

pub(crate) async fn code_generation_modules(
  compilation: &mut Compilation,
  cache_counter: &mut Option<CacheCount>,
  modules: IdentifierSet,
) -> Result<()> {
  let chunk_graph = &compilation.chunk_graph;
  let module_graph = compilation.get_module_graph();
  let mut jobs = Vec::new();
  for module in modules {
    let mut map: HashMap<RspackHashDigest, CodeGenerationJob> = HashMap::default();
    for runtime in chunk_graph.get_module_runtimes_iter(module, &compilation.chunk_by_ukey) {
      let hash = ChunkGraph::get_module_hash(compilation, module, runtime)
        .expect("should have cgm.hash in code generation");
      let scope = compilation
        .plugin_driver
        .compilation_hooks
        .concatenation_scope
        .call(compilation, module)
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
      let s = unsafe { token.used((&compilation, &module_graph, job)) };

      s.spawn(|(this, module_graph, job)| async {
        let options = &this.options;
        let old_cache = &this.old_cache;

        let module = module_graph
          .module_by_identifier(&job.module)
          .expect("should have module");
        let codegen_res = old_cache
          .code_generate_occasion
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
        compilation.push_diagnostic(diagnostic);
        let mut codegen_res = CodeGenerationResult::default();
        codegen_res.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
        );
        codegen_res
      }
    };
    compilation
      .code_generation_results
      .insert(module, codegen_res, runtimes);
    compilation.code_generated_modules.insert(module);
  }
  Ok(())
}
