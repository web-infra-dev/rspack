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
  pub fn to_description(&self) -> StatsCompilation {
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
        asset.chunks = chunks.iter().map(|chunk| chunk.id.clone()).collect();
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

    let errors: Vec<StatsError> = self
      .compilation
      .get_errors()
      .map(|d| StatsError {
        message: d.message.clone(),
      })
      .collect();

    StatsCompilation {
      assets,
      modules,
      chunks: self
        .compilation
        .chunk_by_ukey
        .values()
        .into_iter()
        .map(|c| StatsChunk {
          r#type: "chunk",
          files: c.files.clone(),
          id: c.id.clone(),
        })
        .collect(),
      errors_count: errors.len(),
      errors,
    }
  }
}

#[derive(Debug)]
pub struct StatsCompilation {
  pub assets: Vec<StatsAsset>,
  pub modules: Vec<StatsModule>,
  pub chunks: Vec<StatsChunk>,
  pub errors: Vec<StatsError>,
  pub errors_count: usize,
  // pub entrypoints: HashMap<String, StatsEntrypoint>,
}

#[derive(Debug)]
pub struct StatsError {
  pub message: String,
}

#[derive(Debug)]
pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<String>,
}

#[derive(Debug)]
pub struct StatsModule {
  pub r#type: &'static str,
  pub module_type: ModuleType,
  pub identifier: String,
  pub name: String,
  pub id: String,
  pub chunks: Vec<String>,
  pub size: f64,
}

#[derive(Debug)]
pub struct StatsChunk {
  pub r#type: &'static str,
  pub files: HashSet<String>,
  pub id: String,
}

#[derive(Debug)]
pub struct StatsAssetReference {
  pub name: String,
}

#[derive(Debug)]
pub struct StatsEntrypoint {
  pub name: String,
  pub assets: Vec<StatsAssetReference>,
}
