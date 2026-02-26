use std::{collections::VecDeque, sync::Arc};

use rspack_fs::ReadableFileSystem;
use rustc_hash::FxHashMap as HashMap;

use super::{
  TaskContext, factorize::FactorizeTask, lazy::ProcessUnlazyDependenciesTask,
  process_dependencies::dependency_resource_identifier,
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildContext, BuildResult, CompilationId,
  CompilerId, CompilerOptions, DependencyParents, Module, ModuleCodeTemplate, ResolverFactory,
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
    let mut grouped_dependencies = HashMap::default();
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
        if dependency.lazy().is_none()
          && let Some(resource_identifier) = dependency_resource_identifier(&dependency)
        {
          grouped_dependencies
            .entry(resource_identifier.into_owned())
            .or_insert(vec![])
            .push(dependency.clone());
        }
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
    let original_module_source = module
      .as_normal_module()
      .and_then(|module| module.source().cloned());
    let original_module_context = module.get_context();
    let issuer = module
      .as_normal_module()
      .and_then(|module| module.name_for_condition());
    let issuer_layer = module.get_layer().cloned();
    let resolve_options = module.get_resolve_options();

    module_graph.add_module(module);

    let mut tasks: Vec<Box<dyn Task<TaskContext>>> = vec![];

    if !lazy_dependencies.is_empty() {
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
    } else {
      context
        .artifact
        .module_to_lazy_make
        .update_module_lazy_dependencies(module_identifier, None);
    }

    for dependencies in grouped_dependencies.into_values() {
      let dependency = &dependencies[0];
      let dependency_type = dependency.dependency_type();
      // TODO move module_factory calculate to dependency factories
      let module_factory = context
        .dependency_factories
        .get(dependency_type)
        .unwrap_or_else(|| {
          panic!(
            "No module factory available for dependency type: {}, resourceIdentifier: {:?}",
            dependency_type,
            dependency.resource_identifier()
          )
        })
        .clone();

      tasks.push(Box::new(FactorizeTask {
        compiler_id: context.compiler_id,
        compilation_id: context.compilation_id,
        module_factory,
        original_module_identifier: Some(module_identifier),
        original_module_source: original_module_source.clone(),
        original_module_context: original_module_context.clone(),
        issuer: issuer.clone(),
        issuer_layer: issuer_layer.clone(),
        dependencies,
        resolve_options: resolve_options.clone(),
        options: context.compiler_options.clone(),
        resolver_factory: context.resolver_factory.clone(),
        from_unlazy: false,
      }));
    }

    Ok(tasks)
  }
}
