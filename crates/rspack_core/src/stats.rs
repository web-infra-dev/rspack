use crate::{Chunk, Compilation, ModuleType, SourceType, PATH_START_BYTE_POS_MAP};
use hashbrown::{HashMap, HashSet};
use rspack_error::{
  emitter::{DiagnosticDisplay, StdioDiagnosticDisplay, StringDiagnosticDisplay},
  Result,
};

#[derive(Debug, Clone)]
pub struct Stats<'compilation> {
  pub compilation: &'compilation Compilation,
}

impl<'compilation> Stats<'compilation> {
  pub fn new(compilation: &'compilation Compilation) -> Self {
    Self { compilation }
  }

  pub fn emit_error(&self) -> Result<()> {
    StdioDiagnosticDisplay::default().emit_batch_diagnostic(
      &self.compilation.diagnostic,
      PATH_START_BYTE_POS_MAP.clone(),
    )
  }

  pub fn emit_error_string(&self, sorted: bool) -> Result<String> {
    StringDiagnosticDisplay::default()
      .with_sorted(sorted)
      .emit_batch_diagnostic(
        &self.compilation.diagnostic,
        PATH_START_BYTE_POS_MAP.clone(),
      )
  }
}

impl<'compilation> Stats<'compilation> {
  pub fn to_description(&self) -> StatsDescription {
    let mut compilation_file_to_chunks: HashMap<&String, Vec<&Chunk>> = HashMap::new();
    for (_, chunk) in &self.compilation.chunk_by_ukey {
      for file in &chunk.files {
        // TODO: avoid runtime.js in every chunk.files, delete this once runtime refacted.
        if file == "runtime.js" {
          continue;
        }
        let chunks = compilation_file_to_chunks.entry(file).or_default();
        chunks.push(chunk);
      }
    }
    let mut assets: HashMap<&String, StatsAsset> =
      HashMap::from_iter(self.compilation.assets.iter().map(|(name, asset)| {
        (
          name,
          StatsAsset {
            r#type: "asset",
            name: name.clone(),
            size: asset.get_source().size() as f64,
            chunks: Vec::new(),
          },
        )
      }));
    for (_, asset) in &self.compilation.assets {
      if let Some(source_map) = &asset.get_info().related.source_map {
        assets.remove(source_map);
      }
    }
    for (name, asset) in &mut assets {
      if let Some(chunks) = compilation_file_to_chunks.get(name) {
        asset.chunks = chunks.into_iter().map(|chunk| chunk.id.clone()).collect();
        asset.chunks.sort();
      }
    }
    let mut assets: Vec<StatsAsset> = assets.into_values().collect();
    assets.sort_by(|a, b| {
      // SAFETY: size should not be NAN.
      b.size.partial_cmp(&a.size).unwrap()
    });

    let mut modules: Vec<StatsModule> = self
      .compilation
      .module_graph
      .modules()
      .map(|module| {
        let identifier = module.identifier();
        let mgm = self
          .compilation
          .module_graph
          .module_graph_module_by_identifier(&identifier)
          .unwrap();
        let mut chunks: Vec<String> = self
          .compilation
          .chunk_graph
          .get_chunk_graph_module(&mgm.uri)
          .chunks
          .iter()
          .map(|k| self.compilation.chunk_by_ukey.get(k).unwrap().id.clone())
          .collect();
        chunks.sort();
        StatsModule {
          r#type: "module",
          module_type: module.module_type(),
          identifier,
          name: module.identifier(), // TODO: short it with requestShortener
          id: mgm.id.clone(),
          chunks,
          size: module.size(&SourceType::JavaScript),
        }
      })
      .collect();
    modules.sort_by_cached_key(|m| m.identifier.to_string()); // TODO: sort by module.depth
    StatsDescription {
      assets,
      modules,
      chunks: self
        .compilation
        .chunk_by_ukey
        .values()
        .into_iter()
        .map(|c| StatsChunk {
          files: c.files.clone(),
          id: c.id.clone(),
        })
        .collect(),
    }
  }
}

pub struct StatsDescription {
  pub assets: Vec<StatsAsset>,
  pub modules: Vec<StatsModule>,
  pub chunks: Vec<StatsChunk>,
  // pub entrypoints: HashMap<String, StatsEntrypoint>,
}

pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<String>,
}

pub struct StatsModule {
  pub r#type: &'static str,
  pub module_type: ModuleType,
  pub identifier: String,
  pub name: String,
  pub id: String,
  pub chunks: Vec<String>,
  pub size: f64,
}

pub struct StatsChunk {
  pub files: HashSet<String>,
  pub id: String,
}

pub struct StatsAssetReference {
  pub name: String,
}

pub struct StatsEntrypoint {
  pub name: String,
  pub assets: Vec<StatsAssetReference>,
}
