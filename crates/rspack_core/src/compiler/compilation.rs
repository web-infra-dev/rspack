use std::{
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  path::PathBuf,
  sync::{
    atomic::{AtomicBool, Ordering},
    // time::Instant,
    Arc,
  },
};

use dashmap::DashSet;
use indexmap::{IndexMap, IndexSet};
use rayon::prelude::{
  IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelBridge, ParallelIterator,
};
use rspack_database::Database;
use rspack_error::{
  internal_error, CatchUnwindFuture, Diagnostic, Result, Severity, TWithDiagnosticArray,
};
use rspack_futures::FuturesResults;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rspack_sources::{BoxSource, CachedSource, SourceExt};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use swc_core::ecma::ast::ModuleItem;
use tokio::sync::mpsc::error::TryRecvError;
use tracing::instrument;
use xxhash_rust::xxh3::Xxh3;

use crate::tree_shaking::visitor::OptimizeAnalyzeResult;
use crate::{
  build_chunk_graph::build_chunk_graph,
  cache::{use_code_splitting_cache, Cache, CodeSplittingCache},
  is_source_equal,
  tree_shaking::{
    optimizer, visitor::SymbolRef, webpack_ext::ExportInfo, BailoutFlag, OptimizeDependencyResult,
  },
  utils::fast_drop,
  AddQueue, AddTask, AddTaskResult, AdditionalChunkRuntimeRequirementsArgs, BoxModuleDependency,
  BuildQueue, BuildTask, BuildTaskResult, BundleEntries, Chunk, ChunkByUkey, ChunkGraph,
  ChunkGroup, ChunkGroupUkey, ChunkHashArgs, ChunkKind, ChunkUkey, CleanQueue, CleanTask,
  CleanTaskResult, CodeGenerationResult, CodeGenerationResults, CompilerOptions, ContentHashArgs,
  DependencyId, EntryDependency, EntryItem, EntryOptions, Entrypoint, FactorizeQueue,
  FactorizeTask, FactorizeTaskResult, Filename, Module, ModuleGraph, ModuleIdentifier, ModuleType,
  NormalModuleAstOrSource, PathData, ProcessAssetsArgs, ProcessDependenciesQueue,
  ProcessDependenciesResult, ProcessDependenciesTask, RenderManifestArgs, Resolve, ResolverFactory,
  RuntimeGlobals, RuntimeModule, RuntimeSpec, SharedPluginDriver, Stats, TaskResult, WorkerTask,
};

#[derive(Debug)]
pub struct EntryData {
  pub name: String,
  pub dependencies: Vec<DependencyId>,
  pub options: EntryOptions,
}

pub type BuildDependency = (
  DependencyId,
  Option<ModuleIdentifier>, /* parent module */
);

#[derive(Debug)]
pub enum SetupMakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
}

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: BundleEntries,
  pub entry_dependencies: HashMap<String, Vec<DependencyId>>,
  pub module_graph: ModuleGraph,
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: HashSet<ModuleIdentifier>,
  pub has_module_import_export_change: bool,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_module_code_generation_results: IdentifierMap<(u64, BoxSource)>,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: Database<Chunk>,
  pub chunk_group_by_ukey: Database<ChunkGroup>,
  pub entrypoints: IndexMap<String, ChunkGroupUkey>,
  assets: CompilationAssets,
  pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
  diagnostics: IndexSet<Diagnostic, BuildHasherDefault<FxHasher>>,
  pub plugin_driver: SharedPluginDriver,
  pub resolver_factory: Arc<ResolverFactory>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: IdentifierSet,
  /// Collecting all used export symbol
  pub used_symbol_ref: HashSet<SymbolRef>,
  /// Collecting all module that need to skip in tree-shaking ast modification phase
  pub bailout_module_identifiers: IdentifierMap<BailoutFlag>,
  pub exports_info_map: IdentifierMap<Vec<ExportInfo>>,
  pub optimize_analyze_result_map: IdentifierMap<OptimizeAnalyzeResult>,

  pub code_generation_results: CodeGenerationResults,
  pub code_generated_modules: IdentifierSet,
  pub cache: Arc<Cache>,
  pub code_splitting_cache: CodeSplittingCache,
  pub hash: String,
  // lazy compilation visit module
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub used_chunk_ids: HashSet<String>,
  pub include_module_ids: IdentifierSet,

  pub file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  pub side_effects_free_modules: IdentifierSet,
  pub module_item_map: IdentifierMap<Vec<ModuleItem>>,
}

impl Compilation {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: BundleEntries,
    module_graph: ModuleGraph,
    plugin_driver: SharedPluginDriver,
    resolver_factory: Arc<ResolverFactory>,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      options,
      module_graph,
      make_failed_dependencies: HashSet::default(),
      make_failed_module: HashSet::default(),
      has_module_import_export_change: true,
      runtime_modules: Default::default(),
      runtime_module_code_generation_results: Default::default(),
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
      resolver_factory,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: IdentifierSet::default(),
      used_symbol_ref: HashSet::default(),
      exports_info_map: IdentifierMap::default(),
      optimize_analyze_result_map: IdentifierMap::default(),
      bailout_module_identifiers: IdentifierMap::default(),

      code_generation_results: Default::default(),
      code_generated_modules: Default::default(),

      cache,
      code_splitting_cache: Default::default(),
      hash: Default::default(),
      lazy_visit_modules: Default::default(),
      used_chunk_ids: Default::default(),

      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
      side_effects_free_modules: IdentifierSet::default(),
      module_item_map: IdentifierMap::default(),
      include_module_ids: IdentifierSet::default(),
    }
  }

  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn update_asset(
    &mut self,
    filename: &str,
    updater: impl FnOnce(BoxSource, AssetInfo) -> Result<(BoxSource, AssetInfo)>,
  ) -> Result<()> {
    // Safety: we don't move anything from compilation
    let assets = &mut self.assets;

    let (new_source, new_info) = match assets.remove(filename) {
      Some(CompilationAsset {
        source: Some(source),
        info,
      }) => updater(source, info)?,
      _ => {
        return Err(internal_error!(
          "Called Compilation.updateAsset for not existing filename {filename}"
        ))
      }
    };
    self.emit_asset(
      filename.to_owned(),
      CompilationAsset {
        source: Some(new_source),
        info: new_info,
      },
    );
    Ok(())
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

  pub fn rename_asset(&mut self, filename: &str, new_name: String) {
    if let Some(asset) = self.assets.remove(filename) {
      self.assets.insert(new_name.clone(), asset);
      self.chunk_by_ukey.iter_mut().for_each(|(_, chunk)| {
        if chunk.files.remove(filename) {
          chunk.files.insert(new_name.clone());
        }
      });
    }
  }

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
  }

  pub fn assets_mut(&mut self) -> &mut CompilationAssets {
    &mut self.assets
  }

  pub fn entrypoints(&self) -> &IndexMap<String, ChunkGroupUkey> {
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
  pub fn entry_data(&self) -> IndexMap<String, EntryData> {
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
        .modules_mut()
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

    let mut force_build_module = std::mem::take(&mut self.make_failed_module);
    let mut force_build_deps = std::mem::take(&mut self.make_failed_dependencies);
    let mut origin_module_deps = HashMap::default();
    // handle setup params
    if let SetupMakeParam::ModifiedFiles(files) = &params {
      force_build_module.extend(self.module_graph.modules().values().filter_map(|module| {
        // check has dependencies modified
        if self
          .module_graph
          .has_dependencies(&module.identifier(), files)
        {
          Some(module.identifier())
        } else {
          None
        }
      }));
      // collect origin_module_deps
      for module_id in &force_build_module {
        let mgm = self
          .module_graph
          .module_graph_module_by_identifier(module_id)
          .expect("module graph module not exist");
        let deps = mgm
          .all_depended_modules(&self.module_graph)
          .into_iter()
          .cloned()
          .collect::<Vec<_>>();
        origin_module_deps.insert(*module_id, deps);
      }
    }
    if let SetupMakeParam::ForceBuildDeps(deps) = params {
      force_build_deps.extend(deps);
    }

    // move deps bindings module to force_build_module
    for (dependency_id, _) in &force_build_deps {
      if let Some(mid) = self
        .module_graph
        .module_identifier_by_dependency_id(dependency_id)
      {
        force_build_module.insert(*mid);
      }
    }

    let mut need_check_isolated_module_ids = HashSet::default();
    let mut origin_module_issuers = HashMap::default();
    // calc need_check_isolated_module_ids & regen_module_issues
    for id in &force_build_module {
      if let Some(mgm) = self.module_graph.module_graph_module_by_identifier(id) {
        let depended_modules = mgm
          .all_depended_modules(&self.module_graph)
          .into_iter()
          .copied();
        need_check_isolated_module_ids.extend(depended_modules);
        origin_module_issuers.insert(*id, mgm.get_issuer().clone());
      }
    }

    let mut active_task_count = 0usize;
    let is_expected_shutdown = Arc::new(AtomicBool::new(false));
    let (result_tx, mut result_rx) = tokio::sync::mpsc::unbounded_channel::<Result<TaskResult>>();
    let mut factorize_queue = FactorizeQueue::new();
    let mut add_queue = AddQueue::new();
    let mut build_queue = BuildQueue::new();
    let mut process_dependencies_queue = ProcessDependenciesQueue::new();
    let mut make_failed_dependencies: HashSet<BuildDependency> = HashSet::default();
    let mut make_failed_module = HashSet::default();
    let mut errored = None;

    force_build_deps.extend(
      force_build_module
        .iter()
        .flat_map(|id| self.module_graph.revoke_module(id)),
    );

    force_build_deps
      .into_iter()
      .for_each(|(id, parent_module_identifier)| {
        let dependency = self
          .module_graph
          .dependency_by_id(&id)
          .expect("dependency not found");
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
            .map(|issuer| issuer.to_string()),
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

            let result = CatchUnwindFuture::create(task.run()).await;

            match result {
              Ok(result) => {
                if !is_expected_shutdown.load(Ordering::SeqCst) {
                  result_tx
                    .send(result)
                    .expect("Failed to send factorize result");
                }
              }
              Err(e) => {
                // panic on the tokio worker thread
                result_tx.send(Err(e)).expect("Failed to send panic info");
              }
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

            let result = CatchUnwindFuture::create(task.run()).await;

            match result {
              Ok(result) => {
                if !is_expected_shutdown.load(Ordering::SeqCst) {
                  result_tx.send(result).expect("Failed to send build result");
                }
              }
              Err(e) => {
                // panic on the tokio worker thread
                result_tx.send(Err(e)).expect("Failed to send panic info");
              }
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
              .map(|issuer| issuer.to_string()),
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
                mut module_graph_module,
                diagnostics,
                dependencies,
              } = task_result;
              let module_identifier = factory_result.module.identifier();

              tracing::trace!("Module created: {}", &module_identifier);
              if !diagnostics.is_empty() {
                make_failed_dependencies.insert((dependencies[0], original_module_identifier));
              }

              module_graph_module.set_issuer_if_unset(original_module_identifier);
              module_graph_module.factory_meta = Some(factory_result.factory_meta);
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
                  resolver_factory: self.resolver_factory.clone(),
                  compiler_options: self.options.clone(),
                  plugin_driver: self.plugin_driver.clone(),
                  cache: self.cache.clone(),
                });
              }
              AddTaskResult::ModuleReused { module, .. } => {
                tracing::trace!("Module reused: {}, skipping build", module.identifier());
              }
            },
            Ok(TaskResult::Build(task_result)) => {
              let BuildTaskResult {
                module,
                build_result,
                diagnostics,
              } = task_result;

              if !diagnostics.is_empty() {
                make_failed_module.insert(module.identifier());
              }

              tracing::trace!("Module built: {}", module.identifier());
              self.push_batch_diagnostic(diagnostics);

              self
                .file_dependencies
                .extend(build_result.build_info.file_dependencies.clone());
              self
                .context_dependencies
                .extend(build_result.build_info.context_dependencies.clone());
              self
                .missing_dependencies
                .extend(build_result.build_info.missing_dependencies.clone());
              self
                .build_dependencies
                .extend(build_result.build_info.build_dependencies.clone());

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
              self.module_graph.set_module_build_info_and_meta(
                &module.identifier(),
                build_result.build_info,
                build_result.build_meta,
              );
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

    self.make_failed_dependencies = make_failed_dependencies;
    self.make_failed_module = make_failed_module;
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
    // set origin module issues
    for (id, issuer) in origin_module_issuers {
      if let Some(mgm) = self.module_graph.module_graph_module_by_identifier_mut(&id) {
        mgm.set_issuer(issuer);
      }
    }

    // calc has_module_import_export_change
    self.has_module_import_export_change = if origin_module_deps.is_empty() {
      true
    } else {
      !origin_module_deps.into_iter().all(|(module_id, deps)| {
        if let Some(mgm) = self
          .module_graph
          .module_graph_module_by_identifier(&module_id)
        {
          mgm.all_depended_modules(&self.module_graph) == deps.iter().collect::<Vec<_>>()
        } else {
          false
        }
      })
    };

    // add context module and context element module to bailout_module_identifiers
    if self.options.builtins.tree_shaking.enable() {
      self.bailout_module_identifiers = self
        .module_graph
        .modules()
        .values()
        .par_bridge()
        .filter_map(|module| {
          if module.as_context_module().is_some() {
            let mut values = vec![(module.identifier(), BailoutFlag::CONTEXT_MODULE)];
            if let Some(dependencies) = self
              .module_graph
              .dependencies_by_module_identifier(&module.identifier())
            {
              for dependency in dependencies {
                if let Some(dependency_module) = self
                  .module_graph
                  .module_identifier_by_dependency_id(dependency)
                {
                  values.push((*dependency_module, BailoutFlag::CONTEXT_MODULE));
                }
              }
            }

            Some(values)
          } else {
            None
          }
        })
        .flatten()
        .collect();
    }

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
    issuer: Option<String>,
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
      resolver_factory: self.resolver_factory.clone(),
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
        .modules()
        .par_iter()
        .filter(filter_op)
        .filter(|(module_identifier, _)| {
          let runtimes = compilation
            .chunk_graph
            .get_module_runtimes(**module_identifier, &compilation.chunk_by_ukey);
          !runtimes.is_empty()
        })
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
    let results = self
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
      .collect::<FuturesResults<_>>();

    let chunk_ukey_and_manifest = results.into_inner();

    for (chunk_ukey, manifest) in chunk_ukey_and_manifest.into_iter() {
      for file_manifest in manifest.expect("We should return this error rathen expect") {
        let filename = file_manifest.filename().to_string();

        let current_chunk = self
          .chunk_by_ukey
          .get_mut(&chunk_ukey)
          .unwrap_or_else(|| panic!("chunk({chunk_ukey:?}) should be in chunk_by_ukey",));
        current_chunk.files.insert(filename.clone());

        self.emit_asset(
          filename.clone(),
          CompilationAsset::new(
            Some(CachedSource::new(file_manifest.source).boxed()),
            file_manifest.info,
          ),
        );

        _ = self
          .chunk_asset(chunk_ukey, filename, plugin_driver.clone())
          .await;
      }
      //
      // .into_iter()
      // .for_each(|file_manifest| {
      // });
    }
    // .for_each(|(chunk_ukey, manifest)| {
    // })
  }
  #[instrument(name = "compilation:process_asssets", skip_all)]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .write()
      .await
      .process_assets(ProcessAssetsArgs { compilation: self })
      .await
  }

  #[instrument(name = "compilation:chunk_asset", skip_all)]
  async fn chunk_asset(
    &mut self,
    chunk_ukey: ChunkUkey,
    filename: String,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let current_chunk = self
      .chunk_by_ukey
      .get(&chunk_ukey)
      .unwrap_or_else(|| panic!("chunk({chunk_ukey:?}) should be in chunk_by_ukey",));
    _ = plugin_driver
      .write()
      .await
      .chunk_asset(current_chunk, filename)
      .await;
    Ok(())
  }

  pub async fn optimize_dependency(
    &mut self,
  ) -> Result<TWithDiagnosticArray<OptimizeDependencyResult>> {
    optimizer::CodeSizeOptimizer::new(self).run().await
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

  #[instrument(name = "compilation:finish", skip_all)]
  pub async fn finish(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver.write().await.finish_modules(self).await?;
    Ok(())
  }

  #[instrument(name = "compilation:seal", skip_all)]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    use_code_splitting_cache(self, |compilation| async {
      build_chunk_graph(compilation)?;
      plugin_driver
        .write()
        .await
        .optimize_modules(compilation)
        .await?;
      plugin_driver
        .write()
        .await
        .optimize_chunks(compilation)
        .await?;
      Ok(compilation)
    })
    .await?;
    plugin_driver
      .write()
      .await
      .optimize_chunk_modules(self)
      .await?;

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
    let entries = self.entrypoints.values().map(|entrypoint_ukey| {
      let entrypoint = self
        .chunk_group_by_ukey
        .get(entrypoint_ukey)
        .expect("chunk group not found");
      entrypoint.get_runtime_chunk()
    });
    HashSet::from_iter(entries)
  }

  #[instrument(name = "compilation:process_runtime_requirements", skip_all)]
  pub async fn process_runtime_requirements(
    &mut self,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let mut module_runtime_requirements = self
      .module_graph
      .modules()
      .par_iter()
      .filter_map(|(_, module)| {
        if self
          .chunk_graph
          .get_number_of_module_chunks(module.identifier())
          > 0
        {
          let mut module_runtime_requirements: Vec<(RuntimeSpec, RuntimeGlobals)> = vec![];
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
      let mut set = RuntimeGlobals::default();
      for module in self
        .chunk_graph
        .get_chunk_modules(chunk_ukey, &self.module_graph)
      {
        if let Some(runtime_requirements) = self
          .chunk_graph
          .get_module_runtime_requirements(module.identifier(), &chunk.runtime)
        {
          set.add(*runtime_requirements);
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

      let mut set = RuntimeGlobals::default();

      for chunk_ukey in entry
        .get_all_referenced_chunks(&self.chunk_group_by_ukey)
        .iter()
      {
        let runtime_requirements = self.chunk_graph.get_chunk_runtime_requirements(chunk_ukey);
        set.add(*runtime_requirements);
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
    let chunk_hash_results = self
      .chunk_by_ukey
      .iter()
      .map(|(chunk_key, _)| async {
        let hash = plugin_driver
          .read()
          .await
          .chunk_hash(&ChunkHashArgs {
            chunk_ukey: *chunk_key,
            compilation: self,
          })
          .await;
        (*chunk_key, hash)
      })
      .collect::<FuturesResults<_>>()
      .into_inner();
    for (chunk_ukey, hash) in chunk_hash_results {
      if let Some(chunk) = self.chunk_by_ukey.get_mut(&chunk_ukey) {
        hash?.hash(&mut chunk.hash);
      }
    }
    let mut chunks = self.chunk_by_ukey.values_mut().collect::<Vec<_>>();
    chunks.sort_unstable_by_key(|chunk| chunk.ukey);
    chunks
      .par_iter_mut()
      .flat_map_iter(|chunk| {
        self
          .chunk_graph
          .get_ordered_chunk_modules(&chunk.ukey, &self.module_graph)
          .into_iter()
          .filter_map(|m| {
            self
              .code_generation_results
              .get_hash(&m.identifier(), Some(&chunk.runtime))
          })
          .inspect(|hash| hash.hash(&mut chunk.hash))
      })
      .collect::<Vec<_>>()
      .iter()
      .for_each(|hash| {
        hash.hash(&mut compilation_hasher);
      });

    tracing::trace!("hash normal chunks chunk hash");

    let runtime_chunk_ukeys = self.get_chunk_graph_entries();
    // runtime chunks should be hashed after all other chunks
    let content_hash_chunks = self
      .chunk_by_ukey
      .keys()
      .filter(|key| !runtime_chunk_ukeys.contains(key))
      .copied()
      .collect::<Vec<_>>();
    self.create_chunk_content_hash(content_hash_chunks, plugin_driver.clone())?;

    tracing::trace!("calculate normal chunks content hash");

    self.create_runtime_module_hash();

    tracing::trace!("hash runtime modules");

    let mut entry_chunks = self
      .chunk_by_ukey
      .values_mut()
      .filter(|chunk| runtime_chunk_ukeys.contains(&chunk.ukey))
      .collect::<Vec<_>>();
    entry_chunks.sort_unstable_by_key(|chunk| chunk.ukey);
    entry_chunks
      .par_iter_mut()
      .flat_map_iter(|chunk| {
        self
          .chunk_graph
          .get_chunk_runtime_modules_in_order(&chunk.ukey)
          .iter()
          .filter_map(|identifier| self.runtime_module_code_generation_results.get(identifier))
          .inspect(|(hash, _)| hash.hash(&mut chunk.hash))
      })
      .collect::<Vec<_>>()
      .iter()
      .for_each(|hash| {
        hash.hash(&mut compilation_hasher);
      });
    tracing::trace!("calculate runtime chunks hash");

    self.create_chunk_content_hash(
      Vec::from_iter(runtime_chunk_ukeys.into_iter()),
      plugin_driver.clone(),
    )?;
    tracing::trace!("calculate runtime chunks content hash");

    // TODO(ahabhgk): refactor, upper-level wrapper for format!("{:016x}")
    // 016 for length = 16
    self.hash = format!("{:016x}", compilation_hasher.finish());
    tracing::trace!("compilation hash");
    Ok(())
  }

  #[allow(clippy::unwrap_in_result)]
  fn create_chunk_content_hash(
    &mut self,
    chunks: Vec<ChunkUkey>,
    plugin_driver: SharedPluginDriver,
  ) -> Result<()> {
    let hash_results = chunks
      .iter()
      .map(|chunk_ukey| async {
        let hashes = plugin_driver
          .read()
          .await
          .content_hash(&ContentHashArgs {
            chunk_ukey: *chunk_ukey,
            compilation: self,
          })
          .await;
        (*chunk_ukey, hashes)
      })
      .collect::<FuturesResults<_>>()
      .into_inner();

    for (chunk_ukey, hashes) in hash_results {
      hashes?.into_iter().for_each(|hash| {
        if let Some(chunk) = self.chunk_by_ukey.get_mut(&chunk_ukey) && let Some((source_type, hash)) = hash {
          chunk.content_hash.insert(source_type, hash);
        }
      });
    }

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
    self.runtime_module_code_generation_results = self
      .runtime_modules
      .par_iter()
      .map(|(identifier, module)| {
        let source = module.generate(self);
        let mut hasher = Xxh3::new();
        module.identifier().hash(&mut hasher);
        source.source().hash(&mut hasher);
        (*identifier, (hasher.finish(), source))
      })
      .collect();
  }

  pub fn add_runtime_module(&mut self, chunk_ukey: &ChunkUkey, mut module: Box<dyn RuntimeModule>) {
    // add chunk runtime to prefix module identifier to avoid multiple entry runtime modules conflict
    let chunk = self
      .chunk_by_ukey
      .get(chunk_ukey)
      .expect("chunk not found by ukey");
    let runtime_module_identifier =
      ModuleIdentifier::from(format!("{:?}/{}", chunk.runtime, module.identifier()));
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

  pub fn get_path<'b, 'a: 'b>(&'a self, filename: &Filename, mut data: PathData<'b>) -> String {
    if data.hash.is_none() {
      data.hash = Some(&self.hash);
    }
    filename.render(data, None)
  }

  pub fn get_path_with_info<'b, 'a: 'b>(
    &'a self,
    filename: &Filename,
    mut data: PathData<'b>,
  ) -> (String, AssetInfo) {
    let mut info = AssetInfo::default();
    if data.hash.is_none() {
      data.hash = Some(&self.hash);
    }
    let path = filename.render(data, Some(&mut info));
    (path, info)
  }

  pub fn get_asset_path(&self, filename: &Filename, data: PathData) -> String {
    filename.render(data, None)
  }

  pub fn get_asset_path_with_info(
    &self,
    filename: &Filename,
    data: PathData,
  ) -> (String, AssetInfo) {
    let mut info = AssetInfo::default();
    let path = filename.render(data, Some(&mut info));
    (path, info)
  }
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;

#[derive(Debug, Clone)]
pub struct CompilationAsset {
  pub source: Option<BoxSource>,
  pub info: AssetInfo,
}

impl From<BoxSource> for CompilationAsset {
  fn from(value: BoxSource) -> Self {
    Self::new(Some(value), Default::default())
  }
}

impl CompilationAsset {
  pub fn new(source: Option<BoxSource>, info: AssetInfo) -> Self {
    Self { source, info }
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
  pub immutable: bool,
  /// whether the asset is minimized
  pub minimized: bool,
  /// the value(s) of the full hash used for this asset
  // pub full_hash:
  /// the value(s) of the chunk hash used for this asset
  // pub chunk_hash:
  /// the value(s) of the module hash used for this asset
  // pub module_hash:
  /// the value(s) of the content hash used for this asset
  pub content_hash: HashSet<String>,
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

  pub fn with_content_hashes(mut self, v: HashSet<String>) -> Self {
    self.content_hash = v;
    self
  }

  pub fn set_content_hash(&mut self, v: String) {
    self.content_hash.insert(v);
  }

  pub fn set_immutable(&mut self, v: bool) {
    self.immutable = v;
  }
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}
