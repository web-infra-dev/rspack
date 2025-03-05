mod ctrl;
mod entry;
mod execute;
mod overwrite;

use std::sync::Arc;

use dashmap::{mapref::entry::Entry, DashMap};
pub use execute::{ExecuteModuleId, ExecutedRuntimeModule};
use rspack_collections::{Identifier, IdentifierDashMap, IdentifierDashSet};
use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::sync::{
  mpsc::{unbounded_channel, UnboundedSender},
  oneshot,
};
use tokio::task;

use self::{
  ctrl::{CtrlTask, Event, ExecuteParam},
  execute::{ExecuteModuleResult, ExecuteTask},
  overwrite::OverwriteTask,
};
use super::make::cutout::Cutout;
use super::make::repair::repair;
use super::make::{repair::MakeTaskContext, MakeArtifact, MakeParam};
use crate::cache::MemoryCache;
use crate::{
  task_loop::run_task_loop_with_event, Compilation, CompilationAsset, Context, Dependency,
  DependencyId, LoaderImportDependency, PublicPath,
};

#[derive(Debug)]
struct DepStatus {
  id: DependencyId,
  should_update: bool,
}

#[derive(Debug, Default)]
pub struct ModuleExecutor {
  cutout: Cutout,
  request_dep_map: DashMap<(String, Option<String>), DepStatus>,
  pub make_artifact: MakeArtifact,

  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<MakeArtifact>>,
  module_assets: IdentifierDashMap<HashMap<String, CompilationAsset>>,
  code_generated_modules: IdentifierDashSet,
  module_code_generated_modules: IdentifierDashMap<IdentifierDashSet>,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
}

impl ModuleExecutor {
  pub async fn hook_before_make(&mut self, compilation: &Compilation) -> Result<()> {
    let mut make_artifact = std::mem::take(&mut self.make_artifact);
    let mut params = Vec::with_capacity(5);
    params.push(MakeParam::CheckNeedBuild);
    if !compilation.modified_files.is_empty() {
      params.push(MakeParam::ModifiedFiles(compilation.modified_files.clone()));
    }
    if !compilation.removed_files.is_empty() {
      params.push(MakeParam::RemovedFiles(compilation.removed_files.clone()));
    }
    make_artifact.built_modules = Default::default();
    make_artifact.revoked_modules = Default::default();

    // Modules imported by `importModule` are passively loaded.
    let mut build_dependencies = self.cutout.cutout_artifact(&mut make_artifact, params);

    compilation
      .plugin_driver
      .compilation_hooks
      .revoked_modules
      .call(&make_artifact.revoked_modules)
      .await?;

    let mut build_dependencies_id = build_dependencies
      .iter()
      .map(|(id, _)| *id)
      .collect::<HashSet<_>>();
    for mut dep_status in self.request_dep_map.iter_mut() {
      if build_dependencies_id.contains(&dep_status.id) {
        dep_status.should_update = true;
        build_dependencies_id.remove(&dep_status.id);
      }
    }
    build_dependencies.retain(|dep| build_dependencies_id.contains(&dep.0));
    make_artifact = repair(compilation, make_artifact, build_dependencies)
      .await
      .unwrap_or_default();

    let mut ctx = MakeTaskContext::new(compilation, make_artifact, Arc::new(MemoryCache));
    let (event_sender, event_receiver) = unbounded_channel();
    let (stop_sender, stop_receiver) = oneshot::channel();
    self.event_sender = Some(event_sender.clone());
    self.stop_receiver = Some(stop_receiver);
    // avoid coop budget consumed to zero cause hang risk
    // related to https://tokio.rs/blog/2020-04-preemption
    tokio::spawn(task::unconstrained(async move {
      let _ = run_task_loop_with_event(
        &mut ctx,
        vec![Box::new(CtrlTask::new(event_receiver))],
        |_, task| {
          Box::new(OverwriteTask {
            origin_task: task,
            event_sender: event_sender.clone(),
          })
        },
      )
      .await;

      stop_sender
        .send(ctx.transform_to_make_artifact())
        .expect("should success");
    }));

    Ok(())
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

    let cutout = std::mem::take(&mut self.cutout);
    cutout.fix_artifact(&mut self.make_artifact);

    let mut mg = compilation.make_artifact.get_module_graph_mut();
    let module_assets = std::mem::take(&mut self.module_assets);
    for (original_module_identifier, assets) in module_assets {
      // recursive import module may not exist the module, just skip it
      if let Some(module) = mg.module_by_identifier_mut(&original_module_identifier) {
        module.build_info_mut().assets.extend(assets);
      }
    }

    //    let module_code_generation_modules = std::mem::take(&mut self.module_code_generated_modules);

    let diagnostics = self.make_artifact.diagnostics();
    compilation.extend_diagnostics(diagnostics);

    let code_generated_modules = std::mem::take(&mut self.code_generated_modules);
    for id in code_generated_modules {
      compilation.code_generated_modules.insert(id);
    }

    // remove useless *_dependencies incremental info
    self
      .make_artifact
      .file_dependencies
      .reset_incremental_info();
    self
      .make_artifact
      .context_dependencies
      .reset_incremental_info();
    self
      .make_artifact
      .missing_dependencies
      .reset_incremental_info();
    self
      .make_artifact
      .build_dependencies
      .reset_incremental_info();
  }

  #[allow(clippy::too_many_arguments)]
  pub async fn import_module(
    &self,
    request: String,
    layer: Option<String>,
    public_path: Option<PublicPath>,
    base_uri: Option<String>,
    original_module_context: Option<Context>,
    original_module_identifier: Option<Identifier>,
  ) -> ExecuteModuleResult {
    let sender = self
      .event_sender
      .as_ref()
      .expect("should have event sender");
    let (param, dep_id) = match self.request_dep_map.entry((request.clone(), layer.clone())) {
      Entry::Vacant(v) => {
        let dep = LoaderImportDependency::new(
          request.clone(),
          original_module_context.unwrap_or(Context::from("")),
        );
        let dep_id = *dep.id();
        v.insert(DepStatus {
          id: dep_id,
          should_update: false,
        });
        (ExecuteParam::Entry(Box::new(dep), layer.clone()), dep_id)
      }
      Entry::Occupied(mut v) => {
        let dep_status = v.get_mut();
        let dep_id = dep_status.id;
        if dep_status.should_update {
          let dep = LoaderImportDependency::new_with_id(
            dep_id,
            request.clone(),
            original_module_context.unwrap_or(Context::from("")),
          );
          dep_status.should_update = false;
          (ExecuteParam::Entry(Box::new(dep), layer.clone()), dep_id)
        } else {
          (ExecuteParam::DependencyId(dep_id), dep_id)
        }
      }
    };

    let (tx, rx) = oneshot::channel();
    sender
      .send(Event::ExecuteModule(
        param,
        ExecuteTask {
          entry_dep_id: dep_id,
          layer,
          public_path,
          base_uri,
          result_sender: tx,
        },
      ))
      .expect("should success");
    let (execute_result, assets, code_generated_modules, executed_runtime_modules) =
      rx.await.expect("should receiver success");

    if execute_result.error.is_none()
      && let Some(original_module_identifier) = original_module_identifier
    {
      self
        .module_assets
        .entry(original_module_identifier)
        .or_default()
        .extend(assets);
    }

    for id in code_generated_modules {
      self.code_generated_modules.insert(id);
      if let Some(original_module_identifier) = original_module_identifier {
        self
          .module_code_generated_modules
          .entry(original_module_identifier)
          .or_default()
          .insert(id);
      }
    }

    for runtime_module in executed_runtime_modules {
      self
        .executed_runtime_modules
        .insert(runtime_module.identifier, runtime_module);
    }

    execute_result
  }
}
