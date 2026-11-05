mod context;
mod ctrl;
mod entry;
mod execute;
mod module_tracker;
mod overwrite;

use rspack_collections::{Identifier, IdentifierDashMap, IdentifierDashSet};
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

  // temporary data, used by hook_after_finish_modules
  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<ExecutorTaskContext>>,
  module_assets: IdentifierDashMap<HashMap<String, CompilationAsset>>,
  code_generated_modules: IdentifierDashSet,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
  previous_module_assets: Option<IdentifierDashMap<HashMap<String, CompilationAsset>>>,
  previous_code_generated_modules: Option<IdentifierDashSet>,
  previous_executed_runtime_modules: Option<IdentifierDashMap<ExecutedRuntimeModule>>,
  usable: bool,
}

impl Default for ModuleExecutor {
  fn default() -> Self {
    Self {
      make_artifact: StealCell::new(BuildModuleGraphArtifact::new()),
      entries: Default::default(),
      exports_info_artifact: StealCell::new(Default::default()),
      event_sender: Default::default(),
      stop_receiver: Default::default(),
      module_assets: Default::default(),
      code_generated_modules: Default::default(),
      executed_runtime_modules: Default::default(),
      previous_module_assets: None,
      previous_code_generated_modules: None,
      previous_executed_runtime_modules: None,
      usable: true,
    }
  }
}

impl ModuleExecutor {
  /// Whether a module executor window is open, i.e. the background task loop
  /// servicing `importModule` requests is running.
  ///
  /// The window is open while the make phase (or a `rebuild_module` window)
  /// is in progress.
  pub fn is_running(&self) -> bool {
    self.event_sender.is_some()
  }

  pub fn is_usable(&self) -> bool {
    self.usable
  }

  fn reset_after_failed_window(&mut self) {
    self.make_artifact = BuildModuleGraphArtifact::new().into();
    self.exports_info_artifact = ExportsInfoArtifact::default().into();
    self.entries.clear();
    self.module_assets.clear();
    self.code_generated_modules.clear();
    self.executed_runtime_modules.clear();
    self.previous_module_assets = None;
    self.previous_code_generated_modules = None;
    self.previous_executed_runtime_modules = None;
    self.usable = false;
  }

  pub async fn before_build_module_graph(&mut self, compilation: &Compilation) -> Result<()> {
    if !self.usable {
      *self = Self::default();
    }
    self.start_window(compilation).await
  }

  /// Opens a module executor window around `Compilation::rebuild_module`.
  ///
  /// The rebuild owns the only module-graph update. The executor window only
  /// services `importModule` requests against its existing graph.
  pub fn before_rebuild_module_graph(&mut self, compilation: &Compilation) -> Result<()> {
    if !self.usable {
      return Err(rspack_error::error!(
        "module executor is unavailable after a previous window failure"
      ));
    }
    let mut make_artifact = self.make_artifact.steal();
    make_artifact.reset_temporary_data();
    let exports_info_artifact = self.exports_info_artifact.steal();
    self.start_window_with_artifacts(compilation, make_artifact, exports_info_artifact);
    Ok(())
  }

  async fn start_window(&mut self, compilation: &Compilation) -> Result<()> {
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

    // update the module affected by modified_files
    (make_artifact, exports_info_artifact) =
      match update_module_graph(compilation, make_artifact, exports_info_artifact, params).await {
        Ok(result) => result,
        Err(err) => {
          self.make_artifact = err.artifact.into();
          self.exports_info_artifact = err.exports_info_artifact.into();
          return Err(err.error);
        }
      };

    self.start_window_with_artifacts(compilation, make_artifact, exports_info_artifact);
    Ok(())
  }

  fn start_window_with_artifacts(
    &mut self,
    compilation: &Compilation,
    make_artifact: BuildModuleGraphArtifact,
    exports_info_artifact: ExportsInfoArtifact,
  ) {
    debug_assert!(self.previous_module_assets.is_none());
    debug_assert!(self.previous_code_generated_modules.is_none());
    debug_assert!(self.previous_executed_runtime_modules.is_none());
    self.previous_module_assets = Some(std::mem::take(&mut self.module_assets));
    self.previous_code_generated_modules = Some(std::mem::take(&mut self.code_generated_modules));
    self.previous_executed_runtime_modules =
      Some(std::mem::take(&mut self.executed_runtime_modules));

    let mut ctx = ExecutorTaskContext {
      origin_context: TaskContext::new(compilation, make_artifact, exports_info_artifact),
      tracker: Default::default(),
      entries: std::mem::take(&mut self.entries),
      executed_entry_deps: Default::default(),
    };
    let (event_sender, event_receiver) = unbounded_channel();
    let (stop_sender, stop_receiver) = oneshot::channel();
    self.event_sender = Some(event_sender);
    self.stop_receiver = Some(stop_receiver);
    // avoid coop budget consumed to zero cause hang risk
    // related to https://tokio.rs/blog/2020-04-preemption
    rspack_tasks::spawn_in_compiler_context(task::unconstrained(async move {
      if run_task_loop(&mut ctx, vec![Box::new(CtrlTask { event_receiver })])
        .await
        .is_err()
      {
        // Drop pending ExecuteTasks so their oneshot senders are closed and
        // waiting loaders receive an error instead of hanging forever.
        ctx.tracker = Default::default();
      }

      // ignore error, stop_receiver may be dropped if make stage occur error.
      let _ = stop_sender.send(ctx);
    }));
  }

  fn restore_window_temporary_data(&mut self, merge_runtime_modules: bool) {
    self.module_assets = self.previous_module_assets.take().unwrap_or_default();
    self.code_generated_modules = self
      .previous_code_generated_modules
      .take()
      .unwrap_or_default();
    let current_runtime_modules = std::mem::take(&mut self.executed_runtime_modules);
    let previous_runtime_modules = self
      .previous_executed_runtime_modules
      .take()
      .unwrap_or_default();
    if merge_runtime_modules {
      for entry in current_runtime_modules.iter() {
        previous_runtime_modules.insert(*entry.key(), entry.value().clone());
      }
    }
    self.executed_runtime_modules = previous_runtime_modules;
  }

  /// Stops a rebuild executor window after the rebuild failed.
  ///
  /// The compilation's main artifact may already have been consumed by the
  /// failed rebuild and therefore cannot be merged through
  /// `after_build_module_graph`. The executor's own temporary state still has
  /// to be recovered so it does not get lost when the callback reports the
  /// error.
  pub async fn abort_rebuild_module_graph(&mut self) -> Result<()> {
    let Some(sender) = self.event_sender.take() else {
      return Ok(());
    };
    let _ = sender.send(Event::Stop);
    let Some(stop_receiver) = self.stop_receiver.take() else {
      self.reset_after_failed_window();
      return Err(rspack_error::error!(
        "module executor stopped without a completion receiver"
      ));
    };
    let Ok(ctx) = stop_receiver.await else {
      self.reset_after_failed_window();
      return Err(rspack_error::error!(
        "module executor task exited before returning its state"
      ));
    };
    self.make_artifact = ctx.origin_context.artifact.into();
    self.exports_info_artifact = ctx.origin_context.exports_info_artifact.into();
    self.entries = ctx.entries;
    self.restore_window_temporary_data(false);
    self.usable = false;
    Ok(())
  }

  pub async fn after_build_module_graph(
    &mut self,
    compilation: &mut Compilation,
    clean_entries: bool,
  ) -> Result<()> {
    let Some(sender) = self.event_sender.take() else {
      return Err(rspack_error::error!(
        "module executor was not running when finishing the module graph"
      ));
    };
    let _ = sender.send(Event::Stop);

    let Some(stop_receiver) = self.stop_receiver.take() else {
      self.reset_after_failed_window();
      return Err(rspack_error::error!(
        "module executor stopped without a completion receiver"
      ));
    };
    let Ok(ctx) = stop_receiver.await else {
      self.reset_after_failed_window();
      return Err(rspack_error::error!(
        "module executor task exited before returning its state"
      ));
    };
    let mut make_artifact = ctx.origin_context.artifact;
    let mut entries = ctx.entries;
    let mut exports_info_artifact = ctx.origin_context.exports_info_artifact;

    // Only the make phase cleans entries. A rebuild window may contain a
    // subset of the importModule entries, so cleaning here could remove
    // entries that belong to another rebuild window.
    let update_param = if clean_entries {
      let removed_module = compilation
        .build_module_graph_artifact
        .revoked_modules()
        .chain(make_artifact.revoked_modules())
        .collect::<HashSet<_>>();
      entries.retain(|k, v| {
        !removed_module.contains(&k.origin_module_identifier) || ctx.executed_entry_deps.contains(v)
      });
      UpdateParam::BuildEntryAndClean(entries.values().copied().collect())
    } else {
      UpdateParam::BuildEntry(entries.values().copied().collect())
    };
    (make_artifact, exports_info_artifact) = match update_module_graph(
      compilation,
      make_artifact,
      exports_info_artifact,
      vec![update_param],
    )
    .await
    {
      Ok(result) => result,
      Err(err) => {
        self.make_artifact = err.artifact.into();
        self.exports_info_artifact = err.exports_info_artifact.into();
        self.entries = entries;
        self.restore_window_temporary_data(false);
        self.usable = false;
        return Err(err.error);
      }
    };

    let mg = compilation.build_module_graph_artifact.get_module_graph();
    let module_assets = std::mem::take(&mut self.module_assets);
    for (original_module_identifier, assets) in module_assets {
      // recursive import module may not exist the module, just skip it
      if let Some(module) = mg.module_by_identifier(&original_module_identifier) {
        module.extend_build_assets(assets);
      }
    }

    for (_, module) in make_artifact.get_module_graph().modules() {
      module.freeze_build_info();
    }

    if clean_entries {
      let diagnostics = make_artifact.diagnostics();
      compilation.extend_diagnostics(diagnostics);
    }

    let code_generated_modules = std::mem::take(&mut self.code_generated_modules);
    for id in code_generated_modules {
      compilation.code_generated_modules.insert(id);
    }

    self.restore_window_temporary_data(true);
    self.make_artifact = make_artifact.into();
    self.exports_info_artifact = exports_info_artifact.into();
    self.entries = entries;
    Ok(())
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
  ) -> ExecuteModuleResult {
    let Some(sender) = self.event_sender.as_ref() else {
      // No task loop is running to service this request: the module executor
      // window is closed. Return a proper error for the calling loader
      // instead of panicking, so the failure surfaces as a module build error.
      let id = execute::next_execute_module_id();
      return ExecuteModuleResult {
        id,
        errors: vec![rspack_error::error!(
          "`importModule` is unavailable because no module is being built: it can only be called while the make phase or a `rebuildModule` window is in progress. Request: {request}"
        )
        .into()],
        ..Default::default()
      };
    };

    let request_for_error = request.clone();
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
      return ExecuteModuleResult {
        id: execute::next_execute_module_id(),
        errors: vec![
          rspack_error::error!(
            "`importModule` executor task is no longer running. Request: {request_for_error}"
          )
          .into(),
        ],
        ..Default::default()
      };
    }
    let Ok(ExecuteResult {
      execute_result,
      assets,
      code_generated_modules,
      executed_runtime_modules,
    }) = rx.await
    else {
      return ExecuteModuleResult {
        id: execute::next_execute_module_id(),
        errors: vec![rspack_error::error!(
          "`importModule` executor task exited before completing the request. Request: {request_for_error}"
        )
        .into()],
        ..Default::default()
      };
    };

    if execute_result.errors.is_empty() {
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

    execute_result
  }
}
