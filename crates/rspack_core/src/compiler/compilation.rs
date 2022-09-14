use crate::{
  split_chunks::code_splitting, AssetContent, Chunk, ChunkByUkey, ChunkGraph, ChunkGroup,
  ChunkGroupKind, ChunkGroupUkey, ChunkKind, ChunkUkey, CompilerOptions, Dependency, EntryItem,
  Entrypoint, ModuleDependency, ModuleGraph, PluginDriver, ProcessAssetsArgs, RenderManifestArgs,
  RenderRuntimeArgs, ResolveKind, Runtime, VisitedModuleIdentity,
};
use hashbrown::HashMap;
use rayon::prelude::*;
use rspack_error::{Diagnostic, Result};
use serde::Serialize;
use std::{fmt::Debug, sync::Arc};

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: HashMap<ChunkUkey, Chunk>,
  pub chunk_group_by_ukey: HashMap<ChunkGroupUkey, ChunkGroup>,
  pub runtime: Runtime,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  pub diagnostic: Vec<Diagnostic>,
  pub(crate) _named_chunk: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
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
      chunk_by_ukey: Default::default(),
      chunk_group_by_ukey: Default::default(),
      entries: HashMap::from_iter(entries),
      chunk_graph: Default::default(),
      runtime: Default::default(),
      entrypoints: Default::default(),
      assets: Default::default(),
      diagnostic: vec![],
      _named_chunk: Default::default(),
      named_chunk_groups: Default::default(),
    }
  }
  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
    self.assets.insert(filename, asset);
  }

  pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostic.push(diagnostic);
  }

  pub fn push_batch_diagnostic(&mut self, mut diagnostic: Vec<Diagnostic>) {
    self.diagnostic.append(&mut diagnostic);
  }

  pub(crate) fn add_chunk(
    chunk_by_ukey: &mut ChunkByUkey,
    name: Option<String>,
    id: String,
    kind: ChunkKind,
  ) -> &mut Chunk {
    let chunk = Chunk::new(name, id, kind);
    let ukey = chunk.ukey;
    chunk_by_ukey.insert(chunk.ukey, chunk);
    chunk_by_ukey.get_mut(&ukey).expect("chunk not found")
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
              span: None,
            },
          },
        )
      })
      .collect()
  }

  fn create_chunk_assets(&mut self, plugin_driver: Arc<PluginDriver>) {
    let chunk_ref_and_manifest = self
      .chunk_by_ukey
      .par_values()
      .map(|chunk| {
        let manifest = plugin_driver.render_manifest(RenderManifestArgs {
          chunk_ukey: chunk.ukey,
          compilation: self,
        });
        (chunk.ukey, manifest)
      })
      .collect::<Vec<_>>();

    chunk_ref_and_manifest
      .into_iter()
      .for_each(|(chunk_ref, manifest)| {
        manifest.unwrap().into_iter().for_each(|file_manifest| {
          let current_chunk = self.chunk_by_ukey.get_mut(&chunk_ref).unwrap();
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

  async fn process_assets(&mut self, plugin_driver: Arc<PluginDriver>) {
    plugin_driver
      .process_assets(ProcessAssetsArgs { compilation: self })
      .await
      .map_err(|e| {
        eprintln!("process_assets is not ok, err {:#?}", e);
        e
      })
      .ok();
  }
  pub async fn done(&mut self, plugin_driver: Arc<PluginDriver>) -> Result<()> {
    plugin_driver.done().await?;
    Ok(())
  }
  pub fn render_runtime(&self, plugin_driver: Arc<PluginDriver>) -> Runtime {
    if let Ok(sources) = plugin_driver.render_runtime(RenderRuntimeArgs {
      sources: &vec![],
      compilation: self,
    }) {
      Runtime {
        context_indent: self.runtime.context_indent.clone(),
        sources,
      }
    } else {
      self.runtime.clone()
    }
  }

  pub fn entry_modules(&self) {
    // self.
    todo!()
  }

  pub fn entrypoint_by_name(&self, name: &str) -> &Entrypoint {
    let ukey = self.entrypoints.get(name).expect("entrypoint not found");
    self
      .chunk_group_by_ukey
      .get(ukey)
      .expect("entrypoint not found by ukey")
  }

  pub async fn seal(&mut self, plugin_driver: Arc<PluginDriver>) -> Result<()> {
    code_splitting(self)?;
    // TODO: optmize chunks

    // for chunk in self.chunk_by_ukey.values_mut() {
    //   chunk.calc_exec_order(&self.module_graph)?;
    // }

    tracing::debug!("chunk graph {:#?}", self.chunk_graph);

    let context_indent = if matches!(
      self.options.target.platform,
      crate::TargetPlatform::Web | crate::TargetPlatform::None
    ) {
      String::from("self")
    } else {
      String::from("this")
    };

    self.runtime = Runtime {
      sources: vec![],
      context_indent,
    };

    self.create_chunk_assets(plugin_driver.clone());
    // generate runtime
    self.runtime = self.render_runtime(plugin_driver.clone());

    self.process_assets(plugin_driver).await;
    Ok(())
  }
}

// TODO: This is a temporary struct. This struct should be replaced with `rspack_sources`.
// See https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L159
// See https://github.com/webpack/webpack/blob/9fcaa243573005d6fdece9a3f8d89a0e8b399613/lib/Compilation.js#L159
#[derive(Debug, Clone, Serialize)]
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
