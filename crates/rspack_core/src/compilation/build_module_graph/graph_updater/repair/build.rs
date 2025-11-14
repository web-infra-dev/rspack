use std::{collections::VecDeque, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rustc_hash::FxHashSet;

use super::{
  TaskContext, lazy::ProcessUnlazyDependenciesTask, process_dependencies::ProcessDependenciesTask,
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildContext, BuildResult, CompilationId,
  CompilerId, CompilerOptions, DependencyParents, ModuleProfile, ResolverFactory,
  SharedPluginDriver,
  compilation::build_module_graph::{ForwardedIdSet, HasLazyDependencies, LazyDependencies},
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
  pub current_profile: Option<ModuleProfile>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub forwarded_ids: ForwardedIdSet,
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
      resolver_factory,
      plugin_driver,
      mut current_profile,
      mut module,
      fs,
      forwarded_ids,
    } = *self;
    if let Some(current_profile) = &mut current_profile {
      current_profile.mark_building_start();
    }

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
          resolver_factory: resolver_factory.clone(),
          plugin_driver: plugin_driver.clone(),
          fs: fs.clone(),
        },
        None,
      )
      .await;

    if let Some(current_profile) = &mut current_profile {
      current_profile.mark_building_end();
    }

    result.map::<Vec<Box<dyn Task<TaskContext>>>, _>(|build_result| {
      vec![Box::new(BuildResultTask {
        module,
        build_result: Box::new(build_result),
        plugin_driver,
        current_profile,
        forwarded_ids,
      })]
    })
  }
}

#[derive(Debug)]
struct BuildResultTask {
  pub module: BoxModule,
  pub build_result: Box<BuildResult>,
  pub plugin_driver: SharedPluginDriver,
  pub current_profile: Option<ModuleProfile>,
  pub forwarded_ids: ForwardedIdSet,
}

#[async_trait::async_trait]
impl Task<TaskContext> for BuildResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let BuildResultTask {
      mut module,
      build_result,
      current_profile,
      plugin_driver,
      mut forwarded_ids,
    } = *self;

    plugin_driver
      .compilation_hooks
      .succeed_module
      .call(context.compiler_id, context.compilation_id, &mut module)
      .await?;

    let build_info = module.build_info();

    let artifact = &mut context.artifact;
    let module_graph = &mut TaskContext::get_module_graph_mut(&mut artifact.module_graph_partial);

    if !module.diagnostics().is_empty() {
      artifact.make_failed_module.insert(module.identifier());
    }

    tracing::trace!("Module built: {}", module.identifier());
    module_graph
      .get_optimization_bailout_mut(&module.identifier())
      .extend(build_result.optimization_bailouts);
    let resource_id = ResourceId::from(module.identifier());
    artifact
      .file_dependencies
      .add_files(&resource_id, &build_info.file_dependencies);
    artifact
      .context_dependencies
      .add_files(&resource_id, &build_info.context_dependencies);
    artifact
      .missing_dependencies
      .add_files(&resource_id, &build_info.missing_dependencies);
    artifact
      .build_dependencies
      .add_files(&resource_id, &build_info.build_dependencies);

    let mut lazy_dependencies = LazyDependencies::default();
    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block = |dependencies: Vec<BoxDependency>,
                            blocks: Vec<Box<AsyncDependenciesBlock>>,
                            current_block: Option<Box<AsyncDependenciesBlock>>|
     -> Vec<Box<AsyncDependenciesBlock>> {
      for (index_in_block, dependency) in dependencies.into_iter().enumerate() {
        let dependency_id = *dependency.id();
        if context.compiler_options.experiments.lazy_barrel
          && let Some(until) = dependency.lazy()
        {
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
        module_graph.add_dependency(dependency);
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
      let mgm = module_graph
        .module_graph_module_by_identifier_mut(&module.identifier())
        .expect("Failed to get mgm");
      mgm.all_dependencies = all_dependencies.clone();
      if let Some(current_profile) = current_profile {
        mgm.set_profile(current_profile);
      }
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
      if !forwarded_ids.is_empty() {
        tasks.push(Box::new(ProcessUnlazyDependenciesTask {
          forwarded_ids,
          original_module_identifier: module_identifier,
        }));
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
