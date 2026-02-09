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
  Compilation, CompilationAsset, Context, DependencyId, PublicPath, task_loop::run_task_loop,
};

#[derive(Debug)]
pub struct ModuleExecutor {
  // data
  pub make_artifact: BuildModuleGraphArtifact,
  pub entries: HashMap<ImportModuleMeta, DependencyId>,

  // temporary data, used by hook_after_finish_modules
  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<ExecutorTaskContext>>,
  module_assets: IdentifierDashMap<HashMap<String, CompilationAsset>>,
  code_generated_modules: IdentifierDashSet,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
}

impl Default for ModuleExecutor {
  fn default() -> Self {
    Self {
      make_artifact: BuildModuleGraphArtifact::new(),
      entries: Default::default(),
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
    let mut make_artifact =
      std::mem::replace(&mut self.make_artifact, BuildModuleGraphArtifact::new());
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
    make_artifact = update_module_graph(compilation, make_artifact, params).await?;

    let mut ctx = ExecutorTaskContext {
      origin_context: TaskContext::new(compilation, make_artifact),
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
      let _ = run_task_loop(&mut ctx, vec![Box::new(CtrlTask { event_receiver })]).await;

      // ignore error, stop_receiver may be dropped if make stage occur error.
      let _ = stop_sender.send(ctx);
    }));

    Ok(())
  }

  pub async fn after_build_module_graph(&mut self, compilation: &mut Compilation) -> Result<()> {
    let sender = std::mem::take(&mut self.event_sender);
    sender
      .expect("should have sender")
      .send(Event::Stop)
      .expect("should success");

    let stop_receiver = std::mem::take(&mut self.stop_receiver);
    let Ok(ctx) = stop_receiver.expect("should have receiver").await else {
      panic!("receive make artifact failed");
    };
    self.make_artifact = ctx.origin_context.artifact;
    self.entries = ctx.entries;

    // clean removed entries
    let removed_module = compilation
      .build_module_graph_artifact
      .revoked_modules()
      .chain(self.make_artifact.revoked_modules())
      .collect::<HashSet<_>>();
    self.entries.retain(|k, v| {
      !removed_module.contains(&k.origin_module_identifier) || ctx.executed_entry_deps.contains(v)
    });
    self.make_artifact = update_module_graph(
      compilation,
      std::mem::replace(&mut self.make_artifact, BuildModuleGraphArtifact::new()),
      vec![UpdateParam::BuildEntryAndClean(
        self.entries.values().copied().collect(),
      )],
    )
    .await?;

    let mg = compilation
      .build_module_graph_artifact
      .get_module_graph_mut();
    let module_assets = std::mem::take(&mut self.module_assets);
    for (original_module_identifier, assets) in module_assets {
      // recursive import module may not exist the module, just skip it
      if let Some(module) = mg.module_by_identifier_mut(&original_module_identifier) {
        module.build_info_mut().assets.extend(assets);
      }
    }

    let diagnostics = self.make_artifact.diagnostics();
    compilation.extend_diagnostics(diagnostics);

    let code_generated_modules = std::mem::take(&mut self.code_generated_modules);
    for id in code_generated_modules {
      compilation.code_generated_modules.insert(id);
    }
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
    let sender = self
      .event_sender
      .as_ref()
      .expect("should have event sender");

    let meta = ImportModuleMeta {
      origin_module_identifier,
      request,
      layer,
    };
    let (tx, rx) = oneshot::channel();
    sender
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
      .expect("should success");
    let (execute_result, assets, code_generated_modules, executed_runtime_modules) =
      rx.await.expect("should receiver success");

    if execute_result.error.is_none() {
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
