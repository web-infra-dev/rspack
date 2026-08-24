use std::{
  collections::VecDeque,
  sync::Arc,
  time::{SystemTime, UNIX_EPOCH},
};

use rspack_cacheable::{cacheable, utils::OwnedOrRef};
use rspack_fs::ReadableFileSystem;
use rustc_hash::FxHashSet;

use super::{
  TaskContext, context::ModuleCacheContext, lazy::process_unlazy_dependencies,
  process_dependencies::ProcessDependenciesTask,
};
use crate::{
  AsyncDependenciesBlock, BoxDependency, BoxModule, BuildContext, BuildResult, Cache, CacheFacade,
  CacheValue, CompilationId, CompilerId, CompilerOptions, DependencyParents, Module,
  ModuleCodeTemplate, NormalModuleBuildState, OptimizationBailoutItem, ResolverFactory,
  SharedPluginDriver,
  compilation::build_module_graph::{ForwardedIdSet, HasLazyDependencies, LazyDependencies},
  new_cache::Snapshot,
  utils::{
    ResourceId,
    task_loop::{Task, TaskResult, TaskType},
  },
};

const MODULES_CACHE_NAMESPACE: &str = "Compilation/modules";

type RestoredDependencies = Vec<BoxDependency>;
// AsyncDependenciesBlock is recursive and intentionally stays behind a pointer.
#[allow(clippy::vec_box)]
type RestoredBlocks = Vec<Box<AsyncDependenciesBlock>>;
type RestoredBuildResult = (
  RestoredDependencies,
  RestoredBlocks,
  Vec<OptimizationBailoutItem>,
);

#[cacheable]
struct CachedBuildResult<'a> {
  dependencies: Vec<OwnedOrRef<'a, BoxDependency>>,
  // AsyncDependenciesBlock is recursive and intentionally stays behind a pointer.
  #[allow(clippy::vec_box)]
  blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlock>>,
}

impl CachedBuildResult<'_> {
  fn from_build_result(build_result: &BuildResult) -> CachedBuildResult<'_> {
    CachedBuildResult {
      dependencies: build_result.dependencies.iter().map(Into::into).collect(),
      blocks: build_result
        .blocks
        .iter()
        .map(|block| block.as_ref().into())
        .collect(),
    }
  }

  fn into_parts(self) -> (RestoredDependencies, RestoredBlocks) {
    (
      self
        .dependencies
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .collect(),
      self
        .blocks
        .into_iter()
        .map(|block| Box::new(block.into_owned()))
        .collect(),
    )
  }
}

#[cacheable]
#[derive(Debug)]
struct ModuleBuildCacheEntry {
  state: NormalModuleBuildState,
  snapshot: Snapshot,
  build_result: Vec<u8>,
  optimization_bailouts: Vec<OptimizationBailoutItem>,
}

impl ModuleBuildCacheEntry {
  fn from_build_result(
    build_result: &BuildResult,
    snapshot: Snapshot,
    cache: &Cache,
  ) -> rspack_error::Result<Option<Self>> {
    let Some(module) = build_result.module.as_normal_module() else {
      return Ok(None);
    };
    if !module.build_info().cacheable {
      return Ok(None);
    }
    let Some(codec) = cache.codec() else {
      return Ok(None);
    };
    Ok(Some(Self {
      state: module.build_state(),
      snapshot,
      build_result: codec.encode(&CachedBuildResult::from_build_result(build_result))?,
      optimization_bailouts: build_result.optimization_bailouts.clone(),
    }))
  }

  fn restore(&self, module: &mut BoxModule) -> Option<()> {
    module
      .as_normal_module_mut()?
      .restore_build_state(&self.state);
    Some(())
  }

  fn build_result_parts(&self, cache: &Cache) -> rspack_error::Result<RestoredBuildResult> {
    let codec = cache
      .codec()
      .ok_or_else(|| rspack_error::error!("New cache codec is unavailable"))?;
    let cached: CachedBuildResult<'static> = codec.decode(&self.build_result)?;
    let (dependencies, blocks) = cached.into_parts();
    Ok((dependencies, blocks, self.optimization_bailouts.clone()))
  }
}

#[derive(Debug)]
pub struct BuildTask {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub module: BoxModule,
  pub resolver_factory: Arc<ResolverFactory>,
  pub compiler_options: Arc<CompilerOptions>,
  pub loader_cache: CacheFacade,
  pub runtime_template: ModuleCodeTemplate,
  pub plugin_driver: SharedPluginDriver,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub(super) module_cache_context: Option<ModuleCacheContext>,
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
      loader_cache,
      resolver_factory,
      plugin_driver,
      runtime_template,
      mut module,
      fs,
      module_cache_context,
      forwarded_ids,
    } = *self;

    let module_cache = if module.as_normal_module().is_some() {
      module_cache_context.as_ref().map(|context| {
        context
          .cache
          .facade(MODULES_CACHE_NAMESPACE)
          .get_item_cache(module.identifier().as_str(), None)
      })
    } else {
      None
    };

    if let (Some(module_cache), Some(module_cache_context)) = (&module_cache, &module_cache_context)
    {
      let entry = match module_cache.get::<ModuleBuildCacheEntry>() {
        Ok(entry) => entry,
        Err(error) => {
          tracing::warn!("Restoring NormalModule build cache failed: {error}");
          None
        }
      };
      if let Some(entry) = entry
        && !entry
          .state
          .need_build(&module_cache_context.value_cache_versions)
      {
        match module_cache_context
          .cache
          .check_module_snapshot_valid(&entry.snapshot)
          .await
        {
          Ok(true) => match entry.build_result_parts(&module_cache_context.cache) {
            Ok((dependencies, blocks, optimization_bailouts))
              if entry.restore(&mut module).is_some() =>
            {
              plugin_driver
                .compilation_hooks
                .still_valid_module
                .call(compiler_id, compilation_id, &mut module)
                .await?;
              return Ok(vec![Box::new(BuildResultTask {
                build_result: Box::new(BuildResult {
                  module,
                  dependencies,
                  blocks,
                  optimization_bailouts,
                }),
                plugin_driver,
                forwarded_ids,
                invoke_succeed_module: false,
              })]);
            }
            Ok(_) => {}
            Err(error) => {
              tracing::warn!("Decoding NormalModule build cache failed: {error}");
            }
          },
          Ok(false) => {}
          Err(error) => {
            tracing::warn!("Validating NormalModule build cache failed: {error}");
          }
        }
      }
    }

    let build_start_time = module_cache_context.as_ref().map(|_| {
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
    });

    plugin_driver
      .compilation_hooks
      .build_module
      .call(compiler_id, compilation_id, &mut module)
      .await?;

    let mut result = module
      .build(
        BuildContext {
          compiler_id,
          compilation_id,
          compiler_options: compiler_options.clone(),
          loader_cache,
          resolver_factory: resolver_factory.clone(),
          plugin_driver: plugin_driver.clone(),
          runtime_template,
          fs: fs.clone(),
        },
        None,
      )
      .await;

    if let Ok(build_result) = &mut result
      && let Some(module_cache) = module_cache
      && let Some(module_cache_context) = &module_cache_context
      && let Some(build_start_time) = build_start_time
    {
      let snapshot = {
        let build_info = build_result.module.build_info();
        module_cache_context
          .cache
          .create_module_snapshot(
            build_start_time,
            &build_info.file_dependencies,
            &build_info.context_dependencies,
            &build_info.missing_dependencies,
            &build_info.build_dependencies,
          )
          .await
      };
      match snapshot {
        Ok(Some(snapshot)) => {
          match ModuleBuildCacheEntry::from_build_result(
            build_result,
            snapshot,
            &module_cache_context.cache,
          ) {
            Ok(Some(entry)) => {
              if let Err(error) = module_cache.store(CacheValue::new(entry)) {
                tracing::warn!("Storing NormalModule build cache failed: {error}");
              }
            }
            Ok(None) => {}
            Err(error) => {
              tracing::warn!("Encoding NormalModule build cache failed: {error}");
            }
          }
        }
        Ok(None) => {}
        Err(error) => {
          tracing::warn!("Creating NormalModule build snapshot failed: {error}");
        }
      }
    }

    result.map::<Vec<Box<dyn Task<TaskContext>>>, _>(|build_result| {
      vec![Box::new(BuildResultTask {
        build_result: Box::new(build_result),
        plugin_driver,
        forwarded_ids,
        invoke_succeed_module: true,
      })]
    })
  }
}

#[derive(Debug)]
struct BuildResultTask {
  pub build_result: Box<BuildResult>,
  pub plugin_driver: SharedPluginDriver,
  pub forwarded_ids: ForwardedIdSet,
  pub invoke_succeed_module: bool,
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
      invoke_succeed_module,
    } = *self;
    let mut module = build_result.module;

    if invoke_succeed_module {
      plugin_driver
        .compilation_hooks
        .succeed_module
        .call(context.compiler_id, context.compilation_id, &mut module)
        .await?;
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
