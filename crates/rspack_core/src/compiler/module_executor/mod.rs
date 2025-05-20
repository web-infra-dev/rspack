mod context;
mod ctrl;
mod entry;
mod execute;
mod module_tracker;
mod overwrite;

use std::sync::Arc;

use context::{ExecutorTaskContext, ImportModuleEntry, ImportModuleMeta};
use entry::EntryTask;
pub use execute::{ExecuteModuleId, ExecutedRuntimeModule};
use rspack_collections::{Identifier, IdentifierDashMap, IdentifierDashSet};
use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::{
  sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    oneshot,
  },
  task,
};

use self::{
  ctrl::{CtrlTask, Event},
  execute::ExecuteModuleResult,
};
use super::make::{repair::MakeTaskContext, update_module_graph, MakeArtifact, MakeParam};
use crate::{
  cache::MemoryCache, task_loop::run_task_loop, Compilation, CompilationAsset, Context, PublicPath,
};

#[derive(Debug, Default)]
pub struct ModuleExecutor {
  // data
  pub make_artifact: MakeArtifact,
  pub entries: HashMap<ImportModuleMeta, ImportModuleEntry>,

  // temporary data, used by hook_after_finish_modules
  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<ExecutorTaskContext>>,
  module_assets: IdentifierDashMap<HashMap<String, CompilationAsset>>,
  code_generated_modules: IdentifierDashSet,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
}

impl ModuleExecutor {
  pub async fn hook_before_make(&mut self, compilation: &Compilation) -> Result<()> {
    let mut make_artifact = std::mem::take(&mut self.make_artifact);
    let mut params = Vec::with_capacity(3);
    params.push(MakeParam::CheckNeedBuild);
    if !compilation.modified_files.is_empty() {
      params.push(MakeParam::ModifiedFiles(compilation.modified_files.clone()));
    }
    if !compilation.removed_files.is_empty() {
      params.push(MakeParam::RemovedFiles(compilation.removed_files.clone()));
    }
    make_artifact.reset_temporary_data();

    // update the module affected by modified_files
    make_artifact = update_module_graph(compilation, make_artifact, params).await?;

    compilation
      .plugin_driver
      .compilation_hooks
      .revoked_modules
      .call(&make_artifact.revoked_modules)
      .await?;

    let mut ctx = ExecutorTaskContext {
      origin_context: MakeTaskContext::new(compilation, make_artifact, Arc::new(MemoryCache)),
      tracker: Default::default(),
      entries: std::mem::take(&mut self.entries),
      used_entry: Default::default(),
    };
    let (event_sender, event_receiver) = unbounded_channel();
    let (stop_sender, stop_receiver) = oneshot::channel();
    self.event_sender = Some(event_sender.clone());
    self.stop_receiver = Some(stop_receiver);
    // avoid coop budget consumed to zero cause hang risk
    // related to https://tokio.rs/blog/2020-04-preemption
    tokio::spawn(task::unconstrained(async move {
      let _ = run_task_loop(&mut ctx, vec![Box::new(CtrlTask { event_receiver })]).await;

      // ignore error, stop_receiver may be dropped if make stage occur error.
      let _ = stop_sender.send(ctx);
    }));

    Ok(())
  }

  pub async fn hook_after_finish_modules(&mut self, compilation: &mut Compilation) -> Result<()> {
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
      .make_artifact
      .revoked_modules
      .iter()
      .chain(self.make_artifact.revoked_modules.iter())
      .collect::<HashSet<_>>();
    self.entries.retain(|_k, v| {
      let default = Default::default();
      let used_mid = ctx.used_entry.get(&v.dep_id).unwrap_or(&default);
      v.origin_module_identifiers.retain(|mid| {
        // Because some modules may use different importModule when recompiled,
        // use used_mid instead of artifact.built_module.
        !removed_module.contains(&mid) || used_mid.contains(mid)
      });
      !v.origin_module_identifiers.is_empty()
    });
    self.make_artifact = update_module_graph(
      compilation,
      std::mem::take(&mut self.make_artifact),
      vec![MakeParam::BuildEntryAndClean(
        self.entries.values().map(|item| item.dep_id).collect(),
      )],
    )
    .await?;

    let mut mg = compilation.make_artifact.get_module_graph_mut();
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

    // remove useless *_dependencies incremental info
    self.make_artifact.reset_dependencies_incremental_info();
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
      origin_module_context: origin_module_context.unwrap_or(Context::from("")),
      request,
      layer,
    };
    let (tx, rx) = oneshot::channel();
    sender
      .send(Event::ImportModule(EntryTask {
        meta,
        origin_module_identifier,
        public_path,
        base_uri,
        result_sender: tx,
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
