use std::{collections::VecDeque, sync::Arc};

use rspack_error::{Diagnostic, IntoTWithDiagnosticArray};

use super::{process_dependencies::ProcessDependenciesTask, MakeTaskContext};
use crate::{
  utils::task_loop::{Task, TaskResult, TaskType},
  AsyncDependenciesBlock, BoxDependency, BuildContext, BuildResult, CompilerModuleContext,
  CompilerOptions, DependencyParents, Module, ModuleProfile, ResolverFactory, RunnerContext,
  SharedPluginDriver,
};

#[derive(Debug)]
pub struct BuildTask {
  pub module: Box<dyn Module>,
  pub current_profile: Option<Box<ModuleProfile>>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
}

#[async_trait::async_trait]
impl Task<MakeTaskContext> for BuildTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Async
  }
  async fn async_run(self: Box<Self>) -> TaskResult<MakeTaskContext> {
    let Self {
      compiler_options,
      resolver_factory,
      plugin_driver,
      current_profile,
      mut module,
    } = *self;
    if let Some(current_profile) = &current_profile {
      current_profile.mark_building_start();
    }

    plugin_driver
      .compilation_hooks
      .build_module
      .call(&mut module)
      .await?;

    let result = module
      .build(
        BuildContext {
          runner_context: RunnerContext {
            options: compiler_options.clone(),
            resolver_factory: resolver_factory.clone(),
            module: CompilerModuleContext::from_module(module.as_ref()),
            module_source_map_kind: *module.get_source_map_kind(),
          },
          plugin_driver: plugin_driver.clone(),
          compiler_options: &compiler_options,
        },
        None,
      )
      .await;

    plugin_driver
      .compilation_hooks
      .succeed_module
      .call(&mut module)
      .await?;

    let build_result = result.map(|t| {
      let diagnostics = module
        .clone_diagnostics()
        .into_iter()
        .map(|d| d.with_module_identifier(Some(module.identifier())))
        .collect();
      t.with_diagnostic(diagnostics)
    });

    if let Some(current_profile) = &current_profile {
      current_profile.mark_building_end();
    }

    build_result.map::<Vec<Box<dyn Task<MakeTaskContext>>>, _>(|build_result| {
      let (build_result, diagnostics) = build_result.split_into_parts();
      vec![Box::new(BuildResultTask {
        module,
        build_result: Box::new(build_result),
        diagnostics,
        current_profile,
      })]
    })
  }
}

#[derive(Debug)]
struct BuildResultTask {
  pub module: Box<dyn Module>,
  pub build_result: Box<BuildResult>,
  pub diagnostics: Vec<Diagnostic>,
  pub current_profile: Option<Box<ModuleProfile>>,
}

impl Task<MakeTaskContext> for BuildResultTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }
  fn sync_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let BuildResultTask {
      mut module,
      build_result,
      diagnostics,
      current_profile,
    } = *self;

    let artifact = &mut context.artifact;
    let module_graph =
      &mut MakeTaskContext::get_module_graph_mut(&mut artifact.module_graph_partial);

    if !diagnostics.is_empty() {
      artifact.make_failed_module.insert(module.identifier());
    }

    tracing::trace!("Module built: {}", module.identifier());
    artifact.diagnostics.extend(diagnostics);
    module_graph
      .get_optimization_bailout_mut(&module.identifier())
      .extend(build_result.optimization_bailouts);
    artifact
      .file_dependencies
      .add_batch_file(&build_result.build_info.file_dependencies);
    artifact
      .context_dependencies
      .add_batch_file(&build_result.build_info.context_dependencies);
    artifact
      .missing_dependencies
      .add_batch_file(&build_result.build_info.missing_dependencies);
    artifact
      .build_dependencies
      .add_batch_file(&build_result.build_info.build_dependencies);

    let mut queue = VecDeque::new();
    let mut all_dependencies = vec![];
    let mut handle_block = |dependencies: Vec<BoxDependency>,
                            blocks: Vec<AsyncDependenciesBlock>,
                            current_block: Option<AsyncDependenciesBlock>|
     -> Vec<AsyncDependenciesBlock> {
      for dependency in dependencies {
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

    module.set_build_info(build_result.build_info);
    module.set_build_meta(build_result.build_meta);

    module_graph.add_module(module);

    Ok(vec![Box::new(ProcessDependenciesTask {
      dependencies: all_dependencies,
      original_module_identifier: module_identifier,
    })])
  }
}
