use std::{collections::VecDeque, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rustc_hash::FxHashSet;

use super::{
  TaskContext, lazy::process_unlazy_dependencies, process_dependencies::ProcessDependenciesTask,
};
use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule, BuildContext,
  CompilationId, CompilerId, CompilerOptions, DependencyId, DependencyParents, ModuleCodeTemplate,
  OptimizationBailoutItem, ResolverFactory, SharedPluginDriver,
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
      let mut flat_items: Vec<FlatBuildItem> = Vec::new();
      let mut all_dependencies: Vec<DependencyId> = Vec::new();
      let mut lazy_dependencies = LazyDependencies::default();
      let mut queue: VecDeque<Box<AsyncDependenciesBlock>> = VecDeque::new();

      for dependency in build_result.dependencies {
        let dependency_id = *dependency.id();
        if let Some(until) = dependency.lazy() {
          lazy_dependencies.insert(&dependency, until);
        }
        all_dependencies.push(dependency_id);
        flat_items.push(FlatBuildItem::Dependency {
          dependency,
          parent_block: None,
        });
      }
      queue.extend(build_result.blocks);

      while let Some(mut block) = queue.pop_front() {
        let block_id = block.identifier();
        let inner_dependencies = block.take_dependencies();
        let inner_blocks = block.take_blocks();

        for dependency in inner_dependencies {
          let dependency_id = *dependency.id();
          if let Some(until) = dependency.lazy() {
            lazy_dependencies.insert(&dependency, until);
          }
          all_dependencies.push(dependency_id);
          flat_items.push(FlatBuildItem::Dependency {
            dependency,
            parent_block: Some(block_id),
          });
        }

        flat_items.push(FlatBuildItem::Block(block));
        queue.extend(inner_blocks);
      }

      vec![Box::new(BuildResultTask {
        module: build_result.module,
        optimization_bailouts: build_result.optimization_bailouts,
        plugin_driver,
        forwarded_ids,
        flat_items,
        all_dependencies,
        lazy_dependencies,
      })]
    })
  }
}

/// One item in the pre-flattened build result. Emitted in handle_block iteration order: each
/// block's `Dependency` items appear before its `Block`, so the main thread sees the same
/// deps-then-add_block sequence the original closure-based walk produced.
#[derive(Debug)]
enum FlatBuildItem {
  Dependency {
    dependency: BoxDependency,
    parent_block: Option<AsyncDependenciesBlockIdentifier>,
  },
  Block(Box<AsyncDependenciesBlock>),
}

#[derive(Debug)]
struct BuildResultTask {
  module: BoxModule,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
  plugin_driver: SharedPluginDriver,
  forwarded_ids: ForwardedIdSet,
  flat_items: Vec<FlatBuildItem>,
  all_dependencies: Vec<DependencyId>,
  lazy_dependencies: LazyDependencies,
}

#[async_trait::async_trait]
impl Task<TaskContext> for BuildResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }
  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let BuildResultTask {
      mut module,
      optimization_bailouts,
      plugin_driver,
      mut forwarded_ids,
      flat_items,
      mut all_dependencies,
      lazy_dependencies,
    } = *self;

    plugin_driver
      .compilation_hooks
      .succeed_module
      .call(context.compiler_id, context.compilation_id, &mut module)
      .await?;

    let module_id = module.identifier();
    let build_info = module.build_info();

    if !module.diagnostics().is_empty() {
      context.artifact.make_failed_module.insert(module_id);
    }

    tracing::trace!("Module built: {}", module_id);
    context
      .artifact
      .module_graph
      .get_optimization_bailout_mut(&module_id)
      .extend(optimization_bailouts);
    let resource_id = ResourceId::from(module_id);
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

    // Deps with the same parent block are emitted contiguously by the background pass, so
    // index_in_block is just a counter that resets on every parent-block change. Avoids
    // carrying a per-element usize through the cross-task transfer.
    let mut current_parent: Option<AsyncDependenciesBlockIdentifier> = None;
    let mut index_in_block: usize = 0;
    for item in flat_items {
      match item {
        FlatBuildItem::Dependency {
          dependency,
          parent_block,
        } => {
          let dependency_id = *dependency.id();
          if parent_block != current_parent {
            current_parent = parent_block;
            index_in_block = 0;
          }
          if parent_block.is_none() {
            module.add_dependency_id(dependency_id);
          }
          module_graph.set_parents(
            dependency_id,
            DependencyParents {
              block: parent_block,
              module: module_id,
              index_in_block,
            },
          );
          module_graph.add_dependency(dependency);
          index_in_block += 1;
        }
        FlatBuildItem::Block(block) => {
          module.add_block_id(block.identifier());
          module_graph.add_block(block);
        }
      }
    }

    {
      let mgm = module_graph.module_graph_module_by_identifier_mut(&module_id);
      mgm.all_dependencies_mut().clone_from(&all_dependencies);
    }

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
        .update_module_lazy_dependencies(module_id, Some(lazy_dependencies))
      {
        forwarded_ids.append(pending_forwarded_ids);
      }
      if let Some(task) = process_unlazy_dependencies(
        &context.artifact.module_to_lazy_make,
        module_graph,
        forwarded_ids,
        module_id,
      ) {
        tasks.push(Box::new(task));
      }

      all_dependencies
    } else {
      context
        .artifact
        .module_to_lazy_make
        .update_module_lazy_dependencies(module_id, None);
      all_dependencies
    };

    tasks.push(Box::new(ProcessDependenciesTask {
      dependencies: dependencies_to_process,
      original_module_identifier: module_id,
      from_unlazy: false,
    }));

    Ok(tasks)
  }
}
