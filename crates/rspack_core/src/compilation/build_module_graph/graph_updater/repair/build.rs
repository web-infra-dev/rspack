use std::{collections::VecDeque, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rspack_util::time::current_time;
use rustc_hash::FxHashSet;

use super::{
  TaskContext, lazy::process_unlazy_dependencies, process_dependencies::ProcessDependenciesTask,
};
use crate::{
  AsyncDependenciesBlock, BoxModule, BuildContext, BuildResult, CacheFacade, CompilationId,
  CompilerId, CompilerOptions, DependencyParents, DependencyRef, FileSystemInfo,
  ModuleCodeTemplate, ResolverFactory, SharedPluginDriver,
  compilation::build_module_graph::{
    ForwardedIdSet, HasLazyDependencies, LazyDependencies, module_build_cache::ModuleBuildCache,
  },
  utils::{
    ResourceId,
    task_loop::{Task, TaskResult, TaskType},
  },
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

    let module_build_cache_with_start_time =
      module_build_cache.map(|cache| (cache, current_time()));

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

    let mut build_result = result;
    if let Some((module_build_cache, build_start_time)) = module_build_cache_with_start_time {
      module_build_cache
        .store(&mut build_result, &file_system_info, build_start_time)
        .await?;
    }

    Ok(vec![Box::new(BuildResultTask::built(
      build_result,
      plugin_driver,
      forwarded_ids,
    ))])
  }
}

#[derive(Debug)]
enum BuildResultOrigin {
  Built,
  Cached,
}

#[derive(Debug)]
pub(super) struct BuildResultTask {
  pub build_result: Box<BuildResult>,
  pub plugin_driver: SharedPluginDriver,
  pub forwarded_ids: ForwardedIdSet,
  origin: BuildResultOrigin,
}

impl BuildResultTask {
  fn built(
    build_result: BuildResult,
    plugin_driver: SharedPluginDriver,
    forwarded_ids: ForwardedIdSet,
  ) -> Self {
    Self {
      build_result: Box::new(build_result),
      plugin_driver,
      forwarded_ids,
      origin: BuildResultOrigin::Built,
    }
  }

  pub(super) fn cached(
    build_result: BuildResult,
    plugin_driver: SharedPluginDriver,
    forwarded_ids: ForwardedIdSet,
  ) -> Self {
    Self {
      build_result: Box::new(build_result),
      plugin_driver,
      forwarded_ids,
      origin: BuildResultOrigin::Cached,
    }
  }
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

    match origin {
      BuildResultOrigin::Built => {
        plugin_driver
          .compilation_hooks
          .succeed_module
          .call(context.compiler_id, context.compilation_id, &mut module)
          .await?;
      }
      BuildResultOrigin::Cached => {
        plugin_driver
          .compilation_hooks
          .still_valid_module
          .call(context.compiler_id, context.compilation_id, &mut module)
          .await?;
      }
    }

    let build_info = module.build_info();

    if !module.diagnostics().is_empty() {
      context
        .artifact
        .make_failed_module
        .insert(module.identifier());
    }

    tracing::trace!("Module built: {}", module.identifier());
    context
      .artifact
      .module_graph
      .get_optimization_bailout_mut(&module.identifier())
      .extend(build_result.optimization_bailouts);
    let resource_id = ResourceId::from(module.identifier());
    context
      .artifact
      .file_dependencies
      .add_files(&resource_id, &build_info.dependencies.file);
    context
      .artifact
      .context_dependencies
      .add_files(&resource_id, &build_info.dependencies.context);
    context
      .artifact
      .missing_dependencies
      .add_files(&resource_id, &build_info.dependencies.missing);
    context
      .artifact
      .build_dependencies
      .add_files(&resource_id, &build_info.dependencies.build);

    let module_graph = &mut context.artifact.module_graph;
    let mut lazy_dependencies = LazyDependencies::default();
    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block = |dependencies: Vec<DependencyRef>,
                            blocks: Vec<Box<AsyncDependenciesBlock>>,
                            current_block: Option<Box<AsyncDependenciesBlock>>|
     -> Vec<Box<AsyncDependenciesBlock>> {
      for (index_in_block, dependency) in dependencies.into_iter().enumerate() {
        let dependency_id = *dependency.id();
        if let Some(until) = dependency.lazy() {
          lazy_dependencies.insert(&dependency, until);
        }
        if current_block.is_none() {
          module.add_dependency_id(dependency_id);
        }
        all_dependencies.push(dependency_id);
        module_graph.set_parents(
          dependency_id,
          DependencyParents {
            block: current_block.as_ref().map(|block| block.identifier()),
            module: module.identifier(),
            index_in_block,
          },
        );
        module_graph.add_dependency_ref(dependency);
      }
      if let Some(current_block) = current_block {
        module.add_block_id(current_block.identifier());
        module_graph.add_block(current_block);
      }
      blocks
    };
    let blocks = handle_block(build_result.dependencies, build_result.blocks, None);
    queue.extend(blocks);

    while let Some(mut block) = queue.pop_front() {
      let dependencies = block.take_dependencies();
      let blocks = handle_block(dependencies, block.take_blocks(), Some(block));
      queue.extend(blocks);
    }

    {
      let mgm = module_graph.module_graph_module_by_identifier_mut(&module.identifier());
      mgm.all_dependencies_mut().clone_from(&all_dependencies);
    }

    let module_identifier = module.identifier();

    module_graph.add_module(module);

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
