use std::{fmt::Debug, sync::Arc};

use anyhow::Result;
use hashbrown::HashMap;
use rayon::prelude::*;
use tracing::instrument;

use crate::{
  split_chunks::code_splitting2, Asset, ChunkGraph, CompilerOptions, Dependency, EntryItem,
  ModuleDependency, ModuleGraph, PluginDriver, RenderManifestArgs, RenderRuntimeArgs, ResolveKind,
  Runtime, VisitedModuleIdentity,
};

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub runtime: Runtime,
}

impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: std::collections::HashMap<String, EntryItem>,
    visited_module_id: VisitedModuleIdentity,
    module_graph: ModuleGraph,
  ) -> Self {
    Self {
      options,
      visited_module_id,
      module_graph,
      entries: HashMap::from_iter(entries),
      chunk_graph: Default::default(),
      runtime: Default::default(),
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

  pub fn render_manifest(&self, plugin_driver: Arc<PluginDriver>) -> Result<Vec<Asset>> {
    self
      .chunk_graph
      .id_to_chunk()
      .par_keys()
      .flat_map(|chunk_id| {
        match plugin_driver.render_manifest(RenderManifestArgs {
          chunk_id,
          compilation: self,
        }) {
          Ok(assets) => assets.into_par_iter().map(Ok).collect(),
          Err(err) => vec![Err(err)],
        }
      })
      .collect::<Result<Vec<Asset>>>()
  }

  pub fn render_runtime(&self, plugin_driver: Arc<PluginDriver>) -> Runtime {
    if let Ok(sources) = plugin_driver.render_runtime(RenderRuntimeArgs {
      sources: &vec![],
      compilation: self,
    }) {
      Runtime { sources }
    } else {
      Runtime { sources: vec![] }
    }
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
