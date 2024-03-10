#![allow(clippy::only_used_in_recursion)]
use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::hash::Hasher;

use indexmap::IndexSet;
use rspack_core::concatenated_module::{
  is_harmony_dep_like, ConcatenatedInnerModule, ConcatenatedModule, RootModuleContext,
};
use rspack_core::{
  filter_runtime, merge_runtime, runtime_to_string, Compilation, CompilerContext,
  ExportInfoProvided, ExtendedReferencedExport, LibIdentOptions, Logger, Module, ModuleExt,
  ModuleGraph, ModuleGraphModule, ModuleIdentifier, MutexModuleGraph, OptimizeChunksArgs, Plugin,
  ProvidedExports, RuntimeCondition, RuntimeSpec, SourceType,
};
use rspack_error::Result;
use rspack_util::fx_dashmap::FxDashMap;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

fn format_bailout_reason(msg: &str) -> String {
  format!("ModuleConcatenation bailout: {}", msg)
}

#[derive(Clone, Debug)]
enum Warning {
  Id(ModuleIdentifier),
  Problem(String),
}

#[derive(Debug, Clone)]
struct ConcatConfiguration {
  pub root_module: ModuleIdentifier,
  runtime: Option<RuntimeSpec>,
  modules: HashSet<ModuleIdentifier>,
  warnings: HashMap<ModuleIdentifier, Warning>,
}

#[allow(unused)]
impl ConcatConfiguration {
  fn new(root_module: ModuleIdentifier, runtime: Option<RuntimeSpec>) -> Self {
    let mut modules = HashSet::default();
    modules.insert(root_module);

    ConcatConfiguration {
      root_module,
      runtime,
      modules,
      warnings: HashMap::default(),
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

  fn get_warnings_sorted(&self) -> HashMap<ModuleIdentifier, Warning> {
    let mut sorted_warnings: Vec<_> = self.warnings.clone().into_iter().collect();
    sorted_warnings.sort_by(|a, b| a.0.cmp(&b.0));
    sorted_warnings.into_iter().collect()
  }

  fn get_modules(&self) -> &HashSet<ModuleIdentifier> {
    &self.modules
  }

  fn snapshot(&self) -> usize {
    self.modules.len()
  }

  fn rollback(&mut self, mut snapshot: usize) {
    let modules = &mut self.modules;
    modules.retain(|_| {
      if snapshot == 0 {
        false
      } else {
        snapshot -= 1;
        true
      }
    });
  }
}

#[derive(Debug, Default)]
pub struct ModuleConcatenationPlugin {
  bailout_reason_map: FxDashMap<ModuleIdentifier, String>,
}

impl ModuleConcatenationPlugin {
  fn format_bailout_warning(&self, module: ModuleIdentifier, warning: &Warning) -> String {
    match warning {
      Warning::Problem(id) => {
        format_bailout_reason(&format!("Cannot concat with {}: {}", module, id))
      }
      Warning::Id(id) => {
        let reason = self.get_inner_bailout_reason(id);
        let reason_with_prefix = match reason {
          Some(reason) => format!(": {}", *reason),
          None => "".to_string(),
        };
        if id == &module {
          format_bailout_reason(&format!(
            "Cannot concat with {}{}",
            module, reason_with_prefix
          ))
        } else {
          format_bailout_reason(&format!(
            "Cannot concat with {} because of {}{}",
            module, id, reason_with_prefix
          ))
        }
      }
    }
  }

  fn set_bailout_reason(&self, module: &ModuleIdentifier, reason: String, mg: &mut ModuleGraph) {
    self.set_inner_bailout_reason(module, reason.clone());
    mg.get_optimization_bailout_mut(*module)
      .push(format_bailout_reason(&reason));
  }

  fn set_inner_bailout_reason(&self, module: &ModuleIdentifier, reason: String) {
    self.bailout_reason_map.insert(*module, reason);
  }

  fn get_inner_bailout_reason(
    &self,
    module_id: &ModuleIdentifier,
  ) -> Option<
    dashmap::mapref::one::Ref<
      '_,
      rspack_identifier::Identifier,
      String,
      std::hash::BuildHasherDefault<rustc_hash::FxHasher>,
    >,
  > {
    self.bailout_reason_map.get(module_id)
  }

  pub fn get_imports(
    mg: &ModuleGraph,
    mi: ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> IndexSet<ModuleIdentifier> {
    let mut set = IndexSet::default();
    let module = mg.module_by_identifier(&mi).expect("should have module");
    // let flag = mi.ends_with("node_modules/@ant-design/cssinjs/es/index.js");
    for d in module.get_dependencies() {
      let dep = d.get_dependency(mg);
      let is_harmony_import_like = is_harmony_dep_like(dep);
      if !is_harmony_import_like {
        continue;
      }
      let Some(con) = mg.connection_by_dependency(d) else {
        continue;
      };
      // if flag {
      //   dbg!(&con);
      //   dbg!(&con.is_target_active(mg, runtime));
      // }
      if !con.is_target_active(mg, runtime) {
        continue;
      }
      // SAFETY: because it is extends harmony dep, we can ensure the dep has been
      // implemented ModuleDependency Trait.
      let module_dep = dep.as_module_dependency().expect("should be module dep");
      let imported_names = module_dep.get_referenced_exports(mg, None);
      if imported_names.iter().all(|item| match item {
        ExtendedReferencedExport::Array(arr) => !arr.is_empty(),
        ExtendedReferencedExport::Export(export) => !export.name.is_empty(),
      }) || matches!(mg.get_provided_exports(mi), ProvidedExports::Vec(_))
      {
        set.insert(con.module_identifier);
      }
    }
    set
  }

  #[allow(clippy::too_many_arguments)]
  fn try_to_add(
    compilation: &Compilation,
    config: &mut ConcatConfiguration,
    module_id: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    active_runtime: Option<&RuntimeSpec>,
    possible_modules: &HashSet<ModuleIdentifier>,
    candidates: &mut HashSet<ModuleIdentifier>,
    failure_cache: &mut HashMap<ModuleIdentifier, Warning>,
    avoid_mutate_on_failure: bool,
    statistics: &mut Statistics,
  ) -> Option<Warning> {
    let Compilation {
      chunk_graph,
      options,
      chunk_by_ukey,
      ..
    } = compilation;
    let module_graph = compilation.get_module_graph();
    if let Some(cache_entry) = failure_cache.get(module_id) {
      statistics.cached += 1;
      return Some(cache_entry.clone());
    }

    if config.has(module_id) {
      statistics.already_in_config += 1;
      return None;
    }

    let module = module_graph
      .module_by_identifier(module_id)
      .expect("should have module");
    let module_readable_identifier = module.readable_identifier(&options.context).to_string();

    if !possible_modules.contains(module_id) {
      statistics.invalid_module += 1;
      let problem = Warning::Id(*module_id);
      failure_cache.insert(*module_id, problem.clone());
      return Some(problem);
    }

    let missing_chunks: Vec<_> = chunk_graph
      .get_module_chunks(config.root_module)
      .iter()
      .cloned()
      .filter(|&chunk| !chunk_graph.is_module_in_chunk(module_id, chunk))
      .collect();

    if !missing_chunks.is_empty() {
      let problem_string = {
        let missing_chunks_list = missing_chunks
          .iter()
          .map(|&chunk| {
            let chunk = chunk_by_ukey.expect_get(&chunk);
            chunk.name.clone().unwrap_or("unnamed chunk(s)".to_owned())
          })
          .collect::<Vec<_>>();
        let chunks = chunk_graph
          .get_module_chunks(*module_id)
          .iter()
          .map(|&chunk| {
            let chunk = chunk_by_ukey.expect_get(&chunk);
            chunk.name.clone().unwrap_or("unnamed chunk(s)".to_owned())
          })
          .collect::<Vec<_>>();
        format!("Module {} is not in the same chunk(s) (expected in chunk(s) {}, module is in chunk(s) {})",module_readable_identifier,missing_chunks_list.join(", "),chunks.join(", "))
      };

      statistics.incorrect_chunks += 1;
      let problem = Warning::Problem(problem_string);
      failure_cache.insert(*module_id, problem.clone());
      return Some(problem);
    }

    let get_incoming_connections_by_origin_module =
      module_graph.get_incoming_connections_by_origin_module(module_id);

    let incoming_connections = get_incoming_connections_by_origin_module;

    if let Some(incoming_connections_from_non_modules) = incoming_connections.get(&None) {
      let active_non_modules_connections = incoming_connections_from_non_modules
        .iter()
        .filter(|&connection| connection.is_active(compilation.get_module_graph(), runtime))
        .collect::<Vec<_>>();

      // TODO: ADD module connection explanations
      if !active_non_modules_connections.is_empty() {
        let problem = {
          // let importing_explanations: HashSet<_> = active_non_modules_connections
          //   .iter()
          //   .flat_map(|&c| c.explanation.as_ref())
          //   .cloned()
          //   .collect();
          // let mut explanations: Vec<_> = importing_explanations.into_iter().collect();
          // explanations.sort();
          format!(
            "Module {} is referenced",
            module_readable_identifier,
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
    for (origin_module, connections) in incoming_connections.iter() {
      if let Some(origin_module) = origin_module {
        if chunk_graph.get_number_of_module_chunks(*origin_module) == 0 {
          // Ignore connection from orphan modules
          continue;
        }

        let mut origin_runtime = RuntimeSpec::default();
        for r in chunk_graph
          .get_module_runtimes(*origin_module, chunk_by_ukey)
          .values()
        {
          origin_runtime = merge_runtime(&origin_runtime, r);
        }

        let is_intersect = if let Some(runtime) = runtime {
          runtime.intersection(&origin_runtime).count() > 0
        } else {
          false
        };
        if !is_intersect {
          continue;
        }

        let active_connections: Vec<_> = connections
          .iter()
          .filter(|&connection| connection.is_active(compilation.get_module_graph(), runtime))
          .cloned()
          .collect();

        if !active_connections.is_empty() {
          incoming_connections_from_modules.insert(origin_module, active_connections);
        }
      }
    }
    //
    let mut incoming_modules: Vec<_> = incoming_connections_from_modules.keys().cloned().collect();

    let other_chunk_modules: Vec<_> = incoming_modules
      .iter()
      .filter(|&origin_module| {
        !chunk_graph
          .get_module_chunks(config.root_module)
          .iter()
          .all(|&chunk_ukey| chunk_graph.is_module_in_chunk(origin_module, chunk_ukey))
      })
      .cloned()
      .collect();

    if !other_chunk_modules.is_empty() {
      let problem = {
        let mut names: Vec<_> = other_chunk_modules
          .iter()
          .map(|&mid| {
            let m = compilation
              .get_module_graph()
              .module_by_identifier(mid)
              .expect("should have module");
            m.readable_identifier(&compilation.options.context)
              .to_string()
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

    let mut non_harmony_connections = HashMap::default();
    for (origin_module, connections) in incoming_connections_from_modules.iter() {
      let selected: Vec<_> = connections
        .iter()
        .filter(|&connection| {
          if let Some(dep) = compilation
            .get_module_graph()
            .dependency_by_id(&connection.dependency_id)
          {
            !is_harmony_dep_like(dep)
          } else {
            false
          }
        })
        .cloned()
        .collect();

      if !selected.is_empty() {
        non_harmony_connections.insert(origin_module, connections);
      }
    }

    if !non_harmony_connections.is_empty() {
      let problem = {
        let names: Vec<_> = non_harmony_connections
          .iter()
          .map(|(origin_module, connections)| {
            let module = compilation
              .get_module_graph()
              .module_by_identifier(origin_module)
              .expect("should have module");
            let readable_identifier = module.readable_identifier(&compilation.options.context);
            let mut names = connections
              .iter()
              .filter_map(|item| {
                let dep = compilation
                  .get_module_graph()
                  .dependency_by_id(&item.dependency_id)?;
                Some(dep.dependency_type().to_string())
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
            connection.is_target_active(compilation.get_module_graph(), runtime)
          });

          if runtime_condition == RuntimeCondition::Boolean(false) {
            continue;
          }

          if runtime_condition == RuntimeCondition::Boolean(true) {
            continue 'outer;
          }

          // here two runtime_condition must be `RuntimeCondition::Spec`
          if current_runtime_condition != RuntimeCondition::Boolean(false) {
            current_runtime_condition = RuntimeCondition::Spec(merge_runtime(
              current_runtime_condition.as_spec().expect("should be spec"),
              runtime_condition.as_spec().expect("should be spec"),
            ));
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
                let module = compilation
                  .get_module_graph()
                  .module_by_identifier(origin_module)
                  .expect("should have module");
                let readable_identifier = module.readable_identifier(&compilation.options.context);
                format!(
                  "{} (expected runtime {}, module is only referenced in {})",
                  readable_identifier,
                  runtime_to_string(runtime),
                  runtime_to_string(runtime_condition.as_spec().expect("should be spec"))
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
    let backup = if avoid_mutate_on_failure {
      Some(config.snapshot())
    } else {
      None
    };

    config.add(*module_id);

    incoming_modules.sort();

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
        avoid_mutate_on_failure,
        statistics,
      ) {
        if let Some(backup) = &backup {
          config.rollback(*backup);
        }
        statistics.importer_failed += 1;
        failure_cache.insert(*module_id, problem.clone());
        return Some(problem);
      }
    }
    for imp in Self::get_imports(compilation.get_module_graph(), *module_id, runtime) {
      candidates.insert(imp);
    }
    statistics.added += 1;
    None
  }
}

#[async_trait::async_trait]
impl Plugin for ModuleConcatenationPlugin {
  async fn optimize_chunk_modules(&self, args: OptimizeChunksArgs<'_>) -> Result<()> {
    let OptimizeChunksArgs { compilation } = args;
    let logger = compilation.get_logger("rspack.ModuleConcatenationPlugin");
    let mut relevant_modules = vec![];
    let mut possible_inners = HashSet::default();
    let start = logger.time("select relevant modules");
    let module_id_list = compilation
      .get_module_graph()
      .module_graph_modules()
      .keys()
      .copied()
      .collect::<Vec<_>>();
    for module_id in module_id_list {
      let mut can_be_root = true;
      let mut can_be_inner = true;
      let m = compilation
        .get_module_graph()
        .module_by_identifier(&module_id);
      // If the result is `None`, that means we have some differences with webpack,
      // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ModuleConcatenationPlugin.js#L168-L171
      if compilation
        .get_module_graph()
        .is_async(&module_id)
        .expect("should have async result")
      {
        self.set_bailout_reason(
          &module_id,
          "Module is async".to_string(),
          compilation.get_module_graph_mut(),
        );
        continue;
      }

      if !m
        .and_then(|m| m.build_info())
        .expect("should have build info")
        .strict
      {
        self.set_bailout_reason(
          &module_id,
          "Module is not in strict mode".to_string(),
          compilation.get_module_graph_mut(),
        );
        continue;
      }
      if compilation
        .chunk_graph
        .get_number_of_module_chunks(module_id)
        == 0
      {
        self.set_bailout_reason(
          &module_id,
          "Module is not in any chunk".to_string(),
          compilation.get_module_graph_mut(),
        );
        continue;
      }
      let exports_info = compilation.get_module_graph().get_exports_info(&module_id);
      let relevnat_epxorts =
        exports_info.get_relevant_exports(None, compilation.get_module_graph());
      let unknown_exports = relevnat_epxorts
        .iter()
        .filter(|id| {
          let export_info = id
            .get_export_info_mut(compilation.get_module_graph_mut())
            .clone();
          MutexModuleGraph::new(compilation.get_module_graph_mut()).with_lock(|mut mga| {
            export_info.is_reexport() && export_info.id.get_target(&mut mga, None).is_none()
          })
        })
        .copied()
        .collect::<Vec<_>>();
      if !unknown_exports.is_empty() {
        let bailout_reason = unknown_exports
          .into_iter()
          .map(|id| {
            let export_info = id.get_export_info(compilation.get_module_graph());
            let name = export_info
              .name
              .as_ref()
              .map(|name| name.to_string())
              .unwrap_or("other exports".to_string());
            format!(
              "{} : {}",
              name,
              export_info.id.get_used_info(compilation.get_module_graph())
            )
          })
          .collect::<Vec<String>>()
          .join(", ");
        self.set_bailout_reason(
          &module_id,
          format!("Reexports in this module do not have a static target ({bailout_reason})"),
          compilation.get_module_graph_mut(),
        );

        continue;
      }
      let unknown_provided_exports = relevnat_epxorts
        .iter()
        .filter(|id| {
          let export_info = id.get_export_info(compilation.get_module_graph());
          !matches!(export_info.provided, Some(ExportInfoProvided::True))
        })
        .copied()
        .collect::<Vec<_>>();

      if !unknown_provided_exports.is_empty() {
        let bailout_reason = unknown_provided_exports
          .into_iter()
          .map(|id| {
            let export_info = id.get_export_info(compilation.get_module_graph());
            let name = export_info
              .name
              .as_ref()
              .map(|name| name.to_string())
              .unwrap_or("other exports".to_string());
            format!(
              "{} : {} and {}",
              name,
              export_info
                .id
                .get_provided_info(compilation.get_module_graph()),
              export_info.id.get_used_info(compilation.get_module_graph())
            )
          })
          .collect::<Vec<String>>()
          .join(", ");
        self.set_bailout_reason(
          &module_id,
          format!("List of module exports is dynamic ({bailout_reason})"),
          compilation.get_module_graph_mut(),
        );
        can_be_root = false;
      }

      if compilation.chunk_graph.is_entry_module(&module_id) {
        self.set_bailout_reason(
          &module_id,
          "Module is an entry point".to_string(),
          compilation.get_module_graph_mut(),
        );
        can_be_inner = false;
      }
      if can_be_root {
        relevant_modules.push(module_id);
      }
      if can_be_inner {
        possible_inners.insert(module_id);
      }
    }

    logger.time_end(start);
    logger.debug(format!(
      "{} potential root modules, {} potential inner modules",
      relevant_modules.len(),
      possible_inners.len(),
    ));

    let start = logger.time("sort relevant modules");
    relevant_modules.sort_by(|a, b| {
      let ad = compilation.get_module_graph().get_depth(a);
      let bd = compilation.get_module_graph().get_depth(b);
      ad.cmp(&bd)
    });

    logger.time_end(start);
    let mut statistics = Statistics::default();
    let mut stats_candidates = 0;
    let mut stats_size_sum = 0;
    let mut stats_empty_configurations = 0;

    let start = logger.time("find modules to concatenate");
    let mut concat_configurations: Vec<ConcatConfiguration> = Vec::new();
    let mut used_as_inner: HashSet<ModuleIdentifier> = HashSet::default();

    for current_root in relevant_modules.iter() {
      if used_as_inner.contains(current_root) {
        continue;
      }
      let mut chunk_runtime = HashSet::default();
      for r in compilation
        .chunk_graph
        .get_module_runtimes(*current_root, &compilation.chunk_by_ukey)
        .into_values()
      {
        chunk_runtime = merge_runtime(&chunk_runtime, &r);
      }
      let exports_info_id = compilation
        .get_module_graph()
        .get_exports_info(current_root)
        .id;
      let filtered_runtime = filter_runtime(Some(&chunk_runtime), |r| {
        exports_info_id.is_module_used(compilation.get_module_graph(), r)
      });
      let active_runtime = match filtered_runtime {
        RuntimeCondition::Boolean(true) => Some(chunk_runtime.clone()),
        RuntimeCondition::Boolean(false) => None,
        RuntimeCondition::Spec(spec) => Some(spec),
      };

      let mut current_configuration =
        ConcatConfiguration::new(*current_root, active_runtime.clone());

      let mut failure_cache = HashMap::default();
      let mut candidates_visited = HashSet::default();
      let mut candidates = VecDeque::new();

      let imports = Self::get_imports(
        compilation.get_module_graph(),
        *current_root,
        active_runtime.as_ref(),
      );
      for import in imports {
        candidates.push_back(import);
      }

      while let Some(imp) = candidates.pop_front() {
        if candidates_visited.contains(&imp) {
          continue;
        } else {
          candidates_visited.insert(imp);
        }
        let mut import_candidates = HashSet::default();
        match Self::try_to_add(
          compilation,
          &mut current_configuration,
          &imp,
          Some(&chunk_runtime),
          active_runtime.as_ref(),
          &possible_inners,
          &mut import_candidates,
          &mut failure_cache,
          true,
          &mut statistics,
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
        let optimization_bailouts = compilation
          .get_module_graph_mut()
          .get_optimization_bailout_mut(*current_root);
        for warning in current_configuration.get_warnings_sorted() {
          optimization_bailouts.push(self.format_bailout_warning(warning.0, &warning.1));
        }
      }
    }
    logger.time_end(start);
    if !concat_configurations.is_empty() {
      logger.debug(format!(
        "{} successful concat configurations (avg size: {}), {} bailed out completely",
        concat_configurations.len(),
        stats_size_sum / concat_configurations.len(),
        stats_empty_configurations
      ));
    }

    logger.debug(format!(
        "{} candidates were considered for adding ({} cached failure, {} already in config, {} invalid module, {} incorrect chunks, {} incorrect dependency, {} incorrect chunks of importer, {} incorrect module dependency, {} incorrect runtime condition, {} importer failed, {} added)",
        stats_candidates,
        statistics.cached,
        statistics.already_in_config,
        statistics.invalid_module,
        statistics.incorrect_chunks,
        statistics.incorrect_dependency,
        statistics.incorrect_chunks_of_importer,
        statistics.incorrect_module_dependency,
        statistics.incorrect_runtime_condition,
        statistics.importer_failed,
        statistics.added
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

    for config in concat_configurations {
      let root_module_id = config.root_module;
      if used_modules.contains(&root_module_id) {
        continue;
      }
      let modules_set = config.get_modules();
      for m in modules_set {
        used_modules.insert(*m);
      }
      let box_module = compilation
        .get_module_graph()
        .module_by_identifier(&root_module_id)
        .expect("should have module");
      let root_module_ctxt = RootModuleContext {
        id: root_module_id,
        readable_identifier: box_module
          .readable_identifier(&compilation.options.context)
          .to_string(),
        name_for_condition: box_module.name_for_condition().clone(),
        lib_indent: box_module
          .lib_ident(LibIdentOptions {
            context: compilation.options.context.as_str(),
          })
          .map(|id| id.to_string()),
        resolve_options: box_module.get_resolve_options().clone(),
        code_generation_dependencies: box_module
          .get_code_generation_dependencies()
          .map(|deps| deps.to_vec()),
        presentational_dependencies: box_module
          .get_presentational_dependencies()
          .map(|deps| deps.to_vec()),
        context: Some(compilation.options.context.clone()),
        side_effect_connection_state: box_module.get_side_effects_connection_state(
          compilation.get_module_graph(),
          &mut HashSet::default(),
        ),
        build_meta: box_module.build_meta().cloned(),
      };
      let modules = modules_set
        .iter()
        .map(|id| {
          let module = compilation
            .get_module_graph()
            .module_by_identifier(id)
            .unwrap_or_else(|| panic!("should have module {}", id));
          let inner_module = ConcatenatedInnerModule {
            id: *id,
            size: module.size(&rspack_core::SourceType::JavaScript),
            original_source_hash: module.original_source().map(|source| {
              let mut hasher = DefaultHasher::default();
              source.dyn_hash(&mut hasher);
              hasher.finish()
            }),
            shorten_id: module
              .readable_identifier(&compilation.options.context)
              .to_string(),
          };
          inner_module
        })
        .collect::<Vec<_>>();
      let mut new_module = ConcatenatedModule::create(
        root_module_ctxt,
        modules,
        Some(rspack_hash::HashFunction::MD4),
        config.runtime.clone(),
      );
      let build_result = new_module
        .build(
          rspack_core::BuildContext {
            compiler_context: CompilerContext {
              options: compilation.options.clone(),
              resolver_factory: compilation.resolver_factory.clone(),
              module: new_module.id(),
              module_context: None,
              module_source_map_kind: rspack_util::source_map::SourceMapKind::None,
              cache: compilation.cache.clone(),
              factorize_queue: compilation.factorize_queue.clone(),
              build_queue: compilation.build_queue.clone(),
              add_queue: compilation.add_queue.clone(),
              process_dependencies_queue: compilation.process_dependencies_queue.clone(),
              build_time_execution_queue: compilation.build_time_execution_queue.clone(),
              plugin_driver: compilation.plugin_driver.clone(),
            },
            plugin_driver: compilation.plugin_driver.clone(),
            compiler_options: &compilation.options,
          },
          Some(compilation),
        )
        .await?;
      new_module.set_module_build_info_and_meta(build_result.build_info, build_result.build_meta);
      let root_mgm_epxorts = compilation
        .get_module_graph()
        .module_graph_module_by_identifier(&root_module_id)
        .expect("should have mgm")
        .exports;
      let module_graph_module =
        ModuleGraphModule::new(new_module.id(), *new_module.module_type(), root_mgm_epxorts);
      compilation
        .get_module_graph_mut()
        .add_module_graph_module(module_graph_module);
      compilation
        .get_module_graph_mut()
        .clone_module_attributes(&root_module_id, &new_module.id());
      // integrate
      let mut chunk_graph = std::mem::take(&mut compilation.chunk_graph);
      for m in modules_set {
        if m == &root_module_id {
          continue;
        }
        compilation
          .get_module_graph_mut()
          .copy_outgoing_module_connections(m, &new_module.id(), |c, mg| {
            let dep = c.dependency_id.get_dependency(mg);
            c.original_module_identifier.as_ref() == Some(m)
              && !(is_harmony_dep_like(dep) && modules_set.contains(&c.module_identifier))
          });
        // TODO: optimize asset module https://github.com/webpack/webpack/pull/15515/files
        for chunk_ukey in chunk_graph.get_module_chunks(root_module_id).clone() {
          let module = compilation
            .get_module_graph_mut()
            .module_by_identifier_mut(m)
            .expect("should exist module");

          let source_types = chunk_graph.get_chunk_module_source_types(&chunk_ukey, module);

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
      compilation.chunk_graph = chunk_graph;
      // compilation
      //   .get_module_graph()
      //   .module_identifier_to_module
      //   .remove(&root_module_id);
      // compilation.chunk_graph.clear

      compilation
        .chunk_graph
        .replace_module(&root_module_id, &new_module.id());

      compilation.get_module_graph_mut().move_module_connections(
        &root_module_id,
        &new_module.id(),
        |c, mg| {
          let other_module = if c.module_identifier == root_module_id {
            c.original_module_identifier
          } else {
            Some(c.module_identifier)
          };
          let dep = c.dependency_id.get_dependency(mg);
          let inner_connection = is_harmony_dep_like(dep)
            && if let Some(other_module) = other_module {
              modules_set.contains(&other_module)
            } else {
              false
            };
          !inner_connection
        },
      );
      compilation
        .get_module_graph_mut()
        .add_module(new_module.boxed());
    }
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
