use std::{collections::VecDeque, sync::Arc};

use rspack_util::time::current_time;

use super::{TaskContext, process_dependencies::ProcessDependenciesTask};
use crate::{
  AsyncDependenciesBlockRef, BoxModule, BuildContext, DependenciesBlock, DependencyParents,
  DependencyRef, ModuleRef, SharedPluginDriver,
  compilation::build_module_graph::module_build_cache::ModuleBuildCache,
  module_declared_side_effect_free,
  utils::task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct BuildTask {
  pub build_context: Arc<BuildContext>,
  pub module: BoxModule,
  pub module_build_cache: Option<ModuleBuildCache>,
  pub force_build: bool,
}

#[async_trait::async_trait]
impl Task<TaskContext> for BuildTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Background
  }
  async fn background_run(self: Box<Self>) -> TaskResult<TaskContext> {
    let Self {
      build_context,
      mut module,
      module_build_cache,
      force_build,
    } = *self;
    let plugin_driver = build_context.plugin_driver.clone();

    if let Some(module_build_cache) = &module_build_cache
      && let Some(cached_module) = module_build_cache
        .restore(
          &module,
          &build_context.file_system_info,
          &build_context.value_cache_versions,
          force_build,
        )
        .await?
    {
      return Ok(vec![Box::new(BuildResultTask {
        build_result: ModuleBuildResult::Cached(cached_module),
        plugin_driver,
      })]);
    }

    let build_start_time = module_build_cache.as_ref().map(|_| current_time());

    plugin_driver
      .compilation_hooks
      .build_module
      .call(
        build_context.compiler_id,
        build_context.compilation_id,
        &mut module,
      )
      .await?;

    let result = module.build(build_context, None).await?;

    if let (Some(module_build_cache), Some(build_start_time)) =
      (module_build_cache, build_start_time)
    {
      module_build_cache.mark_pending(result.identifier(), build_start_time);
    }

    Ok(vec![Box::new(BuildResultTask {
      build_result: ModuleBuildResult::Built(result),
      plugin_driver,
    })])
  }
}

#[derive(Debug)]
pub(super) enum ModuleBuildResult {
  Built(BoxModule),
  Cached(ModuleRef),
}

#[derive(Debug)]
pub(super) struct BuildResultTask {
  pub build_result: ModuleBuildResult,
  pub plugin_driver: SharedPluginDriver,
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
    } = *self;
    let module = match build_result {
      ModuleBuildResult::Built(mut module) => {
        plugin_driver
          .compilation_hooks
          .succeed_module
          .call(
            context.build_context.compiler_id,
            context.build_context.compilation_id,
            &mut module,
          )
          .await?;
        ModuleRef::from(module)
      }
      ModuleBuildResult::Cached(module) => {
        plugin_driver
          .compilation_hooks
          .still_valid_module
          .call(
            context.build_context.compiler_id,
            context.build_context.compilation_id,
            module.as_ref(),
          )
          .await?;
        module
      }
    };

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
      .extend_from_slice(&build_info.optimization_bailouts);
    context
      .artifact
      .file_dependencies
      .add_module_files(module.identifier(), &build_info.dependencies.file);
    context
      .artifact
      .context_dependencies
      .add_module_files(module.identifier(), &build_info.dependencies.context);
    context
      .artifact
      .missing_dependencies
      .add_module_files(module.identifier(), &build_info.dependencies.missing);
    context
      .artifact
      .build_dependencies
      .add_module_files(module.identifier(), &build_info.dependencies.build);
    drop(build_info);

    let module_graph = &mut context.artifact.module_graph;
    let side_effect_free = module_declared_side_effect_free(module.as_ref()) == Some(true);
    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block =
      |dependencies: &[DependencyRef], current_block: Option<&AsyncDependenciesBlockRef>| {
        for (index_in_block, dependency) in dependencies.iter().enumerate() {
          let dependency_id = *dependency.id();
          all_dependencies.push(dependency_id);
          module_graph.set_parents(
            dependency_id,
            DependencyParents {
              block: current_block.map(|block| block.identifier()),
              module: module.identifier(),
              index_in_block,
            },
          );
          module_graph.add_dependency_ref(dependency.clone());
        }
        if let Some(current_block) = current_block {
          module_graph.add_block(current_block.clone());
        }
      };
    handle_block(module.get_dependencies(), None);
    queue.extend(module.get_block_refs().iter().cloned());

    while let Some(block) = queue.pop_front() {
      handle_block(block.get_dependencies(), Some(&block));
      queue.extend(block.get_block_refs().iter().cloned());
    }

    {
      let mgm = module_graph.module_graph_module_by_identifier_mut(&module.identifier());
      mgm.all_dependencies_mut().clone_from(&all_dependencies);
    }

    let module_identifier = module.identifier();
    let has_lazy_dependencies = context.artifact.module_to_lazy_make.classify(
      module_identifier,
      side_effect_free,
      module.get_dependencies(),
      context
        .build_context
        .compiler_options
        .optimization
        .provided_exports,
      module_graph,
    );

    module_graph.add_module(module);

    if has_lazy_dependencies {
      all_dependencies.retain(|dependency| !module_graph.is_lazy_dependency(dependency));
    }

    Ok(vec![Box::new(ProcessDependenciesTask {
      dependencies: all_dependencies,
      original_module_identifier: module_identifier,
      from_unlazy: false,
    })])
  }
}
