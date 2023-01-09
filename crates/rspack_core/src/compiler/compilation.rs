use std::{
  borrow::BorrowMut,
  collections::hash_map::Entry,
  collections::VecDeque,
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  marker::PhantomPinned,
  path::PathBuf,
  pin::Pin,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use dashmap::DashSet;
use futures::{stream::FuturesUnordered, StreamExt};
use indexmap::IndexSet;
use petgraph::{
  algo,
  dot::{Config, Dot},
  graphmap::DiGraphMap,
  prelude::GraphMap,
  Directed,
};
use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rspack_database::Database;
use rspack_error::{
  errors_to_diagnostics, internal_error, Diagnostic, Error, InternalError,
  IntoTWithDiagnosticArray, Result, Severity, TWithDiagnosticArray,
};
use rspack_sources::{BoxSource, CachedSource, SourceExt};
use rspack_symbol::{IndirectTopLevelSymbol, IndirectType, Symbol};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use swc_core::ecma::atoms::JsWord;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::instrument;
use xxhash_rust::xxh3::Xxh3;

use crate::{
  build_chunk_graph::build_chunk_graph,
  cache::Cache,
  contextify, is_source_equal, join_string_component, resolve_module_type_by_uri,
  tree_shaking::{
    symbol_graph::SymbolGraph,
    visitor::{ModuleRefAnalyze, SymbolRef, TreeShakingResult},
    BailoutFlog, ModuleUsedType, OptimizeDependencyResult, SideEffect,
  },
  utils::fast_drop,
  AddQueue, AddTask, AddTaskResult, AdditionalChunkRuntimeRequirementsArgs, BoxModuleDependency,
  BuildQueue, BuildTask, BuildTaskResult, BundleEntries, Chunk, ChunkByUkey, ChunkGraph,
  ChunkGroup, ChunkGroupUkey, ChunkKind, ChunkUkey, CleanQueue, CleanTask, CleanTaskResult,
  CodeGenerationResult, CodeGenerationResults, CompilerOptions, ContentHashArgs, DependencyId,
  EntryDependency, EntryItem, EntryOptions, Entrypoint, FactorizeQueue, FactorizeTask,
  FactorizeTaskResult, Identifier, IdentifierLinkedSet, IdentifierMap, IdentifierSet,
  LoaderRunnerRunner, Module, ModuleGraph, ModuleIdentifier, ModuleType, NormalModuleAstOrSource,
  ProcessAssetsArgs, ProcessDependenciesQueue, ProcessDependenciesResult, ProcessDependenciesTask,
  RenderManifestArgs, Resolve, RuntimeModule, SharedPluginDriver, Stats, TaskResult, WorkerTask,
};

#[derive(Debug)]
pub struct EntryData {
  pub name: String,
  pub dependencies: Vec<DependencyId>,
  pub options: EntryOptions,
}

#[derive(Debug)]
pub enum SetupMakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<DependencyId>),
}

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: BundleEntries,
  pub entry_dependencies: HashMap<String, Vec<DependencyId>>,
  pub module_graph: ModuleGraph,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_module_hashes: IdentifierMap<u64>,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: Database<Chunk>,
  pub chunk_group_by_ukey: HashMap<ChunkGroupUkey, ChunkGroup>,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
  diagnostics: IndexSet<Diagnostic, BuildHasherDefault<FxHasher>>,
  // record last make diagnostics
  last_module_diagnostics: IdentifierMap<Vec<Diagnostic>>,
  pub plugin_driver: SharedPluginDriver,
  pub(crate) loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: IdentifierSet,
  /// Collecting all used export symbol
  pub used_symbol: HashSet<Symbol>,
  pub used_indirect_symbol: HashSet<IndirectTopLevelSymbol>,
  /// Collecting all module that need to skip in tree-shaking ast modification phase
  pub bailout_module_identifiers: IdentifierMap<BailoutFlog>,
  #[cfg(debug_assertions)]
  pub tree_shaking_result: IdentifierMap<TreeShakingResult>,

  pub code_generation_results: CodeGenerationResults,
  pub code_generated_modules: IdentifierSet,
  pub cache: Arc<Cache>,
  pub hash: String,
  // TODO: make compilation safer
  _pin: PhantomPinned,
  // lazy compilation visit module
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub used_chunk_ids: HashSet<String>,

  pub file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
}

impl Compilation {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: BundleEntries,
    // visited_module_id: VisitedModuleIdentity,
    module_graph: ModuleGraph,
    plugin_driver: SharedPluginDriver,
    loader_runner_runner: Arc<LoaderRunnerRunner>,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      options,
      // visited_module_id,
      last_module_diagnostics: Default::default(),
      module_graph,
      runtime_modules: Default::default(),
      runtime_module_hashes: Default::default(),
      chunk_by_ukey: Default::default(),
      chunk_group_by_ukey: Default::default(),
      entry_dependencies: Default::default(),
      entries,
      chunk_graph: Default::default(),
      entrypoints: Default::default(),
      assets: Default::default(),
      emitted_assets: Default::default(),
      diagnostics: Default::default(),
      plugin_driver,
      loader_runner_runner,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: IdentifierSet::default(),
      used_symbol: HashSet::default(),
      #[cfg(debug_assertions)]
      tree_shaking_result: IdentifierMap::default(),
      bailout_module_identifiers: IdentifierMap::default(),

      code_generation_results: Default::default(),
      code_generated_modules: Default::default(),

      cache,
      hash: Default::default(),
      _pin: PhantomPinned,
      used_indirect_symbol: HashSet::default(),
      lazy_visit_modules: Default::default(),
      used_chunk_ids: Default::default(),

      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
    }
  }

  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn update_asset(
    self: Pin<&mut Self>,
    filename: &str,
    updater: impl FnOnce(&mut BoxSource, &mut AssetInfo) -> Result<()>,
  ) -> Result<()> {
    // Safety: we don't move anything from compilation
    let assets = unsafe { self.map_unchecked_mut(|c| &mut c.assets) }.get_mut();

    match assets.get_mut(filename) {
      Some(CompilationAsset {
        source: Some(source),
        info,
      }) => updater(source, info),
      _ => Err(internal_error!(
        "Called Compilation.updateAsset for not existing filename {filename}"
      )),
    }
  }

  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
    tracing::trace!("Emit asset {}", filename);
    if let Some(mut original) = self.assets.remove(&filename)
      && let Some(original_source) = &original.source
      && let Some(asset_source) = asset.get_source() {
      let is_source_equal = is_source_equal(original_source, asset_source);
      if !is_source_equal {
        tracing::error!(
          "Emit Duplicate Filename({}), is_source_equal: {:?}",
          filename,
          is_source_equal
        );
        self.push_batch_diagnostic(
          internal_error!(
            "Conflict: Multiple assets emit different content to the same filename {}{}",
            filename,
            // TODO: source file name
            ""
          )
          .into(),
        );
        self.assets.insert(filename, asset);
        return;
      }
      original.info = asset.info;
      self.assets.insert(filename, original);
    } else {
      self.assets.insert(filename, asset);
    }
  }

  pub fn delete_asset(&mut self, filename: &str) {
    if let Some(asset) = self.assets.remove(filename) {
      if let Some(source_map) = asset.info.related.source_map {
        self.delete_asset(&source_map);
      }
      self.chunk_by_ukey.iter_mut().for_each(|(_, chunk)| {
        chunk.files.remove(filename);
      });
    }
  }

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
  }

  pub fn entrypoints(&self) -> &HashMap<String, ChunkGroupUkey> {
    &self.entrypoints
  }

  pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.insert(diagnostic);
  }

  pub fn push_batch_diagnostic(&mut self, diagnostics: Vec<Diagnostic>) {
    self.diagnostics.extend(diagnostics);
  }

  pub fn get_errors(&self) -> impl Iterator<Item = &Diagnostic> {
    self
      .diagnostics
      .iter()
      .filter(|d| matches!(d.severity, Severity::Error))
  }

  pub fn get_warnings(&self) -> impl Iterator<Item = &Diagnostic> {
    self
      .diagnostics
      .iter()
      .filter(|d| matches!(d.severity, Severity::Warn))
  }

  pub fn get_stats(&self) -> Stats {
    Stats::new(self)
  }

  pub fn add_named_chunk<'chunk>(
    name: String,
    chunk_by_ukey: &'chunk mut ChunkByUkey,
    named_chunks: &mut HashMap<String, ChunkUkey>,
  ) -> &'chunk mut Chunk {
    let existed_chunk_ukey = named_chunks.get(&name);
    if let Some(chunk_ukey) = existed_chunk_ukey {
      let chunk = chunk_by_ukey
        .get_mut(chunk_ukey)
        .expect("This should not happen");
      chunk
    } else {
      let chunk = Chunk::new(Some(name.clone()), None, ChunkKind::Normal);
      let ukey = chunk.ukey;
      named_chunks.insert(name, chunk.ukey);
      chunk_by_ukey.entry(ukey).or_insert_with(|| chunk)
    }
  }

  pub fn add_chunk(chunk_by_ukey: &mut ChunkByUkey) -> &mut Chunk {
    let chunk = Chunk::new(None, None, ChunkKind::Normal);
    let ukey = chunk.ukey;
    chunk_by_ukey.add(chunk);
    chunk_by_ukey.get_mut(&ukey).expect("chunk not found")
  }

  #[instrument(name = "entry_data", skip(self))]
  pub fn entry_data(&self) -> HashMap<String, EntryData> {
    self
      .entries
      .iter()
      .map(|(name, item)| {
        let dependencies = self
          .entry_dependencies
          .get(name)
          .expect("should have dependencies")
          .clone();
        (
          name.clone(),
          EntryData {
            dependencies,
            name: name.clone(),
            options: EntryOptions {
              runtime: item.runtime.clone(),
            },
          },
        )
      })
      .collect()
  }

  pub fn setup_entry_dependencies(&mut self) {
    self.entries.iter().for_each(|(name, item)| {
      let dependencies = item
        .import
        .iter()
        .map(|detail| {
          let dependency =
            Box::new(EntryDependency::new(detail.to_string())) as BoxModuleDependency;
          self.module_graph.add_dependency(dependency)
        })
        .collect::<Vec<_>>();
      self
        .entry_dependencies
        .insert(name.to_string(), dependencies);
    })
  }

  #[instrument(name = "compilation:make", skip_all)]
  pub async fn make(&mut self, params: SetupMakeParam) -> Result<()> {
    if let Some(e) = self
      .plugin_driver
      .clone()
      .read()
      .await
      .make(self)
      .await
      .err()
    {
      self.push_batch_diagnostic(e.into());
    }

    // remove prev build ast in modules
    fast_drop(
      self
        .module_graph
        .module_identifier_to_module
        .values_mut()
        .map(|module| {
          if let Some(m) = module.as_normal_module_mut() {
            let is_ast_unbuild = matches!(m.ast_or_source(), NormalModuleAstOrSource::Unbuild);
            if !is_ast_unbuild {
              return Some(std::mem::replace(
                m.ast_or_source_mut(),
                NormalModuleAstOrSource::Unbuild,
              ));
            }
          }
          None
        })
        .collect::<Vec<Option<NormalModuleAstOrSource>>>(),
    );

    let mut force_build_module = HashSet::default();
    let mut force_build_deps = HashSet::default();
    // handle setup params
    if let SetupMakeParam::ModifiedFiles(files) = &params {
      force_build_module.extend(
        self
          .module_graph
          .module_identifier_to_module
          .values()
          .filter_map(|module| {
            // check has dependencies modified
            if module.has_dependencies(files) {
              Some(module.identifier())
            } else {
              None
            }
          }),
      );
    }
    if let SetupMakeParam::ForceBuildDeps(deps) = params {
      force_build_deps.extend(deps);
    }
    // show last module build diagnostics, need exclude force_build_module
    for identifier in force_build_module.iter() {
      self.last_module_diagnostics.remove(identifier);
    }

    self.push_batch_diagnostic(
      self
        .last_module_diagnostics
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>(),
    );
    // move deps bindings module to force_build_module
    // for dependency_id in &force_build_deps {
    //   if let Some(mgm) = self.module_graph.module_by_dependency(dependency_id) {
    //     force_build_module.insert(mgm.module_identifier);
    //   }
    // }

    let mut need_check_isolated_module_ids = HashSet::default();
    // let mut need_clean_visited_module_ids = HashSet::default();
    // handle force build module
    need_check_isolated_module_ids.extend(force_build_module.iter().flat_map(|id| {
      if let Some(mgm) = self.module_graph.module_graph_module_by_identifier(id) {
        mgm
          .all_depended_modules(&self.module_graph)
          .into_iter()
          .copied()
          .collect()
      } else {
        vec![]
      }
    }));

    let mut active_task_count = 0usize;
    let is_expected_shutdown = Arc::new(AtomicBool::new(false));
    let (result_tx, mut result_rx) = tokio::sync::mpsc::unbounded_channel::<Result<TaskResult>>();
    let mut factorize_queue = FactorizeQueue::new();
    let mut add_queue = AddQueue::new();
    let mut build_queue = BuildQueue::new();
    let mut process_dependencies_queue = ProcessDependenciesQueue::new();
    let mut errored = None;

    force_build_deps.extend(
      force_build_module
        .iter()
        .flat_map(|id| self.module_graph.revoke_module(id)),
    );

    force_build_deps.iter().for_each(|id| {
      let dependency = self
        .module_graph
        .dependency_by_id(id)
        .expect("dependency not found");
      let parent_module_identifier = dependency.parent_module_identifier().cloned();
      let parent_module =
        parent_module_identifier.and_then(|id| self.module_graph.module_by_identifier(&id));
      if parent_module_identifier.is_some() && parent_module.is_none() {
        return;
      }

      self.handle_module_creation(
        &mut factorize_queue,
        parent_module_identifier,
        {
          parent_module
            .and_then(|m| m.as_normal_module())
            .map(|module| module.resource_resolved_data().resource_path.clone())
        },
        vec![dependency.clone()],
        parent_module_identifier.is_none(),
        None,
        None,
        parent_module.and_then(|module| module.get_resolve_options().map(ToOwned::to_owned)),
        self.lazy_visit_modules.clone(),
        parent_module
          .and_then(|m| m.as_normal_module())
          .and_then(|module| module.name_for_condition())
          .map(|issuer| issuer.to_string())
          .unwrap_or_else(|| String::from("")),
      );
    });

    tokio::task::block_in_place(|| loop {
      while let Some(task) = factorize_queue.get_task() {
        tokio::spawn({
          let result_tx = result_tx.clone();
          let is_expected_shutdown = is_expected_shutdown.clone();
          active_task_count += 1;
          async move {
            if is_expected_shutdown.load(Ordering::SeqCst) {
              return;
            }

            let result = task.run().await;

            if !is_expected_shutdown.load(Ordering::SeqCst) {
              result_tx
                .send(result)
                .expect("Failed to send factorize result");
            }
          }
        });
      }

      while let Some(task) = build_queue.get_task() {
        tokio::spawn({
          let result_tx = result_tx.clone();
          let is_expected_shutdown = is_expected_shutdown.clone();
          active_task_count += 1;

          async move {
            if is_expected_shutdown.load(Ordering::SeqCst) {
              return;
            }

            let result = task.run().await;

            if !is_expected_shutdown.load(Ordering::SeqCst) {
              result_tx.send(result).expect("Failed to send build result");
            }
          }
        });
      }

      while let Some(task) = add_queue.get_task() {
        active_task_count += 1;
        let result = task.run(self);
        result_tx.send(result).expect("Failed to send add result");
      }

      while let Some(task) = process_dependencies_queue.get_task() {
        active_task_count += 1;

        task.dependencies.into_iter().for_each(|id| {
          let original_module_identifier = &task.original_module_identifier;
          let module = self
            .module_graph
            .module_by_identifier(original_module_identifier)
            .expect("Module expected");
          let dependency = self
            .module_graph
            .dependency_by_id(&id)
            .expect("dependency expected");

          self.handle_module_creation(
            &mut factorize_queue,
            Some(task.original_module_identifier),
            {
              module
                .as_normal_module()
                .map(|module| module.resource_resolved_data().resource_path.clone())
            },
            vec![dependency.clone()],
            false,
            None,
            None,
            task.resolve_options.clone(),
            self.lazy_visit_modules.clone(),
            module
              .as_normal_module()
              .and_then(|module| module.name_for_condition())
              .map(|issuer| issuer.to_string())
              .unwrap_or_else(|| String::from("")),
          );
        });

        result_tx
          .send(Ok(TaskResult::ProcessDependencies(
            ProcessDependenciesResult {
              module_identifier: task.original_module_identifier,
            },
          )))
          .expect("Failed to send process dependencies result");
      }

      match result_rx.try_recv() {
        Ok(item) => {
          match item {
            Ok(TaskResult::Factorize(task_result)) => {
              let FactorizeTaskResult {
                is_entry,
                original_module_identifier,
                factory_result,
                module_graph_module,
                diagnostics,
                dependencies,
              } = task_result;

              tracing::trace!("Module created: {}", factory_result.module.identifier());

              self.push_batch_diagnostic(diagnostics);

              self
                .file_dependencies
                .extend(factory_result.file_dependencies);
              self
                .context_dependencies
                .extend(factory_result.context_dependencies);
              self
                .missing_dependencies
                .extend(factory_result.missing_dependencies);

              add_queue.add_task(AddTask {
                original_module_identifier,
                module: factory_result.module,
                module_graph_module,
                dependencies,
                is_entry,
              });
            }
            Ok(TaskResult::Add(task_result)) => match task_result {
              AddTaskResult::ModuleAdded { module } => {
                tracing::trace!("Module added: {}", module.identifier());
                build_queue.add_task(BuildTask {
                  module,
                  loader_runner_runner: self.loader_runner_runner.clone(),
                  compiler_options: self.options.clone(),
                  plugin_driver: self.plugin_driver.clone(),
                  cache: self.cache.clone(),
                });
              }
              AddTaskResult::ModuleReused { module } => {
                tracing::trace!("Module reused: {}, skipping build", module.identifier());
              }
            },
            Ok(TaskResult::Build(task_result)) => {
              let BuildTaskResult {
                module,
                build_result,
                diagnostics,
              } = task_result;

              tracing::trace!("Module built: {}", module.identifier());
              if !diagnostics.is_empty() {
                self
                  .last_module_diagnostics
                  .insert(module.identifier(), diagnostics.clone());
              }
              self.push_batch_diagnostic(diagnostics);

              self
                .file_dependencies
                .extend(build_result.file_dependencies);
              self
                .context_dependencies
                .extend(build_result.context_dependencies);
              self
                .missing_dependencies
                .extend(build_result.missing_dependencies);
              self
                .build_dependencies
                .extend(build_result.build_dependencies);

              let mut dep_ids = vec![];
              for dependency in build_result.dependencies {
                let dep_id = self.module_graph.add_dependency(dependency);
                dep_ids.push(dep_id);
              }

              {
                let mgm = self
                  .module_graph
                  .module_graph_module_by_identifier_mut(&module.identifier())
                  .expect("Failed to get mgm");
                mgm.dependencies = dep_ids.clone();
              }
              process_dependencies_queue.add_task(ProcessDependenciesTask {
                dependencies: dep_ids.clone(),
                original_module_identifier: module.identifier(),
                resolve_options: module.get_resolve_options().map(ToOwned::to_owned),
              });
              self.module_graph.add_module(module);
            }
            Ok(TaskResult::ProcessDependencies(task_result)) => {
              tracing::trace!(
                "Processing dependencies of {} finished",
                task_result.module_identifier
              );
            }
            Err(err) => {
              // Severe internal error encountered, we should end the compiling here.
              errored = Some(err);
              is_expected_shutdown.store(true, Ordering::SeqCst);
              break;
            }
          }

          active_task_count -= 1;
        }
        Err(TryRecvError::Disconnected) => {
          is_expected_shutdown.store(true, Ordering::SeqCst);
          break;
        }
        Err(TryRecvError::Empty) => {
          if active_task_count == 0 {
            is_expected_shutdown.store(true, Ordering::SeqCst);
            break;
          }
        }
      }
    });

    tracing::debug!("All task is finished");

    // clean isolated module
    let mut clean_queue = CleanQueue::new();
    clean_queue.add_tasks(
      need_check_isolated_module_ids
        .into_iter()
        .map(|module_identifier| CleanTask { module_identifier }),
    );

    while let Some(task) = clean_queue.get_task() {
      match task.run(self) {
        CleanTaskResult::ModuleIsUsed { module_identifier } => {
          tracing::trace!("Module is used: {}", module_identifier);
        }
        CleanTaskResult::ModuleIsCleaned {
          module_identifier,
          dependent_module_identifiers,
        } => {
          tracing::trace!("Module is cleaned: {}", module_identifier);
          clean_queue.add_tasks(
            dependent_module_identifiers
              .into_iter()
              .map(|module_identifier| CleanTask { module_identifier }),
          );
        }
      };
    }

    tracing::debug!("All clean task is finished");

    if let Some(err) = errored {
      Err(err)
    } else {
      Ok(())
    }
  }

  #[allow(clippy::too_many_arguments)]
  fn handle_module_creation(
    &self,
    queue: &mut FactorizeQueue,
    original_module_identifier: Option<ModuleIdentifier>,
    original_resource_path: Option<PathBuf>,
    dependencies: Vec<BoxModuleDependency>,
    is_entry: bool,
    module_type: Option<ModuleType>,
    side_effects: Option<bool>,
    resolve_options: Option<Resolve>,
    lazy_visit_modules: std::collections::HashSet<String>,
    issuer: String,
  ) {
    queue.add_task(FactorizeTask {
      original_module_identifier,
      issuer,
      original_resource_path,
      dependencies,
      is_entry,
      module_type,
      side_effects,
      resolve_options,
      lazy_visit_modules,
      options: self.options.clone(),
      plugin_driver: self.plugin_driver.clone(),
      cache: self.cache.clone(),
    });
  }

  #[instrument(name = "compilation:code_generation", skip(self))]
  async fn code_generation(&mut self) -> Result<()> {
    fn run_iteration(
      compilation: &mut Compilation,
      filter_op: impl Fn(&(&ModuleIdentifier, &Box<dyn Module>)) -> bool + Sync + Send,
    ) -> Result<()> {
      let results = compilation
        .module_graph
        .module_identifier_to_module
        .par_iter()
        .filter(filter_op)
        .map(|(module_identifier, module)| {
          compilation
            .cache
            .code_generate_occasion
            .use_cache(module, |module| module.code_generation(compilation))
            .map(|result| (*module_identifier, result))
        })
        .collect::<Result<Vec<(ModuleIdentifier, CodeGenerationResult)>>>()?;

      results.into_iter().for_each(|(module_identifier, result)| {
        compilation.code_generated_modules.insert(module_identifier);

        let runtimes = compilation
          .chunk_graph
          .get_module_runtimes(module_identifier, &compilation.chunk_by_ukey);

        compilation
          .code_generation_results
          .module_generation_result_map
          .insert(module_identifier, result);
        for runtime in runtimes.values() {
          compilation.code_generation_results.add(
            module_identifier,
            runtime.clone(),
            module_identifier,
          );
        }
      });
      Ok(())
    }

    run_iteration(self, |(_, module)| {
      module.get_code_generation_dependencies().is_none()
    })?;

    run_iteration(self, |(_, module)| {
      module.get_code_generation_dependencies().is_some()
    })?;

    Ok(())
  }

  #[instrument(skip_all)]
  async fn create_chunk_assets(&mut self, plugin_driver: SharedPluginDriver) {
    let chunk_ukey_and_manifest = (self
      .chunk_by_ukey
      .values()
      .map(|chunk| async {
        let manifest = plugin_driver
          .read()
          .await
          .render_manifest(RenderManifestArgs {
            chunk_ukey: chunk.ukey,
            compilation: self,
          })
          .await;

        if let Ok(manifest) = &manifest {
          tracing::debug!(
            "For Chunk({:?}), collected assets: {:?}",
            chunk.id,
            manifest.iter().map(|m| m.filename()).collect::<Vec<_>>()
          );
        };
        (chunk.ukey, manifest)
      })
      .collect::<FuturesUnordered<_>>())
    .collect::<Vec<_>>()
    .await;

    chunk_ukey_and_manifest
      .into_iter()
      .for_each(|(chunk_ukey, manifest)| {
        manifest
          .expect("TODO: we should return this error rathen expect")
          .into_iter()
          .for_each(|file_manifest| {
            let current_chunk = self
              .chunk_by_ukey
              .get_mut(&chunk_ukey)
              .unwrap_or_else(|| panic!("chunk({chunk_ukey:?}) should be in chunk_by_ukey",));
            current_chunk
              .files
              .insert(file_manifest.filename().to_string());

            self.emit_asset(
              file_manifest.filename().to_string(),
              CompilationAsset::with_source(CachedSource::new(file_manifest.source).boxed()),
            );
          });
      })
  }
  #[instrument(name = "compilation:process_asssets", skip_all)]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .write()
      .await
      .process_assets(ProcessAssetsArgs { compilation: self })
      .await
  }

  pub async fn optimize_dependency(
    &mut self,
  ) -> Result<TWithDiagnosticArray<OptimizeDependencyResult>> {
    let resolver_factory = &self.plugin_driver.read().await.resolver_factory;
    let mut analyze_results = self
      .module_graph
      .module_identifier_to_module_graph_module
      .par_iter()
      .filter_map(|(module_identifier, mgm)| {
        let uri_key = *module_identifier;
        let ast = match mgm.module_type {
          crate::ModuleType::Js | crate::ModuleType::Jsx | crate::ModuleType::Tsx
          | crate::ModuleType::Ts => match self
                      .module_graph
                      .module_by_identifier(&mgm.module_identifier)
                      .and_then(|module| module.as_normal_module().and_then(|m| m.ast()))
                      // A module can missing its AST if the module is failed to build
                      .and_then(|ast|ast.as_javascript()) {
              Some(ast) => {ast},
              None => {
                // FIXME: this could be none if you enable both hmr and tree-shaking, should investigate why
                return None;
              },
          }
            ,
          // Of course this is unsafe, but if we can't get a ast of a javascript module, then panic is ideal.
          _ => {
            // Ignore analyzing other module for now
                return None;
          }
        };
        // let normal_module = self.module_graph.module_by_identifier(&m.module_identifier);
        //        let ast = ast.as_javascript().expect("TODO:");
        let analyzer = ast.visit(|program, context| {
          let top_level_mark = context.top_level_mark;
          let unresolved_mark = context.unresolved_mark;
          let helper_mark = context.helpers.mark();

          let mut analyzer = ModuleRefAnalyze::new(
            top_level_mark,
            unresolved_mark,
            helper_mark,
            uri_key,
            &self.module_graph,
            resolver_factory,
            &self.options,
          );
          program.visit_with(&mut analyzer);
          analyzer
        });
        // Keep this debug info until we stabilize the tree-shaking

        // if debug_care_module_id(uri_key) {
        //   dbg!(
        //     &uri_key,
        //     // &analyzer.export_all_list,
        //     &analyzer.export_map,
        //     &analyzer.import_map,
        //     &analyzer.maybe_lazy_reference_map,
        //     &analyzer.immediate_evaluate_reference_map,
        //     &analyzer.reachable_import_and_export,
        //     &analyzer.used_symbol_ref
        //   );
        // }

        Some((uri_key, analyzer.into()))
      })
      .collect::<IdentifierMap<TreeShakingResult>>();

    let mut used_symbol_ref: HashSet<SymbolRef> = HashSet::default();
    let mut bail_out_module_identifiers = IdentifierMap::default();
    let mut evaluated_module_identifiers = IdentifierSet::default();
    let side_effects_options = self.options.builtins.side_effects;
    let mut side_effect_map: IdentifierMap<SideEffect> = IdentifierMap::default();
    for analyze_result in analyze_results.values() {
      side_effect_map.insert(
        analyze_result.module_identifier,
        analyze_result.side_effects,
      );
      // if `side_effects` is disabled, then force every module has side_effects
      let forced_side_effects = !side_effects_options
        || self
          .entry_module_identifiers
          .contains(&analyze_result.module_identifier);
      // side_effects: true
      if forced_side_effects
        || !matches!(
          analyze_result.side_effects,
          SideEffect::Configuration(false)
        )
      {
        evaluated_module_identifiers.insert(analyze_result.module_identifier);
        used_symbol_ref.extend(analyze_result.used_symbol_refs.iter().cloned());
      }
      // merge bailout module identifier
      for (k, &v) in analyze_result.bail_out_module_identifiers.iter() {
        match bail_out_module_identifiers.entry(*k) {
          Entry::Occupied(mut occ) => {
            *occ.get_mut() |= v;
          }
          Entry::Vacant(vac) => {
            vac.insert(v);
          }
        }
      }
      // bail_out_module_identifiers.extend(analyze_result.bail_out_module_identifiers.clone());
    }

    // normalize side_effects, there are two kinds of `side_effects` one from configuration and another from
    for entry_module_ident in self.entry_module_identifiers.iter() {
      normalize_side_effects(
        *entry_module_ident,
        &self.module_graph,
        &mut IdentifierSet::default(),
        &mut side_effect_map,
      );
    }
    let side_effects_free_module_ident = side_effect_map
      .iter()
      .filter_map(|(k, v)| {
        let side_effect = match v {
          SideEffect::Configuration(value) => value,
          SideEffect::Analyze(value) => value,
        };
        if !side_effect {
          Some(*k)
        } else {
          None
        }
      })
      .collect::<IdentifierSet>();

    // dbg!(&used_symbol_ref);

    // calculate relation of module that has `export * from 'xxxx'`
    let mut symbol_graph = SymbolGraph::new();
    let inherit_export_ref_graph = create_inherit_graph(&analyze_results);
    // key is the module_id of module that potential have reexport all symbol from other module
    // value is the set which contains several module_id the key related module need to inherit
    let map_of_inherit_map = get_extends_map(&inherit_export_ref_graph);

    for (module_id, inherit_export_module_id) in map_of_inherit_map.iter() {
      // This is just a work around for rustc checker, because we have immutable and mutable borrow at the same time.
      let mut inherit_export_maps = {
        let main_module = analyze_results.get_mut(module_id).expect("TODO:");
        std::mem::take(&mut main_module.inherit_export_maps)
      };
      for inherit_export_module_identifier in inherit_export_module_id {
        let export_module = analyze_results
          .get(inherit_export_module_identifier)
          .unwrap_or_else(|| {
            panic!(
              "inherit_export_module_identifier not found: {inherit_export_module_identifier:?}"
            )
          })
          .export_map
          .iter()
          .filter_map(|(k, v)| {
            // export * should not reexport default export
            if k == "default" {
              None
            } else {
              Some((k.clone(), v.clone()))
            }
          })
          .collect::<HashMap<JsWord, SymbolRef>>();
        match inherit_export_maps.entry(*inherit_export_module_identifier) {
          hashlink::lru_cache::Entry::Occupied(mut occ) => {
            *occ.borrow_mut().get_mut() = export_module;
          }
          hashlink::lru_cache::Entry::Vacant(vac) => {
            vac.insert(export_module);
          }
        }
      }
      analyze_results
        .get_mut(module_id)
        .unwrap_or_else(|| panic!("Module({module_id:?}) not found"))
        .inherit_export_maps = inherit_export_maps;
    }
    let mut errors = vec![];
    let mut used_symbol = HashSet::new();
    let mut used_indirect_symbol: HashSet<IndirectTopLevelSymbol> = HashSet::new();
    let mut used_export_module_identifiers: IdentifierMap<ModuleUsedType> =
      IdentifierMap::default();
    let mut traced_tuple = HashSet::new();
    // Marking used symbol and all reachable export symbol from the used symbol for each module

    let used_symbol_from_import = mark_used_symbol_with(
      &analyze_results,
      VecDeque::from_iter(used_symbol_ref.into_iter()),
      &bail_out_module_identifiers,
      &mut evaluated_module_identifiers,
      &mut used_indirect_symbol,
      &mut used_export_module_identifiers,
      &inherit_export_ref_graph,
      &mut traced_tuple,
      &self.options,
      &mut symbol_graph,
      &mut errors,
    );

    used_symbol.extend(used_symbol_from_import);

    // We considering all export symbol in each entry module as used for now
    for entry in self.entry_modules() {
      let used_symbol_set = collect_reachable_symbol(
        &analyze_results,
        entry,
        &mut used_indirect_symbol,
        &bail_out_module_identifiers,
        &mut evaluated_module_identifiers,
        &mut used_export_module_identifiers,
        &inherit_export_ref_graph,
        &mut traced_tuple,
        &self.options,
        &mut symbol_graph,
        &mut errors,
      );
      used_symbol.extend(used_symbol_set);
    }

    // All lazy imported module will be treadted as entry module, which means
    // Its export symbol will be marked as used
    for (module_id, reason) in bail_out_module_identifiers.iter() {
      if reason
        .intersection(
          BailoutFlog::DYNAMIC_IMPORT | BailoutFlog::HELPER | BailoutFlog::COMMONJS_REQUIRE,
        )
        .bits()
        .count_ones()
        >= 1
      {
        let used_symbol_set = collect_reachable_symbol(
          &analyze_results,
          *module_id,
          &mut used_indirect_symbol,
          &bail_out_module_identifiers,
          &mut evaluated_module_identifiers,
          &mut used_export_module_identifiers,
          &inherit_export_ref_graph,
          &mut traced_tuple,
          &self.options,
          &mut symbol_graph,
          &mut errors,
        );
        used_symbol.extend(used_symbol_set);
      };
    }

    // dbg!(&used_export_module_identifiers);
    // println!("{:?}", Dot::new(&symbol_graph.graph,));
    // println!("{}", used_export_module_identifiers.len());
    // let direct_used = used_export_module_identifiers
    //   .iter()
    //   .filter(|(k, v)| v.contains(ModuleUsedType::DIRECT))
    //   .count();

    // dbg!(&direct_used);

    dbg!(&used_export_module_identifiers);
    // println!("{:?}", Dot::new(&symbol_graph.graph,));

    if side_effects_options {
      // pruning
      let mut visited = self.entry_module_identifiers.clone();
      let mut q = VecDeque::from_iter(visited.iter().cloned());
      while let Some(module_identifier) = q.pop_front() {
        let result = analyze_results.get(&module_identifier);
        let analyze_result = match result {
          Some(result) => result,
          None => {
            // These are none js like module, we need to keep it.
            let mgm = self
              .module_graph
              .module_graph_module_by_identifier_mut(&module_identifier)
              .unwrap_or_else(|| {
                panic!("Failed to get mgm by module identifier {module_identifier}")
              });
            mgm.used = true;
            continue;
          }
        };
        // let used = used_export_module_identifiers
        //   .get(&analyze_result.module_identifier)
        //   .map(|value| value.contains(UsedType::DIRECT))
        //   .unwrap_or(false);
        let used = used_export_module_identifiers.contains_key(&analyze_result.module_identifier);

        if !used
          && !bail_out_module_identifiers.contains_key(&analyze_result.module_identifier)
          && side_effects_free_module_ident.contains(&analyze_result.module_identifier)
          && !self.entry_module_identifiers.contains(&module_identifier)
        {
          continue;
        } else {
        }

        let mgm = self
          .module_graph
          .module_graph_module_by_identifier_mut(&module_identifier)
          .unwrap_or_else(|| panic!("Failed to get mgm by module identifier {module_identifier}"));
        mgm.used = true;
        let mgm = self
          .module_graph
          .module_graph_module_by_identifier(&module_identifier)
          .unwrap_or_else(|| {
            panic!("Failed to get ModuleGraphModule by module identifier {module_identifier}")
          });
        for dep in mgm.dependencies.iter() {
          let module_ident = match self.module_graph.module_identifier_by_dependency_id(dep) {
            Some(module_identifier) => *module_identifier,
            None => {
              match self
                .module_graph
                .module_by_identifier(&mgm.module_identifier)
                .and_then(|module| module.as_normal_module())
                .map(|normal_module| normal_module.ast_or_source())
              {
                Some(ast_or_source) => {
                  if matches!(ast_or_source, NormalModuleAstOrSource::BuiltFailed(_)) {
                    // We know that the build output can't run, so it is alright to generate a wrong tree-shaking result.
                    continue;
                  } else {
                    panic!("Failed to resolve {dep:?}")
                  }
                }
                None => {
                  panic!("Failed to get normal module of {}", mgm.module_identifier);
                }
              };
            }
          };
          // .unwrap_or_else(|| panic!("Failed to resolve {dep:?}"))
          // .module_identifier;
          if !visited.contains(&module_ident) {
            q.push_back(module_ident);
            visited.insert(module_ident);
          } else {
            continue;
          }
        }
      }

      // dbg!(&used_symbol, &used_indirect_symbol);
    }
    Ok(
      OptimizeDependencyResult {
        used_symbol,
        analyze_results,
        bail_out_module_identifiers,
        used_indirect_symbol,
      }
      .with_diagnostic(errors_to_diagnostics(errors)),
    )
  }

  pub async fn done(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let stats = &mut Stats::new(self);
    plugin_driver.write().await.done(stats).await?;
    Ok(())
  }

  pub fn entry_modules(&self) -> impl Iterator<Item = ModuleIdentifier> {
    self.entry_module_identifiers.clone().into_iter()
  }

  pub fn entrypoint_by_name(&self, name: &str) -> &Entrypoint {
    let ukey = self.entrypoints.get(name).expect("entrypoint not found");
    self
      .chunk_group_by_ukey
      .get(ukey)
      .expect("entrypoint not found by ukey")
  }
  #[instrument(name = "compilation:seal", skip_all)]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    build_chunk_graph(self)?;

    plugin_driver.write().await.optimize_chunks(self)?;

    plugin_driver.write().await.module_ids(self)?;
    plugin_driver.write().await.chunk_ids(self)?;

    self.code_generation().await?;

    self
      .process_runtime_requirements(plugin_driver.clone())
      .await?;

    self.create_hash(plugin_driver.clone()).await?;

    self.create_chunk_assets(plugin_driver.clone()).await;

    self.process_assets(plugin_driver).await?;
    Ok(())
  }

  pub fn get_chunk_graph_entries(&self) -> HashSet<ChunkUkey> {
    let mut entries: HashSet<ChunkUkey> = HashSet::default();
    for entrypoint_ukey in self.entrypoints.values() {
      let entrypoint = self
        .chunk_group_by_ukey
        .get(entrypoint_ukey)
        .expect("chunk group not found");
      entries.insert(entrypoint.get_runtime_chunk());
    }
    entries
  }

  #[instrument(name = "compilation:process_runtime_requirements", skip_all)]
  pub async fn process_runtime_requirements(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let mut module_runtime_requirements = self
      .module_graph
      .module_identifier_to_module
      .par_iter()
      .filter_map(|(_, module)| {
        if self
          .chunk_graph
          .get_number_of_module_chunks(module.identifier())
          > 0
        {
          let mut module_runtime_requirements: Vec<(HashSet<String>, HashSet<&'static str>)> =
            vec![];
          for runtime in self
            .chunk_graph
            .get_module_runtimes(module.identifier(), &self.chunk_by_ukey)
            .values()
          {
            let runtime_requirements = self
              .code_generation_results
              .get_runtime_requirements(&module.identifier(), Some(runtime));
            module_runtime_requirements.push((runtime.clone(), runtime_requirements));
          }
          return Some((module.identifier(), module_runtime_requirements));
        }
        None
      })
      .collect::<Vec<_>>();

    for (module_identifier, runtime_requirements) in module_runtime_requirements.iter_mut() {
      for (runtime, requirements) in runtime_requirements.iter_mut() {
        self.chunk_graph.add_module_runtime_requirements(
          *module_identifier,
          runtime,
          std::mem::take(requirements),
        )
      }
    }
    tracing::trace!("runtime requirements.modules");

    let mut chunk_requirements = HashMap::default();
    for (chunk_ukey, chunk) in self.chunk_by_ukey.iter() {
      let mut set = HashSet::default();
      for module in self
        .chunk_graph
        .get_chunk_modules(chunk_ukey, &self.module_graph)
      {
        if let Some(runtime_requirements) = self
          .chunk_graph
          .get_module_runtime_requirements(module.module_identifier, &chunk.runtime)
        {
          set.extend(runtime_requirements);
        }
      }
      chunk_requirements.insert(*chunk_ukey, set);
    }
    for (chunk_ukey, set) in chunk_requirements.iter_mut() {
      plugin_driver
        .read()
        .await
        .additional_chunk_runtime_requirements(&mut AdditionalChunkRuntimeRequirementsArgs {
          compilation: self,
          chunk: chunk_ukey,
          runtime_requirements: set,
        })?;

      self
        .chunk_graph
        .add_chunk_runtime_requirements(chunk_ukey, std::mem::take(set));
    }
    tracing::trace!("runtime requirements.chunks");

    for entry_ukey in self.get_chunk_graph_entries().iter() {
      let entry = self
        .chunk_by_ukey
        .get(entry_ukey)
        .expect("chunk not found by ukey");

      let mut set = HashSet::default();

      for chunk_ukey in entry
        .get_all_referenced_chunks(&self.chunk_group_by_ukey)
        .iter()
      {
        let runtime_requirements = self.chunk_graph.get_chunk_runtime_requirements(chunk_ukey);
        set.extend(runtime_requirements.clone());
      }

      plugin_driver
        .read()
        .await
        .additional_tree_runtime_requirements(&mut AdditionalChunkRuntimeRequirementsArgs {
          compilation: self,
          chunk: entry_ukey,
          runtime_requirements: &mut set,
        })?;

      plugin_driver.read().await.runtime_requirements_in_tree(
        &mut AdditionalChunkRuntimeRequirementsArgs {
          compilation: self,
          chunk: entry_ukey,
          runtime_requirements: &mut set,
        },
      )?;

      self
        .chunk_graph
        .add_tree_runtime_requirements(entry_ukey, set);
    }
    tracing::trace!("runtime requirements.entries");
    Ok(())
  }

  #[instrument(name = "compilation:create_hash", skip_all)]
  pub async fn create_hash(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    // move to make, only hash changed module at hmr.
    // self.create_module_hash();

    let mut compilation_hasher = Xxh3::new();
    let mut chunks = self.chunk_by_ukey.values_mut().collect::<Vec<_>>();
    chunks.sort_by_key(|chunk| chunk.ukey);
    chunks.par_iter_mut().for_each(|chunk| {
      for mgm in self
        .chunk_graph
        .get_ordered_chunk_modules(&chunk.ukey, &self.module_graph)
      {
        if let Some(hash) = self.module_graph.get_module_hash(&mgm.module_identifier) {
          hash.hash(&mut chunk.hash);
        }
      }
    });
    for chunk in chunks {
      for mgm in self
        .chunk_graph
        .get_ordered_chunk_modules(&chunk.ukey, &self.module_graph)
      {
        if let Some(hash) = self.module_graph.get_module_hash(&mgm.module_identifier) {
          hash.hash(&mut compilation_hasher);
        }
      }
    }
    tracing::trace!("hash chunks");

    let runtime_chunk_ukeys = self.get_chunk_graph_entries();
    let content_hash_chunks = {
      // runtime chunks should be hashed after all other chunks
      let mut chunks = self
        .chunk_by_ukey
        .keys()
        .filter(|key| !runtime_chunk_ukeys.contains(key))
        .copied()
        .collect::<Vec<_>>();
      chunks.extend(runtime_chunk_ukeys.clone());
      chunks
    };
    for chunk_ukey in content_hash_chunks {
      plugin_driver
        .read()
        .await
        .content_hash(&mut ContentHashArgs {
          chunk_ukey,
          compilation: self,
        })
        .await?;
    }
    tracing::trace!("calculate chunks content hash");

    self.create_runtime_module_hash();

    let mut entry_chunks = self
      .chunk_by_ukey
      .values_mut()
      .filter(|chunk| runtime_chunk_ukeys.contains(&chunk.ukey))
      .collect::<Vec<_>>();
    entry_chunks.sort_by_key(|chunk| chunk.ukey);
    entry_chunks.par_iter_mut().for_each(|chunk| {
      for identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_in_order(&chunk.ukey)
      {
        if let Some(hash) = self.runtime_module_hashes.get(identifier) {
          hash.hash(&mut chunk.hash);
        }
      }
    });
    tracing::trace!("hash runtime chunks");

    entry_chunks.iter().for_each(|chunk| {
      for identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_in_order(&chunk.ukey)
      {
        if let Some(hash) = self.runtime_module_hashes.get(identifier) {
          hash.hash(&mut compilation_hasher);
        }
      }
    });
    self.hash = format!("{:x}", compilation_hasher.finish());
    tracing::trace!("compilation hash");
    Ok(())
  }

  // #[instrument(name = "compilation:create_module_hash", skip_all)]
  // pub fn create_module_hash(&mut self) {
  //   let module_hash_map: HashMap<ModuleIdentifier, u64> = self
  //     .module_graph
  //     .module_identifier_to_module
  //     .par_iter()
  //     .map(|(identifier, module)| {
  //       let mut hasher = Xxh3::new();
  //       module.hash(&mut hasher);
  //       (*identifier, hasher.finish())
  //     })
  //     .collect();

  //   for (identifier, hash) in module_hash_map {
  //     for runtime in self
  //       .chunk_graph
  //       .get_module_runtimes(identifier, &self.chunk_by_ukey)
  //       .values()
  //     {
  //       self
  //         .chunk_graph
  //         .set_module_hashes(identifier, runtime, hash);
  //     }
  //   }
  // }

  #[instrument(name = "compilation:create_runtime_module_hash", skip_all)]
  pub fn create_runtime_module_hash(&mut self) {
    self.runtime_module_hashes = self
      .runtime_modules
      .par_iter()
      .map(|(identifier, module)| {
        let mut hasher = Xxh3::new();
        module.hash(self, &mut hasher);
        (*identifier, hasher.finish())
      })
      .collect();
  }

  pub fn add_runtime_module(&mut self, chunk_ukey: &ChunkUkey, mut module: Box<dyn RuntimeModule>) {
    // add chunk runtime to perfix module identifier to avoid multiple entry runtime modules conflict
    let chunk = self
      .chunk_by_ukey
      .get(chunk_ukey)
      .expect("chunk not found by ukey");
    let runtime_module_identifier = format!("{:?}/{}", chunk.runtime, module.identifier());
    let runtime_module_identifier = ModuleIdentifier::from(runtime_module_identifier);
    module.attach(*chunk_ukey);
    self.chunk_graph.add_module(runtime_module_identifier);
    self
      .chunk_graph
      .connect_chunk_and_module(*chunk_ukey, runtime_module_identifier);
    self
      .chunk_graph
      .connect_chunk_and_runtime_module(*chunk_ukey, runtime_module_identifier);
    self
      .runtime_modules
      .insert(runtime_module_identifier, module);
  }
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;

#[derive(Debug, Clone)]
pub struct CompilationAsset {
  pub source: Option<BoxSource>,
  pub info: AssetInfo,
}

impl CompilationAsset {
  pub fn new(source: Option<BoxSource>, info: AssetInfo) -> Self {
    Self { source, info }
  }

  pub fn with_source(source: BoxSource) -> Self {
    Self {
      source: Some(source),
      info: Default::default(),
    }
  }

  pub fn get_source(&self) -> Option<&BoxSource> {
    self.source.as_ref()
  }

  pub fn get_source_mut(&mut self) -> Option<&mut BoxSource> {
    self.source.as_mut()
  }

  pub fn set_source(&mut self, source: Option<BoxSource>) {
    self.source = source;
  }

  pub fn get_info(&self) -> &AssetInfo {
    &self.info
  }

  pub fn get_info_mut(&mut self) -> &mut AssetInfo {
    &mut self.info
  }

  pub fn set_info(&mut self, info: AssetInfo) {
    self.info = info;
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfo {
  /// if the asset can be long term cached forever (contains a hash)
  // pub immutable: bool,
  /// whether the asset is minimized
  pub minimized: bool,
  /// the value(s) of the full hash used for this asset
  // pub full_hash:
  /// the value(s) of the chunk hash used for this asset
  // pub chunk_hash:
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  // pub content_hash:
  /// when asset was created from a source file (potentially transformed), the original filename relative to compilation context
  // pub source_filename:
  /// size in bytes, only set after asset has been emitted
  // pub size: f64,
  /// when asset is only used for development and doesn't count towards user-facing assets
  pub development: bool,
  /// when asset ships data for updating an existing application (HMR)
  pub hot_module_replacement: bool,
  /// when asset is javascript and an ESM
  // pub javascript_module:
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: AssetInfoRelated,
}

impl AssetInfo {
  pub fn with_minimized(mut self, v: bool) -> Self {
    self.minimized = v;
    self
  }

  pub fn with_development(mut self, v: bool) -> Self {
    self.development = v;
    self
  }

  pub fn with_hot_module_replacement(mut self, v: bool) -> Self {
    self.hot_module_replacement = v;
    self
  }

  pub fn with_related(mut self, v: AssetInfoRelated) -> Self {
    self.related = v;
    self
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn collect_reachable_symbol(
  analyze_map: &IdentifierMap<TreeShakingResult>,
  entry_identifier: ModuleIdentifier,
  used_indirect_symbol: &mut HashSet<IndirectTopLevelSymbol>,
  bailout_module_identifiers: &IdentifierMap<BailoutFlog>,
  evaluated_module_identifiers: &mut IdentifierSet,
  used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
  inherit_extend_graph: &GraphMap<ModuleIdentifier, (), Directed>,
  traced_tuple: &mut HashSet<(ModuleIdentifier, ModuleIdentifier)>,
  options: &Arc<CompilerOptions>,
  graph: &mut SymbolGraph,
  errors: &mut Vec<Error>,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::default();
  let mut q = VecDeque::new();
  let entry_module_result = match analyze_map.get(&entry_identifier) {
    Some(result) => result,
    None => {
      // TODO: checking if it is none js type
      return HashSet::default();
      // panic!("Can't get analyze result from entry_identifier {}", entry_identifier);
    }
  };

  // deduplicate reexport in entry module start
  let mut export_symbol_count_map: HashMap<JsWord, (SymbolRef, usize)> = entry_module_result
    .export_map
    .iter()
    .map(|(symbol_name, symbol_ref)| (symbol_name.clone(), (symbol_ref.clone(), 1)))
    .collect();
  // All the reexport star symbol should be included in the bundle
  // TODO: esbuild will hidden the duplicate reexport, webpack will emit an error
  // which should we align to?
  for (_, inherit_map) in entry_module_result.inherit_export_maps.iter() {
    for (atom, symbol_ref) in inherit_map.iter() {
      match export_symbol_count_map.entry(atom.clone()) {
        Entry::Occupied(mut occ) => {
          occ.borrow_mut().get_mut().1 += 1;
        }
        Entry::Vacant(vac) => {
          vac.insert((symbol_ref.clone(), 1));
        }
      };
    }
  }

  q.extend(export_symbol_count_map.into_iter().filter_map(
    |(_, v)| {
      if v.1 == 1 {
        Some(v.0)
      } else {
        None
      }
    },
  ));
  // deduplicate reexport in entry end

  for item in entry_module_result.export_map.values() {
    mark_symbol(
      item.clone(),
      &mut used_symbol_set,
      used_indirect_symbol,
      analyze_map,
      &mut q,
      bailout_module_identifiers,
      evaluated_module_identifiers,
      used_export_module_identifiers,
      inherit_extend_graph,
      traced_tuple,
      options,
      graph,
      errors,
    );
  }

  while let Some(sym_ref) = q.pop_front() {
    mark_symbol(
      sym_ref,
      &mut used_symbol_set,
      used_indirect_symbol,
      analyze_map,
      &mut q,
      bailout_module_identifiers,
      evaluated_module_identifiers,
      used_export_module_identifiers,
      inherit_extend_graph,
      traced_tuple,
      options,
      graph,
      errors,
    );
  }
  used_symbol_set
}

#[allow(clippy::too_many_arguments)]
fn mark_used_symbol_with(
  analyze_map: &IdentifierMap<TreeShakingResult>,
  mut init_queue: VecDeque<SymbolRef>,
  bailout_module_identifiers: &IdentifierMap<BailoutFlog>,
  evaluated_module_identifiers: &mut IdentifierSet,
  used_indirect_symbol_set: &mut HashSet<IndirectTopLevelSymbol>,
  used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
  inherit_extend_graph: &GraphMap<ModuleIdentifier, (), Directed>,
  traced_tuple: &mut HashSet<(ModuleIdentifier, ModuleIdentifier)>,
  options: &Arc<CompilerOptions>,
  graph: &mut SymbolGraph,
  errors: &mut Vec<Error>,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::default();
  let mut visited = HashSet::default();

  while let Some(sym_ref) = init_queue.pop_front() {
    if visited.contains(&sym_ref) {
      continue;
    } else {
      visited.insert(sym_ref.clone());
    }
    mark_symbol(
      sym_ref,
      &mut used_symbol_set,
      used_indirect_symbol_set,
      analyze_map,
      &mut init_queue,
      bailout_module_identifiers,
      evaluated_module_identifiers,
      used_export_module_identifiers,
      inherit_extend_graph,
      traced_tuple,
      options,
      graph,
      errors,
    );
  }
  used_symbol_set
}

#[allow(clippy::too_many_arguments)]
fn mark_symbol(
  current_symbol_ref: SymbolRef,
  used_symbol_set: &mut HashSet<Symbol>,
  used_indirect_symbol_set: &mut HashSet<IndirectTopLevelSymbol>,
  analyze_map: &IdentifierMap<TreeShakingResult>,
  symbol_queue: &mut VecDeque<SymbolRef>,
  bailout_module_identifiers: &IdentifierMap<BailoutFlog>,
  evaluated_module_identifiers: &mut IdentifierSet,
  used_export_module_identifiers: &mut IdentifierMap<ModuleUsedType>,
  inherit_extend_graph: &GraphMap<ModuleIdentifier, (), Directed>,
  traced_tuple: &mut HashSet<(ModuleIdentifier, ModuleIdentifier)>,
  options: &Arc<CompilerOptions>,
  graph: &mut SymbolGraph,
  errors: &mut Vec<Error>,
) {
  // if debug_care_module_id(symbol_ref.module_identifier()) {
  // dbg!(&symbol_ref);
  // }
  // We don't need mark the symbol usage if it is from a bailout module because
  // bailout module will skipping tree-shaking anyway
  let side_effects_enable = options.builtins.side_effects;
  let is_bailout_module_identifier =
    bailout_module_identifiers.contains_key(&current_symbol_ref.module_identifier());
  match &current_symbol_ref {
    SymbolRef::Direct(symbol) => {
      merge_used_export_type(
        used_export_module_identifiers,
        symbol.uri().into(),
        ModuleUsedType::DIRECT,
      );
    }
    SymbolRef::Indirect(indirect) => {
      merge_used_export_type(
        used_export_module_identifiers,
        indirect.uri().into(),
        ModuleUsedType::INDIRECT,
      );
    }
    _ => {}
  };
  match current_symbol_ref {
    SymbolRef::Direct(ref symbol) => match used_symbol_set.entry(symbol.clone()) {
      Occupied(_) => {}
      Vacant(vac) => {
        let module_result = analyze_map.get(&vac.get().uri().into()).expect("TODO:");
        if let Some(set) = module_result
          .reachable_import_of_export
          .get(&symbol.id().atom)
        {
          for symbol_ref_ele in set.iter() {
            graph.add_edge(&current_symbol_ref, symbol_ref_ele);
            symbol_queue.push_back(symbol_ref_ele.clone());
          }
        };

        // Assume the module name is app.js
        // ```js
        // import {myanswer, secret} from './lib'
        // export {myanswer as m, secret as s}
        // ```
        // In such scenario there are two `myanswer` binding would create
        // one for `app.js`, one for `lib.js`
        // the binding in `app.js` used for shake the `export {xxx}`
        // In other words, we need two binding for supporting indirect redirect.
        if let Some(import_symbol_ref) = module_result.import_map.get(vac.get().id()) {
          graph.add_edge(&current_symbol_ref, import_symbol_ref);
          symbol_queue.push_back(import_symbol_ref.clone());
        }
        if !evaluated_module_identifiers.contains(&vac.get().uri().into()) {
          evaluated_module_identifiers.insert(vac.get().uri().into());
          for used_symbol_ref in module_result.used_symbol_refs.iter() {
            graph.add_edge(&current_symbol_ref, used_symbol_ref);
            symbol_queue.push_back(used_symbol_ref.clone());
          }
        }
        if !side_effects_enable {
          vac.insert();
        }
      }
    },
    SymbolRef::Indirect(ref indirect_symbol) => {
      let importer = indirect_symbol.importer();
      if indirect_symbol.ty == IndirectType::ReExport
        && !evaluated_module_identifiers.contains(&importer.into())
      {
        evaluated_module_identifiers.insert(importer.into());
        let module_result = analyze_map.get(&importer.into()).expect("TODO:");
        for used_symbol in module_result.used_symbol_refs.iter() {
          graph.add_edge(&current_symbol_ref, used_symbol);
          symbol_queue.push_back(used_symbol.clone());
        }
      }
      if !side_effects_enable {
        used_indirect_symbol_set.insert(indirect_symbol.clone());
      }
      let module_result = match analyze_map.get(&indirect_symbol.uri.into()) {
        Some(module_result) => module_result,
        None => {
          // eprintln!(
          //   "Can't get optimize dep result for module {}",
          //   indirect_symbol.uri,
          // );
          return;
        }
      };
      let symbol = match module_result.export_map.get(&indirect_symbol.id) {
        Some(symbol) => symbol.clone(),
        None => {
          // TODO: better diagnostic and handle if multiple extends_map has export same symbol
          let mut ret = vec![];
          // Checking if any inherit export map is belong to a bailout module
          let mut has_bailout_module_identifiers = false;
          let mut is_first_result = true;
          for (module_identifier, extends_export_map) in module_result.inherit_export_maps.iter() {
            if let Some(value) = extends_export_map.get(&indirect_symbol.id) {
              ret.push((module_identifier, value));
              if is_first_result {
                let tuple = (indirect_symbol.uri.into(), *module_identifier);
                if !traced_tuple.contains(&tuple) {
                  for mi in algo::all_simple_paths::<Vec<_>, _>(
                    &inherit_extend_graph,
                    indirect_symbol.uri.into(),
                    *module_identifier,
                    0,
                    None,
                  )
                  .into_iter()
                  .flatten()
                  {
                    merge_used_export_type(
                      used_export_module_identifiers,
                      mi,
                      ModuleUsedType::REEXPORT,
                    );
                  }
                  // used_export_module_identifiers.extend();
                  traced_tuple.insert(tuple);
                }
                is_first_result = false;
              }
            }
            has_bailout_module_identifiers = has_bailout_module_identifiers
              || bailout_module_identifiers.contains_key(module_identifier);
          }

          // FIXME: this is just a workaround for dependency replacement
          if !ret.is_empty() && !evaluated_module_identifiers.contains(&indirect_symbol.uri.into())
          {
            for used_symbol_ref in module_result.used_symbol_refs.iter() {
              graph.add_edge(&current_symbol_ref, used_symbol_ref);
              symbol_queue.push_back(used_symbol_ref.clone());
            }
            evaluated_module_identifiers.insert(indirect_symbol.uri().into());
          }
          match ret.len() {
            0 => {
              // TODO: Better diagnostic handle if source module does not have the export
              // let map = analyze_map.get(&module_result.module_identifier).expect("TODO:");
              // dbg!(&map);
              if !is_bailout_module_identifier && !has_bailout_module_identifiers {
                let error_message = format!(
                  "{} did not export `{}`, imported by {}",
                  module_result.module_identifier,
                  indirect_symbol.id,
                  indirect_symbol.importer()
                );
                errors.push(Error::InternalError(InternalError {
                  error_message,
                  severity: Severity::Warn,
                }));
                return;
              } else {
                // TODO: This branch should be remove after we analyze module.exports
                // If one of inherit module is a bailout module, that most probably means that module has some common js export
                // which we don't analyze yet, we just pass it. It is alright because we don't modified the ast of bailout module
                return;
              }
            }
            1 => ret[0].1.clone(),
            // multiple export candidate in reexport
            // mark the first symbol_ref as used, align to webpack
            _ => {
              // TODO: better traceable diagnostic
              let mut error_message = format!(
                "Conflicting star exports for the name '{}' in ",
                indirect_symbol.id
              );
              // let cwd = std::env::current_dir();
              let module_identifier_list = ret
                .iter()
                .map(|(module_identifier, _)| {
                  contextify(options.context.clone(), module_identifier)
                })
                .collect::<Vec<_>>();
              error_message += &join_string_component(module_identifier_list);
              errors.push(Error::InternalError(InternalError {
                error_message,
                severity: Severity::Warn,
              }));
              ret[0].1.clone()
            }
          }
        }
      };

      graph.add_edge(&current_symbol_ref, &symbol);
      symbol_queue.push_back(symbol);
    }
    SymbolRef::Star(src) => {
      // If a star ref is used. e.g.
      // ```js
      // import * as all from './test.js'
      // all
      // ```
      // then, all the exports in `test.js` including
      // export defined in `test.js` and all realted
      // reexport should be marked as used

      let analyze_refsult = match analyze_map.get(&src) {
        Some(analyze_result) => analyze_result,
        None => {
          match resolve_module_type_by_uri(src.as_str()) {
            Some(module_type) => match module_type {
              crate::ModuleType::Js
              | crate::ModuleType::JsDynamic
              | crate::ModuleType::JsEsm
              | crate::ModuleType::Jsx
              | crate::ModuleType::JsxDynamic
              | crate::ModuleType::JsxEsm
              | crate::ModuleType::Tsx
              | crate::ModuleType::Ts => {
                let error_message = format!("Can't get analyze result of {src}");
                errors.push(Error::InternalError(InternalError {
                  error_message,
                  severity: Severity::Warn,
                }));
              }
              _ => {
                // Ignore result module type
              }
            },
            None => {
              let error_message = format!("Can't get analyze result of {src}");
              errors.push(Error::InternalError(InternalError {
                error_message,
                severity: Severity::Warn,
              }));
            }
          };
          return;
        }
      };
      evaluated_module_identifiers.insert(src);
      if !analyze_refsult.export_map.is_empty() {
        merge_used_export_type(used_export_module_identifiers, src, ModuleUsedType::DIRECT);
      }

      for export_symbol_ref in analyze_refsult.export_map.values() {
        graph.add_edge(&current_symbol_ref, export_symbol_ref);
        symbol_queue.push_back(export_symbol_ref.clone());
      }

      for (_, extend_export_map) in analyze_refsult.inherit_export_maps.iter() {
        for export_symbol_ref in extend_export_map.values() {
          graph.add_edge(&current_symbol_ref, export_symbol_ref);
          symbol_queue.push_back(export_symbol_ref.clone());
          let tuple = (src, export_symbol_ref.module_identifier());
          if !traced_tuple.contains(&tuple) {
            for mi in algo::all_simple_paths::<Vec<_>, _>(
              &inherit_extend_graph,
              src,
              export_symbol_ref.module_identifier(),
              0,
              None,
            )
            .into_iter()
            .flatten()
            {
              merge_used_export_type(used_export_module_identifiers, mi, ModuleUsedType::REEXPORT);
            }
            traced_tuple.insert(tuple);
          }
        }
      }
    }
  }
}

fn get_extends_map(
  export_all_ref_graph: &GraphMap<ModuleIdentifier, (), petgraph::Directed>,
) -> IdentifierMap<IdentifierLinkedSet> {
  let mut map = IdentifierMap::default();
  for node in export_all_ref_graph.nodes() {
    let reachable_set = get_reachable(node, export_all_ref_graph);
    map.insert(node, reachable_set);
  }
  map
}

fn get_reachable(
  start: ModuleIdentifier,
  g: &GraphMap<ModuleIdentifier, (), petgraph::Directed>,
) -> IdentifierLinkedSet {
  let mut visited: IdentifierSet = IdentifierSet::default();
  let mut reachable_module_id = IdentifierLinkedSet::default();
  let mut q = VecDeque::from_iter([start]);
  while let Some(cur) = q.pop_front() {
    if !visited.contains(&cur) {
      visited.insert(cur);
    } else {
      continue;
    }
    if cur != start {
      reachable_module_id.insert(cur);
    }
    q.extend(g.neighbors_directed(cur, petgraph::Direction::Outgoing));
  }
  reachable_module_id
}

fn create_inherit_graph(
  analyze_map: &IdentifierMap<TreeShakingResult>,
) -> GraphMap<ModuleIdentifier, (), petgraph::Directed> {
  let mut g = DiGraphMap::new();
  for (module_id, result) in analyze_map.iter() {
    for export_all_module_id in result.inherit_export_maps.keys() {
      g.add_edge(*module_id, *export_all_module_id, ());
    }
  }
  g
}

pub fn merge_used_export_type(
  used_export: &mut IdentifierMap<ModuleUsedType>,
  module_id: ModuleIdentifier,
  ty: ModuleUsedType,
) {
  match used_export.entry(module_id) {
    Entry::Occupied(mut occ) => {
      occ.borrow_mut().get_mut().insert(ty);
    }
    Entry::Vacant(vac) => {
      vac.insert(ty);
    }
  }
}

fn normalize_side_effects(
  cur: Identifier,
  module_graph: &ModuleGraph,
  visited_module: &mut IdentifierSet,
  side_effects_map: &mut IdentifierMap<SideEffect>,
) {
  if visited_module.contains(&cur) {
    return;
  }
  visited_module.insert(cur);
  let mgm = module_graph
    .module_graph_module_by_identifier(&cur)
    .unwrap_or_else(|| panic!("Failed to get mgm by module identifier {cur}"));
  let mut module_ident_list = vec![];
  for dep in mgm.dependencies.iter() {
    let module_ident = match module_graph.module_identifier_by_dependency_id(dep) {
      Some(module_identifier) => *module_identifier,
      None => {
        match module_graph
          .module_by_identifier(&mgm.module_identifier)
          .and_then(|module| module.as_normal_module())
          .map(|normal_module| normal_module.ast_or_source())
        {
          Some(ast_or_source) => {
            if matches!(ast_or_source, NormalModuleAstOrSource::BuiltFailed(_)) {
              // We know that the build output can't run, so it is alright to generate a wrong tree-shaking result.
              continue;
            } else {
              panic!("Failed to resolve {dep:?}")
            }
          }
          None => {
            panic!("Failed to get normal module of {}", mgm.module_identifier);
          }
        };
      }
    };
    module_ident_list.push(module_ident);
    normalize_side_effects(module_ident, module_graph, visited_module, side_effects_map);
  }
  // visited_module.remove(&cur);

  let side_effect_list = match side_effects_map.entry(cur) {
    Entry::Occupied(mut occ) => match occ.get_mut() {
      SideEffect::Configuration(_) => vec![],
      SideEffect::Analyze(value) => {
        if *value {
          vec![]
        } else {
          module_ident_list
            .into_iter()
            .filter(|ident| {
              matches!(
                side_effects_map.get(ident),
                Some(SideEffect::Analyze(true)) | Some(SideEffect::Configuration(true))
              )
            })
            .collect::<Vec<_>>()
        }
      }
    },
    Entry::Vacant(_) => vec![],
  };

  if !side_effect_list.is_empty() {
    if let Some(cur) = side_effects_map.get_mut(&cur) {
      *cur = SideEffect::Analyze(true);
    }
  }
}
