#![allow(clippy::only_used_in_recursion)]
use std::{borrow::Cow, collections::VecDeque, rc::Rc, sync::Arc};

use rayon::prelude::*;
use rspack_collections::{
  Identifiable, IdentifierDashMap, IdentifierIndexSet, IdentifierMap, IdentifierSet,
};
use rspack_core::{
  BoxDependency, BoxModule, Compilation, CompilationOptimizeChunkModules, DependencyId,
  DependencyType, ExportProvided, ExportsInfoArtifact, ExtendedReferencedExport, GetTargetResult,
  ImportedByDeferModulesArtifact, LibIdentOptions, Logger, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleGraphConnection, ModuleGraphModule, ModuleIdentifier, Plugin, PrefetchExportsInfoMode,
  ProvidedExports, RuntimeCondition, RuntimeSpec, SourceType,
  concatenated_module::{
    ConcatenatedInnerModule, ConcatenatedModule, RootModuleContext, is_esm_dep_like,
  },
  filter_runtime, get_cached_readable_identifier, get_target,
  incremental::IncrementalPasses,
};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_hook::{plugin, plugin_hook};
use rspack_util::itoa;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

fn format_bailout_reason(msg: &str) -> String {
  format!("ModuleConcatenation bailout: {msg}")
}

#[derive(Clone, Debug)]
enum Warning {
  Id(ModuleIdentifier),
  Problem(String),
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

  fn get_warnings_sorted(&self) -> Vec<(ModuleIdentifier, Warning)> {
    let mut sorted_warnings: Vec<_> = self.warnings.clone().into_iter().collect();
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
  runtime_map: HashMap<RuntimeSpec, IdentifierMap<T>>,
}

impl<T> RuntimeIdentifierCache<T> {
  fn insert(&mut self, module: ModuleIdentifier, runtime: Option<&RuntimeSpec>, value: T) {
    if let Some(runtime) = runtime {
      if let Some(map) = self.runtime_map.get_mut(runtime) {
        map.insert(module, value);
      } else {
        let mut map = IdentifierMap::default();
        map.insert(module, value);
        self.runtime_map.insert(runtime.clone(), map);
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
  fn format_bailout_warning(&self, module: ModuleIdentifier, warning: &Warning) -> String {
    match warning {
      Warning::Problem(id) => format_bailout_reason(&format!("Cannot concat with {module}: {id}")),
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
      .push(format_bailout_reason(&reason));
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

  pub fn get_imports(
    mg: &ModuleGraph,
    mg_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    mi: ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    imports_cache: &mut RuntimeIdentifierCache<IdentifierIndexSet>,
    module_cache: &HashMap<ModuleIdentifier, NoRuntimeModuleCache>,
  ) -> IdentifierIndexSet {
    if let Some(set) = imports_cache.get(&mi, runtime) {
      return set.clone();
    }

    let cached = module_cache.get(&mi).expect("should have module");

    let mut set = IdentifierIndexSet::default();
    for (con, (has_imported_names, cached_active)) in &cached.connections {
      if set.contains(con.module_identifier()) {
        continue;
      }

      let is_target_active = if let Some(runtime) = runtime {
        if cached.runtime == *runtime {
          // runtime is same, use cached value
          *cached_active
        } else if *cached_active && cached.runtime.is_subset(runtime) {
          // cached runtime is subset and active, means it is also active in current runtime
          true
        } else if !*cached_active && cached.runtime.is_superset(runtime) {
          // cached runtime is superset and inactive, means it is also inactive in current runtime
          false
        } else {
          // can't determine, need to check
          con.is_target_active(mg, Some(runtime), mg_cache, exports_info_artifact)
        }
      } else {
        // no runtime, need to check
        con.is_target_active(mg, None, mg_cache, exports_info_artifact)
      };

      if !is_target_active {
        continue;
      }
      if *has_imported_names || cached.provided_names {
        set.insert(*con.module_identifier());
      }
    }

    imports_cache.insert(mi, runtime, set.clone());
    set
  }

  #[allow(clippy::too_many_arguments)]
  fn try_to_add(
    compilation: &Compilation,
    config: &mut ConcatConfiguration,
    module_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    active_runtime: Option<&RuntimeSpec>,
    possible_modules: &IdentifierSet,
    candidates: &mut IdentifierSet,
    failure_cache: &mut IdentifierMap<Warning>,
    success_cache: &mut RuntimeIdentifierCache<Vec<ModuleIdentifier>>,
    avoid_mutate_on_failure: bool,
    statistics: &mut Statistics,
    imports_cache: &mut RuntimeIdentifierCache<IdentifierIndexSet>,
    module_cache: &HashMap<ModuleIdentifier, NoRuntimeModuleCache>,
  ) -> Option<Warning> {
    statistics
      .module_visit
      .entry(*module_id)
      .and_modify(|count| {
        *count += 1;
      })
      .or_insert(1);

    if let Some(cache_entry) = failure_cache.get(module_id) {
      statistics.cached += 1;
      return Some(cache_entry.clone());
    }

    if config.has(module_id) {
      statistics.already_in_config += 1;
      return None;
    }

    let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
    let chunk_by_ukey = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;

    let incoming_modules = if let Some(incomings) = success_cache.get(module_id, runtime) {
      statistics.cache_hit += 1;
      incomings.clone()
    } else {
      let module_readable_identifier = get_cached_readable_identifier(
        module_id,
        module_graph,
        &compilation.module_static_cache,
        &compilation.options.context,
      );

      if !possible_modules.contains(module_id) {
        statistics.invalid_module += 1;
        let problem = Warning::Id(*module_id);
        failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }

      let missing_chunks: Vec<_> = chunk_graph
        .get_module_chunks(config.root_module)
        .iter()
        .filter(|chunk| !chunk_graph.is_module_in_chunk(module_id, **chunk))
        .collect();

      if !missing_chunks.is_empty() {
        let problem_string = {
          let mut missing_chunks_list = missing_chunks
            .iter()
            .map(|&chunk| {
              let chunk = chunk_by_ukey.expect_get(chunk);
              chunk.name().unwrap_or("unnamed chunk(s)")
            })
            .collect::<Vec<_>>();
          missing_chunks_list.sort_unstable();

          let mut chunks = chunk_graph
            .get_module_chunks(*module_id)
            .iter()
            .map(|&chunk| {
              let chunk = chunk_by_ukey.expect_get(&chunk);
              chunk.name().unwrap_or("unnamed chunk(s)")
            })
            .collect::<Vec<_>>();
          chunks.sort_unstable();

          format!(
            "Module {} is not in the same chunk(s) (expected in chunk(s) {}, module is in chunk(s) {})",
            module_readable_identifier,
            missing_chunks_list.join(", "),
            chunks.join(", ")
          )
        };

        statistics.incorrect_chunks += 1;
        let problem = Warning::Problem(problem_string);
        failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }

      let NoRuntimeModuleCache {
        incomings,
        active_incomings,
        runtime: cached_module_runtime,
        ..
      } = module_cache
        .get(module_id)
        .expect("should have module cache");

      if let Some(incoming_connections_from_non_modules) = incomings.get(&None) {
        let has_active_non_modules_connections =
          incoming_connections_from_non_modules
            .iter()
            .any(|connection| {
              is_connection_active_in_runtime(
                connection,
                runtime,
                active_incomings,
                cached_module_runtime,
                module_graph,
                module_graph_cache,
                &compilation.exports_info_artifact,
              )
            });

        // TODO: ADD module connection explanations
        if has_active_non_modules_connections {
          let problem = {
            // let importing_explanations = active_non_modules_connections
            //   .iter()
            //   .flat_map(|&c| c.explanation())
            //   .collect::<HashSet<_>>();
            // let mut explanations: Vec<_> = importing_explanations.into_iter().collect();
            // explanations.sort();
            format!(
              "Module {module_readable_identifier} is referenced",
              // if !explanations.is_empty() {
              //   format!("by: {}", explanations.join(", "))
              // } else {
              //   "in an unsupported way".to_string()
              // }
            )
          };
          let problem = Warning::Problem(problem);
          statistics.incorrect_dependency += 1;
          failure_cache.insert(*module_id, problem.clone());
          return Some(problem);
        }
      }

      let mut incoming_connections_from_modules = HashMap::default();
      for (origin_module, connections) in incomings.iter() {
        if let Some(origin_module) = origin_module {
          let number_of_chunks = module_cache.get(origin_module).map_or_else(
            || chunk_graph.get_number_of_module_chunks(*origin_module),
            |m| m.number_of_chunks,
          );

          if number_of_chunks == 0 {
            // Ignore connection from orphan modules
            continue;
          }

          let is_intersect = if let Some(runtime) = runtime {
            if let Some(origin_runtime) = module_cache.get(origin_module).map(|m| &m.runtime) {
              !runtime.is_disjoint(origin_runtime)
            } else {
              let mut origin_runtime = RuntimeSpec::default();
              for r in chunk_graph.get_module_runtimes_iter(*origin_module, chunk_by_ukey) {
                origin_runtime.extend(r);
              }
              !runtime.is_disjoint(&origin_runtime)
            }
          } else {
            false
          };

          if !is_intersect {
            continue;
          }

          let active_connections: Vec<_> = connections
            .iter()
            .filter(|&connection| {
              is_connection_active_in_runtime(
                connection,
                runtime,
                active_incomings,
                cached_module_runtime,
                module_graph,
                module_graph_cache,
                &compilation.exports_info_artifact,
              )
            })
            .collect();

          if !active_connections.is_empty() {
            incoming_connections_from_modules.insert(origin_module, active_connections);
          }
        }
      }

      let mut incoming_modules = incoming_connections_from_modules
        .keys()
        .map(|mid| **mid)
        .collect::<Vec<_>>();
      let other_chunk_modules = incoming_modules
        .iter()
        .filter(|&origin_module| {
          chunk_graph
            .get_module_chunks(config.root_module)
            .iter()
            .any(|&chunk_ukey| !chunk_graph.is_module_in_chunk(origin_module, chunk_ukey))
        })
        .collect::<Vec<_>>();

      if !other_chunk_modules.is_empty() {
        let problem = {
          let mut names: Vec<_> = other_chunk_modules
            .into_iter()
            .map(|mid| {
              get_cached_readable_identifier(
                mid,
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
        };

        statistics.incorrect_chunks_of_importer += 1;
        let problem = Warning::Problem(problem);
        failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }

      let mut non_esm_connections = HashMap::default();
      for (origin_module, connections) in incoming_connections_from_modules.iter() {
        let has_non_esm_connections = connections.iter().any(|connection| {
          let dep = module_graph.dependency_by_id(&connection.dependency_id);
          !is_esm_dep_like(dep)
        });

        if has_non_esm_connections {
          non_esm_connections.insert(origin_module, connections);
        }
      }

      if !non_esm_connections.is_empty() {
        let problem = {
          let names: Vec<_> = non_esm_connections
            .iter()
            .map(|(origin_module, connections)| {
              let readable_identifier = get_cached_readable_identifier(
                origin_module,
                module_graph,
                &compilation.module_static_cache,
                &compilation.options.context,
              );
              let mut names = connections
                .iter()
                .map(|item| {
                  let dep = module_graph.dependency_by_id(&item.dependency_id);
                  dep.dependency_type().to_string()
                })
                .collect::<Vec<_>>();
              names.sort();
              format!(
                "{} (referenced with {})",
                readable_identifier,
                names.join(",")
              )
            })
            .collect();

          format!(
            "Module {} is referenced from these modules with unsupported syntax: {}",
            module_readable_identifier,
            names.join(", ")
          )
        };
        let problem = Warning::Problem(problem);
        statistics.incorrect_module_dependency += 1;
        failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }

      if let Some(runtime) = runtime
        && runtime.len() > 1
      {
        let mut other_runtime_connections = Vec::new();
        'outer: for (origin_module, connections) in incoming_connections_from_modules {
          let mut current_runtime_condition = RuntimeCondition::Boolean(false);
          for connection in connections {
            let runtime_condition = filter_runtime(Some(runtime), |runtime| {
              connection.is_target_active(
                module_graph,
                runtime,
                module_graph_cache,
                &compilation.exports_info_artifact,
              )
            });

            if runtime_condition == RuntimeCondition::Boolean(false) {
              continue;
            }

            if runtime_condition == RuntimeCondition::Boolean(true) {
              continue 'outer;
            }

            // here two runtime_condition must be `RuntimeCondition::Spec`
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
            other_runtime_connections.push((origin_module, current_runtime_condition));
          }
        }

        if !other_runtime_connections.is_empty() {
          let problem = {
            format!(
              "Module {} is runtime-dependent referenced by these modules: {}",
              module_readable_identifier,
              other_runtime_connections
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
                    runtime,
                    runtime_condition.as_spec().expect("should be spec")
                  )
                })
                .collect::<Vec<_>>()
                .join(", ")
            )
          };

          let problem = Warning::Problem(problem);
          statistics.incorrect_runtime_condition += 1;
          failure_cache.insert(*module_id, problem.clone());
          return Some(problem);
        }
      }

      incoming_modules.sort();
      success_cache.insert(*module_id, runtime, incoming_modules.clone());
      incoming_modules
    };

    let backup = if avoid_mutate_on_failure {
      Some(config.snapshot())
    } else {
      None
    };

    config.add(*module_id);

    for origin_module in &incoming_modules {
      if let Some(problem) = Self::try_to_add(
        compilation,
        config,
        origin_module,
        runtime,
        active_runtime,
        possible_modules,
        candidates,
        failure_cache,
        success_cache,
        false,
        statistics,
        imports_cache,
        module_cache,
      ) {
        if let Some(backup) = &backup {
          config.rollback(*backup);
        }
        statistics.importer_failed += 1;
        failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }
    }

    for imp in Self::get_imports(
      module_graph,
      module_graph_cache,
      &compilation.exports_info_artifact,
      *module_id,
      runtime,
      imports_cache,
      module_cache,
    ) {
      candidates.insert(imp);
    }
    statistics.added += 1;
    None
  }

  pub async fn process_concatenated_configuration(
    compilation: &mut Compilation,
    config: ConcatConfiguration,
    used_modules: &mut HashSet<ModuleIdentifier>,
  ) -> Result<()> {
    let module_graph = compilation.get_module_graph();

    let root_module_id = config.root_module;
    if used_modules.contains(&root_module_id) {
      return Ok(());
    }

    let modules_set = config.get_modules();
    for m in modules_set {
      used_modules.insert(*m);
    }
    let box_module = module_graph
      .module_by_identifier(&root_module_id)
      .expect("should have module");
    let root_module_source_types = box_module.source_types(module_graph);

    let is_root_module_asset_module = root_module_source_types.contains(&SourceType::Asset);
    if is_root_module_asset_module && !root_module_source_types.contains(&SourceType::JavaScript) {
      return Ok(());
    }

    let root_module_ctxt = RootModuleContext {
      id: root_module_id,
      readable_identifier: get_cached_readable_identifier(
        &root_module_id,
        module_graph,
        &compilation.module_static_cache,
        &compilation.options.context,
      ),
      name_for_condition: box_module.name_for_condition(),
      lib_indent: box_module
        .lib_ident(LibIdentOptions {
          context: compilation.options.context.as_str(),
        })
        .map(|id| id.to_string()),
      layer: box_module.get_layer().cloned(),
      resolve_options: box_module.get_resolve_options(),
      code_generation_dependencies: box_module
        .get_code_generation_dependencies()
        .map(|deps| deps.to_vec()),
      presentational_dependencies: box_module
        .get_presentational_dependencies()
        .map(|deps| deps.to_vec()),
      context: Some(compilation.options.context.clone()),
      side_effect_connection_state: box_module.get_side_effects_connection_state(
        module_graph,
        &compilation.module_graph_cache_artifact,
        &mut IdentifierSet::default(),
        &mut IdentifierMap::default(),
      ),
      factory_meta: box_module.factory_meta().cloned(),
      build_meta: box_module.build_meta().clone(),
      module_argument: box_module.get_module_argument(),
      exports_argument: box_module.get_exports_argument(),
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

    let mut new_module = BoxModule::new(Box::new(ConcatenatedModule::create(
      root_module_ctxt,
      modules,
      Some(rspack_hash::HashFunction::MD4),
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

    let mut chunk_graph = std::mem::take(&mut compilation.build_chunk_graph_artifact.chunk_graph);
    let module_graph = compilation.get_module_graph_mut();
    let module_graph_module = ModuleGraphModule::new(new_module.identifier());
    module_graph.add_module_graph_module(module_graph_module);
    ModuleGraph::clone_module_attributes(compilation, &root_module_id, &new_module.identifier());
    // integrate

    let module_graph = compilation.get_module_graph_mut();
    for m in modules_set {
      if m == &root_module_id {
        continue;
      }
      module_graph.copy_outgoing_module_connections(m, &new_module.identifier(), |con, dep| {
        con.original_module_identifier.as_ref() == Some(m)
          && !(is_esm_dep_like(dep) && modules_set.contains(con.module_identifier()))
      });
      // TODO: optimize asset module https://github.com/webpack/webpack/pull/15515/files
      for chunk_ukey in chunk_graph.get_module_chunks(root_module_id).clone() {
        let module = module_graph
          .module_by_identifier(m)
          .expect("should exist module");

        let source_types =
          chunk_graph.get_chunk_module_source_types(&chunk_ukey, module, module_graph);

        if source_types.len() == 1
          && !matches!(
            source_types.iter().next().expect("has length"),
            SourceType::Css
          )
        {
          chunk_graph.disconnect_chunk_and_module(&chunk_ukey, *m);
        } else {
          let new_source_types = source_types
            .into_iter()
            .filter(|source_type| !matches!(source_type, SourceType::JavaScript))
            .collect();
          chunk_graph.set_chunk_modules_source_types(&chunk_ukey, *m, new_source_types)
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

    module_graph.move_module_connections(&root_module_id, &new_module.identifier(), |c, dep| {
      let other_module = if *c.module_identifier() == root_module_id {
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
    });
    module_graph.add_module(new_module);
    compilation.build_chunk_graph_artifact.chunk_graph = chunk_graph;
    Ok(())
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
          .get_prefetched_exports_info(&module_id, PrefetchExportsInfoMode::Default);
        let relevant_exports = exports_info.get_relevant_exports(None);
        let unknown_exports = relevant_exports
          .iter()
          .filter(|export_info| {
            export_info.is_reexport()
              && !matches!(
                get_target(
                  export_info,
                  module_graph,
                  &compilation.exports_info_artifact,
                  Rc::new(|_| true),
                  &mut Default::default()
                ),
                Some(GetTargetResult::Target(_))
              )
          })
          .copied()
          .collect::<Vec<_>>();
        if !unknown_exports.is_empty() {
          let cur_bailout_reason = unknown_exports
            .into_iter()
            .map(|export_info| {
              let name = export_info
                .name()
                .map_or("other exports".to_string(), |name| name.to_string());
              format!("{} : {}", name, export_info.get_used_info())
            })
            .collect::<Vec<String>>()
            .join(", ");
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
        let unknown_provided_exports = relevant_exports
          .iter()
          .filter(|export_info| !matches!(export_info.provided(), Some(ExportProvided::Provided)))
          .copied()
          .collect::<Vec<_>>();

        if !unknown_provided_exports.is_empty() {
          let cur_bailout_reason = unknown_provided_exports
            .into_iter()
            .map(|export_info| {
              let name = export_info
                .name()
                .map_or("other exports".to_string(), |name| name.to_string());
              format!(
                "{} : {} and {}",
                name,
                export_info.get_provided_info(),
                export_info.get_used_info(),
              )
            })
            .collect::<Vec<String>>()
            .join(", ");
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

        if module_graph.is_deferred(&compilation.imported_by_defer_modules_artifact, &module_id) {
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
    relevant_modules.sort_by(|a, b| {
      let ad = module_graph.get_depth(a);
      let bd = module_graph.get_depth(b);
      ad.cmp(&bd)
    });

    logger.time_end(start);
    let mut statistics = Statistics::default();
    let mut stats_candidates = 0;
    let mut stats_size_sum = 0;
    let mut stats_empty_configurations = 0;

    let start = logger.time("find modules to concatenate");
    let mut concat_configurations: Vec<ConcatConfiguration> = Vec::new();
    let mut used_as_inner: IdentifierSet = IdentifierSet::default();
    let mut imports_cache = RuntimeIdentifierCache::<IdentifierIndexSet>::default();

    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;
    let module_static_cache = &compilation.module_static_cache;
    let compilation_context = &compilation.options.context;
    let cache_modules = relevant_modules
      .iter()
      .chain(possible_inners.iter())
      .copied()
      .collect::<IdentifierSet>();
    let modules_without_runtime_cache = cache_modules
      .into_par_iter()
      .map(|module_id| {
        let exports_info = compilation
          .exports_info_artifact
          .get_prefetched_exports_info(&module_id, PrefetchExportsInfoMode::Default);
        let provided_names = matches!(
          exports_info.get_provided_exports(),
          ProvidedExports::ProvidedNames(_)
        );
        let module = module_graph
          .module_by_identifier(&module_id)
          .expect("should have module");
        let mut runtime = RuntimeSpec::default();
        for r in compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_runtimes_iter(
            module_id,
            &compilation.build_chunk_graph_artifact.chunk_by_ukey,
          )
        {
          runtime.extend(r);
        }

        let _ = get_cached_readable_identifier(
          &module_id,
          module_graph,
          module_static_cache,
          compilation_context,
        );

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

            Some((
              con.clone(),
              (
                imported_names.iter().all(|item| match item {
                  ExtendedReferencedExport::Array(arr) => !arr.is_empty(),
                  ExtendedReferencedExport::Export(export) => !export.name.is_empty(),
                }),
                con.is_target_active(
                  module_graph,
                  Some(&runtime),
                  module_graph_cache,
                  &compilation.exports_info_artifact,
                ),
              ),
            ))
          })
          .collect::<Vec<_>>();

        let incomings: HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>> = module_graph
          .get_incoming_connections_by_origin_module(&module_id)
          .into_iter()
          .map(|(k, v)| (k, v.into_iter().cloned().collect()))
          .collect();
        let mut active_incomings = HashMap::default();
        for connection in incomings.values().flatten() {
          active_incomings.insert(
            connection.dependency_id,
            connection.is_active(
              module_graph,
              Some(&runtime),
              module_graph_cache,
              &compilation.exports_info_artifact,
            ),
          );
        }
        let number_of_chunks = compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_number_of_module_chunks(module_id);
        (
          module_id,
          NoRuntimeModuleCache {
            runtime,
            provided_names,
            connections,
            incomings,
            active_incomings,
            number_of_chunks,
          },
        )
      })
      .collect::<HashMap<_, _>>();

    for current_root in relevant_modules.iter() {
      if used_as_inner.contains(current_root) {
        continue;
      }

      let NoRuntimeModuleCache { runtime, .. } = modules_without_runtime_cache
        .get(current_root)
        .expect("should have module");
      let module_graph = compilation.get_module_graph();
      let module_graph_cache = &compilation.module_graph_cache_artifact;
      let exports_info = compilation
        .exports_info_artifact
        .get_prefetched_exports_info(current_root, PrefetchExportsInfoMode::Default);
      let filtered_runtime = filter_runtime(Some(runtime), |r| exports_info.is_module_used(r));
      let active_runtime = match filtered_runtime {
        RuntimeCondition::Boolean(true) => Some(runtime.clone()),
        RuntimeCondition::Boolean(false) => None,
        RuntimeCondition::Spec(spec) => Some(spec),
      };

      let mut current_configuration =
        ConcatConfiguration::new(*current_root, active_runtime.clone());

      let mut failure_cache = IdentifierMap::default();
      let mut success_cache = RuntimeIdentifierCache::default();
      let mut candidates_visited = HashSet::default();
      let mut candidates = VecDeque::new();

      let imports = Self::get_imports(
        module_graph,
        module_graph_cache,
        &compilation.exports_info_artifact,
        *current_root,
        active_runtime.as_ref(),
        &mut imports_cache,
        &modules_without_runtime_cache,
      );
      for import in imports {
        candidates.push_back(import);
      }

      let mut import_candidates = IdentifierSet::default();
      while let Some(imp) = candidates.pop_front() {
        if candidates_visited.contains(&imp) {
          continue;
        }
        candidates_visited.insert(imp);
        import_candidates.clear();
        match Self::try_to_add(
          compilation,
          &mut current_configuration,
          &imp,
          Some(runtime),
          active_runtime.as_ref(),
          &possible_inners,
          &mut import_candidates,
          &mut failure_cache,
          &mut success_cache,
          true,
          &mut statistics,
          &mut imports_cache,
          &modules_without_runtime_cache,
        ) {
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
        let module_graph = compilation.get_module_graph_mut();
        let optimization_bailouts = module_graph.get_optimization_bailout_mut(current_root);
        for warning in current_configuration.get_warnings_sorted() {
          optimization_bailouts.push(self.format_bailout_warning(warning.0, &warning.1));
        }
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
    concat_configurations.sort_by(|a, b| b.modules.len().cmp(&a.modules.len()));
    logger.time_end(start);

    let mut used_modules = HashSet::default();
    let mut batch = vec![];

    for config in concat_configurations {
      if used_modules.contains(&config.root_module) {
        continue;
      }
      let modules_set = config.get_modules();
      used_modules.extend(modules_set.iter().copied());
      batch.push(config);
    }

    let new_modules = rspack_futures::scope::<_, Result<_>>(|token| {
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
  cache_hit: u32,
  module_visit: IdentifierMap<usize>,
  added: u32,
}

#[derive(Debug)]
pub struct NoRuntimeModuleCache {
  runtime: RuntimeSpec,
  provided_names: bool,
  connections: Vec<(ModuleGraphConnection, (bool, bool))>,
  incomings: HashMap<Option<ModuleIdentifier>, Vec<ModuleGraphConnection>>,
  active_incomings: HashMap<DependencyId, bool>,
  number_of_chunks: usize,
}

async fn create_concatenated_module(
  compilation: &Compilation,
  config: &ConcatConfiguration,
) -> Result<BoxModule> {
  let module_graph = compilation.get_module_graph();
  let root_module_id = config.root_module;
  let modules_set = config.get_modules();

  let box_module = module_graph
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
    name_for_condition: box_module.name_for_condition(),
    lib_indent: box_module
      .lib_ident(LibIdentOptions {
        context: compilation.options.context.as_str(),
      })
      .map(|id| id.to_string()),
    layer: box_module.get_layer().cloned(),
    resolve_options: box_module.get_resolve_options(),
    code_generation_dependencies: box_module
      .get_code_generation_dependencies()
      .map(|deps| deps.to_vec()),
    presentational_dependencies: box_module
      .get_presentational_dependencies()
      .map(|deps| deps.to_vec()),
    context: Some(compilation.options.context.clone()),
    side_effect_connection_state: box_module.get_side_effects_connection_state(
      module_graph,
      &compilation.module_graph_cache_artifact,
      &mut IdentifierSet::default(),
      &mut IdentifierMap::default(),
    ),
    factory_meta: box_module.factory_meta().cloned(),
    build_meta: box_module.build_meta().clone(),
    module_argument: box_module.get_module_argument(),
    exports_argument: box_module.get_exports_argument(),
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
    Some(rspack_hash::HashFunction::MD4),
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
  F: Fn(&ModuleIdentifier, &ModuleGraphConnection, &BoxDependency) -> bool,
{
  let mg = compilation.get_module_graph();
  let mut res = vec![];
  for m in modules_set.iter() {
    if m == new_module {
      continue;
    }

    let old_mgm_connections = mg
      .module_graph_module_by_identifier(m)
      .expect("should have mgm")
      .outgoing_connections();

    // Outgoing connections
    for dep_id in old_mgm_connections {
      let connection = mg
        .connection_by_dependency_id(dep_id)
        .expect("should have connection");
      let dep = mg.dependency_by_id(dep_id);
      if filter_connection(m, connection, dep) {
        res.push(*dep_id);
      }
    }
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

  for m in modules_set.iter() {
    if *m == root_module_id {
      continue;
    }
    let module = module_graph
      .module_by_identifier(m)
      .expect("should exist module");
    // TODO: optimize asset module https://github.com/webpack/webpack/pull/15515/files
    for chunk_ukey in chunk_graph.get_module_chunks(root_module_id).clone() {
      let source_types =
        chunk_graph.get_chunk_module_source_types(&chunk_ukey, module, module_graph);

      if source_types.len() == 1 {
        chunk_graph.disconnect_chunk_and_module(&chunk_ukey, *m);
      } else {
        let new_source_types = source_types
          .into_iter()
          .filter(|source_type| !matches!(source_type, SourceType::JavaScript))
          .collect();
        chunk_graph.set_chunk_modules_source_types(&chunk_ukey, *m, new_source_types)
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
  connection: &ModuleGraphConnection,
  runtime: Option<&RuntimeSpec>,
  cached_active_incomings: &HashMap<DependencyId, bool>,
  cached_runtime: &RuntimeSpec,
  mg: &ModuleGraph,
  mg_cache: &ModuleGraphCacheArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> bool {
  if let (Some(cached_active), Some(runtime)) = (
    cached_active_incomings.get(&connection.dependency_id),
    runtime,
  ) {
    if runtime == cached_runtime {
      return *cached_active;
    }

    if *cached_active && cached_runtime.is_subset(runtime) {
      return true;
    }

    if !*cached_active && cached_runtime.is_superset(runtime) {
      return false;
    }
  }

  connection.is_active(mg, runtime, mg_cache, exports_info_artifact)
}
