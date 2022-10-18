use crate::{
  Chunk, Compilation, CompilationAssets, ModuleType, SourceType, PATH_START_BYTE_POS_MAP,
};
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
  // This function is only used for tests compatible.
  pub fn __should_only_used_in_tests_assets(&self) -> &CompilationAssets {
    &self.compilation.assets
  }

  pub fn to_description(&self) -> StatsDescription {
    let mut compilation_file_to_chunks: HashMap<&str, Vec<&Chunk>> = HashMap::new();
    for (_, chunk) in &self.compilation.chunk_by_ukey {
      for file in &chunk.files {
        let chunks = compilation_file_to_chunks.entry(file).or_default();
        chunks.push(chunk);
      }
    }
    StatsDescription {
      assets: self
        .compilation
        .assets
        .iter()
        // TODO: we needs asset.into.related to filter asset not in chunks, such as '.map' file.
        .filter(|(filename, _)| compilation_file_to_chunks.get(filename.as_str()).is_some())
        .map(|(filename, source)| StatsAsset {
          r#type: "asset",
          name: filename.clone(),
          size: source.size() as f64,
          chunks: compilation_file_to_chunks
            .get(filename.as_str())
            .unwrap()
            .iter()
            .map(|c| c.id.clone())
            .collect(),
        })
        .collect(),
      modules: self
        .compilation
        .module_graph
        .modules()
        .map(|module| StatsModule {
          r#type: "module",
          chunks: self
            .compilation
            .chunk_graph
            .get_chunk_graph_module(&module.uri)
            .chunks
            .iter()
            .map(|k| self.compilation.chunk_by_ukey.get(k).unwrap().id.clone())
            .collect(),
          module_type: module.module_type,
          identifier: module.module.identifier(),
          name: module.module.identifier(),
          id: module.id.clone(),
          size: module.module.size(&SourceType::JavaScript),
        })
        .collect(),
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
