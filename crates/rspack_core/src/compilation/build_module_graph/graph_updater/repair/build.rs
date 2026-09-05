use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
use rspack_util::time::current_time;
use rustc_hash::FxHashSet;

use super::{
  TaskContext, lazy::process_unlazy_dependencies, process_dependencies::ProcessDependenciesTask,
};
use crate::{
  BoxModule, BuildContext, BuildResult, CacheFacade, CompilationId, CompilerId, CompilerOptions,
  FileSystemInfo, ModuleCodeTemplate, ResolverFactory, SharedPluginDriver,
  compilation::build_module_graph::{
    ForwardedIdSet, HasLazyDependencies, module_build_cache::ModuleBuildCache,
  },
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct BuildTask {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub module: BoxModule,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub loader_cache: CacheFacade,
  pub file_system_info: FileSystemInfo,
  pub runtime_template: ModuleCodeTemplate,
  pub plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub forwarded_ids: ForwardedIdSet,
  pub module_build_cache: Option<ModuleBuildCache>,
}

#[async_trait::async_trait]
impl Task<TaskContext> for BuildTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }
  async fn background_run(self: Box<Self>) -> TaskResult<TaskContext> {
    let Self {
      compiler_id,
      compilation_id,
      compiler_options,
      loader_cache,
      file_system_info,
      resolver_factory,
      plugin_driver,
      runtime_template,
      mut module,
      fs,
      forwarded_ids,
      module_build_cache,
    } = *self;

    let build_start_time = module_build_cache.as_ref().map(|_| current_time());

    plugin_driver
      .compilation_hooks
      .build_module
      .call(compiler_id, compilation_id, &mut module)
      .await?;

    let result = module
      .build(
        BuildContext {
          compiler_id,
          compilation_id,
          compiler_options: compiler_options.clone(),
          loader_cache,
          file_system_info: file_system_info.clone(),
          resolver_factory: resolver_factory.clone(),
          plugin_driver: plugin_driver.clone(),
          runtime_template,
          fs: fs.clone(),
        },
        None,
      )
      .await?;

    Ok(vec![Box::new(BuildResultTask {
      build_result: Box::new(result),
      plugin_driver,
      forwarded_ids,
      origin: BuildOrigin::Built(build_start_time),
    })])
  }
}

#[derive(Debug)]
pub(super) enum BuildOrigin {
  Built(Option<u64>),
  CacheHit,
}

#[derive(Debug)]
pub(super) struct BuildResultTask {
  pub build_result: Box<BuildResult>,
  pub origin: BuildOrigin,
  pub plugin_driver: SharedPluginDriver,
  pub forwarded_ids: ForwardedIdSet,
}

#[async_trait::async_trait]
impl Task<TaskContext> for BuildResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let BuildResultTask {
      build_result,
      plugin_driver,
      mut forwarded_ids,
      origin,
    } = *self;
    let mut module = build_result.module;

    plugin_driver
      .compilation_hooks
      .succeed_module
      .call(context.compiler_id, context.compilation_id, &mut module)
      .await?;

    let (module_identifier, mut all_dependencies, lazy_dependencies) =
      context.artifact.apply_build_result(BuildResult {
        module,
        dependencies: build_result.dependencies,
        blocks: build_result.blocks,
        optimization_bailouts: build_result.optimization_bailouts,
      });
    if let BuildOrigin::Built(Some(started_at)) = origin {
      context
        .make_session
        .record_build(module_identifier, started_at);
    }
    let module_graph = &mut context.artifact.module_graph;

    let mut tasks: Vec<Box<dyn Task<TaskContext>>> = vec![];

    let dependencies_to_process = if !lazy_dependencies.is_empty() {
      let lazy_dependency_ids = lazy_dependencies
        .all_lazy_dependencies()
        .collect::<FxHashSet<_>>();
      all_dependencies.retain(|dep| !lazy_dependency_ids.contains(dep));

      if let Some(HasLazyDependencies::Pending(pending_forwarded_ids)) = context
        .artifact
        .module_to_lazy_make
        .update_module_lazy_dependencies(module_identifier, Some(lazy_dependencies))
      {
        forwarded_ids.append(pending_forwarded_ids);
      }
      if let Some(task) = process_unlazy_dependencies(
        &context.artifact.module_to_lazy_make,
        module_graph,
        forwarded_ids,
        module_identifier,
      ) {
        tasks.push(Box::new(task));
      }

      all_dependencies
    } else {
      context
        .artifact
        .module_to_lazy_make
        .update_module_lazy_dependencies(module_identifier, None);
      all_dependencies
    };

    tasks.push(Box::new(ProcessDependenciesTask {
      dependencies: dependencies_to_process,
      original_module_identifier: module_identifier,
      from_unlazy: false,
    }));

    Ok(tasks)
  }
}
