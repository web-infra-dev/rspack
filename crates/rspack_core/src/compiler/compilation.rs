use std::{
  borrow::BorrowMut,
  collections::VecDeque,
  fmt::Debug,
  marker::PhantomPinned,
  path::PathBuf,
  pin::Pin,
  str::FromStr,
  sync::atomic::{AtomicU32, Ordering},
  sync::Arc,
};

use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::{
  hash_map::Entry,
  hash_set::Entry::{Occupied, Vacant},
  HashMap, HashSet,
};
use indexmap::IndexSet;
use petgraph::prelude::GraphMap;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use sugar_path::SugarPath;
use swc_atoms::JsWord;
use tokio::sync::mpsc::{error::TryRecvError, UnboundedSender};
use tracing::instrument;

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
    OptimizeDependencyResult,
  },
  BuildContext, BundleEntries, Chunk, ChunkByUkey, ChunkGraph, ChunkGroup, ChunkGroupUkey,
  ChunkUkey, CodeGenerationResult, CodeGenerationResults, CompilerOptions, Dependency, EntryItem,
  Entrypoint, LoaderRunnerRunner, ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleRule, Msg,
  NormalModule, NormalModuleFactory, NormalModuleFactoryContext, ProcessAssetsArgs,
  RenderManifestArgs, RenderRuntimeArgs, ResolveKind, Runtime, SharedPluginDriver, Stats,
  VisitedModuleIdentity,
};
use rspack_symbol::Symbol;

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, Vec<EntryItem>>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: HashMap<ChunkUkey, Chunk>,
  pub chunk_group_by_ukey: HashMap<ChunkGroupUkey, ChunkGroup>,
  pub runtime: Runtime,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  diagnostics: IndexSet<Diagnostic, hashbrown::hash_map::DefaultHashBuilder>,
  pub(crate) plugin_driver: SharedPluginDriver,
  pub(crate) loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub(crate) _named_chunk: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: HashSet<ModuleIdentifier>,
  pub used_symbol: HashSet<Symbol>,
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
      runtime: Default::default(),
      entrypoints: Default::default(),
      assets: Default::default(),
      diagnostics: Default::default(),
      plugin_driver,
      loader_runner_runner,
      _named_chunk: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: HashSet::new(),
      used_symbol: HashSet::new(),
      #[cfg(debug_assertions)]
      tree_shaking_result: HashMap::new(),

      code_generation_results: Default::default(),
      code_generated_modules: Default::default(),

      _pin: PhantomPinned,
    }
  }
  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, vec![detail]);
  }

  pub fn generate_chunk_entry_code(&self, chunk_ukey: &ChunkUkey) -> BoxSource {
    let entry_modules_uri = self.chunk_graph.get_chunk_entry_modules(chunk_ukey);
    let entry_modules_id = entry_modules_uri
      .into_iter()
      .filter_map(|entry_module_identifier| {
        self
          .module_graph
          .module_by_uri(entry_module_identifier)
          .map(|module| &module.id)
      })
      .collect::<Vec<_>>();
    let namespace = &self.options.output.unique_name;
    self
      .runtime
      .generate_rspack_execute(namespace, "__rspack_require__", &entry_modules_id)
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
    if let Some(mut original) = self.assets.remove(&filename) {
      if !is_source_equal(&original.source, &asset.source) {
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

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
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

  pub fn add_chunk(
    chunk_by_ukey: &mut ChunkByUkey,
    name: Option<String>,
    id: String,
  ) -> &mut Chunk {
    let chunk = Chunk::new(name, id);
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

  #[instrument(name = "compilation:make")]
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
                .entry((module_identifier.clone(), dependency.detail.clone()))
              {
                Occupied(_) => {
                  if let Err(err) = tx.send(Msg::ModuleReused(
                    (original_module_identifier, dependency_id, module_identifier)
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
                  .insert(module_identifier.clone());
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
                module.identifier(),
              ) {
                // If build error message is failed to send, then we should manually decrease the active task count
                // Otherwise, it will be gracefully handled by the error message handler.
                if let Err(err) =
                  tx.send(Msg::ModuleBuiltErrorEncountered(module.identifier(), err))
                {
                  active_task_count.fetch_sub(1, Ordering::SeqCst);
                  tracing::trace!("fail to send msg {:?}", err)
                }
                // Early bail out if task is failed to finish
                return;
              };

              self.module_graph.add_module(*module);

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
            Msg::DependencyReference(dep, resolved_uri) => {
              self.module_graph.add_dependency(dep, resolved_uri);
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
    tracing::trace!("module graph {:#?}", self.module_graph);
  }

  fn handle_module_build_and_dependencies(
    &self,
    original_module_identifier: Option<ModuleIdentifier>,
    mut module: NormalModule,
    dependency_id: u32,
    active_task_count: Arc<AtomicU32>,
    tx: UnboundedSender<Msg>,
  ) {
    let compiler_options = self.options.clone();
    let loader_runner_runner = self.loader_runner_runner.clone();
    let plugin_driver = self.plugin_driver.clone();

    let module_identifier = module.identifier();

    tokio::spawn(async move {
      let resource_data = module.resource_resolved_data();
      let resolved_loaders = match compiler_options
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
        };

      let resolved_loaders = resolved_loaders
        .into_iter()
        .flat_map(|module_rule| module_rule.uses.iter().map(Box::as_ref).rev())
        .collect::<Vec<_>>();

      if let Err(e) = plugin_driver.read().await.build_module(&mut module).await {
        if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module.identifier(), e)) {
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
          if let Err(e) = plugin_driver.read().await.succeed_module(&module).await {
            if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module.identifier(), e)) {
              tracing::trace!("fail to send msg {:?}", err);
            }
          }

          let module_identifier = module.identifier();

          let (build_result, diagnostics) = build_result.split_into_parts();

          let deps = build_result
            .dependencies
            .into_iter()
            .map(|dep| Dependency {
              parent_module_identifier: Some(module_identifier.clone()),
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
              Box::new(module),
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

      self.code_generation_results.add(
        module_identifier,
        // FIXME: hardcoded for using `main` as `RuntimeSpec` by default as multiple runtimes have not been supported yet. cc @underfin
        HashSet::from_iter(["main".to_owned()]),
        result,
      );
    });

    Ok(())
  }

  #[instrument()]
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
  #[instrument(name = "compilation:process_asssets")]
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    plugin_driver
      .write()
      .await
      .process_assets(ProcessAssetsArgs { compilation: self })
      .await
      .map_err(|e| {
        eprintln!("process_assets is not ok, err {:#?}", e);
        e
      })
  }

  pub async fn optimize_dependency(
    &mut self,
  ) -> Result<TWithDiagnosticArray<OptimizeDependencyResult>> {
    let mut analyze_results = self
      .module_graph
      .module_identifier_to_module_graph_module
      .par_iter()
      .filter_map(|(module_identifier, m)| {
        let uri_key = ustr(module_identifier);
        let ast = match m.module_type {
          crate::ModuleType::Js
          | crate::ModuleType::Jsx
          | crate::ModuleType::Tsx
          | crate::ModuleType::Ts => self
            .module_graph
            .module_by_identifier(&m.module_identifier)
            .and_then(|module| module.ast())
            .unwrap()
            .as_javascript()
            .unwrap(),
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
          let mut analyzer =
            ModuleRefAnalyze::new(top_level_mark, unresolved_mark, uri_key, &self.module_graph);
          program.visit_with(&mut analyzer);
          analyzer
        });
        // Keep this debug info until we stabilize the tree-shaking

        // dbg!(
        //   &uri_key,
        //   // &analyzer.export_all_list,
        //   &analyzer.export_map,
        //   &analyzer.import_map,
        //   &analyzer.reference_map,
        //   &analyzer.reachable_import_and_export,
        //   &analyzer.used_symbol_ref
        // );
        Some((uri_key, analyzer.into()))
      })
      .collect::<HashMap<Ustr, TreeShakingResult>>();
    let mut used_symbol_ref: HashSet<SymbolRef> = HashSet::default();
    for analyze_result in analyze_results.values() {
      used_symbol_ref.extend(analyze_result.used_symbol_ref.clone().into_iter());
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
      &mut errors,
    );

    used_symbol.extend(used_symbol_from_import);

    // We considering all export symbol in each entry module as used for now
    for entry in self.entry_modules() {
      let used_symbol_set = collect_reachable_symbol(&analyze_results, ustr(&entry), &mut errors);
      used_symbol.extend(used_symbol_set);
    }

    Ok(
      OptimizeDependencyResult {
        used_symbol,
        analyze_results,
      }
      .with_diagnostic(errors_to_diagnostics(errors)),
    )
  }

  pub async fn done(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    let stats = &mut Stats::new(self);
    plugin_driver.write().await.done(stats).await?;
    Ok(())
  }
  #[instrument(name = "compilation:render_runtime")]
  pub async fn render_runtime(&self, plugin_driver: SharedPluginDriver) -> Runtime {
    if let Ok(sources) = plugin_driver
      .read()
      .await
      .render_runtime(RenderRuntimeArgs {
        sources: vec![],
        compilation: self,
      })
    {
      Runtime {
        context_indent: self.runtime.context_indent.clone(),
        sources,
      }
    } else {
      self.runtime.clone()
    }
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
  #[instrument(name = "compilation:seal")]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    code_splitting(self)?;

    plugin_driver.write().await.optimize_chunks(self)?;

    tracing::trace!("chunk graph {:#?}", self.chunk_graph);

    let context_indent = if matches!(
      self.options.target.platform,
      crate::TargetPlatform::Web | crate::TargetPlatform::None
    ) {
      String::from("self")
    } else {
      String::from("this")
    };

    self.runtime = Runtime {
      sources: vec![],
      context_indent,
    };

    self.code_generation()?;

    self.create_chunk_assets(plugin_driver.clone()).await;
    // generate runtime
    self.runtime = self.render_runtime(plugin_driver.clone()).await;

    self.process_assets(plugin_driver).await?;
    Ok(())
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
  // pub hot_module_replacement:
  /// when asset is javascript and an ESM
  // pub javascript_module:
  /// related object to other assets, keyed by type of relation (only points from parent to child)
  pub related: AssetInfoRelated,
}

#[derive(Debug, Default, Clone)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}

fn collect_reachable_symbol(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  entry_identifier: Ustr,
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
      errors,
    );
  }

  while let Some(sym_ref) = q.pop_front() {
    mark_symbol(sym_ref, &mut used_symbol_set, analyze_map, &mut q, errors);
  }
  used_symbol_set
}

fn mark_used_symbol_with(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  mut init_queue: VecDeque<SymbolRef>,
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
      errors,
    );
  }
  used_symbol_set
}

fn mark_symbol(
  item: SymbolRef,
  used_symbol_set: &mut HashSet<Symbol>,
  analyze_map: &HashMap<Ustr, TreeShakingResult>,
  q: &mut VecDeque<SymbolRef>,
  errors: &mut Vec<Error>,
) {
  match item {
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
        vac.insert();
      }
    },
    SymbolRef::Indirect(indirect_symbol) => {
      let module_result = analyze_map.get(&indirect_symbol.uri).unwrap();
      let symbol = match module_result.export_map.get(&indirect_symbol.id) {
        Some(symbol) => symbol.clone(),
        None => {
          // TODO: better diagnostic and handle if multiple extends_map has export same symbol
          let mut ret = vec![];
          for (module_identifier, extends_export_map) in module_result.inherit_export_maps.iter() {
            if let Some(value) = extends_export_map.get(&indirect_symbol.id) {
              ret.push((module_identifier, value));
            }
          }
          match ret.len() {
            0 => {
              // TODO: Better diagnostic handle if source module does not have the export
              panic!(
                "{} did not export `{}`",
                module_result.module_identifier, indirect_symbol.id
              )
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
