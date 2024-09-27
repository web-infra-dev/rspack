mod ctrl;
mod entry;
mod execute;
mod overwrite;

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
pub use execute::ExecuteModuleId;
pub use execute::ExecutedRuntimeModule;
use rspack_collections::Identifier;
use rspack_error::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{
  mpsc::{unbounded_channel, UnboundedSender},
  oneshot,
};

use self::{
  ctrl::Event,
  entry::EntryParam,
  execute::{ExecuteModuleResult, ExecuteTask},
};
use super::make::repair::MakeTaskContext;
use crate::task_loop::Task;
use crate::{Context, Dependency, DependencyId, LoaderImportDependency, PublicPath};

#[derive(Debug, Default)]
pub struct ModuleExecutor {
  request_dep_map: DashMap<String, DependencyId>,
  event_sender: Option<UnboundedSender<Box<dyn Task<MakeTaskContext>>>>,
}

impl ModuleExecutor {
  pub fn reset(&mut self) -> UnboundedReceiver<Box<dyn Task<MakeTaskContext>>> {
    let (event_sender, event_receiver) = unbounded_channel();
    self.event_sender = Some(event_sender.clone());
    event_receiver
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
  ) -> Result<ExecuteModuleResult> {
    // let sender = self
    //   .event_sender
    //   .as_ref()
    //   .expect("should have event sender");
    // let (param, dep_id) = match self.request_dep_map.entry(request.clone()) {
    //   Entry::Vacant(v) => {
    //     let dep = LoaderImportDependency::new(
    //       request.clone(),
    //       original_module_context.unwrap_or(Context::from("")),
    //     );
    //     let dep_id = *dep.id();
    //     v.insert(dep_id);
    //     (EntryParam::Entry(Box::new(dep)), dep_id)
    //   }
    //   Entry::Occupied(v) => {
    //     let dep_id = *v.get();
    //     (EntryParam::DependencyId(dep_id, sender.clone()), dep_id)
    //   }
    // };

    // let (tx, rx) = oneshot::channel();
    // sender
    //   .send(Event::ExecuteModule(
    //     param,
    //     ExecuteTask {
    //       entry_dep_id: dep_id,
    //       layer,
    //       public_path,
    //       base_uri,
    //       result_sender: tx,
    //     },
    //   ))
    //   .expect("should success");
    // let (execute_result, assets, code_generated_modules, executed_runtime_modules) =
    //   rx.await.expect("should receiver success");

    // execute_result
    todo!()
  }
}
