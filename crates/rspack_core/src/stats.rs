use crate::{Chunk, Compilation, ModuleType, SourceType, PATH_START_BYTE_POS_MAP};
use hashbrown::HashMap;
use rspack_error::{
  emitter::{
    DiagnosticDisplay, DiagnosticDisplayer, StdioDiagnosticDisplay, StringDiagnosticDisplay,
  },
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

  pub fn emit_diagnostics(&self) -> Result<()> {
    let mut displayer = StdioDiagnosticDisplay::default();
    displayer.emit_batch_diagnostic(
      self.compilation.get_warnings(),
      PATH_START_BYTE_POS_MAP.clone(),
    )?;
    displayer.emit_batch_diagnostic(
      self.compilation.get_errors(),
      PATH_START_BYTE_POS_MAP.clone(),
    )
  }

  pub fn emit_diagnostics_string(&self, sorted: bool) -> Result<String> {
    let mut displayer = StringDiagnosticDisplay::default().with_sorted(sorted);
    let warnings = displayer.emit_batch_diagnostic(
      self.compilation.get_warnings(),
      PATH_START_BYTE_POS_MAP.clone(),
    )?;
    let errors = displayer.emit_batch_diagnostic(
      self.compilation.get_errors(),
      PATH_START_BYTE_POS_MAP.clone(),
    )?;
    Ok(warnings + &errors)
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
            chunk_names: Vec::new(),
            info: StatsAssetInfo {
              development: asset.info.development,
              hot_module_replacement: asset.info.hot_module_replacement,
            },
            emitted: self.compilation.emitted_assets.contains(name),
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
        asset.chunk_names = chunks
          .iter()
          .filter_map(|chunk| chunk.name.clone())
          .collect();
        asset.chunk_names.sort();
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
          .get_chunk_graph_module(&mgm.module_identifier)
          .chunks
          .iter()
          .map(|k| self.compilation.chunk_by_ukey.get(k).unwrap().id.clone())
          .collect();
        chunks.sort();
        StatsModule {
          r#type: "module",
          module_type: *module.module_type(),
          identifier: identifier.to_owned(),
          name: module
            .readable_identifier(&self.compilation.options.context)
            .into(),
          id: mgm.id(&self.compilation.chunk_graph).to_string(),
          chunks,
          size: module.size(&SourceType::JavaScript),
        }
      })
      .collect();
    modules.sort_by(|a, b| {
      if a.name.len() != b.name.len() {
        a.name.len().cmp(&b.name.len())
      } else {
        a.name.cmp(&b.name)
      }
    }); // TODO: sort by module.depth

    let mut chunks: Vec<StatsChunk> = self
      .compilation
      .chunk_by_ukey
      .values()
      .into_iter()
      .map(|c| {
        let mut files = Vec::from_iter(c.files.iter().cloned());
        files.sort();
        StatsChunk {
          r#type: "chunk",
          files,
          id: c.id.clone(),
          names: c.name.clone().map(|n| vec![n]).unwrap_or_default(),
          entry: c.has_entry_module(&self.compilation.chunk_graph),
          initial: c.can_be_initial(&self.compilation.chunk_group_by_ukey),
          size: self
            .compilation
            .chunk_graph
            .get_chunk_modules_size(&c.ukey, &self.compilation.module_graph),
        }
      })
      .collect();
    chunks.sort_by_cached_key(|v| v.id.to_string());

    let mut entrypoints: Vec<StatsEntrypoint> = self
      .compilation
      .entrypoints
      .iter()
      .map(|(name, ukey)| {
        let cg = self
          .compilation
          .chunk_group_by_ukey
          .get(ukey)
          .expect("compilation.chunk_group_by_ukey should have ukey from entrypoint");
        let mut chunks: Vec<String> = cg
          .chunks
          .iter()
          .map(|c| {
            self
              .compilation
              .chunk_by_ukey
              .get(c)
              .expect("compilation.chunk_by_ukey should have ukey from chunk_group")
          })
          .map(|c| c.id.clone())
          .collect();
        chunks.sort();
        let mut assets = cg.chunks.iter().fold(Vec::new(), |mut acc, c| {
          let chunk = self
            .compilation
            .chunk_by_ukey
            .get(c)
            .expect("compilation.chunk_by_ukey should have ukey from chunk_group");
          for file in &chunk.files {
            acc.push(StatsEntrypointAsset {
              name: file.clone(),
              size: self
                .compilation
                .assets()
                .get(file)
                .unwrap()
                .get_source()
                .size() as f64,
            });
          }
          acc
        });
        assets.sort_by_cached_key(|v| v.name.to_string());
        StatsEntrypoint {
          name: name.clone(),
          chunks,
          assets_size: assets.iter().fold(0.0, |acc, cur| acc + cur.size),
          assets,
        }
      })
      .collect();
    entrypoints.sort_by_cached_key(|e| e.name.to_string());

    let mut diagnostic_displayer = DiagnosticDisplayer::new(self.compilation.options.stats.colors);
    let errors: Vec<StatsError> = self
      .compilation
      .get_errors()
      .map(|d| StatsError {
        message: d.message.clone(),
        formatted: diagnostic_displayer
          .emit_diagnostic(d, PATH_START_BYTE_POS_MAP.clone())
          .unwrap(),
      })
      .collect();
    let warnings: Vec<StatsWarning> = self
      .compilation
      .get_warnings()
      .map(|d| StatsWarning {
        message: d.message.clone(),
        formatted: diagnostic_displayer
          .emit_diagnostic(d, PATH_START_BYTE_POS_MAP.clone())
          .unwrap(),
      })
      .collect();

    StatsCompilation {
      assets,
      modules,
      chunks,
      entrypoints,
      errors_count: errors.len(),
      errors,
      warnings_count: warnings.len(),
      warnings,
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
  pub warnings: Vec<StatsWarning>,
  pub warnings_count: usize,
  pub entrypoints: Vec<StatsEntrypoint>,
}

#[derive(Debug)]
pub struct StatsError {
  pub message: String,
  pub formatted: String,
}

#[derive(Debug)]
pub struct StatsWarning {
  pub message: String,
  pub formatted: String,
}

#[derive(Debug)]
pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<String>,
  pub chunk_names: Vec<String>,
  pub info: StatsAssetInfo,
  pub emitted: bool,
}

#[derive(Debug)]
pub struct StatsAssetInfo {
  pub development: bool,
  pub hot_module_replacement: bool,
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
  pub files: Vec<String>,
  pub id: String,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
}

#[derive(Debug)]
pub struct StatsEntrypointAsset {
  pub name: String,
  pub size: f64,
}

#[derive(Debug)]
pub struct StatsEntrypoint {
  pub name: String,
  pub assets: Vec<StatsEntrypointAsset>,
  pub chunks: Vec<String>,
  pub assets_size: f64,
}
