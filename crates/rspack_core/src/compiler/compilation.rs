use std::{
  fmt::Debug,
  hash::{BuildHasherDefault, Hash, Hasher},
  path::PathBuf,
  sync::Arc,
};

use dashmap::DashSet;
use indexmap::IndexSet;
use rayon::prelude::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use rspack_database::Database;
use rspack_error::{internal_error, Diagnostic, Result, Severity, TWithDiagnosticArray};
use rspack_futures::FuturesResults;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rspack_sources::{BoxSource, CachedSource, SourceExt};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
use swc_core::ecma::ast::ModuleItem;
use tracing::instrument;
use xxhash_rust::xxh3::Xxh3;

use super::{
  asset::{AssetInfo, CompilationAsset, CompilationAssets},
  make::CompilationMake,
};
#[cfg(debug_assertions)]
use crate::tree_shaking::visitor::TreeShakingResult;
use crate::{
  build_chunk_graph::build_chunk_graph,
  cache::{use_code_splitting_cache, Cache, CodeSplittingCache},
  is_source_equal,
  tree_shaking::{optimizer, visitor::SymbolRef, BailoutFlag, OptimizeDependencyResult},
  AdditionalChunkRuntimeRequirementsArgs, BoxModuleDependency, BundleEntries, Chunk, ChunkByUkey,
  ChunkGraph, ChunkGroup, ChunkGroupUkey, ChunkKind, ChunkUkey, CodeGenerationResult,
  CodeGenerationResults, CompilerOptions, ContentHashArgs, DependencyId, EntryDependency,
  EntryItem, EntryOptions, Entrypoint, LoaderRunnerRunner, Module, ModuleGraph, ModuleIdentifier,
  ProcessAssetsArgs, RenderManifestArgs, RuntimeModule, SharedPluginDriver, Stats,
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
  pub make_failed_dependencies: HashSet<DependencyId>,
  pub has_module_import_export_change: bool,
  pub runtime_modules: IdentifierMap<Box<dyn RuntimeModule>>,
  pub runtime_module_code_generation_results: IdentifierMap<(u64, BoxSource)>,
  pub chunk_graph: ChunkGraph,
  pub chunk_by_ukey: Database<Chunk>,
  pub chunk_group_by_ukey: Database<ChunkGroup>,
  pub entrypoints: HashMap<String, ChunkGroupUkey>,
  pub assets: CompilationAssets,
  pub emitted_assets: DashSet<String, BuildHasherDefault<FxHasher>>,
  diagnostics: IndexSet<Diagnostic, BuildHasherDefault<FxHasher>>,
  pub plugin_driver: SharedPluginDriver,
  pub(crate) loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub entry_module_identifiers: IdentifierSet,
  /// Collecting all used export symbol
  pub used_symbol_ref: HashSet<SymbolRef>,
  /// Collecting all module that need to skip in tree-shaking ast modification phase
  pub bailout_module_identifiers: IdentifierMap<BailoutFlag>,
  #[cfg(debug_assertions)]
  pub tree_shaking_result: IdentifierMap<TreeShakingResult>,

  pub code_generation_results: CodeGenerationResults,
  pub code_generated_modules: IdentifierSet,
  pub cache: Arc<Cache>,
  pub code_splitting_cache: CodeSplittingCache,
  pub hash: String,
  // lazy compilation visit module
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub used_chunk_ids: HashSet<String>,

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
    loader_runner_runner: Arc<LoaderRunnerRunner>,
    cache: Arc<Cache>,
  ) -> Self {
    Self {
      options,
      module_graph,
      make_failed_dependencies: HashSet::default(),
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
      loader_runner_runner,
      named_chunks: Default::default(),
      named_chunk_groups: Default::default(),
      entry_module_identifiers: IdentifierSet::default(),
      used_symbol_ref: HashSet::default(),
      #[cfg(debug_assertions)]
      tree_shaking_result: IdentifierMap::default(),
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
    }
  }

  pub fn add_entry(&mut self, name: String, detail: EntryItem) {
    self.entries.insert(name, detail);
  }

  pub fn update_asset(
    &mut self,
    filename: &str,
    updater: impl FnOnce(&mut BoxSource, &mut AssetInfo) -> Result<()>,
  ) -> Result<()> {
    // Safety: we don't move anything from compilation
    let assets = &mut self.assets;

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
    CompilationMake::new(self, &params).build().await
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

    let chunk_ukey_and_manifest = results
      .into_inner()
      .into_iter()
      .collect::<std::result::Result<Vec<_>, _>>()
      .expect("Failed to resolve render_manifest results");

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
  #[instrument(name = "compilation:seal", skip_all)]
  pub async fn seal(&mut self, plugin_driver: SharedPluginDriver) -> Result<()> {
    use_code_splitting_cache(self, |compilation| async {
      build_chunk_graph(compilation)?;
      plugin_driver.write().await.optimize_chunks(compilation)?;
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
          .get_module_runtime_requirements(module.identifier(), &chunk.runtime)
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
    chunks.sort_unstable_by_key(|chunk| chunk.ukey);
    chunks
      .par_iter_mut()
      .flat_map_iter(|chunk| {
        self
          .chunk_graph
          .get_ordered_chunk_modules(&chunk.ukey, &self.module_graph)
          .into_iter()
          .filter_map(|m| self.module_graph.get_module_hash(&m.identifier()))
          .inspect(|hash| hash.hash(&mut chunk.hash))
      })
      .collect::<Vec<_>>()
      .iter()
      .for_each(|hash| {
        hash.hash(&mut compilation_hasher);
      });

    tracing::trace!("hash chunks");

    let runtime_chunk_ukeys = self.get_chunk_graph_entries();
    // runtime chunks should be hashed after all other chunks
    let content_hash_chunks = self
      .chunk_by_ukey
      .keys()
      .filter(|key| !runtime_chunk_ukeys.contains(key))
      .copied()
      .chain(runtime_chunk_ukeys.clone())
      .collect::<Vec<_>>();

    let hash_results = content_hash_chunks
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

    for item in hash_results {
      let (chunk_ukey, hashes) = item.expect("Failed to resolve content_hash results");
      hashes?.into_iter().for_each(|hash| {
        if let Some(chunk) = self.chunk_by_ukey.get_mut(&chunk_ukey) && let Some((source_type, hash)) = hash {
          chunk.content_hash.insert(source_type, hash);
        }
      });
    }

    tracing::trace!("calculate chunks content hash");

    self.create_runtime_module_hash();

    tracing::trace!("hash runtime chunks");

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
}
