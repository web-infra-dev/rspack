#![allow(clippy::only_used_in_recursion)]
use std::{
  borrow::Cow,
  collections::VecDeque,
  sync::{Arc, OnceLock},
};

use rayon::prelude::*;
use rspack_collections::{
  Identifiable, IdentifierDashMap, IdentifierIndexSet, IdentifierMap, IdentifierSet,
};
use rspack_core::{
  BoxDependency, BoxModule, ChunkUkey, Compilation, CompilationOptimizeChunkModules, DependencyId,
  DependencyType, ExportProvided, ExportsInfoArtifact, GetTargetResult,
  ImportedByDeferModulesArtifact, LibIdentOptions, Logger, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ModuleGraphModule, ModuleIdentifier, OptimizationBailoutItem, Plugin,
  ProvidedExports, RuntimeCondition, RuntimeSpec, RuntimeSpecMap, SideEffectsStateArtifact,
  SourceType,
  concatenated_module::{
    ConcatenatedInnerModule, ConcatenatedModule, RootModuleContext, is_esm_dep_like,
  },
  filter_runtime, get_cached_readable_identifier, get_target,
  incremental::IncrementalPasses,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::itoa;
use rustc_hash::FxHashSet as HashSet;

fn format_bailout_reason(msg: &str) -> String {
  format!("ModuleConcatenation bailout: {msg}")
}

#[derive(Clone, Debug)]
enum Warning {
  Id(ModuleIdentifier),
  Problem(ConcatenationProblem),
}

#[derive(Clone, Debug)]
enum ConcatenationProblem {
  MissingChunks {
    module: ModuleIdentifier,
    root_chunks: Arc<HashSet<ChunkUkey>>,
  },
  ReferencedFromNonModule {
    module: ModuleIdentifier,
  },
  RuntimeDependent {
    module: ModuleIdentifier,
    expected_runtime: RuntimeSpec,
    modules: Arc<[(ModuleIdentifier, RuntimeCondition)]>,
  },
  ReferencedFromDifferentChunks {
    module: ModuleIdentifier,
    chunk_modules: Arc<DifferentChunkModules>,
  },
  UnsupportedSyntax {
    module: ModuleIdentifier,
    modules: Arc<[(ModuleIdentifier, Vec<String>)]>,
  },
}

impl ConcatenationProblem {
  fn module(&self) -> ModuleIdentifier {
    match self {
      Self::MissingChunks { module, .. }
      | Self::ReferencedFromNonModule { module }
      | Self::RuntimeDependent { module, .. }
      | Self::ReferencedFromDifferentChunks { module, .. }
      | Self::UnsupportedSyntax { module, .. } => *module,
    }
  }

  fn collect_readable_identifier_modules(&self, modules: &mut IdentifierSet) {
    modules.insert(self.module());
    match self {
      Self::RuntimeDependent {
        modules: origin_modules,
        ..
      } => {
        modules.extend(origin_modules.iter().map(|(module, _)| *module));
      }
      Self::ReferencedFromDifferentChunks { chunk_modules, .. } => {
        modules.extend(
          chunk_modules
            .incoming_modules
            .iter()
            .map(|incoming_module| incoming_module.module_identifier),
        );
      }
      Self::UnsupportedSyntax {
        modules: origin_modules,
        ..
      } => {
        modules.extend(origin_modules.iter().map(|(module, _)| *module));
      }
      Self::MissingChunks { .. } | Self::ReferencedFromNonModule { .. } => {}
    }
  }

  fn format(&self, module_graph: &ModuleGraph, compilation: &Compilation) -> String {
    let module = self.module();
    let module_readable_identifier = get_cached_readable_identifier(
      &module,
      module_graph,
      &compilation.module_static_cache,
      &compilation.options.context,
    );

    match self {
      Self::MissingChunks { root_chunks, .. } => {
        let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
        let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
        let module_chunks = chunk_graph.get_module_chunks(module);
        let mut missing_chunks = root_chunks
          .iter()
          .filter(|chunk| !module_chunks.contains(*chunk))
          .map(|chunk| {
            chunk_by_ukey
              .expect_get(chunk)
              .name()
              .unwrap_or("unnamed chunk(s)")
              .to_string()
          })
          .collect::<Vec<_>>();
        missing_chunks.sort_unstable();
        let mut chunks = module_chunks
          .iter()
          .map(|chunk| {
            chunk_by_ukey
              .expect_get(chunk)
              .name()
              .unwrap_or("unnamed chunk(s)")
              .to_string()
          })
          .collect::<Vec<_>>();
        chunks.sort_unstable();
        format!(
          "Module {} is not in the same chunk(s) (expected in chunk(s) {}, module is in chunk(s) {})",
          module_readable_identifier,
          missing_chunks.join(", "),
          chunks.join(", ")
        )
      }
      Self::ReferencedFromNonModule { .. } => {
        format!("Module {module_readable_identifier} is referenced")
      }
      Self::RuntimeDependent {
        expected_runtime,
        modules,
        ..
      } => {
        format!(
          "Module {} is runtime-dependent referenced by these modules: {}",
          module_readable_identifier,
          modules
            .iter()
            .map(|(origin_module, runtime_condition)| {
              let readable_identifier = get_cached_readable_identifier(
                origin_module,
                module_graph,
                &compilation.module_static_cache,
                &compilation.options.context,
              );
              format!(
                "{} (expected runtime {}, module is only referenced in {})",
                readable_identifier,
                expected_runtime,
                runtime_condition.as_spec().expect("should be spec")
              )
            })
            .collect::<Vec<_>>()
            .join(", ")
        )
      }
      Self::ReferencedFromDifferentChunks { chunk_modules, .. } => {
        let mut names: Vec<_> = chunk_modules
          .modules(compilation)
          .iter()
          .map(|module| {
            get_cached_readable_identifier(
              module,
              module_graph,
              &compilation.module_static_cache,
              &compilation.options.context,
            )
          })
          .collect();
        names.sort();
        format!(
          "Module {} is referenced from different chunks by these modules: {}",
          module_readable_identifier,
          names.join(", ")
        )
      }
      Self::UnsupportedSyntax { modules, .. } => {
        let names = modules
          .iter()
          .map(|(origin_module, dependency_names)| {
            let readable_identifier = get_cached_readable_identifier(
              origin_module,
              module_graph,
              &compilation.module_static_cache,
              &compilation.options.context,
            );
            format!(
              "{} (referenced with {})",
              readable_identifier,
              dependency_names.join(",")
            )
          })
          .collect::<Vec<_>>();

        format!(
          "Module {} is referenced from these modules with unsupported syntax: {}",
          module_readable_identifier,
          names.join(", ")
        )
      }
    }
  }
}

impl Warning {
  fn collect_readable_identifier_modules(&self, modules: &mut IdentifierSet) {
    if let Self::Problem(problem) = self {
      problem.collect_readable_identifier_modules(modules);
    }
  }
}

#[derive(Debug, Clone)]
pub struct ConcatConfiguration {
  pub root_module: ModuleIdentifier,
  runtime: Option<RuntimeSpec>,
  modules: IdentifierIndexSet,
  warnings: IdentifierMap<Warning>,
}

impl ConcatConfiguration {
  pub fn new(root_module: ModuleIdentifier, runtime: Option<RuntimeSpec>) -> Self {
    let mut modules = IdentifierIndexSet::default();
    modules.insert(root_module);

    ConcatConfiguration {
      root_module,
      runtime,
      modules,
      warnings: IdentifierMap::default(),
    }
  }

  fn add(&mut self, module: ModuleIdentifier) {
    self.modules.insert(module);
  }

  fn has(&self, module: &ModuleIdentifier) -> bool {
    self.modules.contains(module)
  }

  fn is_empty(&self) -> bool {
    self.modules.len() == 1
  }

  fn add_warning(&mut self, module: ModuleIdentifier, problem: Warning) {
    self.warnings.insert(module, problem);
  }

  fn into_warnings_sorted(self) -> Vec<(ModuleIdentifier, Warning)> {
    let mut sorted_warnings: Vec<_> = self.warnings.into_iter().collect();
    sorted_warnings.sort_by_key(|(id, _)| *id);
    sorted_warnings
  }

  fn get_modules(&self) -> &IdentifierIndexSet {
    &self.modules
  }

  fn snapshot(&self) -> usize {
    self.modules.len()
  }

  fn rollback(&mut self, snapshot: usize) {
    let modules = &mut self.modules;
    let len = modules.len();
    for _ in snapshot..len {
      modules.pop();
    }
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct ModuleConcatenationPlugin {
  bailout_reason_map: IdentifierDashMap<Arc<Cow<'static, str>>>,
}

#[derive(Default)]
pub struct RuntimeIdentifierCache<T> {
  no_runtime_map: IdentifierMap<T>,
  runtime_map: RuntimeSpecMap<IdentifierMap<T>>,
}

struct ModuleGraphArtifacts<'a> {
  mg_cache: &'a ModuleGraphCacheArtifact,
  side_effects_state_artifact: &'a SideEffectsStateArtifact,
  exports_info_artifact: &'a ExportsInfoArtifact,
}

struct ConcatenationSearchContext<'a> {
  compilation: &'a Compilation,
  root_chunks: &'a Arc<HashSet<ChunkUkey>>,
  runtime: &'a RuntimeSpec,
  possible_modules: &'a IdentifierSet,
  module_cache: &'a IdentifierMap<NoRuntimeModuleCache>,
}

impl ConcatenationSearchContext<'_> {
  fn module_graph_artifacts(&self) -> ModuleGraphArtifacts<'_> {
    ModuleGraphArtifacts {
      mg_cache: &self.compilation.module_graph_cache_artifact,
      side_effects_state_artifact: &self
        .compilation
        .build_module_graph_artifact
        .side_effects_state_artifact,
      exports_info_artifact: &self.compilation.exports_info_artifact,
    }
  }
}

struct ConcatenationSearchState<'a> {
  candidates: &'a mut IdentifierSet,
  failure_cache: &'a mut IdentifierMap<Warning>,
  incoming_modules_cache: &'a mut RuntimeIdentifierCache<IncomingModulesCacheEntry>,
  statistics: &'a mut Statistics,
  imports_cache: &'a mut RuntimeIdentifierCache<Arc<[ModuleIdentifier]>>,
}

#[derive(Default)]
struct RootSearchScratch {
  failure_cache: IdentifierMap<Warning>,
  candidates_visited: IdentifierSet,
  candidates: VecDeque<ModuleIdentifier>,
  import_candidates: IdentifierSet,
}

impl RootSearchScratch {
  fn reset(&mut self) {
    self.failure_cache.clear();
    self.candidates_visited.clear();
    self.candidates.clear();
    self.import_candidates.clear();
  }
}

impl<T> RuntimeIdentifierCache<T> {
  fn insert(&mut self, module: ModuleIdentifier, runtime: Option<&RuntimeSpec>, value: T) {
    if let Some(runtime) = runtime {
      if let Some(map) = self.runtime_map.get_mut(runtime) {
        map.insert(module, value);
      } else {
        let mut map = IdentifierMap::with_capacity_and_hasher(1, Default::default());
        map.insert(module, value);
        self.runtime_map.set(runtime.clone(), map);
      }
    } else {
      self.no_runtime_map.insert(module, value);
    }
  }

  fn get(&self, module: &ModuleIdentifier, runtime: Option<&RuntimeSpec>) -> Option<&T> {
    if let Some(runtime) = runtime {
      let map = self.runtime_map.get(runtime)?;

      map.get(module)
    } else {
      self.no_runtime_map.get(module)
    }
  }
}

impl ModuleConcatenationPlugin {
  fn format_bailout_warning(
    &self,
    module: ModuleIdentifier,
    warning: &Warning,
    module_graph: &ModuleGraph,
    compilation: &Compilation,
  ) -> String {
    match warning {
      Warning::Problem(problem) => format_bailout_reason(&format!(
        "Cannot concat with {module}: {}",
        problem.format(module_graph, compilation)
      )),
      Warning::Id(id) => {
        let reason = self.get_inner_bailout_reason(id);
        let reason_with_prefix = match reason {
          Some(reason) => format!(": {}", *reason),
          None => String::new(),
        };
        if id == &module {
          format_bailout_reason(&format!("Cannot concat with {module}{reason_with_prefix}"))
        } else {
          format_bailout_reason(&format!(
            "Cannot concat with {module} because of {id}{reason_with_prefix}"
          ))
        }
      }
    }
  }

  fn set_bailout_reason(
    &self,
    module: &ModuleIdentifier,
    reason: Cow<'static, str>,
    mg: &mut ModuleGraph,
  ) {
    self.set_inner_bailout_reason(module, reason.clone());
    mg.get_optimization_bailout_mut(module)
      .push(OptimizationBailoutItem::Message(format_bailout_reason(
        &reason,
      )));
  }

  fn set_inner_bailout_reason(&self, module: &ModuleIdentifier, reason: Cow<'static, str>) {
    self.bailout_reason_map.insert(*module, Arc::new(reason));
  }

  fn get_inner_bailout_reason(
    &self,
    module_id: &ModuleIdentifier,
  ) -> Option<Arc<Cow<'static, str>>> {
    self
      .bailout_reason_map
      .get(module_id)
      .map(|reason| reason.clone())
  }

  fn get_imports(
    mg: &ModuleGraph,
    artifacts: &ModuleGraphArtifacts,
    mi: ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    imports_cache: &mut RuntimeIdentifierCache<Arc<[ModuleIdentifier]>>,
    module_cache: &IdentifierMap<NoRuntimeModuleCache>,
  ) -> Arc<[ModuleIdentifier]> {
    if let Some(imports) = imports_cache.get(&mi, runtime) {
      return Arc::clone(imports);
    }

    let cached = module_cache.get(&mi).expect("should have module");

    let mut imports = Vec::with_capacity(cached.connections.len());
    let mut seen = IdentifierSet::default();
    for cached_connection in &cached.connections {
      if seen.contains(&cached_connection.module_identifier) {
        continue;
      }

      let is_target_active = if let Some(runtime) = runtime {
        if cached.runtime == *runtime {
          // runtime is same, use cached value
          cached_connection.active
        } else if cached_connection.active && cached.runtime.is_subset(runtime) {
          // cached runtime is subset and active, means it is also active in current runtime
          true
        } else if !cached_connection.active && cached.runtime.is_superset(runtime) {
          // cached runtime is superset and inactive, means it is also inactive in current runtime
          false
        } else {
          // can't determine, need to check
          cached_connection.connection.is_target_active(
            mg,
            Some(runtime),
            artifacts.mg_cache,
            artifacts.side_effects_state_artifact,
            artifacts.exports_info_artifact,
          )
        }
      } else {
        // no runtime, need to check
        cached_connection.connection.is_target_active(
          mg,
          None,
          artifacts.mg_cache,
          artifacts.side_effects_state_artifact,
          artifacts.exports_info_artifact,
        )
      };

      if !is_target_active {
        continue;
      }
      if cached_connection.has_imported_names || cached.provided_names {
        imports.push(cached_connection.module_identifier);
        seen.insert(cached_connection.module_identifier);
      }
    }

    let imports: Arc<[ModuleIdentifier]> = Arc::from(imports);
    imports_cache.insert(mi, runtime, Arc::clone(&imports));
    imports
  }

  fn get_incoming_modules(
    context: &ConcatenationSearchContext<'_>,
    state: &mut ConcatenationSearchState<'_>,
    module_id: ModuleIdentifier,
    cached: &NoRuntimeModuleCache,
  ) -> std::result::Result<CachedIncomingModules, RuntimeDependentBailout> {
    if let Some(incomings) = state
      .incoming_modules_cache
      .get(&module_id, Some(context.runtime))
    {
      return match incomings {
        IncomingModulesCacheEntry::Modules(incomings) => Ok(incomings.clone()),
        IncomingModulesCacheEntry::RuntimeDependent(bailout) => Err(bailout.clone()),
      };
    }

    let incoming_modules = Self::compute_incoming_modules(context, cached);
    match &incoming_modules {
      Ok(incomings) => state.incoming_modules_cache.insert(
        module_id,
        Some(context.runtime),
        IncomingModulesCacheEntry::Modules(incomings.clone()),
      ),
      Err(bailout) => state.incoming_modules_cache.insert(
        module_id,
        Some(context.runtime),
        IncomingModulesCacheEntry::RuntimeDependent(bailout.clone()),
      ),
    }
    incoming_modules
  }

  fn compute_incoming_modules(
    context: &ConcatenationSearchContext<'_>,
    cached: &NoRuntimeModuleCache,
  ) -> std::result::Result<CachedIncomingModules, RuntimeDependentBailout> {
    let compilation = context.compilation;
    let runtime = context.runtime;
    let module_cache = context.module_cache;
    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
    let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
    let module_graph = compilation.get_module_graph();
    let module_graph_artifacts = context.module_graph_artifacts();
    let mut modules = Vec::with_capacity(cached.incomings.from_modules.len());

    let needs_runtime_condition = runtime.len() > 1;
    // Keep the common single-runtime path free of the per-origin connection vectors that the
    // multi-runtime condition analysis needs.
    if !needs_runtime_condition {
      for (origin_module, connections) in &cached.incomings.from_modules {
        let origin_cache = module_cache.get(origin_module);
        let number_of_chunks = origin_cache.map_or_else(
          || chunk_graph.get_number_of_module_chunks(*origin_module),
          |module| module.number_of_chunks,
        );

        if number_of_chunks == 0 {
          continue;
        }

        let is_intersect = if let Some(origin_runtime) = origin_cache.map(|module| &module.runtime)
        {
          !runtime.is_disjoint(origin_runtime)
        } else {
          let origin_runtime = RuntimeSpec::from_runtimes(
            chunk_graph.get_module_runtimes_iter(*origin_module, chunk_by_ukey),
          );
          !runtime.is_disjoint(&origin_runtime)
        };

        if !is_intersect {
          continue;
        }

        let mut has_active_connection = false;
        let mut has_non_esm_connection = false;
        for connection in connections {
          if is_connection_active_in_runtime(
            connection,
            Some(runtime),
            &cached.runtime,
            module_graph,
            &module_graph_artifacts,
          ) {
            has_active_connection = true;
            has_non_esm_connection |= !connection.is_esm;
          }
        }

        if has_active_connection {
          modules.push(CachedIncomingModule {
            module_identifier: *origin_module,
            has_non_esm_connection,
          });
        }
      }
    } else {
      let mut incoming_connections_from_modules =
        Vec::with_capacity(cached.incomings.from_modules.len());
      for (origin_module, connections) in &cached.incomings.from_modules {
        let origin_cache = module_cache.get(origin_module);
        let number_of_chunks = origin_cache.map_or_else(
          || chunk_graph.get_number_of_module_chunks(*origin_module),
          |module| module.number_of_chunks,
        );

        if number_of_chunks == 0 {
          continue;
        }

        let is_intersect = if let Some(origin_runtime) = origin_cache.map(|module| &module.runtime)
        {
          !runtime.is_disjoint(origin_runtime)
        } else {
          let origin_runtime = RuntimeSpec::from_runtimes(
            chunk_graph.get_module_runtimes_iter(*origin_module, chunk_by_ukey),
          );
          !runtime.is_disjoint(&origin_runtime)
        };

        if !is_intersect {
          continue;
        }

        let active_connections = connections
          .iter()
          .filter(|connection| {
            is_connection_active_in_runtime(
              connection,
              Some(runtime),
              &cached.runtime,
              module_graph,
              &module_graph_artifacts,
            )
          })
          .collect::<Vec<_>>();

        if !active_connections.is_empty() {
          modules.push(CachedIncomingModule {
            module_identifier: *origin_module,
            has_non_esm_connection: active_connections
              .iter()
              .any(|connection| !connection.is_esm),
          });
          incoming_connections_from_modules.push((*origin_module, active_connections));
        }
      }

      let mut runtime_dependent_modules = Vec::new();
      'outer: for (origin_module, connections) in &incoming_connections_from_modules {
        let mut current_runtime_condition = RuntimeCondition::Boolean(false);
        for connection in connections {
          let runtime_condition = filter_runtime(Some(runtime), |runtime| {
            connection.connection.is_target_active(
              module_graph,
              runtime,
              module_graph_artifacts.mg_cache,
              module_graph_artifacts.side_effects_state_artifact,
              module_graph_artifacts.exports_info_artifact,
            )
          });

          if runtime_condition == RuntimeCondition::Boolean(false) {
            continue;
          }

          if runtime_condition == RuntimeCondition::Boolean(true) {
            continue 'outer;
          }

          if current_runtime_condition != RuntimeCondition::Boolean(false) {
            current_runtime_condition
              .as_spec_mut()
              .expect("should be spec")
              .extend(runtime_condition.as_spec().expect("should be spec"));
          } else {
            current_runtime_condition = runtime_condition;
          }
        }

        if current_runtime_condition != RuntimeCondition::Boolean(false) {
          runtime_dependent_modules.push((*origin_module, current_runtime_condition));
        }
      }

      if !runtime_dependent_modules.is_empty() {
        return Err(RuntimeDependentBailout {
          expected_runtime: runtime.clone(),
          modules: Arc::from(runtime_dependent_modules),
        });
      }
    }

    let modules: Arc<[CachedIncomingModule]> = Arc::from(modules);
    let mut module_identifiers = modules
      .iter()
      .map(|incoming_module| incoming_module.module_identifier)
      .collect::<Vec<_>>();
    module_identifiers.sort_unstable();
    Ok(CachedIncomingModules {
      modules,
      module_identifiers: Arc::from(module_identifiers),
    })
  }

  fn try_to_add(
    context: &ConcatenationSearchContext<'_>,
    state: &mut ConcatenationSearchState<'_>,
    config: &mut ConcatConfiguration,
    module_id: &ModuleIdentifier,
    rollback_on_failure: bool,
  ) -> Option<Warning> {
    if let Some(cache_entry) = state.failure_cache.get(module_id) {
      state.statistics.cached += 1;
      return Some(cache_entry.clone());
    }

    if config.has(module_id) {
      state.statistics.already_in_config += 1;
      return None;
    }

    let compilation = context.compilation;
    let runtime = context.runtime;
    let root_chunks = context.root_chunks;
    let module_cache = context.module_cache;
    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
    let module_graph = compilation.get_module_graph();
    let module_graph_artifacts = context.module_graph_artifacts();

    if !context.possible_modules.contains(module_id) {
      state.statistics.invalid_module += 1;
      let problem = Warning::Id(*module_id);
      state.failure_cache.insert(*module_id, problem.clone());
      return Some(problem);
    }

    let cached = module_cache
      .get(module_id)
      .expect("should have module cache");
    if !root_chunks.is_subset(&cached.chunks) {
      let problem = Warning::Problem(ConcatenationProblem::MissingChunks {
        module: *module_id,
        root_chunks: Arc::clone(root_chunks),
      });

      state.statistics.incorrect_chunks += 1;
      state.failure_cache.insert(*module_id, problem.clone());
      return Some(problem);
    }

    let NoRuntimeModuleCache {
      incomings,
      runtime: cached_module_runtime,
      ..
    } = cached;

    if !incomings.from_non_modules.is_empty() {
      let has_active_non_modules_connections =
        incomings.from_non_modules.iter().any(|connection| {
          is_connection_active_in_runtime(
            connection,
            Some(runtime),
            cached_module_runtime,
            module_graph,
            &module_graph_artifacts,
          )
        });

      // TODO: ADD module connection explanations
      if has_active_non_modules_connections {
        let problem =
          Warning::Problem(ConcatenationProblem::ReferencedFromNonModule { module: *module_id });
        state.statistics.incorrect_dependency += 1;
        state.failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }
    }

    let incoming_modules = match Self::get_incoming_modules(context, state, *module_id, cached) {
      Ok(incoming_modules) => incoming_modules,
      Err(bailout) => {
        let problem = bailout.warning(*module_id);
        state.statistics.incorrect_runtime_condition += 1;
        state.failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }
    };

    let has_other_chunk_module = incoming_modules.modules.iter().any(|incoming_module| {
      let origin_module = incoming_module.module_identifier;
      let origin_cache = module_cache.get(&origin_module);
      if let Some(origin_cache) = origin_cache {
        !root_chunks.is_subset(&origin_cache.chunks)
      } else {
        !root_chunks.is_subset(chunk_graph.get_module_chunks(origin_module))
      }
    });

    if has_other_chunk_module {
      state.statistics.incorrect_chunks_of_importer += 1;
      let problem = Warning::Problem(ConcatenationProblem::ReferencedFromDifferentChunks {
        module: *module_id,
        chunk_modules: Arc::new(DifferentChunkModules {
          root_chunks: Arc::clone(root_chunks),
          incoming_modules: Arc::clone(&incoming_modules.modules),
          modules: OnceLock::new(),
        }),
      });
      state.failure_cache.insert(*module_id, problem.clone());
      return Some(problem);
    }

    let non_esm_modules = incoming_modules
      .modules
      .iter()
      .filter_map(|incoming_module| {
        incoming_module
          .has_non_esm_connection
          .then_some(incoming_module.module_identifier)
      })
      .collect::<Vec<_>>();

    if !non_esm_modules.is_empty() {
      let problem = {
        let modules = non_esm_modules
          .iter()
          .map(|origin_module| {
            let mut names = incomings
              .from_modules
              .get(origin_module)
              .expect("should have incoming connections")
              .iter()
              .filter(|connection| {
                !connection.is_esm
                  && is_connection_active_in_runtime(
                    connection,
                    Some(runtime),
                    cached_module_runtime,
                    module_graph,
                    &module_graph_artifacts,
                  )
              })
              .map(|item| {
                let dep = module_graph.dependency_by_id(&item.connection.dependency_id);
                dep.dependency_type().to_string()
              })
              .collect::<Vec<_>>();
            names.sort();
            (*origin_module, names)
          })
          .collect::<Vec<_>>();

        Warning::Problem(ConcatenationProblem::UnsupportedSyntax {
          module: *module_id,
          modules: modules.into(),
        })
      };
      state.statistics.incorrect_module_dependency += 1;
      state.failure_cache.insert(*module_id, problem.clone());
      return Some(problem);
    }

    let backup = if rollback_on_failure {
      Some(config.snapshot())
    } else {
      None
    };

    config.add(*module_id);

    for origin_module in incoming_modules.module_identifiers.iter() {
      if let Some(problem) = Self::try_to_add(context, state, config, origin_module, false) {
        if let Some(backup) = &backup {
          config.rollback(*backup);
        }
        state.statistics.importer_failed += 1;
        state.failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }
    }

    for imp in Self::get_imports(
      module_graph,
      &module_graph_artifacts,
      *module_id,
      Some(runtime),
      state.imports_cache,
      module_cache,
    )
    .iter()
    {
      state.candidates.insert(*imp);
    }
    state.statistics.added += 1;
    None
  }

  async fn optimize_chunk_modules_impl(&self, compilation: &mut Compilation) -> Result<()> {
    let logger = compilation.get_logger("rspack.ModuleConcatenationPlugin");

    if compilation.options.experiments.defer_import {
      let mut imported_by_defer_modules_artifact = ImportedByDeferModulesArtifact::default();
      let module_graph = compilation.get_module_graph();
      for (_, dep) in module_graph.dependencies() {
        if dep.get_phase().is_defer()
          && matches!(
            dep.dependency_type(),
            DependencyType::EsmImport | DependencyType::EsmExportImport
          )
          && let Some(module) = module_graph.module_identifier_by_dependency_id(dep.id())
        {
          imported_by_defer_modules_artifact.insert(*module);
        }
      }
      compilation.imported_by_defer_modules_artifact = imported_by_defer_modules_artifact.into();
    }

    let mut relevant_modules = vec![];
    let mut possible_inners = IdentifierSet::default();
    let start = logger.time("select relevant modules");
    let module_graph = compilation.get_module_graph();

    // filter modules that can be root
    let modules: Vec<_> = module_graph
      .module_graph_modules()
      .map(|(k, _)| *k)
      .collect();
    let res: Vec<_> = modules
      .into_par_iter()
      .map(|module_id| {
        let mut can_be_root = true;
        let mut can_be_inner = true;
        let mut bailout_reason = vec![];
        let number_of_module_chunks = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_module_chunks(module_id);
        let is_entry_module = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .is_entry_module(&module_id);
        let module_graph = compilation.get_module_graph();
        let m = module_graph
          .module_by_identifier(&module_id)
          .expect("should have module");

        if let Some(reason) = m.get_concatenation_bailout_reason(
          module_graph,
          &compilation.build_chunk_graph_artifact.chunk_graph,
        ) {
          bailout_reason.push(reason);
          return (false, false, module_id, bailout_reason);
        }

        if ModuleGraph::is_async(&compilation.async_modules_artifact, &module_id) {
          bailout_reason.push("Module is async".into());
          return (false, false, module_id, bailout_reason);
        }

        if !m.build_info().strict {
          bailout_reason.push("Module is not in strict mode".into());
          return (false, false, module_id, bailout_reason);
        }
        if number_of_module_chunks == 0 {
          bailout_reason.push("Module is not in any chunk".into());
          return (false, false, module_id, bailout_reason);
        }

        let exports_info = compilation
          .exports_info_artifact
          .get_exports_info_data(&module_id);
        let relevant_exports = exports_info.get_relevant_exports(None);
        let mut unknown_exports = None;
        for export_info in relevant_exports.iter() {
          if export_info.is_reexport()
            && !matches!(
              get_target(
                export_info,
                module_graph,
                &compilation.exports_info_artifact,
                &|_| true,
                &mut Default::default()
              ),
              Some(GetTargetResult::Target(_))
            )
          {
            unknown_exports.get_or_insert_with(Vec::new).push({
              let name = export_info
                .name()
                .map_or("other exports".to_string(), |name| name.to_string());
              format!("{} : {}", name, export_info.get_used_info())
            });
          }
        }
        if let Some(unknown_exports) = unknown_exports {
          let cur_bailout_reason = unknown_exports.join(", ");
          // self.set_bailout_reason(
          //   &module_id,
          //   format!("Reexports in this module do not have a static target ({bailout_reason})"),
          //   &mut module_graph,
          // );

          bailout_reason.push(
            format!("Reexports in this module do not have a static target ({cur_bailout_reason})")
              .into(),
          );

          return (false, false, module_id, bailout_reason);
        }
        let mut unknown_provided_exports = None;
        for export_info in relevant_exports.iter() {
          if !matches!(export_info.provided(), Some(ExportProvided::Provided)) {
            unknown_provided_exports.get_or_insert_with(Vec::new).push({
              let name = export_info
                .name()
                .map_or("other exports".to_string(), |name| name.to_string());
              format!(
                "{} : {} and {}",
                name,
                export_info.get_provided_info(),
                export_info.get_used_info(),
              )
            });
          }
        }

        if let Some(unknown_provided_exports) = unknown_provided_exports {
          let cur_bailout_reason = unknown_provided_exports.join(", ");
          // self.set_bailout_reason(
          //   &module_id,
          //   format!("List of module exports is dynamic ({bailout_reason})"),
          //   &mut module_graph,
          // );
          bailout_reason
            .push(format!("List of module exports is dynamic ({cur_bailout_reason})").into());
          can_be_root = false;
        }

        if is_entry_module {
          // self.set_bailout_reason(
          //   &module_id,
          //   "Module is an entry point".to_string(),
          //   &mut module_graph,
          // );
          can_be_inner = false;
          bailout_reason.push("Module is an entry point".into());
        }

        if compilation.options.experiments.defer_import
          && module_graph.is_deferred(&compilation.imported_by_defer_modules_artifact, &module_id)
        {
          bailout_reason.push("Module is deferred".into());
          can_be_inner = false;
        }

        (can_be_root, can_be_inner, module_id, bailout_reason)
        // if can_be_root {
        //   relevant_modules.push(module_id);
        // }
        // if can_be_inner {
        //   possible_inners.insert(module_id);
        // }
      })
      .collect();

    let module_graph = compilation.get_module_graph_mut();

    for (can_be_root, can_be_inner, module_id, bailout_reason) in res {
      if can_be_root {
        relevant_modules.push(module_id);
      }
      if can_be_inner {
        possible_inners.insert(module_id);
      }
      for bailout_reason in bailout_reason {
        self.set_bailout_reason(&module_id, bailout_reason, module_graph);
      }
    }

    let module_graph = compilation.get_module_graph();
    logger.time_end(start);
    let mut relevant_len_buffer = itoa::Buffer::new();
    let relevant_len_str = relevant_len_buffer.format(relevant_modules.len());
    let mut possible_len_buffer = itoa::Buffer::new();
    let possible_len_str = possible_len_buffer.format(possible_inners.len());
    logger.debug(format!(
      "{relevant_len_str} potential root modules, {possible_len_str} potential inner modules",
    ));

    let start = logger.time("sort relevant modules");
    relevant_modules.sort_by_cached_key(|module| module_graph.get_depth(module));

    logger.time_end(start);
    let mut statistics = Statistics::default();
    let mut stats_candidates = 0;
    let mut stats_size_sum = 0;
    let mut stats_empty_configurations = 0;
    let mut empty_config_warnings = Vec::new();

    let start = logger.time("find modules to concatenate");
    let mut concat_configurations: Vec<ConcatConfiguration> = Vec::new();
    let mut used_as_inner: IdentifierSet = IdentifierSet::default();
    let mut imports_cache = RuntimeIdentifierCache::<Arc<[ModuleIdentifier]>>::default();
    // Incoming activity depends on runtime, while chunk compatibility remains root-specific.
    let mut incoming_modules_cache = RuntimeIdentifierCache {
      no_runtime_map: IdentifierMap::default(),
      runtime_map: Default::default(),
    };
    let mut root_search_scratch = RootSearchScratch::default();

    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;
    let cache_modules = relevant_modules
      .iter()
      .chain(possible_inners.iter())
      .copied()
      .collect::<IdentifierSet>();
    let modules_without_runtime_cache_entries = cache_modules
      .into_par_iter()
      .map(|module_id| {
        let exports_info = compilation
          .exports_info_artifact
          .get_exports_info_data(&module_id);
        let provided_names = matches!(
          exports_info.get_provided_exports(),
          ProvidedExports::ProvidedNames(_)
        );
        let module = module_graph
          .module_by_identifier(&module_id)
          .expect("should have module");
        let chunks = Arc::new(
          compilation
            .build_chunk_graph_artifact
            .chunk_graph
            .get_module_chunks(module_id)
            .clone(),
        );
        let runtime = RuntimeSpec::from_runtimes(chunks.iter().map(|chunk| {
          compilation
            .build_chunk_graph_artifact
            .chunk_by_ukey
            .expect_get(chunk)
            .runtime()
        }));

        let connections = module
          .get_dependencies()
          .iter()
          .filter_map(|d| {
            let dep = module_graph.dependency_by_id(d);
            if !is_esm_dep_like(dep) {
              return None;
            }
            let con = module_graph.connection_by_dependency_id(d)?;
            let module_dep = dep.as_module_dependency().expect("should be module dep");
            let imported_names = module_dep.get_referenced_exports(
              module_graph,
              module_graph_cache,
              &compilation.exports_info_artifact,
              None,
            );

            Some(CachedOutgoingConnection {
              connection: con.clone(),
              module_identifier: *con.module_identifier(),
              has_imported_names: imported_names.iter().all(|item| !item.name.is_empty()),
              active: con.is_target_active(
                module_graph,
                Some(&runtime),
                module_graph_cache,
                &compilation
                  .build_module_graph_artifact
                  .side_effects_state_artifact,
                &compilation.exports_info_artifact,
              ),
            })
          })
          .collect::<Vec<_>>();

        let incoming_connection_ids = module_graph
          .module_graph_module_by_identifier(&module_id)
          .expect("should have mgm")
          .incoming_connections();
        let mut incomings = IncomingConnections::default();
        for dependency_id in incoming_connection_ids {
          let connection = module_graph
            .connection_by_dependency_id(dependency_id)
            .expect("should have connection");
          let origin_module = connection.original_module_identifier;
          let connection = CachedIncomingConnection::new(
            connection,
            &runtime,
            module_graph,
            module_graph_cache,
            &compilation
              .build_module_graph_artifact
              .side_effects_state_artifact,
            &compilation.exports_info_artifact,
          );
          if let Some(origin_module) = origin_module {
            incomings
              .from_modules
              .entry(origin_module)
              .or_default()
              .push(connection);
          } else {
            incomings.from_non_modules.push(connection);
          }
        }
        let number_of_chunks = chunks.len();
        (
          module_id,
          NoRuntimeModuleCache {
            runtime,
            chunks,
            provided_names,
            connections,
            incomings,
            number_of_chunks,
          },
        )
      })
      .collect::<Vec<_>>();
    let mut modules_without_runtime_cache = IdentifierMap::with_capacity_and_hasher(
      modules_without_runtime_cache_entries.len(),
      Default::default(),
    );
    modules_without_runtime_cache.extend(modules_without_runtime_cache_entries);
    for current_root in relevant_modules.iter() {
      if used_as_inner.contains(current_root) {
        continue;
      }

      let NoRuntimeModuleCache {
        runtime,
        chunks: root_chunks,
        ..
      } = modules_without_runtime_cache
        .get(current_root)
        .expect("should have module");
      let module_graph = compilation.get_module_graph();
      let module_graph_cache = &compilation.module_graph_cache_artifact;
      let exports_info = compilation
        .exports_info_artifact
        .get_exports_info_data(current_root);
      let filtered_runtime = filter_runtime(Some(runtime), |r| exports_info.is_module_used(r));
      let active_runtime = match filtered_runtime {
        RuntimeCondition::Boolean(true) => Some(runtime.clone()),
        RuntimeCondition::Boolean(false) => None,
        RuntimeCondition::Spec(spec) => Some(spec),
      };

      let mut current_configuration =
        ConcatConfiguration::new(*current_root, active_runtime.clone());
      let root_chunks = Arc::clone(root_chunks);

      root_search_scratch.reset();
      let RootSearchScratch {
        failure_cache,
        candidates_visited,
        candidates,
        import_candidates,
      } = &mut root_search_scratch;
      let imports = {
        let module_graph_artifacts = ModuleGraphArtifacts {
          mg_cache: module_graph_cache,
          side_effects_state_artifact: &compilation
            .build_module_graph_artifact
            .side_effects_state_artifact,
          exports_info_artifact: &compilation.exports_info_artifact,
        };

        Self::get_imports(
          module_graph,
          &module_graph_artifacts,
          *current_root,
          active_runtime.as_ref(),
          &mut imports_cache,
          &modules_without_runtime_cache,
        )
      };
      for import in imports.iter() {
        candidates.push_back(*import);
      }

      let search_context = ConcatenationSearchContext {
        compilation,
        root_chunks: &root_chunks,
        runtime,
        possible_modules: &possible_inners,
        module_cache: &modules_without_runtime_cache,
      };
      while let Some(imp) = candidates.pop_front() {
        if candidates_visited.contains(&imp) {
          continue;
        }
        candidates_visited.insert(imp);
        import_candidates.clear();
        let result = {
          let mut search_state = ConcatenationSearchState {
            candidates: import_candidates,
            failure_cache,
            incoming_modules_cache: &mut incoming_modules_cache,
            statistics: &mut statistics,
            imports_cache: &mut imports_cache,
          };
          Self::try_to_add(
            &search_context,
            &mut search_state,
            &mut current_configuration,
            &imp,
            true,
          )
        };
        match result {
          Some(problem) => {
            failure_cache.insert(imp, problem.clone());
            current_configuration.add_warning(imp, problem);
          }
          _ => {
            import_candidates.iter().for_each(|c: &ModuleIdentifier| {
              candidates.push_back(*c);
            });
          }
        }
      }
      stats_candidates += candidates.len();
      if !current_configuration.is_empty() {
        let modules = current_configuration.get_modules();
        stats_size_sum += modules.len();
        let root_module = current_configuration.root_module;

        modules.iter().for_each(|module| {
          if *module != root_module {
            used_as_inner.insert(*module);
          }
        });
        concat_configurations.push(current_configuration);
      } else {
        stats_empty_configurations += 1;
        empty_config_warnings.push((*current_root, current_configuration.into_warnings_sorted()));
      }
    }
    logger.time_end(start);

    rayon::spawn(move || drop(modules_without_runtime_cache));

    if !concat_configurations.is_empty() {
      let mut concat_len_buffer = itoa::Buffer::new();
      let concat_len_str = concat_len_buffer.format(concat_configurations.len());
      let mut avg_size_buffer = itoa::Buffer::new();
      let avg_size_str = avg_size_buffer.format(stats_size_sum / concat_configurations.len());
      let mut empty_configs_buffer = itoa::Buffer::new();
      let empty_configs_str = empty_configs_buffer.format(stats_empty_configurations);
      logger.debug(format!(
        "{concat_len_str} successful concat configurations (avg size: {avg_size_str}), {empty_configs_str} bailed out completely"
      ));
    }

    let mut candidates_buffer = itoa::Buffer::new();
    let candidates_str = candidates_buffer.format(stats_candidates);
    let mut cached_buffer = itoa::Buffer::new();
    let cached_str = cached_buffer.format(statistics.cached);
    let mut already_in_config_buffer = itoa::Buffer::new();
    let already_in_config_str = already_in_config_buffer.format(statistics.already_in_config);
    let mut invalid_module_buffer = itoa::Buffer::new();
    let invalid_module_str = invalid_module_buffer.format(statistics.invalid_module);
    let mut incorrect_chunks_buffer = itoa::Buffer::new();
    let incorrect_chunks_str = incorrect_chunks_buffer.format(statistics.incorrect_chunks);
    let mut incorrect_dependency_buffer = itoa::Buffer::new();
    let incorrect_dependency_str =
      incorrect_dependency_buffer.format(statistics.incorrect_dependency);
    let mut incorrect_chunks_of_importer_buffer = itoa::Buffer::new();
    let incorrect_chunks_of_importer_str =
      incorrect_chunks_of_importer_buffer.format(statistics.incorrect_chunks_of_importer);
    let mut incorrect_module_dependency_buffer = itoa::Buffer::new();
    let incorrect_module_dependency_str =
      incorrect_module_dependency_buffer.format(statistics.incorrect_module_dependency);
    let mut incorrect_runtime_condition_buffer = itoa::Buffer::new();
    let incorrect_runtime_condition_str =
      incorrect_runtime_condition_buffer.format(statistics.incorrect_runtime_condition);
    let mut importer_failed_buffer = itoa::Buffer::new();
    let importer_failed_str = importer_failed_buffer.format(statistics.importer_failed);
    let mut added_buffer = itoa::Buffer::new();
    let added_str = added_buffer.format(statistics.added);
    logger.debug(format!(
        "{candidates_str} candidates were considered for adding ({cached_str} cached failure, {already_in_config_str} already in config, {invalid_module_str} invalid module, {incorrect_chunks_str} incorrect chunks, {incorrect_dependency_str} incorrect dependency, {incorrect_chunks_of_importer_str} incorrect chunks of importer, {incorrect_module_dependency_str} incorrect module dependency, {incorrect_runtime_condition_str} incorrect runtime condition, {importer_failed_str} importer failed, {added_str} added)"
    ));

    // Copy from  https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ModuleConcatenationPlugin.js#L368-L371
    // HACK: Sort configurations by length and start with the longest one
    // to get the biggest groups possible. Used modules are marked with usedModules
    // TODO(from webpack): Allow reusing existing configuration while trying to add dependencies.
    // This would improve performance. O(n^2) -> O(n)
    let start = logger.time("sort concat configurations");
    concat_configurations.sort_by_key(|b| std::cmp::Reverse(b.modules.len()));
    logger.time_end(start);

    let mut used_modules = IdentifierSet::default();
    let mut batch = vec![];

    for config in concat_configurations {
      if used_modules.contains(&config.root_module) {
        continue;
      }
      let modules_set = config.get_modules();
      used_modules.extend(modules_set.iter().copied());
      batch.push(config);
    }

    let mut readable_identifier_modules = IdentifierSet::default();
    for config in &batch {
      readable_identifier_modules.extend(config.get_modules().iter().copied());
    }
    for (_, warnings) in &empty_config_warnings {
      for (_, warning) in warnings {
        warning.collect_readable_identifier_modules(&mut readable_identifier_modules);
      }
    }
    let module_graph = compilation.get_module_graph();
    let module_static_cache = &compilation.module_static_cache;
    let compilation_context = &compilation.options.context;
    readable_identifier_modules
      .into_par_iter()
      .for_each(|module_id| {
        let _ = get_cached_readable_identifier(
          &module_id,
          module_graph,
          module_static_cache,
          compilation_context,
        );
      });

    // These lazy warnings inspect the current chunk graph, so materialize them before
    // creating concatenated modules mutates that graph below.
    let formatted_empty_config_warnings = empty_config_warnings
      .into_par_iter()
      .map(|(current_root, warnings)| {
        let module_graph = compilation.get_module_graph();
        let messages = warnings
          .iter()
          .map(|warning| {
            OptimizationBailoutItem::Message(self.format_bailout_warning(
              warning.0,
              &warning.1,
              module_graph,
              compilation,
            ))
          })
          .collect::<Vec<_>>();
        (current_root, messages)
      })
      .collect::<Vec<_>>();
    for (current_root, messages) in formatted_empty_config_warnings {
      let module_graph = compilation.get_module_graph_mut();
      let optimization_bailouts = module_graph.get_optimization_bailout_mut(&current_root);
      optimization_bailouts.extend(messages);
    }
    let new_modules = rspack_parallel::scope::<_, Result<_>>(|token| {
      batch.into_iter().for_each(|config| {
        let s = unsafe { token.used(&*compilation) };
        s.spawn(move |compilation| async move {
          let modules_set = config.get_modules();
          let new_module = create_concatenated_module(compilation, &config).await?;
          let new_module_id = new_module.identifier();
          let connections = prepare_concatenated_module_connections(
            compilation,
            &new_module_id,
            modules_set,
            |m, con, dep| {
              con.original_module_identifier.as_ref() == Some(m)
                && !(is_esm_dep_like(dep) && modules_set.contains(con.module_identifier()))
            },
          );
          let (root_outgoings, root_incomings) = prepare_concatenated_root_module_connections(
            compilation,
            &config.root_module,
            |m, c, dep| {
              let other_module = if c.module_identifier() == m {
                c.original_module_identifier
              } else {
                Some(*c.module_identifier())
              };
              let inner_connection = is_esm_dep_like(dep)
                && if let Some(other_module) = other_module {
                  modules_set.contains(&other_module)
                } else {
                  false
                };
              !inner_connection
            },
          );
          Ok((
            new_module,
            connections,
            root_outgoings,
            root_incomings,
            config,
          ))
        });
      });
    })
    .await
    .into_iter()
    .map(|r| r.to_rspack_result())
    .collect::<Result<Vec<_>>>()?;

    let mut set_original_mid_tasks = vec![];
    let mut set_mid_tasks = vec![];
    let mut add_connection_tasks = vec![];
    let mut remove_connection_tasks = vec![];

    for res in new_modules {
      let (new_module, outgoings, root_outgoings, root_incomings, config) = res?;
      let new_module_id = new_module.identifier();
      let root_module_id = config.root_module;
      add_concatenated_module(compilation, new_module, config);

      for connection in outgoings.iter().chain(root_outgoings.iter()) {
        set_original_mid_tasks.push((*connection, new_module_id));
      }
      for connection in root_incomings.iter() {
        set_mid_tasks.push((*connection, new_module_id));
      }
      let mut all_outgoings = outgoings;
      all_outgoings.extend(root_outgoings.clone());
      add_connection_tasks.push((new_module_id, all_outgoings, root_incomings.clone()));
      remove_connection_tasks.push((root_module_id, root_outgoings, root_incomings));
    }

    let module_graph = compilation.get_module_graph_mut();
    module_graph.batch_set_connections_original_module(set_original_mid_tasks);
    module_graph.batch_set_connections_module(set_mid_tasks);
    module_graph.batch_add_connections(add_connection_tasks);
    module_graph.batch_remove_connections(remove_connection_tasks);

    Ok(())
  }
}

#[plugin_hook(CompilationOptimizeChunkModules for ModuleConcatenationPlugin)]
async fn optimize_chunk_modules(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES
    | IncrementalPasses::MODULE_IDS
    | IncrementalPasses::CHUNK_IDS
    | IncrementalPasses::CHUNKS_RUNTIME_REQUIREMENTS
    | IncrementalPasses::CHUNKS_HASHES,
    "ModuleConcatenationPlugin (optimization.concatenateModules = true)",
    "it requires calculating the modules that can be concatenated based on all the modules, which is a global effect",
  ) && let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
  }

  self.optimize_chunk_modules_impl(compilation).await?;

  Ok(None)
}

impl Plugin for ModuleConcatenationPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_chunk_modules
      .tap(optimize_chunk_modules::new(self));
    Ok(())
  }
}

#[derive(Debug, Default)]
struct Statistics {
  cached: u32,
  already_in_config: u32,
  invalid_module: u32,
  incorrect_chunks: u32,
  incorrect_dependency: u32,
  incorrect_module_dependency: u32,
  incorrect_chunks_of_importer: u32,
  incorrect_runtime_condition: u32,
  importer_failed: u32,
  added: u32,
}

#[derive(Debug, Default)]
struct IncomingConnections {
  from_non_modules: Vec<CachedIncomingConnection>,
  from_modules: IdentifierMap<Vec<CachedIncomingConnection>>,
}

enum IncomingModulesCacheEntry {
  Modules(CachedIncomingModules),
  RuntimeDependent(RuntimeDependentBailout),
}

#[derive(Clone)]
struct RuntimeDependentBailout {
  expected_runtime: RuntimeSpec,
  modules: Arc<[(ModuleIdentifier, RuntimeCondition)]>,
}

impl RuntimeDependentBailout {
  fn warning(&self, module: ModuleIdentifier) -> Warning {
    Warning::Problem(ConcatenationProblem::RuntimeDependent {
      module,
      expected_runtime: self.expected_runtime.clone(),
      modules: Arc::clone(&self.modules),
    })
  }
}

#[derive(Debug, Clone)]
struct CachedIncomingModules {
  modules: Arc<[CachedIncomingModule]>,
  module_identifiers: Arc<[ModuleIdentifier]>,
}

#[derive(Debug)]
struct CachedIncomingModule {
  module_identifier: ModuleIdentifier,
  has_non_esm_connection: bool,
}

#[derive(Debug)]
struct DifferentChunkModules {
  root_chunks: Arc<HashSet<ChunkUkey>>,
  incoming_modules: Arc<[CachedIncomingModule]>,
  modules: OnceLock<Arc<[ModuleIdentifier]>>,
}

impl DifferentChunkModules {
  fn modules(&self, compilation: &Compilation) -> &[ModuleIdentifier] {
    self.modules.get_or_init(|| {
      let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
      self
        .incoming_modules
        .iter()
        .filter_map(|incoming_module| {
          (!self
            .root_chunks
            .is_subset(chunk_graph.get_module_chunks(incoming_module.module_identifier)))
          .then_some(incoming_module.module_identifier)
        })
        .collect::<Vec<_>>()
        .into()
    })
  }
}

#[derive(Debug)]
struct CachedOutgoingConnection {
  connection: ModuleGraphConnection,
  module_identifier: ModuleIdentifier,
  has_imported_names: bool,
  active: bool,
}

#[derive(Debug)]
struct CachedIncomingConnection {
  connection: ModuleGraphConnection,
  active: bool,
  is_esm: bool,
}

impl CachedIncomingConnection {
  fn new(
    connection: &ModuleGraphConnection,
    runtime: &RuntimeSpec,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    side_effects_state_artifact: &SideEffectsStateArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
  ) -> Self {
    let dep = module_graph.dependency_by_id(&connection.dependency_id);
    Self {
      connection: connection.clone(),
      active: connection.is_active(
        module_graph,
        Some(runtime),
        module_graph_cache,
        side_effects_state_artifact,
        exports_info_artifact,
      ),
      is_esm: is_esm_dep_like(dep),
    }
  }
}

#[derive(Debug)]
pub struct NoRuntimeModuleCache {
  runtime: RuntimeSpec,
  chunks: Arc<HashSet<ChunkUkey>>,
  provided_names: bool,
  connections: Vec<CachedOutgoingConnection>,
  incomings: IncomingConnections,
  number_of_chunks: usize,
}

async fn create_concatenated_module(
  compilation: &Compilation,
  config: &ConcatConfiguration,
) -> Result<BoxModule> {
  let module_graph = compilation.get_module_graph();
  let root_module_id = config.root_module;
  let modules_set = config.get_modules();

  let root_module = module_graph
    .module_by_identifier(&root_module_id)
    .expect("should have module");

  let root_module_ctxt = RootModuleContext {
    id: root_module_id,
    readable_identifier: get_cached_readable_identifier(
      &root_module_id,
      module_graph,
      &compilation.module_static_cache,
      &compilation.options.context,
    ),
    name_for_condition: root_module.name_for_condition(),
    lib_indent: root_module
      .lib_ident(LibIdentOptions {
        context: compilation.options.context.as_str(),
      })
      .map(|id| id.to_string()),
    layer: root_module.get_layer().cloned(),
    resolve_options: root_module.get_resolve_options(),
    code_generation_dependencies: root_module
      .get_code_generation_dependencies()
      .map(|deps| deps.to_vec()),
    presentational_dependencies: root_module
      .get_presentational_dependencies()
      .map(|deps| deps.to_vec()),
    context: Some(compilation.options.context.clone()),
    side_effect_connection_state: root_module.get_side_effects_connection_state(
      module_graph,
      &compilation.module_graph_cache_artifact,
      &compilation
        .build_module_graph_artifact
        .side_effects_state_artifact,
      &mut IdentifierSet::default(),
      &mut IdentifierMap::default(),
    ),
    factory_meta: root_module.factory_meta().cloned(),
    build_meta: root_module.build_meta().clone(),
    module_argument: root_module.get_module_argument(),
    exports_argument: root_module.get_exports_argument(),
  };
  let modules = modules_set
    .iter()
    .map(|id| {
      let module = module_graph
        .module_by_identifier(id)
        .unwrap_or_else(|| panic!("should have module {id}"));

      ConcatenatedInnerModule {
        id: *id,
        size: module.size(
          Some(&rspack_core::SourceType::JavaScript),
          Some(compilation),
        ),
        shorten_id: get_cached_readable_identifier(
          id,
          module_graph,
          &compilation.module_static_cache,
          &compilation.options.context,
        ),
      }
    })
    .collect::<Vec<_>>();
  let mut new_module = BoxModule::new(Box::from(ConcatenatedModule::create(
    root_module_ctxt,
    modules,
    Some(rspack_hash::HashFunction::Xxhash64),
    config.runtime.clone(),
    compilation,
  )));
  let build_result = new_module
    .build(
      rspack_core::BuildContext {
        compiler_id: compilation.compiler_id(),
        compilation_id: compilation.id(),
        resolver_factory: compilation.resolver_factory.clone(),
        plugin_driver: compilation.plugin_driver.clone(),
        compiler_options: compilation.options.clone(),
        fs: compilation.input_filesystem.clone(),
        runtime_template: compilation.runtime_template.create_module_code_template(),
      },
      Some(compilation),
    )
    .await?;
  new_module = build_result.module;

  Ok(new_module)
}

fn prepare_concatenated_module_connections<F>(
  compilation: &Compilation,
  new_module: &ModuleIdentifier,
  modules_set: &IdentifierIndexSet,
  filter_connection: F,
) -> Vec<DependencyId>
where
  F: Fn(&ModuleIdentifier, &ModuleGraphConnection, &BoxDependency) -> bool + Sync,
{
  let mg = compilation.get_module_graph();

  let dependency_parts = modules_set
    .par_iter()
    .filter_map(|m| {
      if m == new_module {
        return None;
      }
      let old_mgm_connections = mg
        .module_graph_module_by_identifier(m)
        .expect("should have mgm")
        .outgoing_connections();

      let mut part = vec![];
      for dep_id in old_mgm_connections {
        let connection = mg
          .connection_by_dependency_id(dep_id)
          .expect("should have connection");
        let dep = mg.dependency_by_id(dep_id);
        if filter_connection(m, connection, dep) {
          part.push(*dep_id);
        }
      }
      Some(part)
    })
    .collect::<Vec<_>>();

  let mut res = vec![];
  for part in dependency_parts {
    res.extend(part);
  }
  res
}

fn prepare_concatenated_root_module_connections<F>(
  compilation: &Compilation,
  root_module_id: &ModuleIdentifier,
  filter_connection: F,
) -> (Vec<DependencyId>, Vec<DependencyId>)
where
  F: Fn(&ModuleIdentifier, &ModuleGraphConnection, &BoxDependency) -> bool,
{
  let mg = compilation.get_module_graph();
  let mut outgoings = vec![];
  let old_mgm_connections = mg
    .module_graph_module_by_identifier(root_module_id)
    .expect("should have mgm")
    .outgoing_connections();

  for dep_id in old_mgm_connections {
    let connection = mg
      .connection_by_dependency_id(dep_id)
      .expect("should have connection");

    let dep = mg.dependency_by_id(dep_id);
    if filter_connection(root_module_id, connection, dep) {
      outgoings.push(*dep_id);
    }
  }

  let mut incomings = vec![];
  let incoming_connections = mg
    .module_graph_module_by_identifier(root_module_id)
    .expect("should have mgm")
    .incoming_connections();

  for dep_id in incoming_connections {
    let connection = mg
      .connection_by_dependency_id(dep_id)
      .expect("should have connection");
    let dependency = mg.dependency_by_id(dep_id);
    if filter_connection(root_module_id, connection, dependency) {
      incomings.push(*dep_id);
    }
  }

  (outgoings, incomings)
}

fn add_concatenated_module(
  compilation: &mut Compilation,
  new_module: BoxModule,
  config: ConcatConfiguration,
) {
  let root_module_id = config.root_module;
  let modules_set = config.get_modules();

  let module_graph = compilation.get_module_graph();
  let box_module = module_graph
    .module_by_identifier(&root_module_id)
    .expect("should have module");
  let root_module_source_types = box_module.source_types(module_graph);
  let is_root_module_asset_module = root_module_source_types.contains(&SourceType::Asset);

  let mut chunk_graph = std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_graph);
  let module_graph = compilation.get_module_graph_mut();

  let module_graph_module = ModuleGraphModule::new(new_module.identifier());
  module_graph.add_module_graph_module(module_graph_module);
  ModuleGraph::clone_module_attributes(compilation, &root_module_id, &new_module.identifier());
  // integrate

  let module_graph = compilation.get_module_graph_mut();
  let root_chunks = chunk_graph.get_module_chunks(root_module_id).clone();

  for m in modules_set.iter() {
    if *m == root_module_id {
      continue;
    }
    let module = module_graph
      .module_by_identifier(m)
      .expect("should exist module");
    // TODO: optimize asset module https://github.com/webpack/webpack/pull/15515/files
    for chunk_ukey in &root_chunks {
      let source_types =
        chunk_graph.get_chunk_module_source_types(chunk_ukey, module, module_graph);

      if source_types.len() == 1 && source_types.contains(&SourceType::JavaScript) {
        chunk_graph.disconnect_chunk_and_module(chunk_ukey, *m);
      } else {
        let new_source_types = source_types
          .into_iter()
          .filter(|source_type| !matches!(source_type, SourceType::JavaScript))
          .collect();
        chunk_graph.set_chunk_modules_source_types(chunk_ukey, *m, new_source_types)
      }
    }
  }

  // different from webpack
  // Rspack: if entry is an asset module, outputs a js chunk and a asset chunk
  // Webpack: if entry is an asset module, outputs an asset chunk
  // these lines of codes fix a bug: when asset module (NormalModule) is concatenated into ConcatenatedModule, the asset will be lost
  // because `chunk_graph.replace_module(&root_module_id, &new_module.id());` will remove the asset module from chunk, and I add this module back to fix this bug
  if is_root_module_asset_module {
    chunk_graph.replace_module(&root_module_id, &new_module.identifier());
    chunk_graph.add_module(root_module_id);
    for chunk_ukey in chunk_graph
      .get_module_chunks(new_module.identifier())
      .clone()
    {
      let module = module_graph
        .module_by_identifier(&root_module_id)
        .expect("should exist module");

      let source_types =
        chunk_graph.get_chunk_module_source_types(&chunk_ukey, module, module_graph);
      let new_source_types = source_types
        .iter()
        .filter(|source_type| !matches!(source_type, SourceType::JavaScript))
        .copied()
        .collect();
      chunk_graph.set_chunk_modules_source_types(&chunk_ukey, root_module_id, new_source_types);
      chunk_graph.connect_chunk_and_module(chunk_ukey, root_module_id);
    }
  } else {
    chunk_graph.replace_module(&root_module_id, &new_module.identifier());
  }

  module_graph.add_module(new_module);
  compilation.build_chunk_graph_artifact.chunk_graph = chunk_graph;
}

fn is_connection_active_in_runtime(
  connection: &CachedIncomingConnection,
  runtime: Option<&RuntimeSpec>,
  cached_runtime: &RuntimeSpec,
  mg: &ModuleGraph,
  artifacts: &ModuleGraphArtifacts,
) -> bool {
  if let Some(runtime) = runtime {
    if runtime == cached_runtime {
      return connection.active;
    }

    if connection.active && cached_runtime.is_subset(runtime) {
      return true;
    }

    if !connection.active && cached_runtime.is_superset(runtime) {
      return false;
    }
  }

  connection.connection.is_active(
    mg,
    runtime,
    artifacts.mg_cache,
    artifacts.side_effects_state_artifact,
    artifacts.exports_info_artifact,
  )
}
