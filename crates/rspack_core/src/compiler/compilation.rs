use std::{fmt::Debug, path::PathBuf, sync::Arc};

use dashmap::{DashMap, DashSet};
use hashbrown::HashMap;
use rayon::prelude::*;
use tracing::instrument;

use crate::{
  split_chunks::code_splitting2, Asset, ChunkGraph, CompilerOptions, Dependency, EntryItem,
  Filename, ModuleDependency, ModuleGraph, OutputFilename, PluginDriver, RenderManifestArgs,
  ResolveKind, VisitedModuleIdentity,
};

#[derive(Debug, Default)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub final_out_dir: PathBuf,
}

impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: std::collections::HashMap<String, EntryItem>,
    visited_module_id: VisitedModuleIdentity,
    module_graph: ModuleGraph,
    final_out_dir: PathBuf,
  ) -> Self {
    Self {
      options,
      visited_module_id,
      module_graph,
      entries: HashMap::from_iter(entries),
      chunk_graph: Default::default(),
      final_out_dir,
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

  pub fn assets(&self, plugin_driver: Arc<PluginDriver>) -> Vec<Asset> {
    self
      .chunk_graph
      .id_to_chunk()
      .par_keys()
      .flat_map(|chunk_id| {
        if let Ok(item) = plugin_driver.render_manifest(RenderManifestArgs {
          chunk_id,
          compilation: self,
        }) {
          item
        } else {
          vec![]
        }
      })
      .collect::<Vec<Asset>>()
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
