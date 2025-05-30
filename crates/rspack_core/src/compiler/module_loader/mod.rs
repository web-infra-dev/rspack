mod clean;
mod context;
mod ctrl;
mod entry;
mod execute;
mod module_tracker;
mod overwrite;

use std::sync::Arc;

use clean::{CleanEntryTask, CleanModuleTask};
use context::{Callback, LoadModuleMeta, LoadTaskContext};
use entry::EntryTask;
use execute::ExecuteTask;
use rspack_collections::Identifier;
use rspack_error::{miette::diagnostic, Result};
use rustc_hash::FxHashMap as HashMap;
use tokio::{
  sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    oneshot,
  },
  task,
};

use self::ctrl::{CtrlTask, Event};
use super::make::{repair::MakeTaskContext, MakeArtifact};
use crate::{
  cache::MemoryCache, task_loop::run_task_loop, BoxModule, Compilation, Context, DependencyId,
};

#[derive(Debug, Default)]
pub struct ModuleLoader {
  // data
  pub make_artifact: MakeArtifact,
  pub entries: HashMap<LoadModuleMeta, DependencyId>,

  // temporary data, used by hook_after_finish_modules
  event_sender: Option<UnboundedSender<Event>>,
  stop_receiver: Option<oneshot::Receiver<LoadTaskContext>>,
}

impl ModuleLoader {
  pub async fn hook_before_make(&mut self, compilation: &Compilation) -> Result<()> {
    let mut make_artifact = std::mem::take(&mut self.make_artifact);
    let changed_files = compilation
      .modified_files
      .iter()
      .chain(compilation.removed_files.iter())
      .cloned()
      .collect();
    make_artifact.reset_temporary_data();

    let mut ctx = LoadTaskContext {
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
      let _ = run_task_loop(
        &mut ctx,
        vec![
          Box::new(CleanModuleTask { changed_files }),
          Box::new(CtrlTask { event_receiver }),
        ],
      )
      .await;

      // ignore error, stop_receiver may be dropped if make stage occur error.
      let _ = stop_sender.send(ctx);
    }));

    Ok(())
  }

  pub async fn hook_after_finish_modules(&mut self, compilation: &mut Compilation) -> Result<()> {
    let sender = std::mem::take(&mut self.event_sender);
    sender
      .expect("should have sender")
      .send(Event::Stop(CleanEntryTask {
        revoked_module: compilation.make_artifact.revoked_modules.clone(),
      }))
      .expect("should success");

    let stop_receiver = std::mem::take(&mut self.stop_receiver);
    let Ok(ctx) = stop_receiver.expect("should have receiver").await else {
      panic!("receive make artifact failed");
    };
    self.make_artifact = ctx.origin_context.artifact;
    self.entries = ctx.entries;

    let diagnostics = self.make_artifact.diagnostics();
    compilation.extend_diagnostics(diagnostics);

    // remove useless *_dependencies incremental info
    self.make_artifact.reset_dependencies_incremental_info();
    Ok(())
  }

  pub fn load_module(
    &self,
    request: String,
    origin_module_context: Context,
    origin_module_identifier: Identifier,
    callback: impl FnOnce(Result<&BoxModule>) + 'static,
  ) -> Result<()> {
    let sender = self
      .event_sender
      .as_ref()
      .expect("should have event sender");

    let meta = LoadModuleMeta {
      origin_module_identifier,
      request,
    };
    sender
      .send(Event::LoadModule(EntryTask {
        meta: meta.clone(),
        origin_module_context,
        execute_task: ExecuteTask {
          meta,
          callback: Callback(Box::new(callback)),
        },
      }))
      .map_err(|_| diagnostic!("send to ctrl task failed").into())
  }
}
