use std::fmt::Debug;
use std::path::PathBuf;

use either::Either;
use itertools::Itertools;
use rayon::prelude::*;
use rspack_error::emitter::{DiagnosticDisplay, DiagnosticDisplayer};
use rspack_error::emitter::{StdioDiagnosticDisplay, StringDiagnosticDisplay};
use rspack_error::Result;
use rspack_identifier::Identifier;
use rspack_sources::Source;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  get_chunk_from_ukey, get_chunk_group_from_ukey, BoxModule, BoxRuntimeModule, Chunk,
  ChunkGroupOrderKey, ChunkGroupUkey, ChunkUkey, Compilation, ExecutedRuntimeModule, LogType,
  ModuleGraph, ModuleIdentifier, ModuleType, OriginLocation, ProvidedExports, RuntimeSpec,
  SourceType, UsedExports,
};

fn get_asset_size(file: &str, compilation: &Compilation) -> f64 {
  compilation
    .assets()
    .get(file)
    .unwrap_or_else(|| panic!("Could not find asset by name: {file:?}"))
    .get_source()
    .map_or(-1f64, |s| s.size() as f64)
}

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

      for file in &chunk.auxiliary_files {
        let chunks = compilation_file_to_chunks.entry(file).or_default();
        chunks.push(chunk);
      }
    }

    let mut assets: HashMap<&String, StatsAsset> = self
      .compilation
      .assets()
      .par_iter()
      .filter_map(|(name, asset)| {
        asset.get_source().map(|source| {
          let mut related = vec![];
          if let Some(source_map) = &asset.info.related.source_map {
            related.push(StatsAssetInfoRelated {
              name: "sourceMap".into(),
              value: [source_map.clone()].into(),
            })
          }
          (
            name,
            StatsAsset {
              r#type: "asset",
              name: name.clone(),
              size: source.size() as f64,
              chunks: Vec::new(),
              chunk_names: Vec::new(),
              info: StatsAssetInfo {
                related,
                chunk_hash: asset.info.chunk_hash.iter().cloned().collect_vec(),
                content_hash: asset.info.content_hash.iter().cloned().collect_vec(),
                minimized: asset.info.minimized,
                immutable: asset.info.immutable,
                javascript_module: asset.info.javascript_module,
                development: asset.info.development,
                hot_module_replacement: asset.info.hot_module_replacement,
                source_filename: asset.info.source_filename.clone(),
              },
              emitted: self.compilation.emitted_assets.contains(name),
            },
          )
        })
      })
      .collect::<HashMap<_, _>>();
    for asset in self.compilation.assets().values() {
      if let Some(source_map) = &asset.get_info().related.source_map {
        assets.remove(source_map);
      }
    }
    for (name, asset) in &mut assets {
      if let Some(chunks) = compilation_file_to_chunks.get(name) {
        asset.chunks = chunks.iter().map(|chunk| chunk.id.clone()).collect();
        asset.chunks.sort_unstable();
        asset.chunk_names = chunks
          .iter()
          .filter_map(|chunk| chunk.name.clone())
          .collect();
        asset.chunk_names.sort_unstable();
      }
    }
    let mut assets: Vec<StatsAsset> = assets.into_values().collect();
    assets.sort_unstable_by(|a, b| {
      if b.size == a.size {
        // a to z
        a.name.cmp(&b.name)
      } else {
        // big to small
        b.size.total_cmp(&a.size)
      }
    });

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
      .into_par_iter()
      .map(|(name, mut files)| {
        files.sort_unstable();
        StatsAssetsByChunkName { name, files }
      })
      .collect();

    (assets, assets_by_chunk_name)
  }

  #[allow(clippy::too_many_arguments)]
  pub fn get_modules<T>(
    &self,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
    used_exports: bool,
    provided_exports: bool,
    f: impl Fn(Vec<StatsModule>) -> T,
  ) -> Result<T> {
    let module_graph = self.compilation.get_module_graph();
    let executor_module_graph = &self
      .compilation
      .module_executor
      .as_ref()
      .map(|executor| executor.make_artifact.get_module_graph());

    let mut modules: Vec<StatsModule> = module_graph
      .modules()
      .values()
      .par_bridge()
      .map(|module| {
        self.get_module(
          &module_graph,
          module,
          reasons,
          module_assets,
          nested_modules,
          source,
          used_exports,
          provided_exports,
          false,
          None,
        )
      })
      .collect::<Result<_>>()?;

    let runtime_modules = self
      .compilation
      .runtime_modules
      .par_iter()
      .map(|(identifier, module)| {
        self.get_runtime_module(identifier, module, reasons, module_assets)
      })
      .collect::<Result<Vec<_>>>()?;
    modules.extend(runtime_modules);

    if let Some(executor_module_graph) = &executor_module_graph {
      let executed_modules: Vec<StatsModule> = executor_module_graph
        .modules()
        .values()
        .par_bridge()
        .map(|module| {
          self.get_module(
            executor_module_graph,
            module,
            reasons,
            module_assets,
            nested_modules,
            source,
            used_exports,
            provided_exports,
            true,
            None,
          )
        })
        .collect::<Result<_>>()?;

      modules.extend(executed_modules);
    }

    if let Some(executed_runtime_modules) = self
      .compilation
      .module_executor
      .as_ref()
      .map(|me| &me.executed_runtime_modules)
    {
      let runtime_modules: Vec<StatsModule> = executed_runtime_modules
        .iter()
        .par_bridge()
        .map(|item| {
          let (id, module) = item.pair();
          self.get_executed_runtime_module(id, module, reasons, module_assets)
        })
        .collect::<Result<_>>()?;
      modules.extend(runtime_modules);
    }

    Self::sort_modules(&mut modules);

    Ok(f(modules))
  }

  #[allow(clippy::too_many_arguments)]
  pub fn get_chunks<T>(
    &self,
    chunk_modules: bool,
    chunk_relations: bool,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
    used_exports: bool,
    provided_exports: bool,
    f: impl Fn(Vec<StatsChunk>) -> T,
  ) -> Result<T> {
    let module_graph = self.compilation.get_module_graph();
    let chunk_graph = &self.compilation.chunk_graph;
    let context = &self.compilation.options.context;
    let chunk_group_by_ukey = &self.compilation.chunk_group_by_ukey;

    let orders = [ChunkGroupOrderKey::Prefetch, ChunkGroupOrderKey::Preload];

    let mut chunks: Vec<StatsChunk> = self
      .compilation
      .chunk_by_ukey
      .values()
      .par_bridge()
      .map(|c| -> Result<_> {
        let files: Vec<_> = {
          let mut vec = c.files.iter().cloned().collect::<Vec<_>>();
          vec.sort_unstable();
          vec
        };

        let root_modules = chunk_graph
          .get_chunk_root_modules(&c.ukey, &module_graph)
          .into_iter()
          .collect::<HashSet<Identifier>>();

        let mut auxiliary_files = Vec::from_iter(c.auxiliary_files.iter().cloned());
        auxiliary_files.sort_unstable();

        let chunk_modules = if chunk_modules {
          let chunk_modules = self
            .compilation
            .chunk_graph
            .get_chunk_modules(&c.ukey, &module_graph);
          let mut chunk_modules = chunk_modules
            .into_iter()
            .map(|m| {
              self.get_module(
                &module_graph,
                m,
                reasons,
                module_assets,
                nested_modules,
                source,
                used_exports,
                provided_exports,
                false,
                Some(&root_modules),
              )
            })
            .collect::<Result<Vec<_>>>()?;
          Self::sort_modules(&mut chunk_modules);
          Some(chunk_modules)
        } else {
          None
        };

        let (parents, children, siblings) = chunk_relations
          .then(|| self.get_chunk_relations(c))
          .map_or((None, None, None), |(parents, children, siblings)| {
            (Some(parents), Some(children), Some(siblings))
          });

        let mut children_by_order = HashMap::<ChunkGroupOrderKey, Vec<String>>::default();
        for order in &orders {
          if let Some(order_chlidren) = c.get_child_ids_by_order(order, self.compilation) {
            children_by_order.insert(order.clone(), order_chlidren);
          }
        }

        let origins = c
          .groups
          .iter()
          .sorted()
          .flat_map(|ukey| {
            let chunk_group = chunk_group_by_ukey.expect_get(ukey);
            chunk_group.origins().iter().map(|origin| {
              let module_identifier = origin
                .module_id
                .map(|id| id.to_string())
                .unwrap_or_default();

              let module_name = origin
                .module_id
                .map(|identifier| {
                  module_graph
                    .module_by_identifier(&identifier)
                    .map(|module| module.readable_identifier(context).to_string())
                    .unwrap_or_default()
                })
                .unwrap_or_default();

              let module_id = origin
                .module_id
                .map(|identifier| {
                  self
                    .compilation
                    .chunk_graph
                    .get_module_id(identifier)
                    .clone()
                    .unwrap_or_default()
                })
                .unwrap_or_default();

              StatsOriginRecord {
                module: module_identifier.clone(),
                module_id,
                module_identifier,
                module_name,
                loc: origin
                  .loc
                  .as_ref()
                  .map(|loc| match loc {
                    OriginLocation::Real(l) => format!("{l}"),
                    OriginLocation::Synthetic(l) => l.name.to_string(),
                  })
                  .unwrap_or_default(),
                request: origin.request.clone().unwrap_or_default(),
              }
            })
          })
          .collect::<Vec<_>>();

        let mut id_hints = c.id_name_hints.iter().cloned().collect_vec();
        id_hints.sort_unstable();

        Ok(StatsChunk {
          r#type: "chunk",
          files,
          auxiliary_files,
          id: c.id.clone(),
          id_hints,
          names: c.name.clone().map(|n| vec![n]).unwrap_or_default(),
          entry: c.has_entry_module(chunk_graph),
          initial: c.can_be_initial(&self.compilation.chunk_group_by_ukey),
          size: chunk_graph.get_chunk_modules_size(&c.ukey, self.compilation),
          modules: chunk_modules,
          parents,
          children,
          siblings,
          children_by_order,
          runtime: c.runtime.clone(),
          sizes: chunk_graph.get_chunk_modules_sizes(&c.ukey, self.compilation),
          reason: c.chunk_reason.clone(),
          rendered: c.rendered,
          origins,
          hash: c.rendered_hash.as_ref().map(|i| i.to_string()),
        })
      })
      .collect::<Result<_>>()?;

    // make result deterministic
    chunks.sort_unstable_by_key(|v| {
      // chunk id only exist after chunkIds hook
      if let Some(id) = &v.id {
        Either::Left(id.clone())
      } else {
        Either::Right(v.size as u32)
      }
    });

    Ok(f(chunks))
  }

  fn get_chunk_group(
    &self,
    name: &str,
    ukey: &ChunkGroupUkey,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> StatsChunkGroup {
    let cg = self.compilation.chunk_group_by_ukey.expect_get(ukey);
    let chunks: Vec<Option<String>> = cg
      .chunks
      .iter()
      .map(|c| self.compilation.chunk_by_ukey.expect_get(c).id.clone())
      .collect();

    let assets = cg
      .chunks
      .par_iter()
      .map(|c| {
        let chunk = self.compilation.chunk_by_ukey.expect_get(c);
        chunk.files.par_iter().map(|file| StatsChunkGroupAsset {
          name: file.clone(),
          size: get_asset_size(file, self.compilation),
        })
      })
      .flatten()
      .collect::<Vec<_>>();

    let auxiliary_assets = cg
      .chunks
      .par_iter()
      .map(|c| {
        let chunk = self.compilation.chunk_by_ukey.expect_get(c);
        chunk
          .auxiliary_files
          .par_iter()
          .map(|file| StatsChunkGroupAsset {
            name: file.clone(),
            size: get_asset_size(file, self.compilation),
          })
      })
      .flatten()
      .collect::<Vec<_>>();

    let children = chunk_group_children.then(|| {
      let ordered_children = cg.get_children_by_orders(self.compilation);
      StatsChunkGroupChildren {
        preload: ordered_children
          .get(&ChunkGroupOrderKey::Preload)
          .expect("should have preload chunk groups")
          .par_iter()
          .map(|ukey| {
            let cg = self.compilation.chunk_group_by_ukey.expect_get(ukey);
            self.get_chunk_group(
              cg.name().unwrap_or_default(),
              ukey,
              chunk_group_auxiliary,
              false,
            )
          })
          .collect::<Vec<_>>(),
        prefetch: ordered_children
          .get(&ChunkGroupOrderKey::Prefetch)
          .expect("should have prefetch chunk groups")
          .par_iter()
          .map(|ukey| {
            let cg = self.compilation.chunk_group_by_ukey.expect_get(ukey);
            self.get_chunk_group(
              cg.name().unwrap_or_default(),
              ukey,
              chunk_group_auxiliary,
              false,
            )
          })
          .collect::<Vec<_>>(),
      }
    });
    StatsChunkGroup {
      name: name.to_string(),
      chunks,
      assets_size: assets.iter().map(|i| i.size).sum(),
      assets,
      auxiliary_assets_size: chunk_group_auxiliary
        .then(|| auxiliary_assets.iter().map(|i| i.size).sum()),
      auxiliary_assets: chunk_group_auxiliary.then_some(auxiliary_assets),
      children,
    }
  }

  pub fn get_entrypoints(
    &self,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> Vec<StatsChunkGroup> {
    self
      .compilation
      .entrypoints
      .par_iter()
      .map(|(name, ukey)| {
        self.get_chunk_group(name, ukey, chunk_group_auxiliary, chunk_group_children)
      })
      .collect()
  }

  pub fn get_named_chunk_groups(
    &self,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> Vec<StatsChunkGroup> {
    let mut named_chunk_groups: Vec<StatsChunkGroup> = self
      .compilation
      .named_chunk_groups
      .par_iter()
      .map(|(name, ukey)| {
        self.get_chunk_group(name, ukey, chunk_group_auxiliary, chunk_group_children)
      })
      .collect();
    named_chunk_groups.sort_by_cached_key(|e| e.name.to_string());
    named_chunk_groups
  }

  pub fn get_errors(&self) -> Vec<StatsError> {
    let module_graph = self.compilation.get_module_graph();
    let mut diagnostic_displayer = DiagnosticDisplayer::new(self.compilation.options.stats.colors);
    self
      .compilation
      .get_errors_sorted()
      .map(|d| {
        let module_identifier = d.module_identifier();
        let (module_name, module_id) = module_identifier
          .and_then(|identifier| {
            let module = module_graph.module_by_identifier(&identifier)?;
            Some(get_stats_module_name_and_id(module, self.compilation))
          })
          .unzip();

        let chunk = d
          .chunk()
          .map(ChunkUkey::from)
          .map(|key| self.compilation.chunk_by_ukey.expect_get(&key));

        StatsError {
          message: diagnostic_displayer
            .emit_diagnostic(d)
            .expect("should print diagnostics"),
          module_identifier: module_identifier.map(|i| i.to_string()),
          module_name,
          module_id: module_id.flatten(),
          file: d.file().map(ToOwned::to_owned),

          chunk_name: chunk.and_then(|c| c.name.clone()),
          chunk_entry: chunk.map(|c| c.has_runtime(&self.compilation.chunk_group_by_ukey)),
          chunk_initial: chunk.map(|c| c.can_be_initial(&self.compilation.chunk_group_by_ukey)),
          chunk_id: chunk.and_then(|c| c.id.clone()),
          details: d.details(),
          stack: d.stack(),
        }
      })
      .collect()
  }

  pub fn get_warnings(&self) -> Vec<StatsWarning> {
    let module_graph = self.compilation.get_module_graph();
    let mut diagnostic_displayer = DiagnosticDisplayer::new(self.compilation.options.stats.colors);
    self
      .compilation
      .get_warnings_sorted()
      .map(|d| {
        let module_identifier = d.module_identifier();
        let (module_name, module_id) = module_identifier
          .and_then(|identifier| {
            let module = module_graph.module_by_identifier(&identifier)?;
            Some(get_stats_module_name_and_id(module, self.compilation))
          })
          .unzip();

        let chunk = d
          .chunk()
          .map(ChunkUkey::from)
          .map(|key| self.compilation.chunk_by_ukey.expect_get(&key));

        StatsWarning {
          message: diagnostic_displayer
            .emit_diagnostic(d)
            .expect("should print diagnostics"),
          module_identifier: module_identifier.map(|i| i.to_string()),
          module_name,
          module_id: module_id.flatten(),
          file: d.file().map(ToOwned::to_owned),

          chunk_name: chunk.and_then(|c| c.name.clone()),
          chunk_entry: chunk.map(|c| c.has_runtime(&self.compilation.chunk_group_by_ukey)),
          chunk_initial: chunk.map(|c| c.can_be_initial(&self.compilation.chunk_group_by_ukey)),
          chunk_id: chunk.and_then(|c| c.id.clone()),
          details: d.details(),
          stack: d.stack(),
        }
      })
      .collect()
  }

  pub fn get_logging(&self) -> Vec<(String, LogType)> {
    self
      .compilation
      .get_logging()
      .iter()
      .map(|item| {
        let (name, logs) = item.pair();
        (name.to_owned(), logs.to_owned())
      })
      .sorted_by(|a, b| a.0.cmp(&b.0))
      .flat_map(|item| item.1.into_iter().map(move |log| (item.0.clone(), log)))
      .collect()
  }

  pub fn get_hash(&self) -> Option<&str> {
    self.compilation.get_hash()
  }

  fn sort_modules(modules: &mut [StatsModule]) {
    modules.sort_unstable_by(|a, b| {
      // align with MODULES_SORTER
      // https://github.com/webpack/webpack/blob/ab3e93b19ead869727592d09d36f94e649eb9d83/lib/stats/DefaultStatsFactoryPlugin.js#L1546
      if a.depth != b.depth {
        a.depth.cmp(&b.depth)
      } else if a.pre_order_index != b.pre_order_index {
        a.pre_order_index.cmp(&b.pre_order_index)
      } else if a.name.len() != b.name.len() {
        a.name.len().cmp(&b.name.len())
      } else {
        a.name.cmp(&b.name)
      }
    });
  }

  #[allow(clippy::too_many_arguments)]
  fn get_module<'a>(
    &'a self,
    module_graph: &'a ModuleGraph,
    module: &'a BoxModule,
    reasons: bool,
    module_assets: bool,
    nested_modules: bool,
    source: bool,
    used_exports: bool,
    provided_exports: bool,
    executed: bool,
    root_modules: Option<&HashSet<Identifier>>,
  ) -> Result<StatsModule<'a>> {
    let identifier = module.identifier();
    let mgm = module_graph
      .module_graph_module_by_identifier(&identifier)
      .unwrap_or_else(|| panic!("Could not find ModuleGraphModule by identifier: {identifier:?}"));

    let issuer = module_graph.get_issuer(&module.identifier());
    let (issuer_name, issuer_id) = issuer
      .map(|i| {
        if executed {
          (
            i.readable_identifier(&self.compilation.options.context)
              .to_string(),
            None,
          )
        } else {
          get_stats_module_name_and_id(i, self.compilation)
        }
      })
      .unzip();
    let mut issuer_path = Vec::new();
    let mut current_issuer = issuer;
    while let Some(i) = current_issuer {
      let (name, id) = if executed {
        (
          i.readable_identifier(&self.compilation.options.context)
            .to_string(),
          None,
        )
      } else {
        get_stats_module_name_and_id(i, self.compilation)
      };
      issuer_path.push(StatsModuleIssuer {
        identifier: i.identifier().to_string(),
        name,
        id,
      });
      current_issuer = module_graph.get_issuer(&i.identifier());
    }
    issuer_path.reverse();

    let module_reasons = reasons
      .then(|| -> Result<_> {
        let mut reasons: Vec<StatsModuleReason> = mgm
          .get_incoming_connections_unordered()
          .iter()
          .filter_map(|connection_id| {
            // the connection is removed
            let connection = module_graph.connection_by_connection_id(connection_id)?;
            let (module_name, module_id) = connection
              .original_module_identifier
              .and_then(|i| module_graph.module_by_identifier(&i))
              .map(|m| {
                if executed {
                  (
                    m.readable_identifier(&self.compilation.options.context)
                      .to_string(),
                    None,
                  )
                } else {
                  get_stats_module_name_and_id(m, self.compilation)
                }
              })
              .unzip();
            let dependency = module_graph.dependency_by_id(&connection.dependency_id);
            let (r#type, user_request) =
              if let Some(d) = dependency.and_then(|d| d.as_module_dependency()) {
                (
                  Some(d.dependency_type().to_string()),
                  Some(d.user_request().to_string()),
                )
              } else if let Some(d) = dependency.and_then(|d| d.as_context_dependency()) {
                (
                  Some(d.dependency_type().to_string()),
                  Some(d.request().to_string()),
                )
              } else {
                (None, None)
              };
            Some(StatsModuleReason {
              module_identifier: connection.original_module_identifier.map(|i| i.to_string()),
              module_name,
              module_id: module_id.and_then(|i| i),
              r#type,
              user_request,
            })
          })
          .collect();
        reasons.sort_unstable();
        Ok(reasons)
      })
      .transpose()?;

    let mut chunks: Vec<Option<String>> = if executed {
      vec![]
    } else {
      self
        .compilation
        .chunk_graph
        .get_chunk_graph_module(mgm.module_identifier)
        .chunks
        .iter()
        .map(|k| self.compilation.chunk_by_ukey.expect_get(k).id.clone())
        .collect()
    };
    chunks.sort_unstable();

    let assets = if executed {
      None
    } else {
      module_assets.then(|| {
        let mut assets: Vec<_> = module
          .build_info()
          .as_ref()
          .map(|info| info.asset_filenames.iter().map(|i| i.to_string()).collect())
          .unwrap_or_default();
        assets.sort();
        assets
      })
    };

    let modules = nested_modules
      .then(|| -> Result<_> {
        let Some(module) = module.as_concatenated_module() else {
          return Ok(Vec::new());
        };
        let mut modules: Vec<StatsModule> = module
          .get_modules()
          .par_iter()
          .filter_map(|m| module_graph.module_by_identifier(&m.id))
          .map(|module| {
            self.get_module(
              module_graph,
              module,
              reasons,
              module_assets,
              nested_modules,
              source,
              used_exports,
              provided_exports,
              executed,
              root_modules,
            )
          })
          .collect::<Result<_>>()?;
        Self::sort_modules(&mut modules);
        Ok(modules)
      })
      .transpose()?;
    let profile = if let Some(p) = mgm.get_profile()
      && let Some(factory) = p.factory.duration()
      && let Some(building) = p.building.duration()
    {
      Some(StatsModuleProfile {
        factory: StatsMillisecond::new(factory.as_secs(), factory.subsec_millis()),
        building: StatsMillisecond::new(building.as_secs(), building.subsec_millis()),
      })
    } else {
      None
    };

    let provided_exports =
      if !executed && provided_exports && self.compilation.options.optimization.provided_exports {
        match self
          .compilation
          .get_module_graph()
          .get_provided_exports(module.identifier())
        {
          ProvidedExports::Vec(v) => Some(v.iter().map(|i| i.to_string()).collect_vec()),
          _ => None,
        }
      } else {
        None
      };

    let used_exports = if !executed
      && used_exports
      && self
        .compilation
        .options
        .optimization
        .used_exports
        .is_enable()
    {
      match self
        .compilation
        .get_module_graph()
        .get_used_exports(&module.identifier(), None)
      {
        UsedExports::Null => Some(StatsUsedExports::Null),
        UsedExports::Vec(v) => Some(StatsUsedExports::Vec(
          v.iter().map(|i| i.to_string()).collect_vec(),
        )),
        UsedExports::Bool(b) => Some(StatsUsedExports::Bool(b)),
      }
    } else {
      None
    };

    let built = self.compilation.built_modules.contains(&identifier);
    let code_generated = self
      .compilation
      .code_generated_modules
      .contains(&identifier);

    let errors = self
      .compilation
      .get_errors()
      .filter(|d| d.module_identifier().is_some_and(|id| id == identifier))
      .count() as u32;

    let warnings = self
      .compilation
      .get_warnings()
      .filter(|d| d.module_identifier().is_some_and(|id| id == identifier))
      .count() as u32;

    let sizes = module
      .source_types()
      .iter()
      .map(|t| StatsSourceTypeSize {
        source_type: *t,
        size: module.size(Some(t), self.compilation),
      })
      .collect_vec();

    let dependent = if let Some(root_modules) = root_modules
      && !executed
    {
      Some(!root_modules.contains(&identifier))
    } else {
      None
    };

    Ok(StatsModule {
      r#type: "module",
      module_type: *module.module_type(),
      identifier,
      depth: module_graph.get_depth(&identifier),
      name_for_condition: module.name_for_condition().map(|n| n.to_string()),
      name: module
        .readable_identifier(&self.compilation.options.context)
        .into(),
      id: if executed {
        None
      } else {
        self
          .compilation
          .chunk_graph
          .get_module_id(identifier)
          .clone()
      },
      chunks,
      size: module.size(None, self.compilation),
      sizes,
      issuer: issuer.map(|i| i.identifier().to_string()),
      issuer_name,
      issuer_id: issuer_id.and_then(|i| i),
      issuer_path,
      reasons: module_reasons,
      assets,
      modules,
      source: source.then(|| module.original_source()).flatten(),
      profile,
      orphan: if executed {
        true
      } else {
        self
          .compilation
          .chunk_graph
          .get_number_of_module_chunks(identifier)
          == 0
      },
      provided_exports,
      used_exports,
      optimization_bailout: mgm.optimization_bailout.clone(),
      pre_order_index: module_graph.get_pre_order_index(&identifier),
      post_order_index: module_graph.get_post_order_index(&identifier),
      built,
      code_generated,
      build_time_executed: executed,
      cached: !built && !code_generated,
      cacheable: module.build_info().is_some_and(|i| i.cacheable),
      optional: module_graph.is_optional(&identifier),
      failed: errors > 0,
      errors,
      warnings,
      dependent,
    })
  }

  fn get_executed_runtime_module(
    &self,
    identifier: &ModuleIdentifier,
    module: &ExecutedRuntimeModule,
    reasons: bool,
    module_assets: bool,
  ) -> Result<StatsModule> {
    let built = false;
    let code_generated = self.compilation.code_generated_modules.contains(identifier);

    Ok(StatsModule {
      r#type: "module",
      depth: None,
      module_type: module.module_type,
      identifier: module.identifier,
      name_for_condition: module.name_for_condition.clone(),
      name: module.name.clone(),
      id: Some(String::new()),
      chunks: vec![],
      size: module.size,
      sizes: vec![StatsSourceTypeSize {
        source_type: SourceType::Custom("runtime".into()),
        size: module.size,
      }],
      issuer: None,
      issuer_name: None,
      issuer_id: None,
      issuer_path: Vec::new(),
      reasons: reasons.then_some(vec![]),
      assets: module_assets.then_some(vec![]),
      modules: None,
      source: None,
      profile: None,
      orphan: true,
      provided_exports: Some(vec![]),
      used_exports: None,
      optimization_bailout: vec![],
      pre_order_index: None,
      post_order_index: None,
      built,
      code_generated,
      build_time_executed: true,
      cached: !built && !code_generated,
      cacheable: module.cacheable,
      optional: false,
      failed: false,
      warnings: 0,
      errors: 0,
      dependent: None,
    })
  }

  fn get_runtime_module<'a>(
    &'a self,
    identifier: &ModuleIdentifier,
    module: &'a BoxRuntimeModule,
    reasons: bool,
    module_assets: bool,
  ) -> Result<StatsModule<'a>> {
    let mut chunks: Vec<Option<String>> = self
      .compilation
      .chunk_graph
      .get_chunk_graph_module(*identifier)
      .chunks
      .iter()
      .map(|k| self.compilation.chunk_by_ukey.expect_get(k).id.clone())
      .collect();
    chunks.sort_unstable();

    let built = false;
    let code_generated = self.compilation.code_generated_modules.contains(identifier);
    let size = self
      .compilation
      .runtime_module_code_generation_results
      .get(identifier)
      .map(|(_, source)| source.size() as f64)
      .unwrap_or(0 as f64);

    Ok(StatsModule {
      r#type: "module",
      depth: None,
      module_type: *module.module_type(),
      identifier: module.identifier(),
      name_for_condition: module.name_for_condition().map(|n| n.to_string()),
      name: module.name().to_string(),
      id: Some(String::new()),
      chunks,
      size,
      sizes: vec![StatsSourceTypeSize {
        source_type: SourceType::Custom("runtime".into()),
        size,
      }],
      issuer: None,
      issuer_name: None,
      issuer_id: None,
      issuer_path: Vec::new(),
      reasons: reasons.then_some(vec![]),
      assets: module_assets.then_some(vec![]),
      modules: None,
      source: None,
      profile: None,
      orphan: self
        .compilation
        .chunk_graph
        .get_number_of_module_chunks(*identifier)
        == 0,
      provided_exports: Some(vec![]),
      used_exports: None,
      optimization_bailout: vec![],
      pre_order_index: None,
      post_order_index: None,
      built,
      code_generated,
      build_time_executed: false,
      cached: !built && !code_generated,
      cacheable: module.cacheable(),
      optional: false,
      failed: false,
      warnings: 0,
      errors: 0,
      dependent: Some(false),
    })
  }

  fn get_chunk_relations(&self, chunk: &Chunk) -> (Vec<String>, Vec<String>, Vec<String>) {
    let compilation = &self.compilation;
    let chunk_group_by_ukey = &compilation.chunk_group_by_ukey;
    let chunk_by_ukey = &compilation.chunk_by_ukey;

    let mut parents = HashSet::default();
    let mut children = HashSet::default();
    let mut siblings = HashSet::default();

    for cg in &chunk.groups {
      if let Some(cg) = get_chunk_group_from_ukey(cg, chunk_group_by_ukey) {
        for p in &cg.parents {
          if let Some(pg) = get_chunk_group_from_ukey(p, chunk_group_by_ukey) {
            for c in &pg.chunks {
              if let Some(c) = get_chunk_from_ukey(c, chunk_by_ukey)
                && let Some(id) = &c.id
              {
                parents.insert(id.to_string());
              }
            }
          }
        }

        for p in &cg.children {
          if let Some(pg) = get_chunk_group_from_ukey(p, chunk_group_by_ukey) {
            for c in &pg.chunks {
              if let Some(c) = get_chunk_from_ukey(c, chunk_by_ukey)
                && let Some(id) = &c.id
              {
                children.insert(id.to_string());
              }
            }
          }
        }

        for c in &cg.chunks {
          if let Some(c) = get_chunk_from_ukey(c, chunk_by_ukey)
            && c.id != chunk.id
            && let Some(id) = &c.id
          {
            siblings.insert(id.to_string());
          }
        }
      }
    }

    let mut parents = Vec::from_iter(parents);
    let mut children = Vec::from_iter(children);
    let mut siblings = Vec::from_iter(siblings);

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
  pub module_identifier: Option<String>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub file: Option<PathBuf>,

  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
}

#[derive(Debug)]
pub struct StatsWarning {
  pub message: String,
  pub module_identifier: Option<String>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub file: Option<PathBuf>,

  pub chunk_name: Option<String>,
  pub chunk_entry: Option<bool>,
  pub chunk_initial: Option<bool>,
  pub chunk_id: Option<String>,
  pub details: Option<String>,
  pub stack: Option<String>,
}

#[derive(Debug)]
pub struct StatsAsset {
  pub r#type: &'static str,
  pub name: String,
  pub size: f64,
  pub chunks: Vec<Option<String>>,
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
  pub minimized: bool,
  pub development: bool,
  pub hot_module_replacement: bool,
  pub source_filename: Option<String>,
  pub immutable: bool,
  pub javascript_module: Option<bool>,
  pub chunk_hash: Vec<String>,
  pub content_hash: Vec<String>,
  pub related: Vec<StatsAssetInfoRelated>,
}

#[derive(Debug)]
pub struct StatsAssetInfoRelated {
  pub name: String,
  pub value: Vec<String>,
}

#[derive(Debug)]
pub struct StatsModule<'a> {
  pub r#type: &'static str,
  pub module_type: ModuleType,
  pub identifier: ModuleIdentifier,
  pub name: String,
  pub name_for_condition: Option<String>,
  pub id: Option<String>,
  pub chunks: Vec<Option<String>>, // has id after the call of chunkIds hook
  pub size: f64,
  pub sizes: Vec<StatsSourceTypeSize>,
  pub dependent: Option<bool>,
  pub issuer: Option<String>,
  pub issuer_name: Option<String>,
  pub issuer_id: Option<String>,
  pub issuer_path: Vec<StatsModuleIssuer>,
  pub reasons: Option<Vec<StatsModuleReason>>,
  pub assets: Option<Vec<String>>,
  pub modules: Option<Vec<StatsModule<'a>>>,
  pub source: Option<&'a dyn Source>,
  pub profile: Option<StatsModuleProfile>,
  pub orphan: bool,
  pub provided_exports: Option<Vec<String>>,
  pub used_exports: Option<StatsUsedExports>,
  pub optimization_bailout: Vec<String>,
  pub depth: Option<usize>,
  pub pre_order_index: Option<u32>,
  pub post_order_index: Option<u32>,
  pub built: bool,
  pub code_generated: bool,
  pub build_time_executed: bool,
  pub cached: bool,
  pub cacheable: bool,
  pub optional: bool,
  pub failed: bool,
  pub errors: u32,
  pub warnings: u32,
}

#[derive(Debug)]
pub enum StatsUsedExports {
  Vec(Vec<String>),
  Bool(bool),
  Null,
}

#[derive(Debug)]
pub struct StatsModuleProfile {
  pub factory: StatsMillisecond,
  pub building: StatsMillisecond,
}

#[derive(Debug)]
pub struct StatsOriginRecord {
  pub module: String,
  pub module_id: String,
  pub module_identifier: String,
  pub module_name: String,
  pub loc: String,
  pub request: String,
}

#[derive(Debug)]
pub struct StatsChunk<'a> {
  pub r#type: &'static str,
  pub files: Vec<String>,
  pub auxiliary_files: Vec<String>,
  pub id: Option<String>,
  pub entry: bool,
  pub initial: bool,
  pub names: Vec<String>,
  pub size: f64,
  pub modules: Option<Vec<StatsModule<'a>>>,
  pub parents: Option<Vec<String>>,
  pub children: Option<Vec<String>>,
  pub siblings: Option<Vec<String>>,
  pub children_by_order: HashMap<ChunkGroupOrderKey, Vec<String>>,
  pub runtime: RuntimeSpec,
  pub sizes: HashMap<SourceType, f64>,
  pub reason: Option<String>,
  pub rendered: bool,
  pub origins: Vec<StatsOriginRecord>,
  pub id_hints: Vec<String>,
  pub hash: Option<String>,
}

#[derive(Debug)]
pub struct StatsChunkGroupAsset {
  pub name: String,
  pub size: f64,
}

#[derive(Debug)]
pub struct StatsChunkGroup {
  pub name: String,
  pub chunks: Vec<Option<String>>,
  pub assets: Vec<StatsChunkGroupAsset>,
  pub assets_size: f64,
  pub auxiliary_assets: Option<Vec<StatsChunkGroupAsset>>,
  pub auxiliary_assets_size: Option<f64>,
  pub children: Option<StatsChunkGroupChildren>,
}

#[derive(Debug)]
pub struct StatsChunkGroupChildren {
  pub preload: Vec<StatsChunkGroup>,
  pub prefetch: Vec<StatsChunkGroup>,
}

#[derive(Debug)]
pub struct StatsModuleIssuer {
  pub identifier: String,
  pub name: String,
  pub id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StatsModuleReason {
  pub module_identifier: Option<String>,
  pub module_name: Option<String>,
  pub module_id: Option<String>,
  pub r#type: Option<String>,
  pub user_request: Option<String>,
}

#[derive(Debug)]
pub struct StatsMillisecond {
  pub secs: u64,
  pub subsec_millis: u32,
}

impl StatsMillisecond {
  pub fn new(secs: u64, subsec_millis: u32) -> Self {
    Self {
      secs,
      subsec_millis,
    }
  }
}

#[derive(Debug)]
pub struct StatsSourceTypeSize {
  pub source_type: SourceType,
  pub size: f64,
}
