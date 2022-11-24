use dashmap::DashSet;
use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::{
  hash_map::Entry,
  hash_set::Entry::{Occupied, Vacant},
  HashMap, HashSet,
};
use indexmap::IndexSet;
use petgraph::prelude::GraphMap;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::{
  borrow::BorrowMut,
  collections::VecDeque,
  fmt::Debug,
  hash::{Hash, Hasher},
  marker::PhantomPinned,
  path::PathBuf,
  pin::Pin,
  str::FromStr,
  sync::atomic::{AtomicU32, Ordering},
  sync::Arc,
};
use sugar_path::SugarPath;
use swc_atoms::JsWord;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedSender};
use tracing::instrument;
use xxhash_rust::xxh3::Xxh3;

use rspack_error::{
  errors_to_diagnostics, Diagnostic, Error, IntoTWithDiagnosticArray, Result, Severity,
  TWithDiagnosticArray,
};
use rspack_sources::BoxSource;
use ustr::{ustr, Ustr};

use crate::{
  is_source_equal, join_string_component,
  split_chunks::code_splitting,
  tree_shaking::{
    visitor::{ModuleRefAnalyze, SymbolRef, TreeShakingResult},
    BailoutReason, OptimizeDependencyResult,
  },
  AdditionalChunkRuntimeRequirementsArgs, BoxModule, BuildContext, BundleEntries, Chunk,
  ChunkByUkey, ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkKind, ChunkUkey, CodeGenerationResult,
  CodeGenerationResults, CompilerOptions, Dependency, EntryItem, Entrypoint, LoaderRunnerRunner,
  ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleRule, Msg, NormalModuleFactory,
  NormalModuleFactoryContext, ProcessAssetsArgs, RenderManifestArgs, ResolveKind, RuntimeModule,
  SharedPluginDriver, Stats, VisitedModuleIdentity,
};
use rspack_symbol::Symbol;

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, Vec<EntryItem>>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub runtime_modules: HashMap<String, Box<dyn RuntimeModule>>,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: HashMap<ChunkUkey, Chunk>,
  pub chunk_group_by_ukey: HashMap<ChunkGroupUkey, ChunkGroup>,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  pub emitted_assets: DashSet<String, hashbrown::hash_map::DefaultHashBuilder>,
  diagnostics: IndexSet<Diagnostic, hashbrown::hash_map::DefaultHashBuilder>,
  pub(crate) plugin_driver: SharedPluginDriver,
  pub(crate) loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: HashSet<ModuleIdentifier>,
  /// Collecting all used export symbol
  pub used_symbol: HashSet<Symbol>,
  /// Collecting all module that need to skip in tree-shaking ast modification phase
  pub bailout_module_identifiers: HashMap<Ustr, BailoutReason>,
  #[cfg(debug_assertions)]
  pub tree_shaking_result: HashMap<Ustr, TreeShakingResult>,

  pub code_generation_results: CodeGenerationResults,
  pub code_generated_modules: HashSet<ModuleIdentifier>,

  // TODO: make compilation safer
  _pin: PhantomPinned,
}
impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: BundleEntries,
    visited_module_id: VisitedModuleIdentity,
    module_graph: ModuleGraph,
    plugin_driver: SharedPluginDriver,
    loader_runner_runner: Arc<LoaderRunnerRunner>,
  ) -> Self {
    Self {
      options,
      visited_module_id,
      module_graph,
      chunk_by_ukey: Default::default(),
      chunk_group_by_ukey: Default::default(),
      entries: HashMap::from_iter(entries),
      chunk_graph: Default::default(),
      entrypoints: Default::default(),
      assets: Default::default(),
      emitted_assets: Default::default(),
      diagnostics: Default::default(),
      plugin_driver,
      loader_runner_runner,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: HashSet::new(),
      used_symbol: HashSet::new(),
      #[cfg(debug_assertions)]
      tree_shaking_result: HashMap::new(),
      bailout_module_identifiers: HashMap::new(),

      code_generation_results: Default::default(),
      code_generated_modules: Default::default(),

      runtime_modules: HashMap::default(),
      _pin: PhantomPinned,
    }
  }
  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, vec![detail]);
  }

  pub fn update_asset(
    self: Pin<&mut Self>,
    filename: &str,
    updater: impl FnOnce(&mut CompilationAsset) -> Result<()>,
  ) -> Result<()> {
    // Safety: we don't move anything from compilation
    let assets = unsafe { self.map_unchecked_mut(|c| &mut c.assets) }.get_mut();

    match assets.get_mut(filename) {
      Some(asset) => updater(asset),
      None => Err(Error::InternalError(format!(
        "Called Compilation.updateAsset for not existing filename {}",
        filename
      ))),
    }
  }

  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
    tracing::trace!("Emit asset {}", filename);
    if let Some(mut original) = self.assets.remove(&filename) {
      let is_source_equal = is_source_equal(&original.source, &asset.source);
      if !is_source_equal {
        tracing::error!(
          "Emit Duplicate Filename({}), is_source_equal: {:?}",
          filename,
          is_source_equal
        );
        self.push_batch_diagnostic(
          rspack_error::Error::InternalError(format!(
            "Conflict: Multiple assets emit different content to the same filename {}{}",
            filename,
            // TODO: source file name
            ""
          ))
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
    id: String,
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
      let chunk = Chunk::new(Some(name.clone()), id, ChunkKind::Normal);
      let ukey = chunk.ukey;
      named_chunks.insert(name, chunk.ukey);
      chunk_by_ukey.entry(ukey).or_insert_with(|| chunk)
    }
  }

  pub fn add_chunk(chunk_by_ukey: &mut ChunkByUkey, id: String) -> &mut Chunk {
    let chunk = Chunk::new(None, id, ChunkKind::Normal);
    let ukey = chunk.ukey;
    chunk_by_ukey.insert(chunk.ukey, chunk);
    chunk_by_ukey.get_mut(&ukey).expect("chunk not found")
  }
  #[instrument(name = "entry_deps")]
  pub fn entry_dependencies(&self) -> HashMap<String, Vec<Dependency>> {
    self
      .entries
      .iter()
      .map(|(name, items)| {
        let name = name.clone();
        let items = items
          .iter()
          .map(|detail| Dependency {
            parent_module_identifier: None,
            detail: ModuleDependency {
              specifier: detail.path.clone(),
              kind: ResolveKind::Import,
              span: None,
            },
          })
          .collect();
        (name, items)
      })
      .collect()
  }

  #[instrument(name = "compilation:make", skip_all)]
  pub async fn make(&mut self, entry_deps: HashMap<String, Vec<Dependency>>) {
    if let Some(e) = self.plugin_driver.clone().read().await.make(self).err() {
      self.push_batch_diagnostic(e.into());
    }
    let active_task_count: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    entry_deps.into_iter().for_each(|(_, deps)| {
      deps.into_iter().for_each(|dep| {
        let normal_module_factory = NormalModuleFactory::new(
          NormalModuleFactoryContext {
            module_name: None,
            active_task_count: active_task_count.clone(),
            module_type: None,
            side_effects: None,
            options: self.options.clone(),
          },
          dep,
          tx.clone(),
          self.plugin_driver.clone(),
        );
        tokio::task::spawn(async move { normal_module_factory.create(true).await });
      })
    });

    tokio::task::block_in_place(|| {
      loop {
        match rx.try_recv() {
          Ok(item) => match item {
            Msg::ModuleCreated(module_with_diagnostic) => {
              let (mgm, module, original_module_identifier, dependency_id, dependency, is_entry) =
                *module_with_diagnostic.inner;

              let module_identifier = module.identifier();

              match self
                .visited_module_id
                .entry((module_identifier.clone().into(), dependency.detail.clone()))
              {
                Occupied(_) => {
                  if let Err(err) = tx.send(Msg::ModuleReused(
                    (
                      original_module_identifier,
                      dependency_id,
                      module_identifier.into(),
                    )
                      .with_diagnostic(module_with_diagnostic.diagnostic),
                  )) {
                    tracing::trace!("fail to send msg {:?}", err)
                  }
                  continue;
                }
                Vacant(vac) => {
                  vac.insert();
                }
              }

              if is_entry {
                self
                  .entry_module_identifiers
                  .insert(module_identifier.into());
              }

              self.handle_module_build_and_dependencies(
                original_module_identifier,
                module,
                dependency_id,
                active_task_count.clone(),
                tx.clone(),
              );
              // After module created we add module graph module into module graph
              self.module_graph.add_module_graph_module(mgm);
            }
            Msg::ModuleReused(result_with_diagnostics) => {
              let (original_module_identifier, dependency_id, module_identifier) =
                result_with_diagnostics.inner;
              self.push_batch_diagnostic(result_with_diagnostics.diagnostic);
              if let Err(err) = self.module_graph.set_resolved_module(
                original_module_identifier,
                dependency_id,
                module_identifier.clone(),
              ) {
                // If build error message is failed to send, then we should manually decrease the active task count
                // Otherwise, it will be gracefully handled by the error message handler.
                if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err))
                {
                  active_task_count.fetch_sub(1, Ordering::SeqCst);
                  tracing::trace!("fail to send msg {:?}", err);
                }

                // Early bail out if task is failed to finish
                return;
              };

              // Gracefully exit
              active_task_count.fetch_sub(1, Ordering::SeqCst);
            }
            Msg::ModuleResolved(result_with_diagnostics) => {
              let (original_module_identifier, dependency_id, module, deps) =
                result_with_diagnostics.inner;
              self.push_batch_diagnostic(result_with_diagnostics.diagnostic);

              {
                let mut module_graph_module = self
                  .module_graph
                  .module_graph_module_by_identifier_mut(&module.identifier())
                  .unwrap();
                module_graph_module.all_dependencies = *deps;
              }
              if let Err(err) = self.module_graph.set_resolved_module(
                original_module_identifier,
                dependency_id,
                module.identifier().into(),
              ) {
                // If build error message is failed to send, then we should manually decrease the active task count
                // Otherwise, it will be gracefully handled by the error message handler.
                if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(
                  module.identifier().into(),
                  err,
                )) {
                  active_task_count.fetch_sub(1, Ordering::SeqCst);
                  tracing::trace!("fail to send msg {:?}", err)
                }
                // Early bail out if task is failed to finish
                return;
              };

              self.module_graph.add_module(module);

              // Gracefully exit
              active_task_count.fetch_sub(1, Ordering::SeqCst);
            }
            Msg::ModuleBuiltErrorEncountered(module_identifier, err) => {
              self
                .module_graph
                .module_identifier_to_module
                .remove(&module_identifier);
              self
                .module_graph
                .module_identifier_to_module_graph_module
                .remove(&module_identifier);
              self.push_batch_diagnostic(err.into());
              active_task_count.fetch_sub(1, Ordering::SeqCst);
            }
            Msg::ModuleCreationCanceled => {
              active_task_count.fetch_sub(1, Ordering::SeqCst);
            }
            Msg::DependencyReference(dep, module_identifier) => {
              self.module_graph.add_dependency(dep, module_identifier);
            }
            Msg::ModuleCreationErrorEncountered(err) => {
              active_task_count.fetch_sub(1, Ordering::SeqCst);
              self.push_batch_diagnostic(err.into());
            }
          },
          Err(TryRecvError::Disconnected) => {
            break;
          }
          Err(TryRecvError::Empty) => {
            if active_task_count.load(Ordering::SeqCst) == 0 {
              break;
            }
          }
        }
      }
    });

    tracing::debug!("All task is finished");
  }

  fn handle_module_build_and_dependencies(
    &self,
    original_module_identifier: Option<ModuleIdentifier>,
    mut module: BoxModule,
    dependency_id: u32,
    active_task_count: Arc<AtomicU32>,
    tx: UnboundedSender<Msg>,
  ) {
    let compiler_options = self.options.clone();
    let loader_runner_runner = self.loader_runner_runner.clone();
    let plugin_driver = self.plugin_driver.clone();

    let module_identifier = module.identifier().into_owned();

    tokio::spawn(async move {
      let resolved_loaders = {
        if let Some(normal_module) = module.as_normal_module() {
          let resource_data = normal_module.resource_resolved_data();

          match compiler_options
          .module
          .rules
          .iter()
          .filter_map(|module_rule| -> Option<Result<&ModuleRule>> {
            if let Some(func) = &module_rule.func__ {
              match func(resource_data) {
                Ok(result) => {
                  if result {
                    return Some(Ok(module_rule));
                  }
                  return None
                },
                Err(e) => {
                  return Some(Err(e.into()))
                }
              }
            }

            // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
            // See: https://webpack.js.org/configuration/module/#ruletest
            if let Some(test_rule) = &module_rule.test && test_rule.is_match(&resource_data.resource) {
              return Some(Ok(module_rule));
            } else if let Some(resource_rule) = &module_rule.resource && resource_rule.is_match(&resource_data.resource) {
              return Some(Ok(module_rule));
            }

            if let Some(resource_query_rule) = &module_rule.resource_query && let Some(resource_query) = &resource_data.resource_query && resource_query_rule.is_match(resource_query) {
              return Some(Ok(module_rule));
            }

            None
          })
          .collect::<Result<Vec<_>>>() {
            Ok(result) => result,
            Err(err) => {
              // If build error message is failed to send, then we should manually decrease the active task count
              // Otherwise, it will be gracefully handled by the error message handler.
              if let Err(err) =
                tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err))
              {
                active_task_count.fetch_sub(1, Ordering::SeqCst);
                tracing::trace!("fail to send msg {:?}", err)
              }
              return
            }
          }
        } else {
          vec![]
        }
      };

      let resolved_loaders = resolved_loaders
        .into_iter()
        .flat_map(|module_rule| module_rule.uses.iter().map(Box::as_ref).rev())
        .collect::<Vec<_>>();

      if let Err(e) = plugin_driver
        .read()
        .await
        .build_module(module.as_mut())
        .await
      {
        if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(
          module.identifier().into(),
          e,
        )) {
          tracing::trace!("fail to send msg {:?}", err);
        }
      }

      match module
        .build(BuildContext {
          resolved_loaders,
          loader_runner_runner: &loader_runner_runner,
          compiler_options: &compiler_options,
        })
        .await
      {
        Ok(build_result) => {
          if let Err(e) = plugin_driver
            .read()
            .await
            .succeed_module(module.as_ref())
            .await
          {
            if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(
              module.identifier().into(),
              e,
            )) {
              tracing::trace!("fail to send msg {:?}", err);
            }
          }

          let module_identifier = module.identifier();

          let (build_result, diagnostics) = build_result.split_into_parts();

          let deps = build_result
            .dependencies
            .into_iter()
            .map(|dep| Dependency {
              parent_module_identifier: Some(module_identifier.clone().into()),
              detail: dep,
            })
            .collect::<Vec<_>>();

          Compilation::process_module_dependencies(
            deps.clone(),
            active_task_count.clone(),
            tx.clone(),
            plugin_driver.clone(),
            compiler_options.clone(),
          );

          // If build error message is failed to send, then we should manually decrease the active task count
          // Otherwise, it will be gracefully handled by the error message handler.
          if let Err(err) = tx.send(Msg::ModuleResolved(
            (
              original_module_identifier.clone(),
              dependency_id,
              module,
              Box::new(deps),
            )
              .with_diagnostic(diagnostics),
          )) {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            tracing::trace!("fail to send msg {:?}", err);

            // Manually add return here to prevent the following code from being executed in the future
            #[allow(clippy::needless_return)]
            return;
          };
        }
        Err(err) => {
          // If build error message is failed to send, then we should manually decrease the active task count
          // Otherwise, it will be gracefully handled by the error message handler.
          if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err)) {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            tracing::trace!("fail to send msg {:?}", err);
          }

          // Manually add return here to prevent the following code from being executed in the future
          #[allow(clippy::needless_return)]
          return;
        }
      }
    });
  }

  fn process_module_dependencies(
    dependencies: Vec<Dependency>,
    active_task_count: Arc<AtomicU32>,
    tx: UnboundedSender<Msg>,
    plugin_driver: SharedPluginDriver,
    compiler_options: Arc<CompilerOptions>,
  ) {
    dependencies.into_iter().for_each(|dep| {
      let normal_module_factory = NormalModuleFactory::new(
        NormalModuleFactoryContext {
          module_name: None,
          module_type: None,
          active_task_count: active_task_count.clone(),
          side_effects: None,
          options: compiler_options.clone(),
        },
        dep,
        tx.clone(),
        plugin_driver.clone(),
      );

      tokio::task::spawn(async move {
        normal_module_factory.create(false).await;
      });
    })
  }

  #[instrument(name = "compilation:code_generation")]
  fn code_generation(&mut self) -> Result<()> {
    let results = self
      .module_graph
      .module_identifier_to_module
      .par_iter()
      .map(|(module_identifier, module)| {
        module
          .code_generation(self)
          .map(|result| (module_identifier.clone(), result))
      })
      .collect::<Result<Vec<(ModuleIdentifier, CodeGenerationResult)>>>()?;

    results.into_iter().for_each(|(module_identifier, result)| {
      self
        .code_generated_modules
        .insert(module_identifier.clone());

      let runtimes = self
        .chunk_graph
        .get_module_runtimes(&module_identifier, &self.chunk_by_ukey);

      self
        .code_generation_results
        .module_generation_result_map
        .insert(module_identifier.clone(), result);
      for runtime in runtimes.values() {
        self.code_generation_results.add(
          module_identifier.clone(),
          runtime.clone(),
          module_identifier.clone(),
        );
      }
    });

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
          });

        if let Ok(manifest) = &manifest {
          tracing::debug!(
            "For Chunk({}), collected assets: {:?}",
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
        manifest.unwrap().into_iter().for_each(|file_manifest| {
          let current_chunk = self.chunk_by_ukey.get_mut(&chunk_ukey).unwrap();
          current_chunk
            .files
            .insert(file_manifest.filename().to_string());

          self.emit_asset(
            file_manifest.filename().to_string(),
            CompilationAsset::new(file_manifest.source, AssetInfo::default()),
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
    let mut analyze_results = self
      .module_graph
      .module_identifier_to_module_graph_module
      .par_iter()
      .filter_map(|(module_identifier, mgm)| {
        let uri_key = ustr(module_identifier);
        let ast = match mgm.module_type {
          crate::ModuleType::Js
          | crate::ModuleType::Jsx | crate::ModuleType::Tsx
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
        //        let ast = ast.as_javascript().unwrap();
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
        //     &analyzer.reference_map,
        //     &analyzer.reachable_import_and_export,
        //     &analyzer.used_symbol_ref
        //   );
        // }
        Some((uri_key, analyzer.into()))
      })
      .collect::<HashMap<Ustr, TreeShakingResult>>();
    let mut used_symbol_ref: HashSet<SymbolRef> = HashSet::default();
    let mut bail_out_module_identifiers = HashMap::default();
    for analyze_result in analyze_results.values() {
      used_symbol_ref.extend(analyze_result.used_symbol_ref.iter().cloned());
      bail_out_module_identifiers.extend(analyze_result.bail_out_module_identifiers.clone());
    }

    // calculate relation of module that has `export * from 'xxxx'`
    let inherit_export_ref_graph = create_inherit_graph(&analyze_results);
    // key is the module_id of module that potential have reexport all symbol from other module
    // value is the set which contains several module_id the key related module need to inherit
    let map_of_inherit_map = get_extends_map(&inherit_export_ref_graph);

    for (module_id, inherit_export_module_id) in map_of_inherit_map.iter() {
      // This is just a work around for rustc checker, because we have immutable and mutable borrow at the same time.
      let mut inherit_export_maps = {
        let main_module = analyze_results.get_mut(module_id).unwrap();
        std::mem::take(&mut main_module.inherit_export_maps)
      };
      for inherit_export_module_identifier in inherit_export_module_id {
        let export_module = analyze_results
          .get(inherit_export_module_identifier)
          .unwrap()
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
        inherit_export_maps.insert(*inherit_export_module_identifier, export_module);
      }
      analyze_results
        .get_mut(module_id)
        .unwrap()
        .inherit_export_maps = inherit_export_maps;
    }
    let mut errors = vec![];
    let mut used_symbol = HashSet::new();
    // Marking used symbol and all reachable export symbol from the used symbol for each module
    let used_symbol_from_import = mark_used_symbol_with(
      &analyze_results,
      VecDeque::from_iter(used_symbol_ref.into_iter()),
      &bail_out_module_identifiers,
      &mut errors,
    );

    used_symbol.extend(used_symbol_from_import);

    // We considering all export symbol in each entry module as used for now
    for entry in self.entry_modules() {
      let used_symbol_set = collect_reachable_symbol(
        &analyze_results,
        ustr(&entry),
        &bail_out_module_identifiers,
        &mut errors,
      );
      used_symbol.extend(used_symbol_set);
    }
    // dbg!(&used_symbol, &bail_out_module_identifiers);
    Ok(
      OptimizeDependencyResult {
        used_symbol,
        analyze_results,
        bail_out_module_identifiers,
      }
      .with_diagnostic(errors_to_diagnostics(errors)),
    )
  }

  pub async fn done(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let stats = &mut Stats::new(self);
    plugin_driver.write().await.done(stats).await?;
    Ok(())
  }

  pub fn entry_modules(&self) -> impl Iterator<Item = String> {
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
    code_splitting(self)?;

    plugin_driver.write().await.optimize_chunks(self)?;

    self.code_generation()?;

    self
      .process_runtime_requirements(plugin_driver.clone())
      .await?;

    self.create_hash();

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
          .get_number_of_module_chunks(&module.identifier().into())
          > 0
        {
          let mut module_runtime_requirements: Vec<(HashSet<String>, HashSet<String>)> = vec![];
          for runtime in self
            .chunk_graph
            .get_module_runtimes(&module.identifier().into(), &self.chunk_by_ukey)
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
          module_identifier,
          runtime,
          std::mem::take(requirements),
        )
      }
    }
    tracing::trace!("runtime requirements.modules");

    let mut chunk_requirements = HashMap::new();
    for (chunk_ukey, chunk) in self.chunk_by_ukey.iter() {
      let mut set = HashSet::new();
      for module in self
        .chunk_graph
        .get_chunk_modules(chunk_ukey, &self.module_graph)
      {
        if let Some(runtime_requirements) = self
          .chunk_graph
          .get_module_runtime_requirements(&module.module_identifier, &chunk.runtime)
        {
          set.extend(runtime_requirements.clone());
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

      let mut set = HashSet::new();

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

  #[instrument(skip_all)]
  pub fn create_hash(&mut self) {
    for (chunk_ukey, chunk) in self.chunk_by_ukey.iter_mut() {
      for mgm in self
        .chunk_graph
        .get_chunk_modules(chunk_ukey, &self.module_graph)
      {
        if let Some(module) = self
          .module_graph
          .module_by_identifier(&mgm.module_identifier)
        {
          module.hash(&mut chunk.hash);
        }
      }
    }
    tracing::trace!("hash chunks");

    for entry_ukey in self.get_chunk_graph_entries().iter() {
      let mut hasher = Xxh3::new();
      for identifier in self
        .chunk_graph
        .get_chunk_runtime_modules_in_order(entry_ukey)
      {
        if let Some(module) = self.runtime_modules.get(identifier) {
          module.hash(self, &mut hasher);
        }
      }
      let entry = self
        .chunk_by_ukey
        .get_mut(entry_ukey)
        .expect("chunk not found by ukey");
      format!("{:x}", hasher.finish()).hash(&mut entry.hash);
    }
    tracing::trace!("hash runtime chunks");
  }

  pub fn add_runtime_module(&mut self, chunk_ukey: &ChunkUkey, mut module: Box<dyn RuntimeModule>) {
    module.attach(*chunk_ukey);
    self.chunk_graph.add_module(module.identifier());
    self
      .chunk_graph
      .connect_chunk_and_module(*chunk_ukey, module.identifier());
    self
      .chunk_graph
      .connect_chunk_and_runtime_module(*chunk_ukey, module.identifier());
    self.runtime_modules.insert(module.identifier(), module);
  }
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;

#[derive(Debug, Clone)]
pub struct CompilationAsset {
  pub source: BoxSource,
  pub info: AssetInfo,
}

impl CompilationAsset {
  pub fn new(source: BoxSource, info: AssetInfo) -> Self {
    Self { source, info }
  }

  pub fn get_source(&self) -> &BoxSource {
    &self.source
  }

  pub fn get_source_mut(&mut self) -> &mut BoxSource {
    &mut self.source
  }

  pub fn set_source(&mut self, source: BoxSource) {
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

fn collect_reachable_symbol(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  entry_identifier: Ustr,
  bailout_module_identifiers: &HashMap<Ustr, BailoutReason>,
  errors: &mut Vec<Error>,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::new();
  let mut q = VecDeque::new();
  let entry_module_result = match analyze_map.get(&entry_identifier) {
    Some(result) => result,
    None => {
      panic!("Can't get analyze result from entry_identifier");
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
      analyze_map,
      &mut q,
      bailout_module_identifiers,
      errors,
    );
  }

  while let Some(sym_ref) = q.pop_front() {
    mark_symbol(
      sym_ref,
      &mut used_symbol_set,
      analyze_map,
      &mut q,
      bailout_module_identifiers,
      errors,
    );
  }
  used_symbol_set
}

fn mark_used_symbol_with(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  mut init_queue: VecDeque<SymbolRef>,
  bailout_module_identifiers: &HashMap<Ustr, BailoutReason>,
  errors: &mut Vec<Error>,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::new();
  let mut visited = HashSet::new();

  while let Some(sym_ref) = init_queue.pop_front() {
    if visited.contains(&sym_ref) {
      continue;
    } else {
      visited.insert(sym_ref.clone());
    }
    mark_symbol(
      sym_ref,
      &mut used_symbol_set,
      analyze_map,
      &mut init_queue,
      bailout_module_identifiers,
      errors,
    );
  }
  used_symbol_set
}

fn mark_symbol(
  symbol_ref: SymbolRef,
  used_symbol_set: &mut HashSet<Symbol>,
  analyze_map: &HashMap<Ustr, TreeShakingResult>,
  q: &mut VecDeque<SymbolRef>,
  bailout_module_identifiers: &HashMap<Ustr, BailoutReason>,
  errors: &mut Vec<Error>,
) {
  // We don't need mark the symbol usage if it is from a bailout module because
  // bailout module will skipping tree-shaking anyway
  // if debug_care_module_id(symbol_ref.module_identifier()) {
  //   dbg!(&symbol_ref);
  // }
  let is_bailout_module_identifier =
    bailout_module_identifiers.contains_key(&symbol_ref.module_identifier());
  match symbol_ref {
    SymbolRef::Direct(symbol) => match used_symbol_set.entry(symbol) {
      Occupied(_) => {}
      Vacant(vac) => {
        let module_result = analyze_map.get(&vac.get().uri()).unwrap();
        if let Some(set) = module_result
          .reachable_import_of_export
          .get(&vac.get().id().atom)
        {
          q.extend(set.iter().cloned());
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
        if let Some(symbol_ref) = module_result.import_map.get(vac.get().id()) {
          q.push_back(symbol_ref.clone());
        }
        vac.insert();
      }
    },
    SymbolRef::Indirect(indirect_symbol) => {
      let module_result = match analyze_map.get(&indirect_symbol.uri) {
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
          for (module_identifier, extends_export_map) in module_result.inherit_export_maps.iter() {
            if let Some(value) = extends_export_map.get(&indirect_symbol.id) {
              ret.push((module_identifier, value));
            }
            has_bailout_module_identifiers = has_bailout_module_identifiers
              || bailout_module_identifiers.contains_key(module_identifier);
          }
          match ret.len() {
            0 => {
              // TODO: Better diagnostic handle if source module does not have the export
              // let map = analyze_map.get(&module_result.module_identifier).unwrap();
              // dbg!(&map);
              if !is_bailout_module_identifier && !has_bailout_module_identifiers {
                eprint!(
                  "{} did not export `{}`, imported by {}",
                  module_result.module_identifier,
                  indirect_symbol.id,
                  indirect_symbol.importer()
                );
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
              let cwd = std::env::current_dir();
              let module_identifier_list = ret
                .iter()
                .map(|(module_identifier, _)| {
                  // try to use relative path which should have better DX
                  match cwd {
                    Ok(ref cwd) => {
                      let p = PathBuf::from_str(module_identifier.as_str()).unwrap();
                      p.relative(cwd.as_path())
                        .as_path()
                        .to_string_lossy()
                        .to_string()
                    }
                    // if we can't get the cwd, fallback to module identifier
                    Err(_) => module_identifier.to_string(),
                  }
                })
                .collect::<Vec<_>>();
              error_message += &join_string_component(module_identifier_list);
              errors.push(Error::InternalError(error_message));
              ret[0].1.clone()
            }
          }
        }
      };
      q.push_back(symbol);
    }
    SymbolRef::Star(star) => {
      // If a star ref is used. e.g.
      // ```js
      // import * as all from './test.js'
      // all
      // ```
      // then, all the exports in `test.js` including
      // export defined in `test.js` and all realted
      // reexport should be marked as used
      let module_result = analyze_map.get(&star).unwrap();
      for symbol_ref in module_result.export_map.values() {
        q.push_back(symbol_ref.clone());
      }

      for (_, extend_map) in module_result.inherit_export_maps.iter() {
        q.extend(extend_map.values().cloned());
      }
    }
  }
}

fn get_extends_map(
  export_all_ref_graph: &GraphMap<&Ustr, (), petgraph::Directed>,
) -> HashMap<Ustr, HashSet<Ustr>> {
  let mut map = HashMap::new();
  for node in export_all_ref_graph.nodes() {
    let reachable_set = get_reachable(*node, export_all_ref_graph);
    map.insert(*node, reachable_set);
  }
  map
}
fn get_reachable(start: Ustr, g: &GraphMap<&Ustr, (), petgraph::Directed>) -> HashSet<Ustr> {
  let mut visited: HashSet<Ustr> = HashSet::new();
  let mut reachable_module_id = HashSet::new();
  let mut q = VecDeque::from_iter([start]);
  while let Some(cur) = q.pop_front() {
    match visited.entry(cur) {
      hashbrown::hash_set::Entry::Occupied(_) => continue,
      hashbrown::hash_set::Entry::Vacant(vac) => vac.insert(),
    }
    if cur != start {
      reachable_module_id.insert(cur);
    }
    q.extend(g.neighbors_directed(&cur, petgraph::Direction::Outgoing));
  }
  reachable_module_id
}

fn create_inherit_graph(
  analyze_map: &HashMap<Ustr, TreeShakingResult>,
) -> GraphMap<&Ustr, (), petgraph::Directed> {
  let mut g = petgraph::graphmap::DiGraphMap::new();
  for (module_id, result) in analyze_map.iter() {
    for export_all_module_id in result.inherit_export_maps.keys() {
      g.add_edge(module_id, export_all_module_id, ());
    }
  }
  g
}
