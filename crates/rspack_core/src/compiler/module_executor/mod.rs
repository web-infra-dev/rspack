mod ctrl;
mod entry;
mod execute;
mod overwrite;

use dashmap::DashMap;
use dashmap::{mapref::entry::Entry, DashSet};
pub use execute::ExecuteModuleId;
pub use execute::ExecutedRuntimeModule;
use rspack_error::Result;
use rspack_identifier::Identifier;
use tokio::sync::{
  mpsc::{unbounded_channel, UnboundedSender},
  oneshot,
};

use self::{
  ctrl::{CtrlTask, Event},
  entry::EntryParam,
  execute::{ExecuteModuleResult, ExecuteTask},
  overwrite::OverwriteTask,
};
use super::make::{repair::MakeTaskContext, update_module_graph, MakeArtifact, MakeParam};
use crate::{
  task_loop::run_task_loop_with_event, Compilation, CompilationAsset, Context, Dependency,
  DependencyId, LoaderImportDependency,
};

#[derive(Debug, Default)]
pub struct ModuleExecutor {
  request_dep_map: DashMap<String, DependencyId>,
  pub make_artifact: MakeArtifact,

  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<MakeArtifact>>,
  assets: DashMap<String, CompilationAsset>,
  code_generated_modules: DashSet<Identifier>,
  pub executed_runtime_modules: DashMap<Identifier, ExecutedRuntimeModule>,
}

impl ModuleExecutor {
  pub async fn hook_before_make(&mut self, compilation: &Compilation) {
    let mut make_artifact = std::mem::take(&mut self.make_artifact);
    let mut params = Vec::with_capacity(5);
    params.push(MakeParam::CheckNeedBuild);
    if !compilation.modified_files.is_empty() {
      params.push(MakeParam::ModifiedFiles(compilation.modified_files.clone()));
    }
    if !compilation.removed_files.is_empty() {
      params.push(MakeParam::RemovedFiles(compilation.removed_files.clone()));
    }
    if !make_artifact.make_failed_dependencies.is_empty() {
      let deps = std::mem::take(&mut make_artifact.make_failed_dependencies);
      params.push(MakeParam::ForceBuildDeps(deps));
    }
    if !make_artifact.make_failed_module.is_empty() {
      let modules = std::mem::take(&mut make_artifact.make_failed_module);
      params.push(MakeParam::ForceBuildModules(modules));
    }
    make_artifact.diagnostics = Default::default();
    make_artifact.has_module_graph_change = false;

    make_artifact = if let Ok(artifact) = update_module_graph(compilation, make_artifact, params) {
      artifact
    } else {
      MakeArtifact::default()
    };

    let mut ctx = MakeTaskContext::new(compilation, make_artifact);
    let (event_sender, event_receiver) = unbounded_channel();
    let (stop_sender, stop_receiver) = oneshot::channel();
    self.event_sender = Some(event_sender.clone());
    self.stop_receiver = Some(stop_receiver);

    tokio::spawn(async move {
      let _ = run_task_loop_with_event(
        &mut ctx,
        vec![Box::new(CtrlTask::new(event_receiver))],
        |_, task| {
          Box::new(OverwriteTask {
            origin_task: task,
            event_sender: event_sender.clone(),
          })
        },
      );

      stop_sender
        .send(ctx.transform_to_make_artifact())
        .expect("should success");
    });
  }

  pub async fn hook_after_finish_modules(&mut self, compilation: &mut Compilation) {
    let sender = std::mem::take(&mut self.event_sender);
    sender
      .expect("should have sender")
      .send(Event::Stop())
      .expect("should success");

    let stop_receiver = std::mem::take(&mut self.stop_receiver);
    if let Ok(make_artifact) = stop_receiver.expect("should have receiver").await {
      self.make_artifact = make_artifact;
    } else {
      panic!("receive make artifact failed");
    }

    let assets = std::mem::take(&mut self.assets);
    for (filename, asset) in assets {
      compilation.emit_asset(filename, asset);
    }

    let diagnostics = self.make_artifact.take_diagnostics();
    compilation.extend_diagnostics(diagnostics);

    let built_modules = self.make_artifact.take_built_modules();
    for id in built_modules {
      compilation.built_modules.insert(id);
    }

    let code_generated_modules = std::mem::take(&mut self.code_generated_modules);
    for id in code_generated_modules {
      compilation.code_generated_modules.insert(id);
    }
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn import_module(
    &self,
    request: String,
    public_path: Option<String>,
    base_uri: Option<String>,
    original_module_context: Option<Context>,
  ) -> Result<ExecuteModuleResult> {
    let sender = self
      .event_sender
      .as_ref()
      .expect("should have event sender");
    let (param, dep_id) = match self.request_dep_map.entry(request.clone()) {
      Entry::Vacant(v) => {
        let dep = LoaderImportDependency::new(
          request.clone(),
          original_module_context.unwrap_or(Context::from("")),
        );
        let dep_id = *dep.id();
        v.insert(dep_id);
        (EntryParam::Entry(Box::new(dep)), dep_id)
      }
      Entry::Occupied(v) => {
        let dep_id = *v.get();
        (EntryParam::DependencyId(dep_id, sender.clone()), dep_id)
      }
    };

    let (tx, rx) = oneshot::channel();
    sender
      .send(Event::ExecuteModule(
        param,
        ExecuteTask {
          entry_dep_id: dep_id,
          public_path,
          base_uri,
          result_sender: tx,
        },
      ))
      .expect("should success");
    let (execute_result, assets, code_generated_modules, executed_runtime_modules) =
      rx.await.expect("should receiver success");

    for (key, value) in assets {
      self.assets.insert(key, value);
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
