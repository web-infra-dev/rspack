use rspack_error::{
  emitter::{
    DiagnosticDisplay, DiagnosticDisplayer, StdioDiagnosticDisplay, StringDiagnosticDisplay,
  },
  Result,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  BoxModule, Chunk, ChunkGroupUkey, Compilation, ModuleIdentifier, ModuleType, SourceType,
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
    displayer.emit_batch_diagnostic(self.compilation.get_warnings())?;
    displayer.emit_batch_diagnostic(self.compilation.get_errors())
  }

  pub fn emit_diagnostics_string(&self, sorted: bool) -> Result<String> {
    let mut displayer = StringDiagnosticDisplay::default().with_sorted(sorted);
    let warnings = displayer.emit_batch_diagnostic(self.compilation.get_warnings())?;
    let errors = displayer.emit_batch_diagnostic(self.compilation.get_errors())?;
    Ok(format!("{warnings}{errors}"))
  }
}

impl Stats<'_> {
  pub fn get_assets(&self) -> (Vec<StatsAsset>, Vec<StatsAssetsByChunkName>) {
    let mut compilation_file_to_chunks: HashMap<&String, Vec<&Chunk>> = HashMap::default();
    for chunk in self.compilation.chunk_by_ukey.values() {
      for file in &chunk.files {
        let chunks = compilation_file_to_chunks.entry(file).or_default();
        chunks.push(chunk);
      }
    }

    let mut assets: HashMap<&String, StatsAsset> = HashMap::from_iter(
      self
        .compilation
        .assets()
        .iter()
        .filter_map(|(name, asset)| {
          asset.get_source().map(|source| {
            (
              name,
              StatsAsset {
                r#type: "asset",
                name: name.clone(),
                size: source.size() as f64,
                chunks: Vec::new(),
                chunk_names: Vec::new(),
                info: StatsAssetInfo {
                  development: asset.info.development,
                  hot_module_replacement: asset.info.hot_module_replacement,
                },
                emitted: self.compilation.emitted_assets.contains(name),
              },
            )
          })
        }),
    );
    for asset in self.compilation.assets().values() {
      if let Some(source_map) = &asset.get_info().related.source_map {
        assets.remove(source_map);
      }
    }
    for (name, asset) in &mut assets {
      if let Some(chunks) = compilation_file_to_chunks.get(name) {
        asset.chunks = chunks
          .iter()
          .map(|chunk| chunk.id.clone().expect("Chunk should have id"))
          .collect();
        asset.chunks.sort_unstable();
        asset.chunk_names = chunks
          .iter()
          .filter_map(|chunk| chunk.name.clone())
          .collect();
        asset.chunk_names.sort_unstable();
      }
    }
    let mut assets: Vec<StatsAsset> = assets.into_values().collect();
    assets.sort_unstable_by(|a, b| b.size.partial_cmp(&a.size).expect("size should not be NAN"));

    let mut assets_by_chunk_name: HashMap<String, Vec<String>> = HashMap::default();
    for (file, chunks) in compilation_file_to_chunks {
      for chunk in chunks {
        if let Some(name) = &chunk.name {
          if let Some(assets) = assets_by_chunk_name.get_mut(name) {
            assets.push(file.to_string());
          } else {
            assets_by_chunk_name.insert(name.to_string(), vec![file.to_string()]);
          }
        }
      }
    }
    let assets_by_chunk_name = assets_by_chunk_name
      .into_iter()
      .map(|(name, mut files)| {
        files.sort_unstable();
        StatsAssetsByChunkName { name, files }
      })
      .collect();

    (assets, assets_by_chunk_name)
  }

  pub fn get_modules(
    &self,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
  ) -> Result<Vec<StatsModule>> {
    let mut modules: Vec<StatsModule> = self
      .compilation
      .module_graph
      .modules()
      .values()
      .map(|module| self.get_module(module, reasons, module_assets, nested_modules))
      .collect::<Result<_>>()?;
    Self::sort_modules(&mut modules);
    Ok(modules)
  }

  pub fn get_chunks(
    &self,
    chunk_modules: bool,
    chunk_relations: bool,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
  ) -> Result<Vec<StatsChunk>> {
    let mut chunks: Vec<StatsChunk> = self
      .compilation
      .chunk_by_ukey
      .values()
      .map(|c| -> Result<_> {
        let mut files = Vec::from_iter(c.files.iter().cloned());
        files.sort_unstable();
        let chunk_modules = if chunk_modules {
          let chunk_modules = self
            .compilation
            .chunk_graph
            .get_chunk_modules(&c.ukey, &self.compilation.module_graph);
          let mut chunk_modules = chunk_modules
            .into_iter()
            .map(|m| self.get_module(m, reasons, module_assets, nested_modules))
            .collect::<Result<Vec<_>>>()?;
          Self::sort_modules(&mut chunk_modules);
          Some(chunk_modules)
        } else {
          None
        };
        let (parents, children, siblings) = if let Some((parents, children, siblings)) =
          chunk_relations.then(|| self.get_chunk_relations(c))
        {
          (Some(parents), Some(children), Some(siblings))
        } else {
          (None, None, None)
        };
        Ok(StatsChunk {
          r#type: "chunk",
          files,
          id: c.expect_id().to_string(),
          names: c.name.clone().map(|n| vec![n]).unwrap_or_default(),
          entry: c.has_entry_module(&self.compilation.chunk_graph),
          initial: c.can_be_initial(&self.compilation.chunk_group_by_ukey),
          size: self
            .compilation
            .chunk_graph
            .get_chunk_modules_size(&c.ukey, &self.compilation.module_graph),
          modules: chunk_modules,
          parents,
          children,
          siblings,
        })
      })
      .collect::<Result<_>>()?;
    chunks.sort_by_cached_key(|v| v.id.to_string());
    Ok(chunks)
  }

  fn get_chunk_group(&self, name: &str, ukey: &ChunkGroupUkey) -> StatsChunkGroup {
    let cg = self
      .compilation
      .chunk_group_by_ukey
      .get(ukey)
      .expect("compilation.chunk_group_by_ukey should have ukey from entrypoint");
    let chunks: Vec<String> = cg
      .chunks
      .iter()
      .map(|c| {
        self
          .compilation
          .chunk_by_ukey
          .get(c)
          .expect("compilation.chunk_by_ukey should have ukey from chunk_group")
      })
      .map(|c| c.expect_id().to_string())
      .collect();
    let assets = cg.chunks.iter().fold(Vec::new(), |mut acc, c| {
      let chunk = self
        .compilation
        .chunk_by_ukey
        .get(c)
        .expect("compilation.chunk_by_ukey should have ukey from chunk_group");
      for file in &chunk.files {
        acc.push(StatsChunkGroupAsset {
          name: file.clone(),
          size: self
            .compilation
            .assets()
            .get(file)
            .unwrap_or_else(|| panic!("Could not find asset by name: {file:?}"))
            .get_source()
            .map_or(-1f64, |s| s.size() as f64),
        });
      }
      acc
    });
    StatsChunkGroup {
      name: name.to_string(),
      chunks,
      assets_size: assets.iter().map(|i| i.size).sum(),
      assets,
    }
  }

  pub fn get_entrypoints(&self) -> Vec<StatsChunkGroup> {
    self
      .compilation
      .entrypoints
      .iter()
      .map(|(name, ukey)| self.get_chunk_group(name, ukey))
      .collect()
  }

  pub fn get_named_chunk_groups(&self) -> Vec<StatsChunkGroup> {
    let mut named_chunk_groups: Vec<StatsChunkGroup> = self
      .compilation
      .named_chunk_groups
      .iter()
      .map(|(name, ukey)| self.get_chunk_group(name, ukey))
      .collect();
    named_chunk_groups.sort_by_cached_key(|e| e.name.to_string());
    named_chunk_groups
  }

  pub fn get_errors(&self) -> Vec<StatsError> {
    let mut diagnostic_displayer = DiagnosticDisplayer::new(self.compilation.options.stats.colors);
    self
      .compilation
      .get_errors()
      .map(|d| StatsError {
        title: d.title.clone(),
        message: d.message.clone(),
        formatted: diagnostic_displayer.emit_diagnostic(d).expect("TODO:"),
      })
      .collect()
  }

  pub fn get_warnings(&self) -> Vec<StatsWarning> {
    let mut diagnostic_displayer = DiagnosticDisplayer::new(self.compilation.options.stats.colors);
    self
      .compilation
      .get_warnings()
      .map(|d| StatsWarning {
        message: d.message.clone(),
        formatted: diagnostic_displayer.emit_diagnostic(d).expect("TODO:"),
      })
      .collect()
  }

  pub fn get_hash(&self) -> String {
    self.compilation.hash.to_owned()
  }

  fn sort_modules(modules: &mut [StatsModule]) {
    // TODO: sort by module.depth
    modules.sort_unstable_by(|a, b| {
      if a.name.len() != b.name.len() {
        a.name.len().cmp(&b.name.len())
      } else {
        a.name.cmp(&b.name)
      }
    });
  }

  fn get_module(
    &self,
    module: &BoxModule,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
  ) -> Result<StatsModule> {
    let identifier = module.identifier();
    let mgm = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(&identifier)
      .unwrap_or_else(|| panic!("Could not find ModuleGraphModule by identifier: {identifier:?}"));

    let issuer = self.compilation.module_graph.get_issuer(module);
    let (issuer_name, issuer_id) = issuer
      .map(|i| get_stats_module_name_and_id(i, self.compilation))
      .unzip();
    let mut issuer_path = Vec::new();
    let mut current_issuer = issuer;
    while let Some(i) = current_issuer {
      let (name, id) = get_stats_module_name_and_id(i, self.compilation);
      issuer_path.push(StatsModuleIssuer {
        identifier: i.identifier().to_string(),
        name,
        id,
      });
      current_issuer = self.compilation.module_graph.get_issuer(i);
    }
    issuer_path.reverse();

    let reasons = reasons
      .then(|| -> Result<_> {
        let mut reasons: Vec<StatsModuleReason> = mgm
          .incoming_connections_unordered(&self.compilation.module_graph)?
          .map(|connection| {
            let (module_name, module_id) = connection
              .original_module_identifier
              .and_then(|i| self.compilation.module_graph.module_by_identifier(&i))
              .map(|m| get_stats_module_name_and_id(m, self.compilation))
              .unzip();
            let dependency = self
              .compilation
              .module_graph
              .dependency_by_id(&connection.dependency_id);

            let r#type = dependency.map(|d| d.dependency_type().to_string());

            let user_request = dependency.map(|d| d.user_request().to_string());
            StatsModuleReason {
              module_identifier: connection.original_module_identifier.map(|i| i.to_string()),
              module_name,
              module_id: module_id.and_then(|i| i),
              r#type,
              user_request,
            }
          })
          .collect();
        reasons.sort_unstable_by(|a, b| a.module_identifier.cmp(&b.module_identifier));
        Ok(reasons)
      })
      .transpose()?;

    let mut chunks: Vec<String> = self
      .compilation
      .chunk_graph
      .get_chunk_graph_module(mgm.module_identifier)
      .chunks
      .iter()
      .map(|k| {
        self
          .compilation
          .chunk_by_ukey
          .get(k)
          .unwrap_or_else(|| panic!("Could not find chunk by ukey: {k:?}"))
          .expect_id()
          .to_string()
      })
      .collect();
    chunks.sort_unstable();

    let assets = module_assets.then(|| {
      let mut assets: Vec<_> = mgm
        .build_info
        .as_ref()
        .map(|info| info.asset_filenames.iter().map(|i| i.to_string()).collect())
        .unwrap_or_default();
      assets.sort();
      assets
    });

    // TODO: a placeholder for concatenation modules
    let modules = nested_modules.then(Vec::new);

    Ok(StatsModule {
      r#type: "module",
      module_type: *module.module_type(),
      identifier,
      name: module
        .readable_identifier(&self.compilation.options.context)
        .into(),
      id: self
        .compilation
        .chunk_graph
        .get_module_id(identifier)
        .clone(),
      chunks,
      size: module.size(&SourceType::JavaScript),
      issuer: issuer.map(|i| i.identifier().to_string()),
      issuer_name,
      issuer_id: issuer_id.and_then(|i| i),
      issuer_path,
      reasons,
      assets,
      modules,
    })
  }

  fn get_chunk_relations(&self, chunk: &Chunk) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut parents = HashSet::default();
    let mut children = HashSet::default();
    let mut siblings = HashSet::default();
    for cg in &chunk.groups {
      if let Some(cg) = self.compilation.chunk_group_by_ukey.get(cg) {
        for p in &cg.parents {
          if let Some(pg) = self.compilation.chunk_group_by_ukey.get(p) {
            for c in &pg.chunks {
              if let Some(c) = self.compilation.chunk_by_ukey.get(c) && let Some(id) = &c.id {
                parents.insert(id.to_string());
              }
            }
          }
        }
      }
      if let Some(cg) = self.compilation.chunk_group_by_ukey.get(cg) {
        for p in &cg.children {
          if let Some(pg) = self.compilation.chunk_group_by_ukey.get(p) {
            for c in &pg.chunks {
              if let Some(c) = self.compilation.chunk_by_ukey.get(c) && let Some(id) = &c.id {
                children.insert(id.to_string());
              }
            }
          }
        }
      }
      if let Some(cg) = self.compilation.chunk_group_by_ukey.get(cg) {
        for c in &cg.chunks {
          if let Some(c) = self.compilation.chunk_by_ukey.get(c) && c.id != chunk.id && let Some(id) = &c.id  {
            siblings.insert(id.to_string());
          }
        }
      }
    }
    let mut parents = Vec::from_iter(parents.into_iter());
    let mut children = Vec::from_iter(children.into_iter());
    let mut siblings = Vec::from_iter(siblings.into_iter());
    parents.sort();
    children.sort();
    siblings.sort();
    (parents, children, siblings)
  }
}

fn get_stats_module_name_and_id(
  module: &BoxModule,
  compilation: &Compilation,
) -> (String, Option<String>) {
  let identifier = module.identifier();
  let name = module.readable_identifier(&compilation.options.context);
  let id = compilation.chunk_graph.get_module_id(identifier).to_owned();
  (name.to_string(), id)
}

#[derive(Debug)]
pub struct StatsError {
  pub message: String,
  pub formatted: String,
  pub title: String,
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
pub struct StatsAssetsByChunkName {
  pub name: String,
  pub files: Vec<String>,
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
  pub identifier: ModuleIdentifier,
  pub name: String,
  pub id: Option<String>,
  pub chunks: Vec<String>,
  pub size: f64,
  pub issuer: Option<String>,
  pub issuer_name: Option<String>,
  pub issuer_id: Option<String>,
  pub issuer_path: Vec<StatsModuleIssuer>,
  pub reasons: Option<Vec<StatsModuleReason>>,
  pub assets: Option<Vec<String>>,
  pub modules: Option<Vec<StatsModule>>,
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
  pub modules: Option<Vec<StatsModule>>,
  pub parents: Option<Vec<String>>,
  pub children: Option<Vec<String>>,
  pub siblings: Option<Vec<String>>,
}

#[derive(Debug)]
pub struct StatsChunkGroupAsset {
  pub name: String,
  pub size: f64,
}

#[derive(Debug)]
pub struct StatsChunkGroup {
  pub name: String,
  pub assets: Vec<StatsChunkGroupAsset>,
  pub chunks: Vec<String>,
  pub assets_size: f64,
}

#[derive(Debug)]
pub struct StatsModuleIssuer {
  pub identifier: String,
  pub name: String,
  pub id: Option<String>,
}

#[derive(Debug)]
pub struct StatsModuleReason {
  pub module_identifier: Option<String>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub r#type: Option<String>,
  pub user_request: Option<String>,
}
