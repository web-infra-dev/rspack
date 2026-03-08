use std::{collections::VecDeque, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rustc_hash::FxHashSet;

use super::{
  TaskContext, lazy::process_unlazy_dependencies, process_dependencies::ProcessDependenciesTask,
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildContext, BuildResult, CompilationId,
  CompilerId, CompilerOptions, DependencyParents, ModuleCodeTemplate, ResolverFactory,
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
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub runtime_template: ModuleCodeTemplate,
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
      runtime_template,
      mut module,
      fs,
      forwarded_ids,
    } = *self;

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
          runtime_template,
          fs: fs.clone(),
        },
        None,
      )
      .await;

    result.map::<Vec<Box<dyn Task<TaskContext>>>, _>(|build_result| {
      vec![Box::new(BuildResultTask {
        build_result: Box::new(build_result),
        plugin_driver,
        forwarded_ids,
      })]
    })
  }
}

#[derive(Debug)]
struct BuildResultTask {
  pub build_result: Box<BuildResult>,
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
    } = *self;
    let mut module = build_result.module;

    plugin_driver
      .compilation_hooks
      .succeed_module
      .call(context.compiler_id, context.compilation_id, &mut module)
      .await?;

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
      .add_files(&resource_id, &build_info.file_dependencies);
    context
      .artifact
      .context_dependencies
      .add_files(&resource_id, &build_info.context_dependencies);
    context
      .artifact
      .missing_dependencies
      .add_files(&resource_id, &build_info.missing_dependencies);
    context
      .artifact
      .build_dependencies
      .add_files(&resource_id, &build_info.build_dependencies);

    let module_graph = &mut context.artifact.module_graph;
    let mut lazy_dependencies = LazyDependencies::default();
    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block = |dependencies: Vec<BoxDependency>,
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
      let mgm = module_graph.module_graph_module_by_identifier_mut(&module.identifier());
      mgm.all_dependencies.clone_from(&all_dependencies);
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
