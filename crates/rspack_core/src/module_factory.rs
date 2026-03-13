use std::{fmt::Debug, sync::Arc};

use dashmap::mapref::entry::Entry;
use rspack_collections::IdentifierDashMap;
use rspack_error::{Diagnostic, Result};
use rspack_paths::{ArcPath, ArcPathSet};

use crate::{
  BoxDependency, BoxModule, CompilationId, CompilerId, CompilerOptions, Context, FactorizeInfo,
  ModuleIdentifier, ModuleLayer, Resolve, ResolverFactory,
};

#[derive(Debug)]
pub struct NormalModuleDedupeWaiter {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub dependencies: Vec<BoxDependency>,
  pub factorize_info: FactorizeInfo,
  pub from_unlazy: bool,
}

#[derive(Debug, Default)]
struct NormalModuleDedupeEntry {
  ready: bool,
  waiters: Vec<NormalModuleDedupeWaiter>,
}

#[derive(Debug, Default)]
pub struct NormalModuleDedupeTracker {
  entries: IdentifierDashMap<NormalModuleDedupeEntry>,
}

impl NormalModuleDedupeTracker {
  pub fn claim(&self, module_identifier: ModuleIdentifier) -> Option<ModuleIdentifier> {
    match self.entries.entry(module_identifier) {
      Entry::Occupied(entry) => Some(*entry.key()),
      Entry::Vacant(entry) => {
        entry.insert(Default::default());
        None
      }
    }
  }

  pub fn register_waiter_or_get_ready(
    &self,
    module_identifier: ModuleIdentifier,
    waiter: NormalModuleDedupeWaiter,
  ) -> Option<NormalModuleDedupeWaiter> {
    match self.entries.entry(module_identifier) {
      Entry::Occupied(mut entry) => {
        let entry = entry.get_mut();
        if entry.ready {
          Some(waiter)
        } else {
          entry.waiters.push(waiter);
          None
        }
      }
      Entry::Vacant(entry) => {
        entry.insert(NormalModuleDedupeEntry {
          ready: true,
          waiters: vec![],
        });
        Some(waiter)
      }
    }
  }

  pub fn mark_ready(&self, module_identifier: ModuleIdentifier) -> Vec<NormalModuleDedupeWaiter> {
    match self.entries.entry(module_identifier) {
      Entry::Occupied(mut entry) => {
        let entry = entry.get_mut();
        entry.ready = true;
        std::mem::take(&mut entry.waiters)
      }
      Entry::Vacant(entry) => {
        entry.insert(NormalModuleDedupeEntry {
          ready: true,
          waiters: vec![],
        });
        vec![]
      }
    }
  }

  pub fn take_waiters_and_clear(
    &self,
    module_identifier: ModuleIdentifier,
  ) -> Vec<NormalModuleDedupeWaiter> {
    self
      .entries
      .remove(&module_identifier)
      .map(|(_, entry)| entry.waiters)
      .unwrap_or_default()
  }
}

#[derive(Debug, Clone)]
pub struct ModuleFactoryCreateData {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub resolve_options: Option<Arc<Resolve>>,
  pub options: Arc<CompilerOptions>,
  pub request: String,
  pub context: Context,
  pub dependencies: Vec<BoxDependency>,
  pub issuer: Option<Box<str>>,
  pub issuer_identifier: Option<ModuleIdentifier>,
  pub issuer_layer: Option<ModuleLayer>,
  pub claimed_normal_module_identifier: Option<ModuleIdentifier>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub normal_module_dedupe_tracker: Arc<NormalModuleDedupeTracker>,

  pub file_dependencies: ArcPathSet,
  pub context_dependencies: ArcPathSet,
  pub missing_dependencies: ArcPathSet,
  pub diagnostics: Vec<Diagnostic>,
}

impl ModuleFactoryCreateData {
  pub fn add_file_dependency<F: Into<ArcPath>>(&mut self, file: F) {
    self.file_dependencies.insert(file.into());
  }

  pub fn add_file_dependencies<F: Into<ArcPath>>(&mut self, files: impl IntoIterator<Item = F>) {
    self
      .file_dependencies
      .extend(files.into_iter().map(Into::into));
  }

  pub fn add_context_dependency<F: Into<ArcPath>>(&mut self, context: F) {
    self.context_dependencies.insert(context.into());
  }

  pub fn add_context_dependencies<F: Into<ArcPath>>(
    &mut self,
    contexts: impl IntoIterator<Item = F>,
  ) {
    self
      .context_dependencies
      .extend(contexts.into_iter().map(Into::into));
  }

  pub fn add_missing_dependency<F: Into<ArcPath>>(&mut self, missing: F) {
    self.missing_dependencies.insert(missing.into());
  }

  pub fn add_missing_dependencies<F: Into<ArcPath>>(
    &mut self,
    missing: impl IntoIterator<Item = F>,
  ) {
    self
      .missing_dependencies
      .extend(missing.into_iter().map(Into::into));
  }
}

#[derive(Debug, Default)]
pub struct ModuleFactoryResult {
  pub module: Option<BoxModule>,
  pub normal_module_dedup: Option<ModuleIdentifier>,
}

impl ModuleFactoryResult {
  pub fn new_with_module(module: BoxModule) -> Self {
    Self {
      module: Some(module),
      normal_module_dedup: None,
    }
  }

  pub fn new_with_normal_module_dedup(module_identifier: ModuleIdentifier) -> Self {
    Self {
      module: None,
      normal_module_dedup: Some(module_identifier),
    }
  }

  pub fn module(mut self, module: Option<BoxModule>) -> Self {
    self.module = module;
    self
  }
}

#[async_trait::async_trait]
pub trait ModuleFactory: Debug + Sync + Send {
  async fn create(&self, data: &mut ModuleFactoryCreateData) -> Result<ModuleFactoryResult>;
}
