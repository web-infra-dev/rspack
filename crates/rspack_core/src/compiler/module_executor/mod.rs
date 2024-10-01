mod ctrl;
mod entry;
mod execute;
mod overwrite;

use ctrl::Event;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use dashmap::DashSet;
pub use execute::ExecuteModuleId;
pub use execute::ExecutedRuntimeModule;
use rspack_collections::Identifier;
use rspack_collections::IdentifierDashMap;
use rspack_collections::IdentifierDashSet;
use rspack_collections::IdentifierMap;
use rspack_error::Result;
use rustc_hash::FxHashSet as HashSet;
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
use super::BuildDependency;
use super::Compilation;
use super::CompilationAsset;
use crate::task_loop::Task;
use crate::{Context, Dependency, DependencyId, LoaderImportDependency, PublicPath};

#[derive(Debug, Default)]
pub struct ModuleExecutor {
  request_dep_map: DashMap<String, DependencyId>,
  running_module_map: IdentifierMap<Vec<UnboundedSender<Event>>>,
  event_sender: Option<UnboundedSender<Box<dyn Task<MakeTaskContext>>>>,

  assets: DashMap<String, CompilationAsset>,
  module_assets: IdentifierDashMap<DashSet<String>>,
  code_generated_modules: IdentifierDashSet,
  module_code_generated_modules: IdentifierDashMap<IdentifierDashSet>,
  pub executed_runtime_modules: IdentifierDashMap<ExecutedRuntimeModule>,
}

impl ModuleExecutor {
  pub fn reset(
    &mut self,
    build_dependencies: &mut HashSet<BuildDependency>,
  ) -> (
    UnboundedReceiver<Box<dyn Task<MakeTaskContext>>>,
    HashSet<BuildDependency>,
  ) {
    self.running_module_map.clear();
    let mut module_executor_build_dependencies = HashSet::default();
    for item in self.request_dep_map.iter() {
      build_dependencies.retain(|&build_dependency| {
        if build_dependency.0 == *item.value() {
          module_executor_build_dependencies.insert(build_dependency);
          false
        } else {
          true
        }
      });
    }

    let (event_sender, event_receiver) = unbounded_channel();
    self.event_sender = Some(event_sender.clone());
    (event_receiver, module_executor_build_dependencies)
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
    &mut self,
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
        (EntryParam::DependencyId(dep_id), dep_id)
      }
    };

    sender
      .send(Box::new(entry::EntryTask {
        param,
        event_sender: tx.clone(),
      }))
      .expect("should success");

    let mut finish_counter = 1;
    while finish_counter > 0 {
      let event = rx.recv().await.expect("should success");
      match event {
        Event::StartBuild(module_id) => {
          self.running_module_map.insert(module_id, Vec::new());
        }
        Event::FinishDeps(module_id) => {
          if let Some(module_id) = module_id
            && let Some(senders) = self.running_module_map.get_mut(&module_id)
          {
            senders.push(tx.clone());
          } else {
            finish_counter -= 1;
          }
        }
        Event::FinishModule(module_id, size) => {
          finish_counter += size;
          finish_counter -= 1;
          if let Some(senders) = self.running_module_map.remove(&module_id) {
            for sender in senders {
              sender
                .send(Event::FinishDeps(None))
                .expect("should success");
            }
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
