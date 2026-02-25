use either::Either;
use itertools::Itertools;
use rayon::iter::{
  IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge,
  ParallelIterator,
};
use rspack_collections::{DatabaseItem, IdentifierSet};
use rspack_error::{Diagnostic, Display, Result, StringDisplayer};
use rustc_hash::FxHashMap as HashMap;

mod utils;
pub use utils::*;
mod r#struct;
pub use r#struct::*;

use crate::{
  BoxModule, BoxRuntimeModule, Chunk, ChunkGraph, ChunkGroupOrderKey, ChunkGroupUkey, ChunkUkey,
  Compilation, ExportsInfoArtifact, LogType, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleIdentifier, PrefetchExportsInfoMode, ProvidedExports, RuntimeSpec, SourceType, UsedExports,
  compilation::build_module_graph::ExecutedRuntimeModule,
};

#[derive(Debug, Clone)]
pub struct Stats<'compilation> {
  pub compilation: &'compilation Compilation,
  pub exports_info_artifact: &'compilation ExportsInfoArtifact,
}

impl<'compilation> Stats<'compilation> {
  pub fn new(
    compilation: &'compilation Compilation,
    exports_info_artifact: &'compilation ExportsInfoArtifact,
  ) -> Self {
    Self {
      compilation,
      exports_info_artifact,
    }
  }
}

impl Stats<'_> {
  pub fn get_assets(&self) -> (Vec<StatsAsset<'_>>, Vec<StatsAssetsByChunkName<'_>>) {
    let mut compilation_file_to_chunks: HashMap<&String, Vec<&Chunk>> = HashMap::default();
    let mut compilation_file_to_auxiliary_chunks: HashMap<&String, Vec<&Chunk>> =
      HashMap::default();
    for chunk in self
      .compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .values()
    {
      for file in chunk.files() {
        let chunks = compilation_file_to_chunks.entry(file).or_default();
        chunks.push(chunk);
      }

      for file in chunk.auxiliary_files() {
        let auxiliary_chunks = compilation_file_to_auxiliary_chunks
          .entry(file)
          .or_default();
        auxiliary_chunks.push(chunk);
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
              name: "sourceMap",
              value: vec![source_map.as_str()],
            })
          }
          (
            name,
            StatsAsset {
              r#type: "asset",
              name: name.as_str(),
              size: source.size() as f64,
              chunks: Vec::new(),
              chunk_names: Vec::new(),
              chunk_id_hints: Vec::new(),
              auxiliary_chunks: Vec::new(),
              auxiliary_chunk_id_hints: Vec::new(),
              auxiliary_chunk_names: Vec::new(),
              info: StatsAssetInfo {
                related,
                full_hash: asset
                  .info
                  .full_hash
                  .iter()
                  .map(|s| s.as_str())
                  .collect_vec(),
                chunk_hash: asset
                  .info
                  .chunk_hash
                  .iter()
                  .map(|s| s.as_str())
                  .collect_vec(),
                content_hash: asset
                  .info
                  .content_hash
                  .iter()
                  .map(|s| s.as_str())
                  .collect_vec(),
                minimized: asset.info.minimized,
                immutable: asset.info.immutable,
                javascript_module: asset.info.javascript_module,
                development: asset.info.development,
                hot_module_replacement: asset.info.hot_module_replacement,
                source_filename: asset.info.source_filename.as_deref(),
                copied: asset.info.copied,
                is_over_size_limit: asset.info.is_over_size_limit,
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
    assets.par_iter_mut().for_each(|(name, asset)| {
      if let Some(chunks) = compilation_file_to_chunks.get(name) {
        asset.chunks = chunks
          .par_iter()
          .map(|chunk| chunk.id().map(|id| id.as_str()))
          .collect();
        asset.chunks.sort_unstable();
        asset.chunk_names = chunks
          .par_iter()
          .filter_map(|chunk| chunk.name())
          .collect::<Vec<_>>();
        asset.chunk_names.sort_unstable();
        asset.chunk_id_hints = chunks
          .par_iter()
          .flat_map(|chunk| {
            chunk
              .id_name_hints()
              .iter()
              .map(|name_hint| name_hint.as_str())
              .collect_vec()
          })
          .collect::<Vec<_>>();
        asset.chunk_id_hints.sort_unstable();
      }

      if let Some(auxiliary_chunks) = compilation_file_to_auxiliary_chunks.get(name) {
        asset.auxiliary_chunks = auxiliary_chunks
          .par_iter()
          .map(|chunk| chunk.id().map(|id| id.as_str()))
          .collect();
        asset.auxiliary_chunks.sort_unstable();
        asset.auxiliary_chunk_names = auxiliary_chunks
          .par_iter()
          .filter_map(|chunk| chunk.name())
          .collect::<Vec<_>>();
        asset.auxiliary_chunk_names.sort_unstable();
        asset.auxiliary_chunk_id_hints = auxiliary_chunks
          .par_iter()
          .flat_map(|chunk| {
            chunk
              .id_name_hints()
              .iter()
              .map(|name_hint| name_hint.as_str())
              .collect_vec()
          })
          .collect::<Vec<_>>();
        asset.auxiliary_chunk_id_hints.sort_unstable();
      }
    });
    let mut assets: Vec<StatsAsset> = assets.into_values().collect();
    assets.sort_unstable_by(|a, b| {
      if b.size == a.size {
        // a to z
        a.name.cmp(b.name)
      } else {
        // big to small
        b.size.total_cmp(&a.size)
      }
    });

    let mut assets_by_chunk_name: HashMap<&str, Vec<&str>> = HashMap::default();
    for (file, chunks) in compilation_file_to_chunks {
      for chunk in chunks {
        if let Some(name) = chunk.name() {
          if let Some(assets) = assets_by_chunk_name.get_mut(name) {
            assets.push(file);
          } else {
            assets_by_chunk_name.insert(name, vec![file]);
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
    options: &ExtendedStatsOptions,
    f: impl Fn(Vec<StatsModule>) -> T,
  ) -> Result<T> {
    let module_graph = self.compilation.get_module_graph();
    let module_graph_cache = &self.compilation.module_graph_cache_artifact;

    let mut modules: Vec<StatsModule> = module_graph
      .modules()
      .map(|(_, module)| module)
      .par_bridge()
      .map(|module| {
        self.get_module(
          module_graph,
          module_graph_cache,
          self.exports_info_artifact,
          module,
          false,
          None,
          None,
          options,
        )
      })
      .collect::<Result<_>>()?;

    let runtime_modules = self
      .compilation
      .runtime_modules
      .par_iter()
      .map(|(identifier, module)| self.get_runtime_module(identifier, module, options))
      .collect::<Result<Vec<_>>>()?;
    modules.extend(runtime_modules);

    let executor_module_graph = self
      .compilation
      .module_executor
      .as_ref()
      .map(|executor| executor.make_artifact.get_module_graph());
    let executor_module_graph_cache = ModuleGraphCacheArtifact::default();
    let executor_exports_info_artifact = self
      .compilation
      .module_executor
      .as_ref()
      .map(|executor| &executor.exports_info_artifact);
    if let Some(executor_module_graph) = &executor_module_graph
      && let Some(executor_exports_info_artifact) = executor_exports_info_artifact
    {
      let executed_modules: Vec<StatsModule> = executor_module_graph
        .modules()
        .map(|(_, module)| module)
        .par_bridge()
        .map(|module| {
          self.get_module(
            executor_module_graph,
            &executor_module_graph_cache,
            executor_exports_info_artifact,
            module,
            true,
            None,
            None,
            options,
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
          self.get_executed_runtime_module(id, module, options)
        })
        .collect::<Result<_>>()?;
      modules.extend(runtime_modules);
    }

    sort_modules(&mut modules);

    Ok(f(modules))
  }

  #[allow(clippy::too_many_arguments)]
  pub fn get_chunks<T>(
    &self,
    options: &ExtendedStatsOptions,
    f: impl Fn(Vec<StatsChunk>) -> T,
  ) -> Result<T> {
    let module_graph = self.compilation.get_module_graph();
    let module_graph_cache = &self.compilation.module_graph_cache_artifact;
    let chunk_graph = &self.compilation.build_chunk_graph_artifact.chunk_graph;
    let context = &self.compilation.options.context;
    let chunk_group_by_ukey = &self
      .compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey;

    let orders = [ChunkGroupOrderKey::Prefetch, ChunkGroupOrderKey::Preload];

    let mut chunks: Vec<StatsChunk> = self
      .compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .values()
      .par_bridge()
      .map(|c| -> Result<_> {
        let files: Vec<_> = {
          let mut vec = c.files().iter().map(|s| s.as_str()).collect::<Vec<_>>();
          vec.sort_unstable();
          vec
        };

        let root_modules = chunk_graph
          .get_chunk_root_modules(
            &c.ukey(),
            module_graph,
            module_graph_cache,
            self.exports_info_artifact,
          )
          .into_iter()
          .collect::<IdentifierSet>();

        let mut auxiliary_files = c
          .auxiliary_files()
          .iter()
          .map(|s| s.as_str())
          .collect::<Vec<_>>();
        auxiliary_files.sort_unstable();

        let chunk_modules = if options.chunk_modules {
          let chunk_modules = self
            .compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .get_chunk_modules(&c.ukey(), module_graph);
          let mut chunk_modules = chunk_modules
            .into_iter()
            .map(|m| {
              self.get_module(
                module_graph,
                module_graph_cache,
                self.exports_info_artifact,
                m,
                false,
                Some(&root_modules),
                Some(c.runtime()),
                options,
              )
            })
            .collect::<Result<Vec<_>>>()?;
          sort_modules(&mut chunk_modules);
          Some(chunk_modules)
        } else {
          None
        };

        let (parents, children, siblings) = options
          .chunk_relations
          .then(|| get_chunk_relations(c, self.compilation))
          .map_or((None, None, None), |(parents, children, siblings)| {
            (Some(parents), Some(children), Some(siblings))
          });

        let mut children_by_order = HashMap::<ChunkGroupOrderKey, Vec<String>>::default();
        let chunk_filter = |_: &ChunkUkey, __: &Compilation| true;
        for order in &orders {
          if let Some(order_children) =
            c.get_child_ids_by_order(order, self.compilation, &chunk_filter)
          {
            children_by_order.insert(
              order.clone(),
              order_children
                .into_iter()
                .map(|id| id.to_string())
                .collect(),
            );
          }
        }

        let origins = c
          .groups()
          .iter()
          .sorted()
          .flat_map(|ukey| {
            let chunk_group = chunk_group_by_ukey.expect_get(ukey);
            chunk_group.origins().iter().map(|origin| {
              let module_identifier = origin.module;

              let module_name = origin
                .module
                .map(|identifier| {
                  module_graph
                    .module_by_identifier(&identifier)
                    .map(|module| module.readable_identifier(context))
                    .unwrap_or_default()
                })
                .unwrap_or_default();

              let module_id = origin.module.and_then(|identifier| {
                ChunkGraph::get_module_id(&self.compilation.module_ids_artifact, identifier)
                  .cloned()
              });

              StatsOriginRecord {
                module: module_identifier,
                module_id,
                module_identifier,
                module_name,
                loc: origin
                  .loc
                  .as_ref()
                  .map(|loc| loc.to_string())
                  .unwrap_or_default(),
                request: origin.request.as_deref().unwrap_or_default(),
              }
            })
          })
          .collect::<Vec<_>>();

        let mut id_hints = c.id_name_hints().iter().map(|s| s.as_str()).collect_vec();
        id_hints.sort_unstable();

        Ok(StatsChunk {
          r#type: "chunk",
          files,
          auxiliary_files,
          id: c.id().map(|id| id.as_str()),
          id_hints,
          names: c.name().map(|n| vec![n]).unwrap_or_default(),
          entry: c.has_entry_module(chunk_graph),
          initial: c.can_be_initial(
            &self
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
          ),
          size: chunk_graph.get_chunk_modules_size(&c.ukey(), self.compilation),
          modules: chunk_modules,
          parents,
          children,
          siblings,
          children_by_order,
          runtime: c.runtime(),
          sizes: chunk_graph.get_chunk_modules_sizes(&c.ukey(), self.compilation),
          reason: c.chunk_reason(),
          rendered: c.rendered(),
          origins,
          hash: c.rendered_hash(
            &self.compilation.chunk_hashes_artifact,
            self.compilation.options.output.hash_digest_length,
          ),
        })
      })
      .collect::<Result<_>>()?;

    // make result deterministic
    chunks.sort_unstable_by_key(|v| {
      // chunk id only exist after chunkIds hook
      if let Some(id) = &v.id {
        Either::Left(*id)
      } else {
        Either::Right(v.size as u32)
      }
    });

    Ok(f(chunks))
  }

  fn get_chunk_group<'a>(
    &'a self,
    name: &'a str,
    ukey: &ChunkGroupUkey,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> StatsChunkGroup<'a> {
    let cg = self
      .compilation
      .build_chunk_graph_artifact
      .chunk_group_by_ukey
      .expect_get(ukey);
    let chunks: Vec<&'a str> = cg
      .chunks
      .iter()
      .filter_map(|c| {
        self
          .compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(c)
          .id()
          .map(|id| id.as_str())
      })
      .collect();

    let assets = cg
      .chunks
      .par_iter()
      .map(|c| {
        let chunk = self
          .compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(c);
        chunk.files().par_iter().map(|file| StatsChunkGroupAsset {
          name: file.as_str(),
          size: get_asset_size(file, self.compilation),
        })
      })
      .flatten()
      .collect::<Vec<_>>();

    let auxiliary_assets = if chunk_group_auxiliary {
      cg.chunks
        .par_iter()
        .map(|c| {
          let chunk = self
            .compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(c);
          chunk
            .auxiliary_files()
            .par_iter()
            .map(|file| StatsChunkGroupAsset {
              name: file.as_str(),
              size: get_asset_size(file, self.compilation),
            })
        })
        .flatten()
        .collect::<Vec<_>>()
    } else {
      vec![]
    };

    let children_info = chunk_group_children.then(|| {
      let ordered_children = cg.get_children_by_orders(self.compilation);
      (
        StatsChunkGroupChildren {
          preload: get_chunk_group_ordered_children(
            self,
            &ordered_children,
            &ChunkGroupOrderKey::Preload,
            &self
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
            chunk_group_auxiliary,
          ),
          prefetch: get_chunk_group_ordered_children(
            self,
            &ordered_children,
            &ChunkGroupOrderKey::Prefetch,
            &self
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
            chunk_group_auxiliary,
          ),
        },
        StatschunkGroupChildAssets {
          preload: get_chunk_group_oreded_child_assets(
            &ordered_children,
            &ChunkGroupOrderKey::Preload,
            &self
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
            &self.compilation.build_chunk_graph_artifact.chunk_by_ukey,
          ),
          prefetch: get_chunk_group_oreded_child_assets(
            &ordered_children,
            &ChunkGroupOrderKey::Prefetch,
            &self
              .compilation
              .build_chunk_graph_artifact
              .chunk_group_by_ukey,
            &self.compilation.build_chunk_graph_artifact.chunk_by_ukey,
          ),
        },
      )
    });

    let (children, child_assets) = match children_info {
      Some(children_info) => (Some(children_info.0), Some(children_info.1)),
      None => (None, None),
    };

    StatsChunkGroup {
      name,
      chunks,
      assets_size: assets.iter().map(|i| i.size).sum(),
      assets,
      auxiliary_assets_size: chunk_group_auxiliary
        .then(|| auxiliary_assets.iter().map(|i| i.size).sum()),
      auxiliary_assets: chunk_group_auxiliary.then_some(auxiliary_assets),
      children,
      child_assets,
      is_over_size_limit: cg.is_over_size_limit,
    }
  }

  pub fn get_entrypoints(
    &self,
    chunk_group_auxiliary: bool,
    chunk_group_children: bool,
  ) -> Vec<StatsChunkGroup<'_>> {
    self
      .compilation
      .build_chunk_graph_artifact
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
  ) -> Vec<StatsChunkGroup<'_>> {
    let mut named_chunk_groups: Vec<StatsChunkGroup> = self
      .compilation
      .build_chunk_graph_artifact
      .named_chunk_groups
      .par_iter()
      .map(|(name, ukey)| {
        self.get_chunk_group(name, ukey, chunk_group_auxiliary, chunk_group_children)
      })
      .collect();
    named_chunk_groups.sort_by_cached_key(|e| e.name.to_string());
    named_chunk_groups
  }

  pub fn get_errors<T>(&self, f: impl Fn(Vec<StatsError>) -> T) -> T {
    let diagnostic_displayer = StringDisplayer::new(self.compilation.options.stats.colors, false);

    let module_graph = self.compilation.get_module_graph();

    let errors = self
      .compilation
      .get_errors_sorted()
      .map(|d| {
        let module_identifier = d.module_identifier;
        let (module_name, module_id) = module_identifier
          .as_ref()
          .and_then(move |identifier| {
            Some(get_stats_module_name_and_id(
              self.compilation.module_by_identifier(identifier)?,
              self.compilation,
            ))
          })
          .unzip();

        let chunk = d.chunk.map(ChunkUkey::from).map(|key| {
          self
            .compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(&key)
        });

        let module_trace = get_module_trace(
          module_identifier,
          module_graph,
          self.compilation,
          &self.compilation.options,
        );
        let code = d.code.clone();
        StatsError {
          name: code.clone(),
          message: diagnostic_displayer
            .emit_diagnostic(d)
            .expect("should print diagnostics"),
          code,
          module_identifier,
          module_name,
          module_id: module_id.flatten(),
          loc: d.loc.as_ref().map(|loc| loc.to_string()),
          file: d.file.as_ref().map(|file| file.as_path()),

          chunk_name: chunk.and_then(|c| c.name()),
          chunk_entry: chunk.map(|c| {
            c.has_runtime(
              &self
                .compilation
                .build_chunk_graph_artifact
                .chunk_group_by_ukey,
            )
          }),
          chunk_initial: chunk.map(|c| {
            c.can_be_initial(
              &self
                .compilation
                .build_chunk_graph_artifact
                .chunk_group_by_ukey,
            )
          }),
          chunk_id: chunk.and_then(|c| c.id().map(|id| id.as_str())),
          details: d.details.clone(),
          stack: d.stack.clone(),
          module_trace,
        }
      })
      .collect::<Vec<_>>();

    f(errors)
  }

  pub fn get_warnings<T>(&self, f: impl Fn(Vec<StatsError>) -> T) -> T {
    let diagnostic_displayer = StringDisplayer::new(self.compilation.options.stats.colors, false);

    let module_graph = self.compilation.get_module_graph();

    let warnings = self
      .compilation
      .get_warnings_sorted()
      .map(|d| {
        let module_identifier = d.module_identifier;
        let (module_name, module_id) = module_identifier
          .as_ref()
          .and_then(|identifier| {
            Some(get_stats_module_name_and_id(
              self.compilation.module_by_identifier(identifier)?,
              self.compilation,
            ))
          })
          .unzip();

        let chunk = d.chunk.map(ChunkUkey::from).map(|key| {
          self
            .compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(&key)
        });

        let module_trace = get_module_trace(
          module_identifier,
          module_graph,
          self.compilation,
          &self.compilation.options,
        );

        let code = d.code.clone();

        StatsError {
          name: code.clone(),
          message: diagnostic_displayer
            .emit_diagnostic(d)
            .expect("should print diagnostics"),
          code,
          module_identifier,
          module_name,
          module_id: module_id.flatten(),
          loc: d.loc.as_ref().map(|loc| loc.to_string()),
          file: d.file.as_ref().map(|file| file.as_path()),

          chunk_name: chunk.and_then(|c| c.name()),
          chunk_entry: chunk.map(|c| {
            c.has_runtime(
              &self
                .compilation
                .build_chunk_graph_artifact
                .chunk_group_by_ukey,
            )
          }),
          chunk_initial: chunk.map(|c| {
            c.can_be_initial(
              &self
                .compilation
                .build_chunk_graph_artifact
                .chunk_group_by_ukey,
            )
          }),
          chunk_id: chunk.and_then(|c| c.id().map(|id| id.as_str())),
          details: d.details.clone(),
          stack: d.stack.clone(),
          module_trace,
        }
      })
      .collect::<Vec<_>>();

    f(warnings)
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

  #[allow(clippy::too_many_arguments)]
  fn get_module<'a>(
    &'a self,
    module_graph: &'a ModuleGraph,
    module_graph_cache: &'a ModuleGraphCacheArtifact,
    exports_info_artifact: &'a ExportsInfoArtifact,
    module: &'a BoxModule,
    executed: bool,
    root_modules: Option<&IdentifierSet>,
    runtime: Option<&RuntimeSpec>,
    options: &ExtendedStatsOptions,
  ) -> Result<StatsModule<'a>> {
    let identifier = module.identifier();
    let mgm = module_graph
      .module_graph_module_by_identifier(&identifier)
      .unwrap_or_else(|| panic!("Could not find ModuleGraphModule by identifier: {identifier:?}"));

    let built = if executed {
      self
        .compilation
        .module_executor
        .as_ref()
        .map(|executor| executor.make_artifact.built_modules().contains(&identifier))
        .unwrap_or_default()
    } else {
      self
        .compilation
        .build_module_graph_artifact
        .built_modules()
        .contains(&identifier)
    };

    let code_generated = self
      .compilation
      .code_generated_modules
      .contains(&identifier);

    let sizes = module
      .source_types(module_graph)
      .iter()
      .map(|t| StatsSourceTypeSize {
        source_type: *t,
        size: module.size(Some(t), Some(self.compilation)),
      })
      .collect_vec();

    let issuer = module_graph.get_issuer(&module.identifier());
    let (issuer_name, issuer_id) = issuer
      .map(|i| {
        if executed {
          (
            i.readable_identifier(&self.compilation.options.context),
            None,
          )
        } else {
          get_stats_module_name_and_id(i, self.compilation)
        }
      })
      .unzip();

    let mut stats = StatsModule {
      r#type: "module",
      module_type: *module.module_type(),
      layer: module.get_layer().map(|layer| layer.into()),
      size: module.size(None, Some(self.compilation)),
      sizes,
      built,
      code_generated,
      build_time_executed: executed,
      cached: !built && !code_generated,
      identifier: None,
      name: None,
      name_for_condition: None,
      id: None,
      chunks: None,
      dependent: None,
      issuer: None,
      issuer_name: None,
      issuer_id: None,
      issuer_path: None,
      reasons: None,
      assets: None,
      modules: None,
      source: None,
      orphan: None,
      provided_exports: None,
      used_exports: None,
      optimization_bailout: None,
      depth: None,
      pre_order_index: None,
      post_order_index: None,
      cacheable: None,
      optional: None,
      failed: None,
      errors: None,
      warnings: None,
    };

    // module$visible
    if stats.built || stats.code_generated || options.cached_modules {
      let orphan = if executed {
        true
      } else {
        self
          .compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_module_chunks(identifier)
          == 0
      };

      let dependent = if let Some(root_modules) = root_modules
        && !executed
      {
        Some(!root_modules.contains(&identifier))
      } else {
        None
      };

      let mut issuer_path = Vec::new();
      let mut current_issuer = issuer;
      while let Some(i) = current_issuer {
        let (name, id) = if executed {
          (
            i.readable_identifier(&self.compilation.options.context),
            None,
          )
        } else {
          get_stats_module_name_and_id(i, self.compilation)
        };
        issuer_path.push(StatsModuleIssuer {
          identifier: i.identifier(),
          name,
          id,
        });
        current_issuer = module_graph.get_issuer(&i.identifier());
      }
      issuer_path.reverse();

      let errors = self
        .compilation
        .get_errors()
        .filter(|d| d.module_identifier.is_some_and(|id| id == identifier))
        .count() as u32;

      let warnings = self
        .compilation
        .get_warnings()
        .filter(|d| d.module_identifier.is_some_and(|id| id == identifier))
        .count() as u32;

      stats.identifier = Some(identifier);
      stats.name = Some(module.readable_identifier(&self.compilation.options.context));
      stats.name_for_condition = module.name_for_condition().map(|n| n.to_string());
      stats.pre_order_index = module_graph.get_pre_order_index(&identifier);
      stats.post_order_index = module_graph.get_post_order_index(&identifier);
      stats.cacheable = Some(module.build_info().cacheable);
      stats.optional =
        Some(module_graph.is_optional(&identifier, module_graph_cache, exports_info_artifact));
      stats.orphan = Some(orphan);
      stats.dependent = dependent;
      stats.issuer = issuer.map(|i| i.identifier());
      stats.issuer_name = issuer_name;
      stats.issuer_path = Some(issuer_path);
      stats.failed = Some(errors > 0);
      stats.errors = Some(errors);
      stats.warnings = Some(warnings);
    }

    if options.ids {
      stats.id = if executed {
        None
      } else {
        ChunkGraph::get_module_id(&self.compilation.module_ids_artifact, identifier).cloned()
      };
      stats.issuer_id = issuer_id.flatten();

      let mut chunks: Vec<&str> = if executed {
        vec![]
      } else {
        self
          .compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_chunk_graph_module(mgm.module_identifier)
          .map(|cgm| {
            cgm
              .chunks
              .iter()
              .filter_map(|k| {
                self
                  .compilation
                  .build_chunk_graph_artifact
                  .chunk_by_ukey
                  .expect_get(k)
                  .id()
                  .map(|id| id.as_str())
              })
              .collect::<Vec<_>>()
          })
          .unwrap_or_default()
      };
      chunks.sort_unstable();
      stats.chunks = Some(chunks);
    }

    if options.module_assets {
      stats.assets = if executed {
        None
      } else {
        let module = module_graph
          .module_by_identifier(&identifier)
          .expect("should have module");
        let mut assets = module
          .build_info()
          .assets
          .keys()
          .map(|s| s.as_str())
          .collect_vec();
        assets.sort_unstable();
        Some(assets)
      };
    }

    if options.reasons {
      let mut reasons: Vec<StatsModuleReason> = mgm
        .incoming_connections()
        .iter()
        .filter_map(|dep_id| {
          // the connection is removed
          let connection = module_graph.connection_by_dependency_id(dep_id)?;
          let (module_name, module_id) = connection
            .original_module_identifier
            .and_then(|i| module_graph.module_by_identifier(&i))
            .map(|m| {
              if executed {
                (
                  m.readable_identifier(&self.compilation.options.context),
                  None,
                )
              } else {
                get_stats_module_name_and_id(m, self.compilation)
              }
            })
            .unzip();
          let (resolved_module_name, resolved_module_id) = connection
            .resolved_original_module_identifier
            .and_then(|i| module_graph.module_by_identifier(&i))
            .map(|m| {
              if executed {
                (
                  m.readable_identifier(&self.compilation.options.context),
                  None,
                )
              } else {
                get_stats_module_name_and_id(m, self.compilation)
              }
            })
            .unzip();
          let dependency = module_graph.dependency_by_id(&connection.dependency_id);
          let (r#type, user_request) = if let Some(d) = dependency.as_module_dependency() {
            (Some(d.dependency_type().as_str()), Some(d.user_request()))
          } else if let Some(d) = dependency.as_context_dependency() {
            (Some(d.dependency_type().as_str()), Some(d.request()))
          } else {
            (None, None)
          };
          let loc = dependency.loc().map(|l| l.to_string());
          let explanation = module_graph
            .get_dep_meta_if_existing(&connection.dependency_id)
            .and_then(|extra| extra.explanation);
          Some(StatsModuleReason {
            module_identifier: connection.original_module_identifier,
            module_name,
            module_id: module_id.flatten(),
            module_chunks: connection.original_module_identifier.and_then(|id| {
              if self
                .compilation
                .build_chunk_graph_artifact
                .chunk_graph
                .chunk_graph_module_by_module_identifier
                .contains_key(&id)
              {
                Some(
                  self
                    .compilation
                    .build_chunk_graph_artifact
                    .chunk_graph
                    .get_number_of_module_chunks(id) as u32,
                )
              } else {
                None
              }
            }),
            resolved_module_identifier: connection.resolved_original_module_identifier,
            resolved_module_name,
            resolved_module_id: resolved_module_id.and_then(|i| i),
            r#type,
            user_request,
            explanation,
            active: connection.is_active(
              module_graph,
              runtime,
              module_graph_cache,
              exports_info_artifact,
            ),
            loc,
          })
        })
        .collect();
      reasons.sort_unstable();
      stats.reasons = Some(reasons);
    }

    if options.used_exports {
      stats.used_exports = if !executed
        && self
          .compilation
          .options
          .optimization
          .used_exports
          .is_enable()
      {
        let exports_info = exports_info_artifact
          .get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);
        let used_exports = exports_info.get_used_exports(None);
        match used_exports {
          UsedExports::Unknown => Some(StatsUsedExports::Null),
          UsedExports::UsedNames(v) => Some(StatsUsedExports::Vec(v)),
          UsedExports::UsedNamespace(b) => Some(StatsUsedExports::Bool(b)),
        }
      } else {
        None
      };
    }

    if options.provided_exports {
      stats.provided_exports =
        if !executed && self.compilation.options.optimization.provided_exports {
          let exports_info = exports_info_artifact
            .get_prefetched_exports_info(&module.identifier(), PrefetchExportsInfoMode::Default);
          let provided_exports = exports_info.get_provided_exports();
          match provided_exports {
            ProvidedExports::ProvidedNames(v) => Some(v),
            _ => None,
          }
        } else {
          None
        };
    }

    if options.optimization_bailout {
      stats.optimization_bailout = Some(&mgm.optimization_bailout);
    }

    // 'depth' is used for sorting in the JavaScript side, so it should always be computed.
    stats.depth = module_graph.get_depth(&identifier);

    if options.nested_modules
      && let Some(module) = module.as_concatenated_module()
    {
      let mut modules: Vec<StatsModule> = module
        .get_modules()
        .par_iter()
        .filter_map(|m| module_graph.module_by_identifier(&m.id))
        .map(|module| {
          self.get_module(
            module_graph,
            module_graph_cache,
            exports_info_artifact,
            module,
            executed,
            root_modules,
            runtime,
            options,
          )
        })
        .collect::<Result<_>>()?;
      sort_modules(&mut modules);
      stats.modules = Some(modules);
    };

    if options.source {
      stats.source = module.source();
    }

    Ok(stats)
  }

  fn get_executed_runtime_module(
    &self,
    identifier: &ModuleIdentifier,
    module: &ExecutedRuntimeModule,
    options: &ExtendedStatsOptions,
  ) -> Result<StatsModule<'_>> {
    let built = false;
    let code_generated = self.compilation.code_generated_modules.contains(identifier);

    let mut stats = StatsModule {
      r#type: "module",
      module_type: module.module_type,
      layer: None,
      size: module.size,
      sizes: vec![StatsSourceTypeSize {
        source_type: SourceType::Custom("runtime".into()),
        size: module.size,
      }],
      built,
      code_generated,
      build_time_executed: true,
      cached: !built && !code_generated,
      identifier: None,
      name: None,
      name_for_condition: None,
      id: None,
      chunks: None,
      dependent: None,
      issuer: None,
      issuer_name: None,
      issuer_id: None,
      issuer_path: None,
      reasons: None,
      assets: None,
      modules: None,
      source: None,
      orphan: None,
      provided_exports: None,
      used_exports: None,
      optimization_bailout: None,
      depth: None,
      pre_order_index: None,
      post_order_index: None,
      cacheable: None,
      optional: None,
      failed: None,
      errors: None,
      warnings: None,
    };

    // module$visible
    if stats.built || stats.code_generated || options.cached_modules {
      stats.identifier = Some(module.identifier);
      stats.name = Some(module.name.clone().into());
      stats
        .name_for_condition
        .clone_from(&module.name_for_condition);
      stats.cacheable = Some(module.cacheable);
      stats.optional = Some(false);
      stats.orphan = Some(true);
      stats.issuer = None;
      stats.issuer_name = None;
      stats.issuer_path = None;
      stats.failed = Some(false);
      stats.errors = Some(0);
      stats.warnings = Some(0);
    }

    if options.ids {
      stats.id = Some("".into());
      stats.chunks = Some(vec![]);
    }

    if options.reasons {
      stats.reasons = Some(vec![]);
    }

    if options.module_assets {
      stats.assets = Some(vec![]);
    }

    if options.used_exports {
      stats.used_exports = Some(StatsUsedExports::Null)
    }

    if options.provided_exports {
      stats.provided_exports = Some(vec![]);
    }

    if options.optimization_bailout {
      stats.optimization_bailout = Some(Default::default());
    }

    Ok(stats)
  }

  fn get_runtime_module<'a>(
    &'a self,
    identifier: &ModuleIdentifier,
    module: &'a BoxRuntimeModule,
    options: &ExtendedStatsOptions,
  ) -> Result<StatsModule<'a>> {
    let mut chunks: Vec<&str> = self
      .compilation
      .build_chunk_graph_artifact
      .chunk_graph
      .get_module_chunks(*identifier)
      .iter()
      .filter_map(|k| {
        self
          .compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(k)
          .id()
          .map(|id| id.as_str())
      })
      .collect();
    chunks.sort_unstable();

    let built = false;
    let code_generated = self.compilation.code_generated_modules.contains(identifier);
    let size = self
      .compilation
      .runtime_modules_code_generation_source
      .get(identifier)
      .map_or(0 as f64, |source| source.size() as f64);

    let mut stats = StatsModule {
      r#type: "module",
      module_type: *module.module_type(),
      layer: module.get_layer().map(|layer| layer.into()),
      size,
      sizes: vec![StatsSourceTypeSize {
        source_type: SourceType::Custom("runtime".into()),
        size,
      }],
      built,
      code_generated,
      build_time_executed: false,
      cached: !built && !code_generated,
      identifier: None,
      name: None,
      name_for_condition: None,
      id: None,
      chunks: None,
      dependent: None,
      issuer: None,
      issuer_name: None,
      issuer_id: None,
      issuer_path: None,
      reasons: None,
      assets: None,
      modules: None,
      source: None,
      orphan: None,
      provided_exports: None,
      used_exports: None,
      optimization_bailout: None,
      depth: None,
      pre_order_index: None,
      post_order_index: None,
      cacheable: None,
      optional: None,
      failed: None,
      errors: None,
      warnings: None,
    };

    if stats.built || stats.code_generated || options.cached_modules {
      let orphan = self
        .compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_number_of_module_chunks(*identifier)
        == 0;

      stats.identifier = Some(module.identifier());
      stats.name = Some(module.readable_identifier(&self.compilation.options.context));
      stats.name_for_condition = module.name_for_condition().map(|n| n.to_string());
      stats.cacheable = Some(!(module.full_hash() || module.dependent_hash()));
      stats.optional = Some(false);
      stats.orphan = Some(orphan);
      stats.dependent = Some(false);
      stats.failed = Some(false);
      stats.errors = Some(0);
      stats.warnings = Some(0);
    }

    if options.ids {
      stats.id = Some("".into());
      stats.chunks = Some(chunks);
    }

    if options.reasons {
      stats.reasons = Some(vec![]);
    }

    if options.module_assets {
      stats.assets = Some(vec![]);
    }

    if options.used_exports {
      stats.used_exports = Some(StatsUsedExports::Null)
    }

    if options.provided_exports {
      stats.provided_exports = Some(vec![]);
    }

    if options.optimization_bailout {
      stats.optimization_bailout = Some(Default::default());
    }

    Ok(stats)
  }
}

pub fn create_stats_errors<'a>(
  compilation: &'a Compilation,
  module_graph: &'a ModuleGraph,
  diagnostics: &'a mut Vec<Diagnostic>,
  colored: bool,
) -> Vec<StatsError<'a>> {
  diagnostics
    .par_iter()
    .map(|d| {
      let module_identifier = d.module_identifier;
      let (module_name, module_id) = module_identifier
        .as_ref()
        .and_then(|identifier| {
          Some(get_stats_module_name_and_id(
            compilation.module_by_identifier(identifier)?,
            compilation,
          ))
        })
        .unzip();

      let chunk = d.chunk.map(ChunkUkey::from).map(|key| {
        compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(&key)
      });

      let module_trace = get_module_trace(
        module_identifier,
        module_graph,
        compilation,
        &compilation.options,
      );

      let code = d.code.clone();

      let diagnostic_displayer = StringDisplayer::new(colored, false);
      StatsError {
        name: code.clone(),
        message: diagnostic_displayer
          .emit_diagnostic(d)
          .expect("should print diagnostics"),
        code,
        module_identifier,
        module_name,
        module_id: module_id.flatten(),
        loc: d.loc.as_ref().map(|loc| loc.to_string()),
        file: d.file.as_ref().map(|file| file.as_path()),

        chunk_name: chunk.and_then(|c| c.name()),
        chunk_entry: chunk
          .map(|c| c.has_runtime(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)),
        chunk_initial: chunk
          .map(|c| c.can_be_initial(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)),
        chunk_id: chunk.and_then(|c| c.id().map(|id| id.as_str())),
        details: d.details.clone(),
        stack: d.stack.clone(),
        module_trace,
      }
    })
    .collect::<Vec<_>>()
}
