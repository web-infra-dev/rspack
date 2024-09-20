use std::path::PathBuf;
use std::{iter::once, sync::atomic::AtomicU32};

use itertools::Itertools;
use rspack_collections::{Identifier, IdentifierSet};
use rspack_error::Result;
use rustc_hash::FxHashMap as HashMap;
use rustc_hash::FxHashSet as HashSet;
use tokio::{runtime::Handle, sync::oneshot::Sender};

use crate::{
  compiler::make::repair::MakeTaskContext,
  utils::task_loop::{Task, TaskResult, TaskType},
  Chunk, ChunkGraph, ChunkKind, CodeGenerationDataAssetInfo, CodeGenerationDataFilename,
  CodeGenerationResult, CompilationAsset, CompilationAssets, DependencyId, EntryOptions,
  Entrypoint, ModuleType, PublicPath, RuntimeSpec, SourceType,
};

#[derive(Debug, Clone)]
pub struct ExecutedRuntimeModule {
  pub identifier: Identifier,
  pub name: String,
  pub name_for_condition: Option<String>,
  pub module_type: ModuleType,
  pub size: f64,
  pub cacheable: bool,
}

static EXECUTE_MODULE_ID: AtomicU32 = AtomicU32::new(0);
pub type ExecuteModuleId = u32;

#[derive(Debug, Default)]
pub struct ExecuteModuleResult {
  pub cacheable: bool,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub code_generated_modules: IdentifierSet,
  pub assets: HashSet<String>,
  pub id: ExecuteModuleId,
}

#[derive(Debug)]
pub struct ExecuteTask {
  pub entry_dep_id: DependencyId,
  pub layer: Option<String>,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub result_sender: Sender<(
    Result<ExecuteModuleResult>,
    CompilationAssets,
    IdentifierSet,
    Vec<ExecutedRuntimeModule>,
  )>,
}

impl Task<MakeTaskContext> for ExecuteTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self {
      entry_dep_id,
      layer,
      public_path,
      base_uri,
      result_sender,
    } = *self;

    let mut compilation = context.transform_to_temp_compilation();

    let id = EXECUTE_MODULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let mg = compilation.get_module_graph_mut();
    let entry_module_identifier = mg
      .get_module_by_dependency_id(&entry_dep_id)
      .expect("should have module")
      .identifier();
    let mut queue = vec![entry_module_identifier];
    let mut modules = IdentifierSet::default();

    while let Some(m) = queue.pop() {
      modules.insert(m);
      for m in mg.get_outgoing_connections(&m) {
        // TODO: handle circle
        if !modules.contains(m.module_identifier()) {
          queue.push(*m.module_identifier());
        }
      }
    }

    tracing::info!("modules: {:?}", &modules.iter().collect::<Vec<_>>());

    let mut chunk_graph = ChunkGraph::default();

    let mut chunk = Chunk::new(Some("build time chunk".into()), ChunkKind::Normal);

    chunk.id = chunk.name.clone();
    chunk.ids = vec![chunk.id.clone().expect("id is set")];
    let runtime: RuntimeSpec = once("build time".into()).collect();

    chunk.runtime = runtime.clone();

    let mut entrypoint = Entrypoint::new(crate::ChunkGroupKind::Entrypoint {
      initial: true,
      options: Box::new(EntryOptions {
        name: Some("build time".into()),
        runtime: Some("runtime".into()),
        chunk_loading: Some(crate::ChunkLoading::Disable),
        async_chunks: None,
        public_path,
        base_uri,
        filename: None,
        library: None,
        depend_on: None,
        layer,
      }),
    });

    // add chunk to this compilation
    let chunk = compilation.chunk_by_ukey.add(chunk);
    let chunk_ukey = chunk.ukey;

    chunk_graph.connect_chunk_and_entry_module(
      chunk.ukey,
      entry_module_identifier,
      entrypoint.ukey,
    );
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

    compilation.create_module_hashes(modules.clone())?;

    compilation.code_generation_modules(&mut None, modules.clone())?;

    Handle::current().block_on(async {
      compilation
        .process_runtime_requirements(
          modules.clone(),
          once(chunk_ukey),
          once(chunk_ukey),
          compilation.plugin_driver.clone(),
        )
        .await
    })?;

    let runtime_modules = compilation
      .chunk_graph
      .get_chunk_runtime_modules_iterable(&chunk_ukey)
      .copied()
      .collect::<IdentifierSet>();

    tracing::info!(
      "runtime modules: {:?}",
      &runtime_modules.iter().collect::<Vec<_>>()
    );

    let mut runtime_module_size = HashMap::default();
    for runtime_id in &runtime_modules {
      let runtime_module = compilation
        .runtime_modules
        .get(runtime_id)
        .expect("runtime module exist");

      let runtime_module_source = runtime_module.generate(&compilation)?;
      runtime_module_size.insert(
        runtime_module.identifier(),
        runtime_module_source.size() as f64,
      );
      let result = CodeGenerationResult::default().with_javascript(runtime_module_source);
      let result_id = result.id;

      compilation
        .code_generation_results
        .module_generation_result_map
        .insert(result.id, result);
      compilation
        .code_generation_results
        .add(*runtime_id, runtime.clone(), result_id);
      compilation
        .code_generated_modules
        .insert(runtime_module.identifier());
    }

    let codegen_results = compilation.code_generation_results.clone();
    let exports = compilation
      .plugin_driver
      .compilation_hooks
      .execute_module
      .call(
        &entry_module_identifier,
        &runtime_modules,
        &codegen_results,
        &id,
      );

    let module_graph = compilation.get_module_graph();
    let mut execute_result = match exports {
      Ok(_) => {
        let mut result = modules.iter().fold(
          ExecuteModuleResult {
            cacheable: true,
            ..Default::default()
          },
          |mut res, m| {
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
              if !info.cacheable {
                res.cacheable = false;
              }
            }
            res
          },
        );

        result.id = id;

        for m in modules.iter() {
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
        }

        Ok(result)
      }
      Err(e) => Err(e),
    };

    let assets = std::mem::take(compilation.assets_mut());
    let code_generated_modules = std::mem::take(&mut compilation.code_generated_modules);
    let module_assets = std::mem::take(&mut compilation.module_assets);
    if let Ok(ref mut result) = execute_result {
      result.assets = assets.keys().cloned().collect::<HashSet<_>>();
      result.assets.extend(
        module_assets
          .values()
          .flat_map(|m| m.iter().map(|i| i.to_owned()).collect_vec())
          .collect::<HashSet<String>>(),
      );
    }
    let executed_runtime_modules = runtime_modules
      .iter()
      .map(|runtime_id| {
        let runtime_module = compilation
          .runtime_modules
          .get(runtime_id)
          .expect("runtime module exist");
        let identifier = runtime_module.identifier();
        ExecutedRuntimeModule {
          identifier,
          name: runtime_module.name().to_string(),
          name_for_condition: runtime_module.name_for_condition().map(|n| n.to_string()),
          module_type: *runtime_module.module_type(),
          cacheable: runtime_module.cacheable(),
          size: runtime_module_size
            .get(&identifier)
            .map_or(0 as f64, |s| s.to_owned()),
        }
      })
      .collect_vec();
    context.recovery_from_temp_compilation(compilation);
    result_sender
      .send((
        execute_result,
        assets,
        code_generated_modules,
        executed_runtime_modules,
      ))
      .expect("should send result success");
    Ok(vec![])
  }
}
