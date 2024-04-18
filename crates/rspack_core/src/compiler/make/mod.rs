mod rebuild_deps_builder;
mod tasks;

use std::path::PathBuf;

use rayon::prelude::*;
use rspack_error::Result;
use rspack_identifier::Identifier;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

pub use self::rebuild_deps_builder::RebuildDepsBuilder;
use self::tasks::{clean::CleanTask, factorize::FactorizeTask, MakeTaskContext};
use crate::{
  tree_shaking::BailoutFlag,
  utils::task_loop::{run_task_loop, Task},
  AsyncDependenciesBlockIdentifier, BuildDependency, Compilation, Context, DependencyId,
  DependencyType, GroupOptions, Module, ModuleGraphPartial, ModuleIdentifier, ModuleIssuer,
  ModuleProfile, NormalModuleSource, Resolve,
};

#[derive(Debug)]
pub enum MakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  DeletedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

impl MakeParam {
  pub fn new_force_build_dep_param(dep: DependencyId, module: Option<ModuleIdentifier>) -> Self {
    let mut data = HashSet::default();
    data.insert((dep, module));
    Self::ForceBuildDeps(data)
  }
}

pub async fn update_module_graph(
  compilation: &mut Compilation,
  params: Vec<MakeParam>,
) -> Result<()> {
  let mut builder = UpdateModuleGraph::default();
  let build_dependencies = builder.cutout(compilation, params)?;
  builder.repair(compilation, build_dependencies)
}

type ModuleDeps = (
  Vec<Identifier>,
  Vec<(AsyncDependenciesBlockIdentifier, Option<GroupOptions>)>,
);

#[derive(Default)]
struct UpdateModuleGraph {
  origin_module_deps: HashMap<Identifier, ModuleDeps>,
  /// Rebuild module issuer mappings
  origin_module_issuers: HashMap<Identifier, ModuleIssuer>,

  need_check_isolated_module_ids: HashSet<Identifier>,
}

impl UpdateModuleGraph {
  fn cutout(
    &mut self,
    compilation: &mut Compilation,
    params: Vec<MakeParam>,
  ) -> Result<HashSet<BuildDependency>> {
    let deps_builder = RebuildDepsBuilder::new(params, &compilation.get_module_graph());

    self.origin_module_deps = HashMap::from_iter(
      deps_builder
        .get_force_build_modules()
        .iter()
        .map(|module_identifier| {
          (
            *module_identifier,
            Self::module_deps(compilation, module_identifier),
          )
        }),
    );

    let module_graph = compilation.get_module_graph();
    // calc need_check_isolated_module_ids & regen_module_issues
    for id in deps_builder.get_force_build_modules() {
      if let Some(mgm) = compilation
        .get_module_graph()
        .module_graph_module_by_identifier(id)
      {
        let depended_modules = module_graph
          .get_module_all_depended_modules(id)
          .expect("module graph module not exist")
          .into_iter()
          .copied();
        self.need_check_isolated_module_ids.extend(depended_modules);
        self
          .origin_module_issuers
          .insert(*id, mgm.get_issuer().clone());
      }
    }

    Ok(deps_builder.revoke_modules(&mut compilation.get_module_graph_mut()))
  }

  fn repair(
    &mut self,
    compilation: &mut Compilation,
    build_dependencies: HashSet<BuildDependency>,
  ) -> Result<()> {
    let module_graph = compilation.get_module_graph();
    let init_tasks = build_dependencies
      .into_iter()
      .filter_map(|(id, parent_module_identifier)| {
        let dependency = module_graph
          .dependency_by_id(&id)
          .expect("dependency not found");
        if dependency.as_module_dependency().is_none()
          && dependency.as_context_dependency().is_none()
        {
          return None;
        }

        let parent_module =
          parent_module_identifier.and_then(|id| module_graph.module_by_identifier(&id));
        if parent_module_identifier.is_some() && parent_module.is_none() {
          return None;
        }
        Some(
          self.handle_module_creation(
            compilation,
            parent_module_identifier,
            parent_module.and_then(|m| m.get_context()),
            vec![id],
            parent_module_identifier.is_none(),
            parent_module.and_then(|module| module.get_resolve_options()),
            parent_module
              .and_then(|m| m.as_normal_module())
              .and_then(|module| module.name_for_condition()),
          ),
        )
      })
      .collect::<Vec<_>>();

    let mut make_module_graph = ModuleGraphPartial::new(true);
    compilation.swap_make_module_graph(&mut make_module_graph);
    let mut ctx = MakeTaskContext::new(compilation, make_module_graph);
    let res = run_task_loop(&mut ctx, init_tasks);

    tracing::debug!("All task is finished");

    // clean isolated module
    let mut clean_tasks: Vec<Box<dyn Task<MakeTaskContext>>> =
      Vec::with_capacity(self.need_check_isolated_module_ids.len());
    for module_identifier in &self.need_check_isolated_module_ids {
      clean_tasks.push(Box::new(CleanTask {
        module_identifier: *module_identifier,
      }));
    }
    run_task_loop(&mut ctx, clean_tasks)?;

    ctx.emit_data_to_compilation(compilation);

    tracing::debug!("All clean task is finished");
    // set origin module issues
    for (id, issuer) in self.origin_module_issuers.drain() {
      if let Some(mgm) = compilation
        .get_module_graph_mut()
        .module_graph_module_by_identifier_mut(&id)
      {
        mgm.set_issuer(issuer);
      }
    }

    // calc has_module_import_export_change
    compilation.has_module_import_export_change = if self.origin_module_deps.is_empty() {
      true
    } else {
      compilation.has_module_import_export_change
        || !self.origin_module_deps.drain().all(|(module_id, deps)| {
          if compilation
            .get_module_graph_mut()
            .module_by_identifier(&module_id)
            .is_none()
          {
            false
          } else {
            let (now_deps, mut now_blocks) = Self::module_deps(compilation, &module_id);
            let (origin_deps, mut origin_blocks) = deps;
            if now_deps.len() != origin_deps.len() || now_blocks.len() != origin_blocks.len() {
              false
            } else {
              for index in 0..origin_deps.len() {
                if origin_deps[index] != now_deps[index] {
                  return false;
                }
              }

              now_blocks.sort_unstable();
              origin_blocks.sort_unstable();

              for index in 0..origin_blocks.len() {
                if origin_blocks[index].0 != now_blocks[index].0 {
                  return false;
                }
                if origin_blocks[index].1 != now_blocks[index].1 {
                  return false;
                }
              }

              true
            }
          }
        })
    };

    // Avoid to introduce too much overhead,
    // until we find a better way to align with webpack hmr behavior

    // add context module and context element module to bailout_module_identifiers
    if compilation.options.builtins.tree_shaking.enable() {
      compilation.bailout_module_identifiers = compilation
        .get_module_graph()
        .dependencies()
        .values()
        .par_bridge()
        .filter_map(|dep| {
          if dep.as_context_dependency().is_some()
            && let Some(module) = compilation
              .get_module_graph()
              .get_module_by_dependency_id(dep.id())
          {
            let mut values = vec![(module.identifier(), BailoutFlag::CONTEXT_MODULE)];
            if let Some(dependencies) = compilation
              .get_module_graph()
              .get_module_all_dependencies(&module.identifier())
            {
              for dependency in dependencies {
                if let Some(dependency_module) = compilation
                  .get_module_graph()
                  .module_identifier_by_dependency_id(dependency)
                {
                  values.push((*dependency_module, BailoutFlag::CONTEXT_MODULE));
                }
              }
            }

            Some(values)
          } else if matches!(
            dep.dependency_type(),
            DependencyType::ContainerExposed | DependencyType::ProvideModuleForShared
          ) && let Some(module) = compilation
            .get_module_graph()
            .get_module_by_dependency_id(dep.id())
          {
            Some(vec![(module.identifier(), BailoutFlag::CONTAINER_EXPOSED)])
          } else {
            None
          }
        })
        .flatten()
        .collect();
    }

    res
  }

  #[allow(clippy::too_many_arguments)]
  fn handle_module_creation(
    &mut self,
    compilation: &Compilation,
    original_module_identifier: Option<ModuleIdentifier>,
    original_module_context: Option<Box<Context>>,
    dependencies: Vec<DependencyId>,
    is_entry: bool,
    resolve_options: Option<Box<Resolve>>,
    issuer: Option<Box<str>>,
  ) -> Box<dyn Task<MakeTaskContext>> {
    let current_profile = compilation
      .options
      .profile
      .then(Box::<ModuleProfile>::default);
    let dependency = compilation
      .get_module_graph()
      .dependency_by_id(&dependencies[0])
      .expect("should have dependency")
      .clone();
    let module_graph = compilation.get_module_graph();
    let original_module_source = original_module_identifier
      .and_then(|i| module_graph.module_by_identifier(&i))
      .and_then(|m| m.as_normal_module())
      .and_then(|m| {
        if let NormalModuleSource::BuiltSucceed(s) = m.source() {
          Some(s.clone())
        } else {
          None
        }
      });
    Box::new(FactorizeTask {
      module_factory: compilation.get_dependency_factory(&dependency),
      original_module_identifier,
      original_module_source,
      issuer,
      original_module_context,
      dependency,
      dependencies,
      is_entry,
      resolve_options,
      resolver_factory: compilation.resolver_factory.clone(),
      loader_resolver_factory: compilation.loader_resolver_factory.clone(),
      options: compilation.options.clone(),
      plugin_driver: compilation.plugin_driver.clone(),
      cache: compilation.cache.clone(),
      current_profile,
    })
  }

  fn module_deps(compilation: &Compilation, module_identifier: &ModuleIdentifier) -> ModuleDeps {
    let module_graph = compilation.get_module_graph();
    let (deps, blocks) = module_graph.get_module_dependencies_modules_and_blocks(module_identifier);

    let blocks_with_option: Vec<_> = blocks
      .iter()
      .map(|block| {
        (
          *block,
          compilation
            .get_module_graph()
            .block_by_id(block)
            .expect("block muse be exist")
            .get_group_options()
            .cloned(),
        )
      })
      .collect();
    (deps, blocks_with_option)
  }
}
