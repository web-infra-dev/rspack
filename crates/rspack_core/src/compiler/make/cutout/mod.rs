mod clean_isolated_module;
mod fix_issuers;
mod has_module_graph_change;

use rustc_hash::FxHashSet as HashSet;

use self::{
  clean_isolated_module::CleanIsolatedModule, fix_issuers::FixIssuers,
  has_module_graph_change::HasModuleGraphChange,
};
use super::{MakeArtifact, MakeParam};
use crate::BuildDependency;

#[derive(Debug, Default)]
pub struct Cutout {
  fix_issuers: FixIssuers,
  clean_isolated_module: CleanIsolatedModule,
  has_module_graph_change: HasModuleGraphChange,
}

impl Cutout {
  pub fn cutout_artifact(
    &mut self,
    artifact: &mut MakeArtifact,
    params: Vec<MakeParam>,
  ) -> HashSet<BuildDependency> {
    let module_graph = artifact.get_module_graph();

    let mut force_build_modules = HashSet::default();
    let mut force_build_deps = HashSet::default();

    for item in params {
      match item {
        MakeParam::ModifiedFiles(files) => {
          force_build_modules.extend(module_graph.modules().values().filter_map(|module| {
            // check has dependencies modified
            if !module.is_available(&files) {
              Some(module.identifier())
            } else {
              None
            }
          }))
        }
        MakeParam::DeletedFiles(files) => {
          force_build_modules.extend(module_graph.modules().values().flat_map(|module| {
            let mut res = vec![];

            // check has dependencies modified
            if !module.is_available(&files) {
              // add module id
              res.push(module.identifier());
              // add parent module id
              res.extend(
                module_graph
                  .get_incoming_connections(&module.identifier())
                  .iter()
                  .filter_map(|connect| connect.original_module_identifier),
              )
            }
            res
          }))
        }
        MakeParam::ForceBuildDeps(deps) => {
          for item in deps {
            let (dependency_id, _) = &item;
            // add deps bindings module to force_build_modules
            if let Some(mid) = module_graph.module_identifier_by_dependency_id(dependency_id) {
              force_build_modules.insert(*mid);
            }
            force_build_deps.insert(item);
          }
        }
        MakeParam::ForceBuildModules(modules) => {
          force_build_modules.extend(modules);
        }
      };
    }

    for module_identifier in &force_build_modules {
      self
        .fix_issuers
        .analyze_force_build_module(artifact, module_identifier);
      self
        .clean_isolated_module
        .analyze_force_build_module(artifact, module_identifier);
      self
        .has_module_graph_change
        .analyze_force_build_module(artifact, module_identifier);
    }

    let mut module_graph = artifact.get_module_graph_mut();
    // do revoke module and collect deps
    force_build_deps.extend(
      force_build_modules
        .iter()
        .flat_map(|id| module_graph.revoke_module(id)),
    );

    force_build_deps
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let Self {
      fix_issuers,
      clean_isolated_module,
      has_module_graph_change,
    } = self;
    fix_issuers.fix_artifact(artifact);
    clean_isolated_module.fix_artifact(artifact);
    has_module_graph_change.fix_artifact(artifact);
  }
}
