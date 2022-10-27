use core::panic;
use std::{
  collections::VecDeque,
  fmt::Debug,
  hash::Hash,
  sync::atomic::{AtomicU32, Ordering},
  sync::Arc,
};

use futures::{stream::FuturesUnordered, StreamExt};
use hashbrown::{HashMap, HashSet};
use petgraph::prelude::GraphMap;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use swc_common::GLOBALS;
use tokio::sync::mpsc::UnboundedSender;
use tracing::instrument;

use rspack_error::{Diagnostic, IntoTWithDiagnosticArray, Result, Severity};
use rspack_sources::BoxSource;
use swc_ecma_visit::VisitWith;
use ustr::{ustr, Ustr};

use crate::{
  split_chunks::code_splitting,
  tree_shaking::{
    symbol::Symbol,
    visitor::{ModuleRefAnalyze, SymbolRef, TreeShakingResult},
  },
  BuildContext, Chunk, ChunkByUkey, ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkKind, ChunkUkey,
  CompilerOptions, Dependency, EntryItem, Entrypoint, JavascriptAstExtend, LoaderRunnerRunner,
  ModuleDependency, ModuleGraph, ModuleIdentifier, ModuleRule, Msg, NormalModule,
  NormalModuleFactory, NormalModuleFactoryContext, ProcessAssetsArgs, RenderManifestArgs,
  RenderRuntimeArgs, ResolveKind, Runtime, SharedPluginDriver, Stats, VisitedModuleIdentity,
};

#[derive(Debug)]
pub struct Compilation {
  pub options: Arc<CompilerOptions>,
  entries: HashMap<String, EntryItem>,
  pub(crate) visited_module_id: VisitedModuleIdentity,
  pub module_graph: ModuleGraph,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: HashMap<ChunkUkey, Chunk>,
  pub chunk_group_by_ukey: HashMap<ChunkGroupUkey, ChunkGroup>,
  pub runtime: Runtime,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  pub diagnostic: Vec<Diagnostic>,
  pub(crate) plugin_driver: SharedPluginDriver,
  pub(crate) loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub(crate) _named_chunk: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: HashSet<String>,
  pub used_symbol: HashSet<Symbol>,
  #[cfg(debug_assertions)]
  pub tree_shaking_result: HashMap<Ustr, TreeShakingResult>,
}
impl Compilation {
  pub fn new(
    options: Arc<CompilerOptions>,
    entries: std::collections::HashMap<String, EntryItem>,
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
      diagnostic: vec![],
      plugin_driver,
      loader_runner_runner,
      _named_chunk: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: HashSet::new(),
      used_symbol: HashSet::new(),
      #[cfg(debug_assertions)]
      tree_shaking_result: HashMap::new(),
    }
  }
  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn emit_asset(&mut self, filename: String, asset: CompilationAsset) {
    self.assets.insert(filename, asset);
  }

  pub fn assets(&self) -> &CompilationAssets {
    &self.assets
  }

  pub fn push_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostic.push(diagnostic);
  }

  pub fn push_batch_diagnostic(&mut self, mut diagnostic: Vec<Diagnostic>) {
    self.diagnostic.append(&mut diagnostic);
  }

  pub fn get_errors(&self) -> impl Iterator<Item = &Diagnostic> {
    self
      .diagnostic
      .iter()
      .filter(|d| matches!(d.severity, Severity::Error))
  }

  pub fn add_chunk(
    chunk_by_ukey: &mut ChunkByUkey,
    name: Option<String>,
    id: String,
    kind: ChunkKind,
  ) -> &mut Chunk {
    let chunk = Chunk::new(name, id, kind);
    let ukey = chunk.ukey;
    chunk_by_ukey.insert(chunk.ukey, chunk);
    chunk_by_ukey.get_mut(&ukey).expect("chunk not found")
  }
  #[instrument(name = "entry_deps")]
  pub fn entry_dependencies(&self) -> HashMap<String, Dependency> {
    self
      .entries
      .iter()
      .map(|(name, detail)| {
        (
          name.clone(),
          Dependency {
            importer: None,
            parent_module_identifier: None,
            detail: ModuleDependency {
              specifier: detail.path.clone(),
              kind: ResolveKind::Import,
              span: None,
            },
          },
        )
      })
      .collect()
  }

  #[instrument(name = "compilation:make")]
  pub async fn make(&mut self, entry_deps: HashMap<String, Dependency>) {
    if let Some(e) = self.plugin_driver.clone().read().await.make(self).err() {
      self.push_batch_diagnostic(e.into());
    }
    let active_task_count: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Msg>();

    entry_deps.into_iter().for_each(|(name, dep)| {
      let normal_module_factory = NormalModuleFactory::new(
        NormalModuleFactoryContext {
          module_name: Some(name),
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
    });

    while active_task_count.load(Ordering::SeqCst) != 0 {
      match rx.recv().await {
        Some(job) => match job {
          Msg::ModuleCreated(module_with_diagnostic) => {
            let (mgm, module, original_module_identifier, dependency_id, dependency, is_entry) =
              *module_with_diagnostic.inner;

            let module_identifier = module.identifier();

            if self
              .visited_module_id
              .contains(&(module_identifier.clone(), dependency.detail.clone()))
            {
              if let Err(err) = tx.send(Msg::ModuleReused(
                (original_module_identifier, dependency_id, module_identifier)
                  .with_diagnostic(module_with_diagnostic.diagnostic),
              )) {
                tracing::trace!("fail to send msg {:?}", err)
              }
              continue;
            }

            self
              .visited_module_id
              .insert((module_identifier.clone(), dependency.detail.clone()));

            if let Err(err) = tx.send(Msg::ModuleGraphModuleCreated(mgm)) {
              tracing::trace!("fail to send msg {:?}", err)
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
          }
          Msg::ModuleGraphModuleCreated(mgm) => {
            self.module_graph.add_module_graph_module(mgm);
          }
          Msg::ModuleReused(result_with_diagnostics) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);

            let (original_module_identifier, dependency_id, module_identifier) =
              result_with_diagnostics.inner;
            self.push_batch_diagnostic(result_with_diagnostics.diagnostic);
            if let Err(err) = self.module_graph.set_resolved_module(
              original_module_identifier,
              dependency_id,
              module_identifier.clone(),
            ) {
              if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err)) {
                tracing::trace!("fail to send msg {:?}", err)
              }
            };
          }
          Msg::ModuleResolved(result_with_diagnostics) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);

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
              if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module.identifier(), err))
              {
                tracing::trace!("fail to send msg {:?}", err)
              }
            };
            self.module_graph.add_module(*module);
          }
          Msg::ModuleBuiltErrorEncountered(module_identifier, err) => {
            active_task_count.fetch_sub(1, Ordering::SeqCst);
            self
              .module_graph
              .module_identifier_to_module
              .remove(&module_identifier);
            self
              .module_graph
              .module_identifier_to_module_graph_module
              .remove(&module_identifier);
            self.push_batch_diagnostic(err.into());
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
        None => {
          tracing::trace!("All sender is dropped");
        }
      }
    }
    tracing::debug!("module graph {:#?}", self.module_graph);
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
            if let Err(err) =
              tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err))
            {
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
              importer: Some(module_identifier.clone()),
              parent_module_identifier: Some(module_identifier.clone()),
              detail: dep,
            })
            .collect::<Vec<_>>();

          if let Err(err) = tx.send(Msg::ModuleResolved(
            (
              original_module_identifier.clone(),
              dependency_id,
              Box::new(module),
              Box::new(deps.clone()),
            )
              .with_diagnostic(diagnostics),
          )) {
            tracing::trace!("fail to send msg {:?}", err);
            return;
          };

          Compilation::process_module_dependencies(
            deps,
            active_task_count.clone(),
            tx.clone(),
            plugin_driver.clone(),
            compiler_options.clone(),
          );
        }
        Err(err) => {
          if let Err(err) = tx.send(Msg::ModuleBuiltErrorEncountered(module_identifier, err)) {
            tracing::trace!("fail to send msg {:?}", err)
          }
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
  async fn process_assets(&mut self, plugin_driver: SharedPluginDriver) {
    plugin_driver
      .write()
      .await
      .process_assets(ProcessAssetsArgs { compilation: self })
      .await
      .map_err(|e| {
        eprintln!("process_assets is not ok, err {:#?}", e);
        e
      })
      .ok();
  }

  pub async fn optimize_dependency(
    &mut self,
  ) -> Result<(HashSet<Symbol>, HashMap<Ustr, TreeShakingResult>)> {
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
            .unwrap(),
          // Of course this is unsafe, but if we can't get a ast of a javascript module, then panic is ideal.
          _ => {
            // Ignore analyzing other module for now
            return None;
          }
        };
        let normal_module = self.module_graph.module_by_identifier(&m.module_identifier);
        let globals = normal_module.and_then(|module| module.parse_phase_global());
        let JavascriptAstExtend {
          ast,
          top_level_mark,
          unresolved_mark,
        } = ast.as_javascript().unwrap();
        let mut analyzer = ModuleRefAnalyze::new(
          *top_level_mark,
          *unresolved_mark,
          uri_key,
          &self.module_graph,
        );
        GLOBALS.set(globals.unwrap(), || {
          ast.visit_with(&mut analyzer);
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
    // calculate each module that has `export * from 'xxxx'`
    let export_all_ref_graph = create_graph(&analyze_results);
    let extends_map = get_extends_map(&export_all_ref_graph);
    for (module_id, include_export_module_id) in extends_map.iter() {
      let mut extends_map = {
        let main_module = analyze_results.get_mut(module_id).unwrap();
        std::mem::take(&mut main_module.export_all_extend_map)
      };
      for export_module_id in include_export_module_id {
        let export_module = &analyze_results.get(export_module_id).unwrap().export_map;
        extends_map.insert(export_module_id.clone(), export_module.clone());
      }
      analyze_results
        .get_mut(module_id)
        .unwrap()
        .export_all_extend_map = extends_map;
    }
    let mut used_symbol = HashSet::new();
    // Marking used symbol and all reachable export symbol from the used symbol for each module
    let used_symbol_from_import = mark_used_symbol(
      &analyze_results,
      VecDeque::from_iter(used_symbol_ref.into_iter()),
    );

    used_symbol.extend(used_symbol_from_import);

    // We considering all export symbol in each entry module as used for now
    // TODO: we also need to mark all the  extends_export_map export symbol as used
    for entry in self.entry_modules() {
      let used_symbol_set = collect_reachable_symbol(&analyze_results, ustr(&entry));
      used_symbol.extend(used_symbol_set);
    }
    dbg!(&used_symbol);
    Ok((used_symbol, analyze_results))
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

    tracing::debug!("chunk graph {:#?}", self.chunk_graph);

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

    self.create_chunk_assets(plugin_driver.clone()).await;
    // generate runtime
    self.runtime = self.render_runtime(plugin_driver.clone()).await;

    self.process_assets(plugin_driver).await;
    Ok(())
  }
}

fn get_extends_map(
  export_all_ref_graph: &GraphMap<&Ustr, (), petgraph::Directed>,
) -> HashMap<Ustr, HashSet<Ustr>> {
  let mut map = HashMap::new();
  for node in export_all_ref_graph.nodes() {
    let reachable_set = get_reachable(node.clone(), export_all_ref_graph);
    map.insert(node.clone(), reachable_set);
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

fn create_graph(
  analyze_map: &HashMap<Ustr, TreeShakingResult>,
) -> GraphMap<&Ustr, (), petgraph::Directed> {
  let mut g = petgraph::graphmap::DiGraphMap::new();
  for (module_id, result) in analyze_map.iter() {
    for export_all_module_id in result.export_all_extend_map.keys() {
      g.add_edge(module_id, export_all_module_id, ());
    }
  }
  g
}

pub type CompilationAssets = HashMap<String, CompilationAsset>;

#[derive(Debug)]
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

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct AssetInfoRelated {
  pub source_map: Option<String>,
}

fn collect_reachable_symbol(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  entry_identifier: Ustr,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::new();
  let mut q = VecDeque::new();
  let entry_module_result = match analyze_map.get(&entry_identifier) {
    Some(result) => result,
    None => {
      panic!("Can't get analyze result from entry_identifier");
    }
  };

  for import_list in entry_module_result.reachable_import_of_export.values() {
    q.extend(import_list.clone());
  }

  for item in entry_module_result.export_map.values() {
    mark_symbol(item.clone(), &mut used_symbol_set, analyze_map, &mut q);
  }

  while let Some(sym_ref) = q.pop_front() {
    mark_symbol(sym_ref, &mut used_symbol_set, analyze_map, &mut q);
  }
  used_symbol_set
}

fn mark_used_symbol(
  analyze_map: &hashbrown::HashMap<Ustr, TreeShakingResult>,
  mut init_queue: VecDeque<SymbolRef>,
) -> HashSet<Symbol> {
  let mut used_symbol_set = HashSet::new();

  while let Some(sym_ref) = init_queue.pop_front() {
    mark_symbol(sym_ref, &mut used_symbol_set, analyze_map, &mut init_queue);
  }
  used_symbol_set
}

fn mark_symbol(
  item: SymbolRef,
  used_symbol_set: &mut HashSet<Symbol>,
  analyze_map: &HashMap<Ustr, TreeShakingResult>,
  q: &mut VecDeque<SymbolRef>,
) {
  match item {
    SymbolRef::Direct(symbol) => match used_symbol_set.entry(symbol) {
      hashbrown::hash_set::Entry::Occupied(_) => {}
      hashbrown::hash_set::Entry::Vacant(vac) => {
        let module_result = analyze_map.get(&vac.get().uri).unwrap();
        if let Some(set) = module_result
          .reachable_import_of_export
          .get(&vac.get().id.atom)
        {
          q.extend(set.clone());
        };
        vac.insert();
      }
    },
    SymbolRef::Indirect(indirect_symbol) => {
      let module_result = analyze_map.get(&indirect_symbol.uri).unwrap();
      let symbol = module_result
        .export_map
        .get(&indirect_symbol.id)
        .or_else(|| {
          // TODO: better diagnostic and handle if multiple extends_map has export same symbol
          for (_, extends_export_map) in module_result.export_all_extend_map.iter() {
            if let Some(value) = extends_export_map.get(&indirect_symbol.id) {
              return Some(value);
            }
          }
          return None;
        })
        .unwrap();
      q.push_back(symbol.clone());
    }
    SymbolRef::Star(star) => {
      let module_result = analyze_map.get(&star).unwrap();
      for symbol_ref in module_result.export_map.values() {
        q.push_back(symbol_ref.clone());
      }
    }
  }
}
