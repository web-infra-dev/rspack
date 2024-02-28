use std::sync::atomic::AtomicU32;
use std::{hash::BuildHasherDefault, iter::once, sync::Arc};

use rayon::prelude::*;
use rspack_error::Result;
use rspack_identifier::{Identifiable, IdentifierMap};
use rustc_hash::{FxHashSet as HashSet, FxHasher};
use tokio::runtime::Handle;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};
use tracing::instrument;

use crate::{
  cache::Cache, BuildTimeExecutionOption, BuildTimeExecutionTask, Chunk, ChunkByUkey, ChunkGraph,
  ChunkKind, CodeGenerationDataAssetInfo, CodeGenerationDataFilename, CodeGenerationResult,
  CompilerOptions, Dependency, EntryOptions, Entrypoint, ExecuteModuleResult, FactorizeTask,
  LoaderImportDependency, ModuleIdentifier, NormalModuleFactory, ResolverFactory,
  SharedPluginDriver, SourceType,
};
use crate::{
  BuildTimeExecutionQueueHandler, Compilation, CompilationAsset, FactorizeQueueHandler,
  ProcessDependenciesQueueHandler,
};
use crate::{Context, RuntimeModule};

static EXECUTE_MODULE_ID: AtomicU32 = AtomicU32::new(0);

impl Compilation {
  #[allow(clippy::unwrap_in_result)]
  #[instrument(name = "compilation:execute_module")]
  pub fn execute_module(
    &mut self,
    module: ModuleIdentifier,
    request: &str,
    options: BuildTimeExecutionOption,
    result_tx: UnboundedSender<Result<ExecuteModuleResult>>,
  ) -> Result<()> {
    let id = EXECUTE_MODULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mut modules: std::collections::HashSet<
      rspack_identifier::Identifier,
      BuildHasherDefault<FxHasher>,
    > = HashSet::default();
    let mut queue = vec![module];

    while let Some(m) = queue.pop() {
      modules.insert(m);
      let m = self
        .module_graph
        .module_by_identifier(&m)
        .unwrap_or_else(|| panic!("should have module: {m}"));
      for m in self.module_graph.get_outgoing_connections(m) {
        // TODO: handle circle
        if !modules.contains(&m.module_identifier) {
          queue.push(m.module_identifier);
        }
      }
    }

    tracing::info!(
      "modules: {:?}",
      &modules.iter().map(|m| m.to_string()).collect::<Vec<_>>()
    );

    let mut chunk_graph = ChunkGraph::default();

    let mut chunk = Chunk::new(Some("build time chunk".into()), ChunkKind::Normal);

    chunk.id = chunk.name.clone();
    chunk.ids = vec![chunk.id.clone().expect("id is set")];
    let runtime = {
      let mut runtime = HashSet::default();
      runtime.insert("build time".into());
      runtime
    };

    chunk.runtime = runtime.clone();

    let mut entrypoint = Entrypoint::new(crate::ChunkGroupKind::Entrypoint {
      initial: true,
      options: Box::new(EntryOptions {
        name: Some("build time".into()),
        runtime: Some("runtime".into()),
        chunk_loading: Some(crate::ChunkLoading::Disable),
        async_chunks: None,
        public_path: options.public_path.clone().map(crate::PublicPath::String),
        base_uri: options.base_uri.clone(),
        filename: None,
        library: None,
      }),
    });

    // add chunk to this compilation
    let chunk_by_ukey = ChunkByUkey::default();
    let old_chunk_by_ukey = std::mem::replace(&mut self.chunk_by_ukey, chunk_by_ukey);

    let chunk = self.chunk_by_ukey.add(chunk);
    let chunk_ukey = chunk.ukey;

    chunk_graph.connect_chunk_and_entry_module(chunk.ukey, module, entrypoint.ukey);
    entrypoint.connect_chunk(chunk);
    entrypoint.set_runtime_chunk(chunk.ukey);
    entrypoint.set_entry_point_chunk(chunk.ukey);

    let entry_ukey = entrypoint.ukey;
    self.chunk_group_by_ukey.add(entrypoint);

    // Assign ids to modules and modules to the chunk
    for m in &modules {
      let module = self
        .module_graph
        .module_by_identifier(m)
        .expect("should have module");

      let id = module.identifier();

      chunk_graph.add_module(id);
      chunk_graph.set_module_id(*m, id.to_string());
      chunk_graph.connect_chunk_and_module(chunk_ukey, *m);
    }

    // Webpack uses this trick to make sure process_runtime_requirements access
    // the new chunk_graph
    // in rspack, if we decouple compilation and chunk_graph, we can't get exclusive ref
    // to the chunk_graph in API that receives both compilation and chunk_graph
    //
    // replace code_generation_results is the same reason
    let old_runtime_modules = std::mem::take(&mut self.runtime_modules);
    let old_chunk_graph = std::mem::replace(&mut self.chunk_graph, chunk_graph);

    self.code_generation_modules(&mut None, false, modules.par_iter().copied())?;

    Handle::current().block_on(async {
      self
        .process_runtime_requirements(
          modules.clone(),
          once(chunk_ukey),
          once(chunk_ukey),
          self.plugin_driver.clone(),
        )
        .await
    })?;

    let runtime_modules = self
      .chunk_graph
      .get_chunk_runtime_modules_iterable(&chunk_ukey)
      .copied()
      .collect::<HashSet<ModuleIdentifier>>();

    tracing::info!(
      "runtime modules: {:?}",
      &runtime_modules
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
    );

    for runtime_id in &runtime_modules {
      let runtime_module = self
        .runtime_modules
        .get(runtime_id)
        .expect("runtime module exist");

      let result = CodeGenerationResult::default().with_javascript(runtime_module.generate(self));
      let result_id = result.id;

      self
        .code_generation_results
        .module_generation_result_map
        .insert(result.id, result);
      self
        .code_generation_results
        .add(*runtime_id, runtime.clone(), result_id);
    }

    let codegen_results = self.code_generation_results.clone();
    let exports = self.plugin_driver.execute_module(
      module,
      request,
      &options,
      runtime_modules.iter().cloned().collect(),
      &codegen_results,
      id,
    );

    let execute_result = match exports {
      Ok(_) => {
        let mut result = modules
          .iter()
          .fold(ExecuteModuleResult::default(), |mut res, m| {
            let module = self
              .module_graph
              .module_by_identifier(m)
              .expect("unreachable");

            let build_info = &module.build_info();
            if let Some(info) = build_info {
              res
                .file_dependencies
                .extend(info.file_dependencies.iter().cloned());
              res
                .context_dependencies
                .extend(info.context_dependencies.iter().cloned());
              res
                .missing_dependencies
                .extend(info.missing_dependencies.iter().cloned());
              res
                .build_dependencies
                .extend(info.build_dependencies.iter().cloned());
              res.assets.extend(info.asset_filenames.iter().cloned());
            }
            res
          });

        result.id = id;

        modules.iter().for_each(|m| {
          let codegen_result = codegen_results.get(m, Some(&runtime));

          if let Some(source) = codegen_result.get(&SourceType::Asset)
            && let Some(filename) = codegen_result.data.get::<CodeGenerationDataFilename>()
            && let Some(asset_info) = codegen_result.data.get::<CodeGenerationDataAssetInfo>()
          {
            let filename = filename.filename();
            self.emit_asset(
              filename.to_owned(),
              CompilationAsset::new(Some(source.clone()), asset_info.inner().clone()),
            );
            result.assets.insert(filename.to_owned());
          }
        });

        Ok(result)
      }
      Err(e) => Err(e),
    };

    // clear side effects stuff we caused
    self.clear_execute_module_effects(old_chunk_graph, old_chunk_by_ukey, old_runtime_modules);

    self.chunk_group_by_ukey.remove(&entry_ukey);

    result_tx.send(execute_result).expect("todo");
    Ok(())
  }

  #[instrument(name = "compilation::import_module")]
  pub async fn import_module(
    &self,
    request: String,
    public_path: Option<String>,
    base_uri: Option<String>,
    original_module_identifier: Option<ModuleIdentifier>,
    original_module_context: Option<Box<Context>>,
  ) -> Result<ExecuteModuleResult> {
    Self::import_module_impl(
      self
        .factorize_queue
        .clone()
        .expect("call import module without queueHandler"),
      self
        .process_dependencies_queue
        .clone()
        .expect("call import module without queueHandler"),
      self
        .build_time_execution_queue
        .clone()
        .expect("call import module without queueHandler"),
      self.resolver_factory.clone(),
      self.options.clone(),
      self.plugin_driver.clone(),
      self.cache.clone(),
      request,
      public_path,
      base_uri,
      original_module_identifier,
      original_module_context,
    )
    .await
  }

  #[allow(clippy::too_many_arguments)]
  #[instrument(name = "compilation::import_module_impl")]
  pub async fn import_module_impl(
    factorize_queue: FactorizeQueueHandler,
    process_dependencies_queue: ProcessDependenciesQueueHandler,
    build_time_execution_queue: BuildTimeExecutionQueueHandler,
    resolver_factory: Arc<ResolverFactory>,
    options: Arc<CompilerOptions>,
    plugin_driver: SharedPluginDriver,
    cache: Arc<Cache>,

    request: String,
    public_path: Option<String>,
    base_uri: Option<String>,
    original_module_identifier: Option<ModuleIdentifier>,
    original_module_context: Option<Box<Context>>,
  ) -> Result<ExecuteModuleResult> {
    let (tx, mut rx) = unbounded_channel::<(ModuleIdentifier, Option<Vec<ModuleIdentifier>>)>();

    let dep = LoaderImportDependency::new(request.clone());
    let dep_id = *dep.id();

    let tx_clone = tx.clone();

    factorize_queue.add_task(FactorizeTask {
      module_factory: Arc::new(NormalModuleFactory::new(
        options.clone(),
        resolver_factory.clone(),
        plugin_driver.clone(),
        cache.clone(),
      )),
      original_module_source: None,
      original_module_identifier,
      original_module_context,
      issuer: None,
      dependency: Box::new(dep),
      dependencies: vec![dep_id],
      is_entry: false,
      resolve_options: Some(Box::new(options.resolve.clone())),
      resolver_factory: resolver_factory.clone(),
      loader_resolver_factory: resolver_factory.clone(),
      options: options.clone(),
      lazy_visit_modules: Default::default(),
      plugin_driver: plugin_driver.clone(),
      cache: cache.clone(),
      current_profile: None,
      connect_origin: false,
      callback: Some(Box::new(move |m| {
        tx_clone
          .send((m.identifier(), None))
          .expect("failed to send entry module of buildtime execution modules")
      })),
    });

    let mut to_be_completed = 0;
    let mut completed = 0;
    let mut waited = vec![];
    let mut modules = HashSet::default();

    let mut entry = None;

    // wait for every child to be built
    loop {
      if let Some((module, deps)) = rx.recv().await {
        if let Some(deps) = deps {
          completed += 1;
          modules.insert(module);

          for dep in deps {
            if !modules.contains(&dep) {
              waited.push(dep);
              to_be_completed += 1;
            }
          }
        } else {
          entry = Some(module);
          waited.push(module);
          to_be_completed += 1;
        }

        while let Some(module) = waited.pop() {
          let tx_clone = tx.clone();

          process_dependencies_queue.wait_for(
            module,
            Box::new(move |module, compilation| {
              let m = compilation
                .module_graph
                .module_by_identifier(&module)
                .expect("todo");

              let deps = compilation
                .module_graph
                .get_outgoing_connections(m)
                .into_iter()
                .map(|conn| conn.module_identifier)
                .filter(|m| compilation.module_graph.module_by_identifier(m).is_none())
                .collect();

              tx_clone.send((module, Some(deps))).expect("todo");
            }),
          );
        }
      } else {
        unreachable!();
      }

      if to_be_completed == completed {
        break;
      }
    }

    let (tx, mut rx) = unbounded_channel();

    build_time_execution_queue.add_task(BuildTimeExecutionTask {
      module: entry.expect("should has entry module"),
      request,
      options: BuildTimeExecutionOption {
        public_path,
        base_uri,
      },
      sender: tx,
    });

    rx.recv().await.unwrap_or_else(|| {
      Err(
        rspack_error::InternalError::new(
          "failed on calling importModule at execution stage".into(),
          rspack_error::RspackSeverity::Error,
        )
        .into(),
      )
    })
  }

  // modules that execute during build are called build-time modules.
  // executeModule will invoke some modules' codegen and process their runtime,
  // however during this process, some plugins might access compilation.chunk_graph
  // or compilation.runtime_modules, and modify them, for example add runtimeModule
  // to compilation.runtime_modules, but those runtime modules are only needed by
  // build-time modules, and should not present in the output, thus we should
  // clear them after we finish executing build-time modules
  // compare to webpack, we have extra items to restore: runtime_modules, because webpack
  // store runtimeModules in ChunkGraph, but we store them in compilation
  fn clear_execute_module_effects(
    &mut self,
    chunk_graph: ChunkGraph,
    chunk_by_ukey: ChunkByUkey,
    runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  ) {
    self.chunk_graph = chunk_graph;
    self.chunk_by_ukey = chunk_by_ukey;
    self.runtime_modules = runtime_modules;
  }
}
