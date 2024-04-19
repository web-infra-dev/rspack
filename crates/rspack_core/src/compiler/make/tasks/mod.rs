mod add;
mod build;
pub mod clean;
pub mod factorize;
mod process_dependencies;

use std::{hash::BuildHasherDefault, path::PathBuf, sync::Arc};

use indexmap::IndexSet;
use rspack_error::Diagnostic;
use rspack_identifier::{IdentifierMap, IdentifierSet};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};

use crate::{
  cache::Cache, module_graph::ModuleGraph, tree_shaking::visitor::OptimizeAnalyzeResult,
  BuildDependency, CacheCount, CacheOptions, Compilation, CompilationLogger, CompilerOptions,
  DependencyType, Logger, ModuleFactory, ModuleGraphPartial, ModuleIdentifier, ResolverFactory,
  SharedPluginDriver,
};

pub struct MakeTaskContext {
  plugin_driver: SharedPluginDriver,
  compiler_options: Arc<CompilerOptions>,
  resolver_factory: Arc<ResolverFactory>,
  loader_resolver_factory: Arc<ResolverFactory>,
  cache: Arc<Cache>,
  dependency_factories: HashMap<DependencyType, Arc<dyn ModuleFactory>>,

  module_graph_partial: ModuleGraphPartial,
  make_failed_dependencies: HashSet<BuildDependency>,
  make_failed_module: HashSet<ModuleIdentifier>,
  entry_module_identifiers: IdentifierSet,
  diagnostics: Vec<Diagnostic>,

  // TODO move outof context
  logger: CompilationLogger,
  build_cache_counter: Option<CacheCount>,
  factorize_cache_counter: Option<CacheCount>,
  //  add_timer: StartTimeAggregate,
  //  process_deps_timer: StartTimeAggregate,
  //  factorize_timer: StartTimeAggregate,
  //  build_timer: StartTimeAggregate,
  /// Collecting all module that need to skip in tree-shaking ast modification phase
  //  bailout_module_identifiers: IdentifierMap<BailoutFlag>,
  optimize_analyze_result_map: IdentifierMap<OptimizeAnalyzeResult>,

  file_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  context_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  missing_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
  build_dependencies: IndexSet<PathBuf, BuildHasherDefault<FxHasher>>,
}

impl MakeTaskContext {
  pub fn new(compilation: &Compilation, make_module_graph_partial: ModuleGraphPartial) -> Self {
    let logger = compilation.get_logger("rspack.Compilation");
    let mut build_cache_counter = None;
    let mut factorize_cache_counter = None;
    if !(matches!(compilation.options.cache, CacheOptions::Disabled)) {
      build_cache_counter = Some(logger.cache("module build cache"));
      factorize_cache_counter = Some(logger.cache("module factorize cache"));
    }

    Self {
      plugin_driver: compilation.plugin_driver.clone(),
      compiler_options: compilation.options.clone(),
      resolver_factory: compilation.resolver_factory.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      cache: compilation.cache.clone(),
      dependency_factories: compilation.dependency_factories.clone(),

      module_graph_partial: make_module_graph_partial,
      make_failed_dependencies: Default::default(),
      make_failed_module: Default::default(),
      entry_module_identifiers: Default::default(),
      diagnostics: Default::default(),
      optimize_analyze_result_map: Default::default(),

      // TODO use timer in tasks
      logger,
      build_cache_counter,
      factorize_cache_counter,
      //      add_timer: logger.time_aggregate("module add task"),
      //      process_deps_timer: logger.time_aggregate("module process dependencies task"),
      //      factorize_timer: logger.time_aggregate("module factorize task"),
      //      build_timer: logger.time_aggregate("module build task"),
      file_dependencies: Default::default(),
      context_dependencies: Default::default(),
      missing_dependencies: Default::default(),
      build_dependencies: Default::default(),
    }
  }

  pub fn emit_data_to_compilation(mut self, compilation: &mut Compilation) {
    if let Some(counter) = self.build_cache_counter {
      self.logger.cache_end(counter);
    }
    if let Some(counter) = self.factorize_cache_counter {
      self.logger.cache_end(counter);
    }

    compilation
      .make_failed_dependencies
      .extend(self.make_failed_dependencies.drain());
    compilation
      .make_failed_module
      .extend(self.make_failed_module.drain());
    compilation.file_dependencies.extend(self.file_dependencies);
    compilation
      .context_dependencies
      .extend(self.context_dependencies);
    compilation
      .missing_dependencies
      .extend(self.missing_dependencies);
    compilation
      .build_dependencies
      .extend(self.build_dependencies);

    compilation.push_batch_diagnostic(self.diagnostics);
    compilation
      .entry_module_identifiers
      .extend(self.entry_module_identifiers);
    compilation.swap_make_module_graph(&mut self.module_graph_partial);
    compilation.optimize_analyze_result_map = self.optimize_analyze_result_map;
  }

  // TODO use module graph with make artifact
  pub fn get_module_graph(module_graph_partial: &mut ModuleGraphPartial) -> ModuleGraph {
    ModuleGraph::new(vec![], Some(module_graph_partial))
  }
}
