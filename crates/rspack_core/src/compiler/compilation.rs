use std::{collections::HashMap, sync::Arc};

use tracing::instrument;

use crate::{
  split_chunks::code_splitting2, ChunkGraph, CompilerOptions, Dependency, EntryItem,
  ModuleDependency, ModuleGraph, ResolveKind, VisitedModuleIdentity,
};

#[derive(Debug, Default)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
}

impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: HashMap<String, EntryItem>,
    visited_module_id: VisitedModuleIdentity,
    module_graph: ModuleGraph,
  ) -> Self {
    Self {
      options,
      visited_module_id,
      module_graph,
      entries,
      chunk_graph: Default::default(),
    }
  }

  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn entry_dependencies(&self) -> HashMap<String, Dependency> {
    self
      .entries
      .iter()
      .map(|(name, detail)| {
        (
          name.clone(),
          Dependency {
            importer: None,
            detail: ModuleDependency {
              specifier: detail.path.clone(),
              kind: ResolveKind::Import,
            },
          },
        )
      })
      .collect()
  }

  pub fn entry_modules(&self) {
    // self.
    todo!()
  }

  #[instrument(skip_all)]
  pub fn seal(&mut self) {
    code_splitting2(self);

    // optmize chunks
  }
}
