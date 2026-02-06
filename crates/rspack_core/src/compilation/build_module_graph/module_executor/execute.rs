use std::{collections::VecDeque, iter::once, sync::atomic::AtomicU32};

use itertools::Itertools;
use rspack_collections::{DatabaseItem, Identifier, IdentifierSet, UkeySet};
use rspack_error::Error;
use rspack_paths::ArcPathSet;
use rustc_hash::FxHashMap as HashMap;
use tokio::sync::oneshot::Sender;

use super::context::{ExecutorTaskContext, ImportModuleMeta};
use crate::{
  Chunk, ChunkGraph, ChunkKind, CodeGenerationDataAssetInfo, CodeGenerationDataFilename,
  CodeGenerationResult, CompilationAsset, CompilationAssets, EntryOptions, Entrypoint,
  FactorizeInfo, ModuleCodeGenerationContext, ModuleType, PublicPath, RuntimeSpec, SourceType,
  utils::task_loop::{Task, TaskResult, TaskType},
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
  pub error: Option<String>,
  pub cacheable: bool,
  pub file_dependencies: ArcPathSet,
  pub context_dependencies: ArcPathSet,
  pub missing_dependencies: ArcPathSet,
  pub build_dependencies: ArcPathSet,
  pub code_generated_modules: IdentifierSet,
  pub id: ExecuteModuleId,
}

pub(super) type ExecuteResultSender = Sender<(
  ExecuteModuleResult,
  CompilationAssets,
  IdentifierSet,
  Vec<ExecutedRuntimeModule>,
)>;

#[derive(Debug)]
pub(super) struct ExecuteTask {
  pub meta: ImportModuleMeta,
  pub public_path: Option<PublicPath>,
  pub base_uri: Option<String>,
  pub result_sender: ExecuteResultSender,
}

impl ExecuteTask {
  pub(super) fn finish_with_error(self, error: Error) {
    let id = EXECUTE_MODULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    self
      .result_sender
      .send((
        ExecuteModuleResult {
          id,
          error: Some(error.to_string()),
          ..Default::default()
        },
        Default::default(),
        Default::default(),
        Default::default(),
      ))
      .expect("should send result success");
  }
}

#[async_trait::async_trait]
impl Task<ExecutorTaskContext> for ExecuteTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(
    self: Box<Self>,
    context: &mut ExecutorTaskContext,
  ) -> TaskResult<ExecutorTaskContext> {
    let Self {
      meta,
      public_path,
      base_uri,
      result_sender,
    } = *self;

    let ExecutorTaskContext {
      origin_context,
      entries,
      ..
    } = context;

    let id = EXECUTE_MODULE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let mut execute_result = ExecuteModuleResult {
      cacheable: true,
      id,
      ..Default::default()
    };

    let entry_dep_id = entries.get(&meta).expect("should have dep_id");
    // collect module info
    let mg = origin_context.artifact.get_module_graph();
    let Some(entry_module_identifier) =
      mg.module_identifier_by_dependency_id(entry_dep_id).copied()
    else {
      // no entry module, entry dependency factorize failed.
      result_sender
        .send((
          execute_result,
          CompilationAssets::default(),
          IdentifierSet::default(),
          vec![],
        ))
        .expect("should send result success");

      return Ok(vec![]);
    };
    let make_failed_module = &origin_context.artifact.make_failed_module;
    let make_failed_dependencies = &origin_context.artifact.make_failed_dependencies;
    let mut queue = VecDeque::from(vec![entry_module_identifier]);
    let mut assets = CompilationAssets::default();
    let mut modules = IdentifierSet::default();
    let mut has_error = false;

    while let Some(m) = queue.pop_front() {
      // to avoid duplicate calculations in https://github.com/web-infra-dev/rspack/issues/10987
      if !modules.insert(m) {
        continue;
      }
      let module = mg.module_by_identifier(&m).expect("should have module");
      let build_info = module.build_info();
      execute_result
        .file_dependencies
        .extend(build_info.file_dependencies.iter().cloned());
      execute_result
        .context_dependencies
        .extend(build_info.context_dependencies.iter().cloned());
      execute_result
        .missing_dependencies
        .extend(build_info.missing_dependencies.iter().cloned());
      execute_result
        .build_dependencies
        .extend(build_info.build_dependencies.iter().cloned());
      if !build_info.cacheable {
        execute_result.cacheable = false;
      }
      for (name, asset) in build_info.assets.as_ref() {
        assets.insert(name.clone(), asset.clone());
      }
      if !has_error && make_failed_module.contains(&m) {
        let diagnostics = module.diagnostics();
        let errors: Vec<_> = diagnostics
          .iter()
          .filter(|d| d.is_error())
          .map(|d| d.message.clone())
          .collect();
        if !errors.is_empty() {
          has_error = true;
          if let Some(existing_error) = &mut execute_result.error {
            existing_error.push('\n');
            existing_error.push_str(&errors.join("\n"));
          } else {
            execute_result.error = Some(errors.join("\n"));
          }
        }
      }
      for dep_id in module.get_dependencies() {
        if !has_error && make_failed_dependencies.contains(dep_id) {
          let dep = mg.dependency_by_id(dep_id);
          let diagnostics = FactorizeInfo::get_from(dep)
            .expect("should have factorize info")
            .diagnostics();
          let errors: Vec<_> = diagnostics
            .iter()
            .filter(|d| d.is_error())
            .map(|d| d.message.clone())
            .collect();
          if !errors.is_empty() {
            has_error = true;
            if let Some(existing_error) = &mut execute_result.error {
              existing_error.push('\n');
              existing_error.push_str(&errors.join("\n"));
            } else {
              execute_result.error = Some(errors.join("\n"));
            }
          }
        }
        if let Some(c) = mg.connection_by_dependency_id(dep_id)
          && !modules.contains(c.module_identifier())
        {
          queue.push_back(*c.module_identifier());
        }
      }
    }

    if has_error {
      result_sender
        .send((
          execute_result,
          CompilationAssets::default(),
          IdentifierSet::default(),
          vec![],
        ))
        .expect("should send result success");

      return Ok(vec![]);
    }

    let mut compilation = origin_context.transform_to_temp_compilation();
    let main_compilation_plugin_driver = compilation.plugin_driver.clone();
    compilation.plugin_driver = compilation.buildtime_plugin_driver.clone();

    tracing::debug!("modules: {:?}", &modules);

    let mut chunk_graph = ChunkGraph::default();

    let mut chunk = Chunk::new(Some("build time chunk".into()), ChunkKind::Normal);

    if let Some(name) = chunk.name() {
      let name = name.to_string();
      chunk.set_id(name);
    }
    let runtime: RuntimeSpec = once("build time".into()).collect();

    chunk.set_runtime(runtime.clone());

    let mut entrypoint = Entrypoint::new(crate::ChunkGroupKind::Entrypoint {
      initial: true,
      options: Box::new(EntryOptions {
        name: Some("build time".into()),
        runtime: Some("runtime".into()),
        chunk_loading: Some(crate::ChunkLoading::Disable),
        wasm_loading: Some(crate::WasmLoading::Disable),
        async_chunks: Some(false),
        public_path,
        base_uri,
        filename: None,
        library: None,
        depend_on: None,
        layer: meta.layer,
      }),
    });

    // add chunk to this compilation
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .add(chunk);
    let chunk_ukey = chunk.ukey();

    chunk_graph.connect_chunk_and_entry_module(
      chunk.ukey(),
      entry_module_identifier,
      entrypoint.ukey,
    );
    entrypoint.connect_chunk(chunk);
    entrypoint.set_runtime_chunk(chunk.ukey());
    entrypoint.set_entrypoint_chunk(chunk.ukey());

    compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .add(entrypoint);

    // Assign ids to modules and modules to the chunk
    for &m in &modules {
      chunk_graph.add_module(m);
      ChunkGraph::set_module_id(&mut compilation.module_ids_artifact, m, m.as_str().into());
      chunk_graph.connect_chunk_and_module(chunk_ukey, m);
    }

    // Webpack uses this trick to make sure process_runtime_requirements access
    // the new chunk_graph
    // in rspack, if we decouple compilation and chunk_graph, we can't get exclusive ref
    // to the chunk_graph in API that receives both compilation and chunk_graph
    //
    // replace code_generation_results is the same reason
    compilation.build_chunk_graph_artifact.chunk_graph = chunk_graph;

    compilation.create_module_hashes(modules.clone()).await?;

    compilation
      .code_generation_modules(&mut None, modules.clone())
      .await?;
    compilation
      .process_modules_runtime_requirements(modules.clone(), compilation.plugin_driver.clone())
      .await?;
    compilation
      .process_chunks_runtime_requirements(
        UkeySet::from_iter([chunk_ukey]),
        UkeySet::from_iter([chunk_ukey]),
        compilation.plugin_driver.clone(),
      )
      .await?;
    let runtime_modules = compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_chunk_runtime_modules_iterable(&chunk_ukey)
      .copied()
      .collect::<IdentifierSet>();

    tracing::debug!(
      "runtime modules: {:?}",
      &runtime_modules.iter().collect::<Vec<_>>()
    );

    let mut runtime_module_size = HashMap::default();
    for runtime_id in &runtime_modules {
      let runtime_module = compilation
        .runtime_modules
        .get(runtime_id)
        .expect("runtime module exist");

      let mut runtime_template = compilation
        .runtime_template
        .create_module_codegen_runtime_template();
      let mut code_generation_context = ModuleCodeGenerationContext {
        compilation: &compilation,
        runtime: None,
        concatenation_scope: None,
        runtime_template: &mut runtime_template,
      };

      let result = runtime_module
        .code_generation(&mut code_generation_context)
        .await?;
      #[allow(clippy::unwrap_used)]
      let runtime_module_source = result.get(&SourceType::Runtime).unwrap();
      runtime_module_size.insert(
        runtime_module.identifier(),
        runtime_module_source.size() as f64,
      );
      let result = CodeGenerationResult::default().with_javascript(runtime_module_source.clone());

      compilation.code_generation_results.insert(
        *runtime_id,
        result,
        std::iter::once(runtime.clone()),
      );
      compilation
        .code_generated_modules
        .insert(runtime_module.identifier());
    }

    let exports = main_compilation_plugin_driver
      .compilation_hooks
      .execute_module
      .call(
        &entry_module_identifier,
        &runtime_modules,
        &compilation.code_generation_results,
        &id,
      )
      .await;

    match exports {
      Ok(_) => {
        for m in modules.iter() {
          let codegen_result = compilation.code_generation_results.get(m, Some(&runtime));

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
      }
      Err(e) => {
        execute_result.cacheable = false;
        if let Some(existing_error) = &mut execute_result.error {
          existing_error.push('\n');
          existing_error.push_str(&e.to_string());
        } else {
          execute_result.error = Some(e.to_string());
        }
      }
    };

    assets.extend(std::mem::take(compilation.assets_mut()));
    let code_generated_modules = std::mem::take(&mut compilation.code_generated_modules);
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
          name: runtime_module
            .readable_identifier(&compilation.options.context)
            .into(),
          name_for_condition: runtime_module.name_for_condition().map(|n| n.to_string()),
          module_type: *runtime_module.module_type(),
          cacheable: !(runtime_module.full_hash() || runtime_module.dependent_hash()),
          size: runtime_module_size
            .get(&identifier)
            .map_or(0 as f64, |s| s.to_owned()),
        }
      })
      .collect_vec();
    origin_context.recovery_from_temp_compilation(compilation);
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
