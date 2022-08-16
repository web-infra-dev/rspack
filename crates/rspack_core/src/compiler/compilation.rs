use std::{fmt::Debug, sync::Arc};

use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::Result;
use tracing::instrument;

use crate::{
  split_chunks::code_splitting2, AssetContent, ChunkGraph, CompilerOptions, Dependency, EntryItem,
  Entrypoint, ModuleDependency, ModuleGraph, PluginDriver, ProcessAssetsArgs, RenderManifestArgs,
  RenderRuntimeArgs, ResolveKind, Runtime, VisitedModuleIdentity,
};

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub runtime: Runtime,
  pub entrypoints: HashMap<String, Entrypoint>,
  pub assets: CompilationAssets,
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
      entrypoints: Default::default(),
      assets: Default::default(),
    }
  }

  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
    self.assets.insert(filename, asset);
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

  fn create_chunk_assets(&mut self, plugin_driver: Arc<PluginDriver>) {
    let chunk_id_and_manifest = self
      .chunk_graph
      .id_to_chunk()
      .par_keys()
      .map(|chunk_id| {
        let manifest = plugin_driver.render_manifest(RenderManifestArgs {
          chunk_id,
          compilation: self,
        });
        (chunk_id.to_string(), manifest)
      })
      .collect::<Vec<_>>();

    chunk_id_and_manifest
      .into_iter()
      .for_each(|(chunk_id, manifest)| {
        manifest.unwrap().into_iter().for_each(|file_manifest| {
          let current_chunk = self.chunk_graph.chunk_by_id_mut(&chunk_id).unwrap();
          current_chunk
            .files
            .insert(file_manifest.filename().to_string());

          self.emit_asset(
            file_manifest.filename().to_string(),
            CompilationAsset {
              source: file_manifest.content,
            },
          );
        });
      })
  }

  fn process_assets(&mut self, plugin_driver: Arc<PluginDriver>) {
    plugin_driver
      .process_assets(ProcessAssetsArgs { compilation: self })
      .map_err(|e| {
        eprintln!("process_assets is not ok, err {:#?}", e);
        e
      })
      .ok();
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
  pub fn seal(&mut self, plugin_driver: Arc<PluginDriver>) -> Result<()> {
    code_splitting2(self);
    // TODO: optmize chunks

    for chunk in self.chunk_graph.chunks_mut() {
      chunk.calc_exec_order(&self.module_graph)?;
    }

    tracing::debug!("chunk graph {:#?}", self.chunk_graph);

    // generate runtime
    self.runtime = self.render_runtime(plugin_driver.clone());

    self.create_chunk_assets(plugin_driver.clone());

    self.entries.iter().for_each(|(name, _entry)| {
      let mut entrypoint = Entrypoint::new();
      self
        .chunk_graph
        .chunks()
        .filter(|chunk| &chunk.id == name)
        .map(|chunk| chunk.id.clone())
        .for_each(|chunk_id| {
          entrypoint.chunk_ids.push(chunk_id);
        });
      self.entrypoints.insert(name.clone(), entrypoint);
    });

    self.process_assets(plugin_driver);
    Ok(())
  }
}

// TODO: This is a temporary struct. This struct should be replaced with `rspack_sources`.
// See https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L159
// See https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L159
#[derive(Debug)]
pub struct CompilationAsset {
  pub source: AssetContent,
}

impl CompilationAsset {
  pub fn source(&self) -> &AssetContent {
    &self.source
  }

  pub fn buffer(&self) -> &[u8] {
    match &self.source {
      AssetContent::String(s) => s.as_bytes(),
      AssetContent::Buffer(b) => b,
    }
  }
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;
