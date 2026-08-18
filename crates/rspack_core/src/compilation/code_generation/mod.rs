use std::future::Future;

use async_trait::async_trait;

use super::*;
use crate::{
  CacheValue, Etag, ModuleCodeGenerationContext, MultiItemCache, compilation::pass::PassExt,
  get_runtime_key, logger::Logger,
};

const CODE_GENERATION_CACHE_NAME: &str = "Compilation/codeGeneration";

pub struct CodeGenerationPass;

#[async_trait]
impl PassExt for CodeGenerationPass {
  fn name(&self) -> &'static str {
    "code generation"
  }

  fn incremental_passes(&self) -> IncrementalPasses {
    IncrementalPasses::MODULES_CODEGEN
  }

  async fn run_pass(&self, compilation: &mut Compilation) -> Result<()> {
    code_generation_pass_impl(compilation).await
  }
}

async fn code_generation_pass_impl(compilation: &mut Compilation) -> Result<()> {
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
      compilation.get_module_graph().modules_len()
    ));
    modules
  } else {
    *compilation.code_generation_results = Default::default();
    compilation
      .get_module_graph()
      .modules_keys()
      .copied()
      .collect()
  };
  code_generation(compilation, code_generation_modules).await?;

  let mut diagnostics = vec![];
  compilation
    .plugin_driver
    .clone()
    .compilation_hooks
    .after_code_generation
    .call(compilation, &mut diagnostics)
    .await
    .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.afterCodeGeneration"))?;
  compilation.extend_diagnostics(diagnostics);

  Ok(())
}

#[instrument("Compilation:code_generation",target=TRACING_BENCH_TARGET, skip_all)]
pub async fn code_generation(compilation: &mut Compilation, modules: IdentifierSet) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let codegen_cache_counter = match compilation.options.cache {
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
    codegen_cache_counter.as_ref(),
    no_codegen_dependencies_modules,
  )
  .await?;
  code_generation_modules(
    compilation,
    codegen_cache_counter.as_ref(),
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
  cache_counter: Option<&CacheCount>,
  modules: IdentifierSet,
) -> Result<()> {
  let new_cache = compilation
    .options
    .experiments
    .new_cache
    .then(|| compilation.get_cache(CODE_GENERATION_CACHE_NAME));
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let module_graph = compilation.get_module_graph();
  let mut jobs = Vec::new();
  for module in modules {
    let mut map: HashMap<RspackHashDigest, CodeGenerationJob> = HashMap::default();
    for runtime in chunk_graph.get_module_runtimes_iter(
      module,
      &compilation.build_chunk_graph_artifact.chunk_by_ukey,
    ) {
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

  let compilation_ref = &*compilation;
  let results = rspack_parallel::scope::<_, _>(|token| {
    jobs.into_iter().for_each(|job| {
      // SAFETY: await immediately and trust caller to poll future entirely
      let s = unsafe {
        token.used((
          compilation_ref,
          &module_graph,
          cache_counter,
          new_cache.as_ref(),
          job,
        ))
      };

      s.spawn(
        |(this, module_graph, cache_counter, new_cache, mut job)| async move {
          let options = &this.options;

          let module = module_graph
            .module_by_identifier(&job.module)
            .expect("should have module");
          let new_code_generation_cache = new_cache.map(|cache| {
            let etag = Etag::from(job.hash.encoded());
            MultiItemCache::new(job.runtimes.iter().map(|runtime| {
              cache.get_item_cache(
                &format!("{}|{}", job.module, get_runtime_key(runtime)),
                Some(etag.clone()),
              )
            }))
          });
          let mut concatenation_scope = job.scope.take();
          let generator = async {
            let mut runtime_template = this.runtime_template.create_module_code_template();
            let mut code_generation_context = ModuleCodeGenerationContext {
              compilation: this,
              runtime: Some(&job.runtime),
              concatenation_scope: concatenation_scope.as_mut(),
              concatenation_source: None,
              runtime_template: &mut runtime_template,
            };

            module
              .code_generation(&mut code_generation_context)
              .await
              .map(|mut codegen_result_builder| {
                codegen_result_builder
                  .runtime_requirements_mut()
                  .extend(*runtime_template.runtime_requirements());
                codegen_result_builder.set_hash(
                  &options.output.hash_function,
                  &options.output.hash_digest,
                  &options.output.hash_salt,
                  module
                    .as_concatenated_module()
                    .is_some()
                    .then_some(&job.hash),
                );
                codegen_result_builder.build()
              })
          };
          let (codegen_res, from_cache) = if let Some(new_cache) = new_code_generation_cache {
            use_new_cache(&new_cache, generator).await
          } else {
            this
              .code_generate_cache_artifact
              .use_cache(&job, generator)
              .await
          };
          if let Some(counter) = cache_counter {
            if from_cache {
              counter.hit();
            } else {
              counter.miss();
            }
          }

          (job.module, job.runtimes, codegen_res.map(Box::new))
        },
      )
    })
  })
  .await;
  let results = results
    .into_iter()
    .map(|res| res.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

  for (module, runtimes, codegen_res) in results {
    let codegen_res = match codegen_res {
      Ok(codegen_res) => *codegen_res,
      Err(err) => {
        let mut diagnostic = Diagnostic::from(err);
        diagnostic.module_identifier = Some(module);
        compilation.push_diagnostic(diagnostic);
        let mut codegen_result_builder = CodeGenerationResultBuilder::default();
        codegen_result_builder.set_hash(
          &compilation.options.output.hash_function,
          &compilation.options.output.hash_digest,
          &compilation.options.output.hash_salt,
          None,
        );
        codegen_result_builder.build()
      }
    };
    compilation
      .code_generation_results
      .insert(module, codegen_res, runtimes);
    compilation.code_generated_modules.insert(module);
  }
  Ok(())
}

async fn use_new_cache<F>(
  cache: &MultiItemCache,
  generator: F,
) -> (Result<CodeGenerationResult>, bool)
where
  F: Future<Output = Result<CodeGenerationResult>>,
{
  match cache.get::<CodeGenerationResult>() {
    Ok(Some(cached)) => {
      let result = cached.as_arc().as_ref().clone();
      return (Ok(result), true);
    }
    Ok(None) => {}
    Err(error) => return (Err(error), false),
  }

  match generator.await {
    Ok(result) => {
      if let Err(error) = cache.store(CacheValue::new(result.clone())) {
        return (Err(error), false);
      }
      (Ok(result), false)
    }
    Err(error) => (Err(error), false),
  }
}
