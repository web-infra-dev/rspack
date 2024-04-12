use std::sync::{atomic::AtomicU32, Arc};
use std::{collections::hash_map, hash::BuildHasherDefault, iter::once};

use rayon::prelude::*;
use rspack_error::Result;
use rspack_identifier::{Identifiable, IdentifierSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

use crate::cache::Cache;
use crate::{
  Chunk, ChunkGraph, ChunkKind, CodeGenerationDataAssetInfo, CodeGenerationDataFilename,
  CodeGenerationResult, CompilationAssets, Dependency, DependencyType, EntryDependency,
  EntryOptions, Entrypoint, ModuleFactory, ModuleGraphPartial, SourceType,
};
use crate::{Compilation, CompilationAsset, DependencyId, MakeParam};
use crate::{CompilerOptions, Context, ResolverFactory, SharedPluginDriver};

static EXECUTE_MODULE_ID: AtomicU32 = AtomicU32::new(0);
pub type ExecuteModuleId = u32;

#[derive(Debug, Default)]
pub struct ExecuteModuleResult {
  pub file_dependencies: HashSet<std::path::PathBuf>,
  pub context_dependencies: HashSet<std::path::PathBuf>,
  pub missing_dependencies: HashSet<std::path::PathBuf>,
  pub build_dependencies: HashSet<std::path::PathBuf>,
  pub assets: CompilationAssets,
  pub id: ExecuteModuleId,
}

#[derive(Debug)]
pub struct ModuleExecutor {
  pub make_module_graph: ModuleGraphPartial,
  request_dep_map: HashMap<String, DependencyId>,
}

impl ModuleExecutor {
  pub fn new(is_new_treeshaking: bool) -> Self {
    Self {
      make_module_graph: ModuleGraphPartial::new(is_new_treeshaking),
      request_dep_map: HashMap::default(),
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn import_module(
    &mut self,
    options: Arc<CompilerOptions>,
    plugin_driver: SharedPluginDriver,
    resolver_factory: Arc<ResolverFactory>,
    loader_resolver_factory: Arc<ResolverFactory>,
    cache: Arc<Cache>,
    dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,

    request: String,
    public_path: Option<String>,
    base_uri: Option<String>,
    original_module_context: Option<Context>,
  ) -> Result<ExecuteModuleResult> {
    let mut compilation = Compilation::new(
      options,
      plugin_driver,
      resolver_factory,
      loader_resolver_factory,
      None,
      cache,
      None,
    );
    compilation.dependency_factories = dependency_factories;
    compilation.swap_module_graph_with_module_executor(self);

    let mut mg = compilation.get_module_graph_mut();
    let dep_id = match self.request_dep_map.entry(request.clone()) {
      hash_map::Entry::Vacant(v) => {
        let dep = EntryDependency::new(
          request,
          original_module_context.unwrap_or(Context::from("")),
        );
        let dep_id = *dep.id();
        mg.add_dependency(Box::new(dep));
        v.insert(dep_id);
        dep_id
      }
      hash_map::Entry::Occupied(v) => *v.get(),
    };

    compilation
      .make(vec![MakeParam::new_force_build_dep_param(dep_id, None)])
      .await?;

    let id = EXECUTE_MODULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mg = compilation.get_module_graph_mut();
    let module = mg
      .get_module_by_dependency_id(&dep_id)
      .expect("should have module")
      .identifier();
    let mut queue = vec![module];
    let mut modules: std::collections::HashSet<
      rspack_identifier::Identifier,
      BuildHasherDefault<FxHasher>,
    > = HashSet::default();

    while let Some(m) = queue.pop() {
      modules.insert(m);
      for m in mg.get_outgoing_connections(&m) {
        // TODO: handle circle
        if !modules.contains(m.module_identifier()) {
          queue.push(*m.module_identifier());
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
        public_path: public_path.map(crate::PublicPath::String),
        base_uri,
        filename: None,
        library: None,
        depend_on: None,
      }),
    });

    // add chunk to this compilation
    let chunk = compilation.chunk_by_ukey.add(chunk);
    let chunk_ukey = chunk.ukey;

    chunk_graph.connect_chunk_and_entry_module(chunk.ukey, module, entrypoint.ukey);
    entrypoint.connect_chunk(chunk);
    entrypoint.set_runtime_chunk(chunk.ukey);
    entrypoint.set_entry_point_chunk(chunk.ukey);

    compilation.chunk_group_by_ukey.add(entrypoint);

    // Assign ids to modules and modules to the chunk
    let module_graph = compilation.get_module_graph();
    for m in &modules {
      let module = module_graph
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
    compilation.chunk_graph = chunk_graph;

    compilation.code_generation_modules(&mut None, false, modules.par_iter().copied())?;

    compilation
      .process_runtime_requirements(
        modules.clone(),
        once(chunk_ukey),
        once(chunk_ukey),
        compilation.plugin_driver.clone(),
      )
      .await?;

    let runtime_modules = compilation
      .chunk_graph
      .get_chunk_runtime_modules_iterable(&chunk_ukey)
      .copied()
      .collect::<IdentifierSet>();

    tracing::info!(
      "runtime modules: {:?}",
      &runtime_modules
        .iter()
        .map(|m| m.to_string())
        .collect::<Vec<_>>()
    );

    for runtime_id in &runtime_modules {
      let runtime_module = compilation
        .runtime_modules
        .get(runtime_id)
        .expect("runtime module exist");

      let result =
        CodeGenerationResult::default().with_javascript(runtime_module.generate(&compilation)?);
      let result_id = result.id;

      compilation
        .code_generation_results
        .module_generation_result_map
        .insert(result.id, result);
      compilation
        .code_generation_results
        .add(*runtime_id, runtime.clone(), result_id);
    }

    let codegen_results = compilation.code_generation_results.clone();
    let exports = compilation
      .plugin_driver
      .compilation_hooks
      .execute_module
      .call(&module, &runtime_modules, &codegen_results, &id);

    let module_graph = compilation.get_module_graph();
    let mut execute_result = match exports {
      Ok(_) => {
        let mut result = modules
          .iter()
          .fold(ExecuteModuleResult::default(), |mut res, m| {
            let module = module_graph.module_by_identifier(m).expect("unreachable");

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
            compilation.emit_asset(
              filename.to_owned(),
              CompilationAsset::new(Some(source.clone()), asset_info.inner().clone()),
            );
          }
        });

        Ok(result)
      }
      Err(e) => Err(e),
    };

    if let Ok(ref mut result) = execute_result {
      std::mem::swap(&mut result.assets, compilation.assets_mut());
    }

    let mut has_error = false;
    for error in compilation.get_errors() {
      has_error = true;
      error.render_report(true)?;
    }
    if !has_error {
      // save make module_graph for next import_module
      compilation.swap_module_graph_with_module_executor(self);
    }

    execute_result
  }
}
