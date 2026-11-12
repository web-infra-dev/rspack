mod context;
mod ctrl;
mod entry;
mod execute;
mod module_tracker;
mod overwrite;

use rspack_collections::{Identifier, IdentifierDashMap, IdentifierDashSet, IdentifierSet};
use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::{
  sync::{
    mpsc::{UnboundedSender, unbounded_channel},
    oneshot,
  },
  task,
};

pub use self::execute::{ExecuteModuleId, ExecutedRuntimeModule};
use self::{
  context::{ExecutorTaskContext, ImportModuleMeta},
  ctrl::{CtrlTask, Event},
  entry::EntryTask,
  execute::{ExecuteModuleResult, ExecuteTask},
};
use super::{
  BuildModuleGraphArtifact,
  graph_updater::{UpdateParam, repair::context::TaskContext, update_module_graph},
};
use crate::{
  Compilation, CompilationAsset, Context, DependencyId, ExportsInfoArtifact, PublicPath, StealCell,
  build_module_graph::module_executor::execute::ExecuteResult, task_loop::run_task_loop,
};

#[derive(Debug)]
pub struct ModuleExecutor {
  // data
  pub make_artifact: StealCell<BuildModuleGraphArtifact>,
  pub entries: HashMap<ImportModuleMeta, DependencyId>,
  pub exports_info_artifact: StealCell<ExportsInfoArtifact>,
  pub has_build_cycle: bool,
  executed_entry_deps: HashSet<DependencyId>,

  // temporary data, used by hook_after_finish_modules
  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<(ExecutorTaskContext, Result<()>)>>,
  module_assets: IdentifierDashMap<HashMap<String, CompilationAsset>>,
  code_generated_modules: IdentifierDashSet,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
}

impl Default for ModuleExecutor {
  fn default() -> Self {
    Self {
      make_artifact: StealCell::new(BuildModuleGraphArtifact::new()),
      entries: Default::default(),
      exports_info_artifact: StealCell::new(Default::default()),
      has_build_cycle: false,
      executed_entry_deps: Default::default(),
      event_sender: Default::default(),
      stop_receiver: Default::default(),
      module_assets: Default::default(),
      code_generated_modules: Default::default(),
      executed_runtime_modules: Default::default(),
    }
  }
}

impl ModuleExecutor {
  pub async fn before_build_module_graph(&mut self, compilation: &Compilation) -> Result<()> {
    let mut make_artifact = self.make_artifact.steal();
    let mut exports_info_artifact = self.exports_info_artifact.steal();
    let mut params = Vec::with_capacity(5);
    params.push(UpdateParam::CheckNeedBuild);
    if !compilation.modified_files.is_empty() {
      params.push(UpdateParam::ModifiedFiles(
        compilation.modified_files.clone(),
      ));
    }
    if !compilation.removed_files.is_empty() {
      params.push(UpdateParam::RemovedFiles(compilation.removed_files.clone()));
    }
    make_artifact.reset_temporary_data();
    self.executed_entry_deps.clear();
    self.has_build_cycle = false;

    let update =
      update_module_graph(compilation, make_artifact, exports_info_artifact, params).await;
    make_artifact = update.artifact;
    exports_info_artifact = update.exports_info_artifact;
    if let Err(error) = update.result {
      self.make_artifact = make_artifact.into();
      self.exports_info_artifact = exports_info_artifact.into();
      return Err(error);
    }

    self.make_artifact = make_artifact.into();
    self.exports_info_artifact = exports_info_artifact.into();
    self.start(compilation);
    Ok(())
  }

  fn start(&mut self, compilation: &Compilation) {
    let mut ctx = ExecutorTaskContext {
      origin_context: TaskContext::new(
        compilation,
        self.make_artifact.steal(),
        self.exports_info_artifact.steal(),
      ),
      tracker: Default::default(),
      entries: std::mem::take(&mut self.entries),
      executed_entry_deps: std::mem::take(&mut self.executed_entry_deps),
      has_build_cycle: self.has_build_cycle,
    };
    let (event_sender, event_receiver) = unbounded_channel();
    let (stop_sender, stop_receiver) = oneshot::channel();
    let shutdown_sender = event_sender.downgrade();
    self.event_sender = Some(event_sender);
    self.stop_receiver = Some(stop_receiver);
    // avoid coop budget consumed to zero cause hang risk
    // related to https://tokio.rs/blog/2020-04-preemption
    rspack_tasks::spawn_in_compiler_context(task::unconstrained(async move {
      let result = run_task_loop(
        &mut ctx,
        vec![Box::new(CtrlTask { event_receiver })],
        |ctx| {
          // Pending imports must reject before draining their loaders. Wake the
          // control task too, so its receiver closes and rejects later imports.
          ctx.tracker = Default::default();
          if let Some(sender) = shutdown_sender.upgrade() {
            let _ = sender.send(Event::Stop);
          }
        },
      )
      .await;
      let _ = stop_sender.send((ctx, result));
    }));
  }

  pub async fn after_build_module_graph(&mut self, compilation: &mut Compilation) -> Result<()> {
    self.pause().await?;

    // clean removed entries
    let removed_module = compilation
      .build_module_graph_artifact
      .revoked_modules()
      .chain(self.make_artifact.revoked_modules())
      .collect::<HashSet<_>>();
    self.entries.retain(|k, v| {
      !removed_module.contains(&k.origin_module_identifier) || self.executed_entry_deps.contains(v)
    });
    self.clean_entries(compilation).await?;

    let mg = compilation.build_module_graph_artifact.get_module_graph();
    let module_assets = std::mem::take(&mut self.module_assets);
    for (original_module_identifier, assets) in module_assets {
      // recursive import module may not exist the module, just skip it
      if let Some(module) = mg.module_by_identifier(&original_module_identifier) {
        module.extend_build_assets(assets);
      }
    }

    for (_, module) in self.make_artifact.get_module_graph().modules() {
      module.freeze_build_info();
    }

    let diagnostics = self.make_artifact.diagnostics();
    compilation.extend_diagnostics(diagnostics);

    let code_generated_modules = std::mem::take(&mut self.code_generated_modules);
    for id in code_generated_modules {
      compilation.code_generated_modules.insert(id);
    }

    Ok(())
  }

  async fn clean_entries(&mut self, compilation: &Compilation) -> Result<()> {
    loop {
      let update = update_module_graph(
        compilation,
        self.make_artifact.steal(),
        self.exports_info_artifact.steal(),
        vec![UpdateParam::BuildEntryAndClean(
          self.entries.values().copied().collect(),
        )],
      )
      .await;
      self.make_artifact = update.artifact.into();
      self.exports_info_artifact = update.exports_info_artifact.into();
      update.result?;

      // Removing an import can remove a module that itself called importModule.
      // Prune those nested entries before collecting diagnostics and dependencies.
      let previous_len = self.entries.len();
      self.entries.retain(|meta, _| {
        compilation
          .get_module_graph()
          .module_by_identifier(&meta.origin_module_identifier)
          .is_some()
          || self
            .make_artifact
            .get_module_graph()
            .module_by_identifier(&meta.origin_module_identifier)
            .is_some()
      });
      if previous_len == self.entries.len() {
        return Ok(());
      }
    }
  }

  // Suspend execution between main-graph update batches. Keep all contributions
  // until the end of finishMake, replacing only those of rebuilt origin modules.
  pub async fn prepare_rebuild(
    &mut self,
    compilation: &Compilation,
    modules: &IdentifierSet,
  ) -> Result<()> {
    self.pause().await?;
    self.entries.retain(|meta, dependency| {
      if modules.contains(&meta.origin_module_identifier) {
        self.executed_entry_deps.remove(dependency);
        return false;
      }
      true
    });
    for module in modules {
      self.module_assets.remove(module);
    }
    self.clean_entries(compilation).await?;
    self.start(compilation);
    Ok(())
  }

  async fn pause(&mut self) -> Result<()> {
    let (ctx, result) = self
      .stop()
      .await?
      .ok_or_else(|| rspack_error::error!("Module executor is not running"))?;
    self.make_artifact = ctx.origin_context.artifact.into();
    self.exports_info_artifact = ctx.origin_context.exports_info_artifact.into();
    self.entries = ctx.entries;
    self.executed_entry_deps = ctx.executed_entry_deps;
    self.has_build_cycle = ctx.has_build_cycle;
    result
  }

  async fn stop(&mut self) -> Result<Option<(ExecutorTaskContext, Result<()>)>> {
    if let Some(sender) = self.event_sender.take() {
      // The task loop may already have stopped because a loader or hook failed.
      let _ = sender.send(Event::Stop);
    }
    match self.stop_receiver.take() {
      Some(receiver) => receiver
        .await
        .map(Some)
        .map_err(|_| rspack_error::error!("Module executor stopped without returning its graph")),
      None => Ok(None),
    }
  }

  pub async fn abort(mut self) {
    let _ = self.stop().await;
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn import_module(
    &self,
    request: String,
    layer: Option<String>,
    public_path: Option<PublicPath>,
    base_uri: Option<String>,
    origin_module_context: Option<Context>,
    origin_module_identifier: Identifier,
  ) -> Result<ExecuteModuleResult> {
    let stopped = || rspack_error::error!("Module executor stopped before importModule completed");
    let Some(sender) = self.event_sender.as_ref() else {
      return Err(stopped());
    };

    let meta = ImportModuleMeta {
      origin_module_identifier,
      request,
      layer,
    };
    let (tx, rx) = oneshot::channel();
    if sender
      .send(Event::ImportModule(EntryTask {
        meta: meta.clone(),
        origin_module_context,
        execute_task: ExecuteTask {
          meta,
          public_path,
          base_uri,
          result_sender: tx,
        },
      }))
      .is_err()
    {
      return Err(stopped());
    }
    let result = rx.await.map_err(|_| stopped())?;
    let ExecuteResult {
      execute_result,
      assets,
      code_generated_modules,
      executed_runtime_modules,
    } = result?;

    if execute_result.errors.is_empty() && !assets.is_empty() {
      self
        .module_assets
        .entry(origin_module_identifier)
        .or_default()
        .extend(assets);
    }

    for id in code_generated_modules {
      self.code_generated_modules.insert(id);
    }

    for runtime_module in executed_runtime_modules {
      self
        .executed_runtime_modules
        .insert(runtime_module.identifier, runtime_module);
    }

    Ok(execute_result)
  }
}
