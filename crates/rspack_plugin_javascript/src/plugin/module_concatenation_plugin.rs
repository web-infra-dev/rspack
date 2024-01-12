use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::os::unix::prelude::OpenOptionsExt;
use std::sync::Arc;

use rspack_core::concatenated_module::{
  is_harmony_dep_like, ConcatenatedInnerModule, ConcatenatedModule, RootModuleContext,
};
use rspack_core::{
  filter_runtime, merge_runtime, BoxDependency, Compilation, CompilerContext, ExportInfoProvided,
  ExtendedReferencedExport, LibIdentOptions, Logger, Module, ModuleExt, ModuleGraph,
  ModuleGraphModule, ModuleIdentifier, OptimizeChunksArgs, Plugin, ProvidedExports,
  RuntimeCondition, RuntimeSpec, WrappedModuleIdentifier,
};
use rspack_error::Result;
use rspack_util::ext::DynHash;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::dependency::{
  HarmonyExportImportedSpecifierDependency, HarmonyImportSideEffectDependency,
  HarmonyImportSpecifierDependency,
};
use crate::inner_graph_plugin;
fn format_bailout_reason(msg: &str) -> String {
  format!("ModuleConcatenation bailout: {}", msg)
}

#[derive(Clone)]
enum Warning {
  Id(ModuleIdentifier),
  Problem(Arc<dyn Fn(String) -> String + Send + Sync + 'static>),
}

impl std::fmt::Debug for Warning {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Id(arg0) => f.debug_tuple("Id").field(arg0).finish(),
      Self::Problem(arg0) => f.write_str("Fn(String) -> String"),
    }
  }
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

#[derive(Debug)]
pub struct ModuleConcatenationPlugin;

impl ModuleConcatenationPlugin {
  pub fn get_imports(
    mg: &ModuleGraph,
    mi: WrappedModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
  ) -> HashSet<ModuleIdentifier> {
    let mut set = HashSet::default();
    let module = mg.module_by_identifier(&mi).expect("should have module");
    for d in module.get_dependencies() {
      let dep = d.get_dependency(mg);
      let is_harmony_import_like = is_harmony_dep_like(dep);
      if !is_harmony_import_like {
        continue;
      }
      let Some(con) = mg.connection_by_dependency(d) else {
        continue;
      };
      if !con.is_target_active(mg, runtime) {
        continue;
      }
      // SAFETY: because it is extends harmony dep, we can ensure the dep has been
      // implemented ModuleDepdency Trait.
      let module_dep = dep.as_module_dependency().expect("should be module dep");
      let imported_names = module_dep.get_referenced_exports(mg, None);
      if imported_names.iter().all(|item| match item {
        ExtendedReferencedExport::Array(arr) => !arr.is_empty(),
        ExtendedReferencedExport::Export(export) => !export.name.is_empty(),
      }) || matches!(mg.get_provided_exports(*mi), ProvidedExports::Vec(_))
      {
        set.insert(con.module_identifier);
      }
    }
    set
  }

  pub fn try_to_add() -> Option<Warning> {
    todo!()
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
      .module_graph
      .module_graph_modules()
      .keys()
      .copied()
      .map(WrappedModuleIdentifier::from)
      .collect::<Vec<_>>();
    let mg = &compilation.module_graph;
    let Compilation {
      module_graph: mg,
      chunk_graph,
      ..
    } = compilation;
    for module_id in module_id_list {
      let mut can_be_root = true;
      let mut can_be_inner = true;
      let m = module_id.module(mg);
      // If the result is `None`, that means we have some differences with webpack,
      // https://github.com/webpack/webpack/blob/1f99ad6367f2b8a6ef17cce0e058f7a67fb7db18/lib/optimize/ModuleConcatenationPlugin.js#L168-L171
      if mg.is_async(&*module_id).expect("should have async result") {
        // TODO: bailout
        continue;
      }
      if !m.build_info().expect("should have build info").strict {
        // TODO: bailout
        continue;
      }
      if chunk_graph.get_number_of_module_chunks(*module_id) == 0 {
        // TODO: bailout
        continue;
      }
      let exports_info = mg.get_exports_info(&*module_id);
      let relevnat_epxorts = exports_info.get_relevant_exports(None, mg);
      let unknown_exports = relevnat_epxorts
        .iter()
        .filter(|id| {
          let export_info = id.get_export_info_mut(mg).clone();
          export_info.is_reexport() && export_info.id.get_target(mg, None).is_none()
        })
        .copied()
        .collect::<Vec<_>>();
      if unknown_exports.len() > 0 {
        // TODO: bailout
        continue;
      }
      let unknown_provided_exports = relevnat_epxorts
        .iter()
        .filter(|id| {
          let export_info = id.get_export_info(mg);
          !matches!(export_info.provided, Some(ExportInfoProvided::True))
        })
        .copied()
        .collect::<Vec<_>>();

      if unknown_provided_exports.len() > 0 {
        // TODO: bailout
        can_be_root = false;
      }

      if chunk_graph.is_entry_module(&*module_id) {
        // TODO: bailout
        can_be_inner = false;
      }
      if can_be_root {
        relevant_modules.push(*module_id);
      }
      if can_be_inner {
        possible_inners.insert(*module_id);
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
      let ad = mg.get_depth(a);
      let bd = mg.get_depth(b);
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
      for r in chunk_graph
        .get_module_runtimes(*current_root, &compilation.chunk_by_ukey)
        .into_values()
      {
        chunk_runtime = merge_runtime(&chunk_runtime, &r);
      }
      let exports_info_id = mg.get_exports_info(current_root).id;
      let filtered_runtime = filter_runtime(Some(&chunk_runtime), |r| {
        exports_info_id.is_module_used(mg, r)
      });
      let active_runtime = match filtered_runtime {
        RuntimeCondition::Boolean(true) => Some(chunk_runtime),
        RuntimeCondition::Boolean(false) => None,
        RuntimeCondition::Spec(spec) => Some(spec),
      };

      let mut current_configuration =
        ConcatConfiguration::new(*current_root, active_runtime.clone());

      let mut failure_cache = HashMap::default();
      let mut candidates = HashSet::default();

      let imports = Self::get_imports(mg, (*current_root).into(), active_runtime.as_ref());
      for import in imports {
        candidates.insert(import);
      }

      let mut imports_to_extends = vec![];
      for imp in candidates.iter() {
        let import_candidates = HashSet::default();
        if let Some(problem) = Self::try_to_add() {
          failure_cache.insert(*imp, problem.clone());
          current_configuration.add_warning(*imp, problem);
        } else {
          import_candidates.iter().for_each(|c: &ModuleIdentifier| {
            imports_to_extends.push(*c);
          });
        }
      }
      candidates.extend(imports_to_extends);
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
        // TODO: bailout
      }
    }
    logger.time_end(start);
    logger.debug(format!(
      "{} successful concat configurations (avg size: {}), {} bailed out completely",
      concat_configurations.len(),
      stats_size_sum / concat_configurations.len(),
      stats_empty_configurations
    ));

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
    // TODO: Allow reusing existing configuration while trying to add dependencies.
    // This would improve performance. O(n^2) -> O(n)
    let start = logger.time("sort concat configurations");
    concat_configurations.sort_by(|a, b| b.modules.len().cmp(&a.modules.len()));
    logger.time_end(start);

    let start = logger.time("create concatenated modules");
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
        .module_graph
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
          .map(|deps| deps.into_iter().cloned().collect::<Vec<_>>()),
        presentational_dependencies: box_module
          .get_presentational_dependencies()
          .map(|deps| deps.into_iter().cloned().collect::<Vec<_>>()),
        context: Some(compilation.options.context.clone()),
        side_effect_connection_state: box_module
          .get_side_effects_connection_state(&compilation.module_graph, &mut HashSet::default()),
        build_meta: box_module.build_meta().cloned(),
      };
      let modules = modules_set
        .iter()
        .map(|id| {
          let module = compilation
            .module_graph
            .module_by_identifier(id)
            .expect("should have module");
          ConcatenatedInnerModule {
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
          }
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
            },
            plugin_driver: compilation.plugin_driver.clone(),
            compiler_options: &compilation.options,
          },
          Some(&compilation),
        )
        .await?;
      let root_mgm_epxorts = compilation
        .module_graph
        .module_graph_module_by_identifier(&root_module_id)
        .expect("should have mgm")
        .exports;
      let module_graph_module =
        ModuleGraphModule::new(new_module.id(), *new_module.module_type(), root_mgm_epxorts);
      compilation
        .module_graph
        .add_module_graph_module(module_graph_module);
      compilation
        .module_graph
        .clone_module_attributes(&root_module_id, &new_module.id());
      // integrate
      for m in modules_set {
        if m == &root_module_id {
          continue;
        }
        compilation
          .module_graph
          .copy_outgoing_module_connections(m, &new_module.id(), |c, mg| {
            let dep = c.dependency_id.get_dependency(mg);
            c.original_module_identifier.as_ref() == Some(m)
              && !(is_harmony_dep_like(dep) && modules_set.contains(&c.module_identifier))
          });
        // TODO: optimize asset module https://github.com/webpack/webpack/pull/15515/files
        for chunk_ukey in compilation
          .chunk_graph
          .get_module_chunks(root_module_id)
          .clone()
        {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(&chunk_ukey, *m);
        }
      }
      compilation
        .module_graph
        .module_identifier_to_module
        .remove(&root_module_id);
      // compilation.chunk_graph.clear

      compilation
        .chunk_graph
        .replace_module(&root_module_id, &new_module.id());
      compilation.module_graph.move_module_connections(
        &root_module_id,
        &new_module.id(),
        |c, mg| {
          let other_module = if c.module_identifier == root_module_id {
            c.original_module_identifier
              .expect("should have original_module_identifier")
          } else {
            c.module_identifier
          };
          let dep = c.dependency_id.get_dependency(mg);
          let inner_connection = is_harmony_dep_like(dep) && modules_set.contains(&other_module);
          !inner_connection
        },
      );
      compilation.module_graph.add_module(new_module.boxed());
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
