mod ctrl;
mod entry;
mod execute;
mod overwrite;

use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use dashmap::DashSet;
pub use execute::ExecuteModuleId;
pub use execute::ExecutedRuntimeModule;
use rspack_collections::Identifier;
use rspack_collections::IdentifierDashMap;
use rspack_collections::IdentifierDashSet;
use rspack_error::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{
  mpsc::{unbounded_channel, UnboundedSender},
  oneshot,
};

use self::{
  entry::EntryParam,
  execute::{ExecuteModuleResult, ExecuteTask},
};
use super::make::repair::MakeTaskContext;
use super::Compilation;
use super::CompilationAsset;
use crate::task_loop::Task;
use crate::{Context, Dependency, DependencyId, LoaderImportDependency, PublicPath};

#[derive(Debug, Default)]
pub struct ModuleExecutor {
  request_dep_map: DashMap<String, DependencyId>,
  event_sender: Option<UnboundedSender<Box<dyn Task<MakeTaskContext>>>>,

  assets: DashMap<String, CompilationAsset>,
  module_assets: IdentifierDashMap<DashSet<String>>,
  code_generated_modules: IdentifierDashSet,
  module_code_generated_modules: IdentifierDashMap<IdentifierDashSet>,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
}

impl ModuleExecutor {
  pub fn reset(&mut self) -> UnboundedReceiver<Box<dyn Task<MakeTaskContext>>> {
    let (event_sender, event_receiver) = unbounded_channel();
    self.event_sender = Some(event_sender.clone());
    event_receiver
  }
  pub async fn hook_after_finish_modules(&mut self, compilation: &mut Compilation) {
    let module_assets = std::mem::take(&mut self.module_assets);
    for (original_module_identifier, files) in module_assets {
      let assets = compilation
        .module_assets
        .entry(original_module_identifier)
        .or_default();
      for file in files {
        assets.insert(file);
      }
    }

    let module_code_generation_modules = std::mem::take(&mut self.module_code_generated_modules);
    for (original_module_identifier, code_generation_modules) in module_code_generation_modules {
      for module_identifier in code_generation_modules {
        if let Some(module_assets) = compilation.module_assets.remove(&module_identifier) {
          compilation
            .module_assets
            .entry(original_module_identifier)
            .or_default()
            .extend(module_assets);
        }
      }
    }

    let assets = std::mem::take(&mut self.assets);
    for (filename, asset) in assets {
      compilation.emit_asset(filename, asset);
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
    layer: Option<String>,
    public_path: Option<PublicPath>,
    base_uri: Option<String>,
    original_module_context: Option<Context>,
    original_module_identifier: Option<Identifier>,
  ) -> Result<ExecuteModuleResult> {
    let sender = self
      .event_sender
      .as_ref()
      .expect("should have event sender");

    let (tx, mut rx) = unbounded_channel();
    let (is_created, param, dep_id) = match self.request_dep_map.entry(request.clone()) {
      Entry::Vacant(v) => {
        let dep = LoaderImportDependency::new(
          request.clone(),
          original_module_context.unwrap_or(Context::from("")),
        );
        let dep_id = *dep.id();
        v.insert(dep_id);
        (false, EntryParam::Entry(Box::new(dep)), dep_id)
      }
      Entry::Occupied(v) => {
        let dep_id = *v.get();
        (true, EntryParam::DependencyId(dep_id), dep_id)
      }
    };
    sender
      .send(Box::new(entry::EntryTask {
        param,
        event_sender: tx,
      }))
      .expect("should success");

    if !is_created {
      let mut finish_counter = 1;
      while finish_counter != 0 {
        let event = rx.recv().await.expect("should success");
        match event {
          ctrl::Event::FinishDeps => {
            finish_counter -= 1;
          }
          ctrl::Event::FinishModule(size) => {
            finish_counter += size;
            finish_counter -= 1;
          }
        }
      }
    }

    let (tx, rx) = oneshot::channel();
    sender
      .send(Box::new(ExecuteTask {
        entry_dep_id: dep_id,
        layer,
        public_path,
        base_uri,
        result_sender: tx,
      }))
      .expect("should success");
    let (execute_result, assets, code_generated_modules, executed_runtime_modules) =
      rx.await.expect("should receiver success");

    if let Ok(execute_result) = &execute_result
      && let Some(original_module_identifier) = original_module_identifier
    {
      self
        .module_assets
        .entry(original_module_identifier)
        .or_default()
        .extend(execute_result.assets.clone());
    }

    for (key, value) in assets {
      self.assets.insert(key.clone(), value);
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
