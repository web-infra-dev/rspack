use std::{collections::VecDeque, sync::Arc};

use rspack_fs::ReadableFileSystem;

use super::{process_dependencies::ProcessDependenciesTask, MakeTaskContext};
use crate::{
  utils::task_loop::{Task, TaskResult, TaskType},
  AsyncDependenciesBlock, BoxDependency, BuildContext, BuildResult, CompilationId, CompilerId,
  CompilerOptions, DependencyParents, Module, ModuleProfile, ResolverFactory, SharedPluginDriver,
};

#[derive(Debug)]
pub struct BuildTask {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub module: Box<dyn Module>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for BuildTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Async
  }
  async fn background_run(self: Box<Self>) -> TaskResult<MakeTaskContext> {
    let Self {
      compiler_id,
      compilation_id,
      compiler_options,
      resolver_factory,
      plugin_driver,
      current_profile,
      mut module,
      fs,
    } = *self;
    if let Some(current_profile) = &current_profile {
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

    if let Some(current_profile) = &current_profile {
      current_profile.mark_building_end();
    }

    result.map::<Vec<Box<dyn Task<MakeTaskContext>>>, _>(|build_result| {
      vec![Box::new(BuildResultTask {
        module,
        build_result: Box::new(build_result),
        plugin_driver,
        current_profile,
      })]
    })
  }
}

#[derive(Debug)]
struct BuildResultTask {
  pub module: Box<dyn Module>,
  pub build_result: Box<BuildResult>,
  pub plugin_driver: SharedPluginDriver,
  pub current_profile: Option<Box<ModuleProfile>>,
}
#[async_trait::async_trait]
impl Task<MakeTaskContext> for BuildResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }
  async fn main_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let BuildResultTask {
      mut module,
      build_result,
      current_profile,
      plugin_driver,
    } = *self;

    plugin_driver
      .compilation_hooks
      .succeed_module
      .call(context.compiler_id, context.compilation_id, &mut module)
      .await?;

    let build_info = module.build_info();

    let artifact = &mut context.artifact;
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut artifact.module_graph_partial);

    if !module.diagnostics().is_empty() {
      artifact.make_failed_module.insert(module.identifier());
    }

    tracing::trace!("Module built: {}", module.identifier());
    module_graph
      .get_optimization_bailout_mut(&module.identifier())
      .extend(build_result.optimization_bailouts);
    artifact
      .file_dependencies
      .add_batch_file(&build_info.file_dependencies);
    artifact
      .context_dependencies
      .add_batch_file(&build_info.context_dependencies);
    artifact
      .missing_dependencies
      .add_batch_file(&build_info.missing_dependencies);
    artifact
      .build_dependencies
      .add_batch_file(&build_info.build_dependencies);

    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block = |dependencies: Vec<BoxDependency>,
                            blocks: Vec<Box<AsyncDependenciesBlock>>,
                            current_block: Option<Box<AsyncDependenciesBlock>>|
     -> Vec<Box<AsyncDependenciesBlock>> {
      for (index_in_block, dependency) in dependencies.into_iter().enumerate() {
        let dependency_id = *dependency.id();
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

    Ok(vec![Box::new(ProcessDependenciesTask {
      dependencies: all_dependencies,
      original_module_identifier: module_identifier,
    })])
  }
}
