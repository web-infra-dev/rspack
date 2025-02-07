mod clean_isolated_module;
mod fix_build_meta;
mod fix_issuers;
mod has_module_graph_change;

use rspack_collections::{IdentifierSet, UkeySet};
use rustc_hash::FxHashSet as HashSet;

use self::{
  clean_isolated_module::CleanIsolatedModule, fix_build_meta::FixBuildMeta,
  fix_issuers::FixIssuers, has_module_graph_change::HasModuleGraphChange,
};
use super::{MakeArtifact, MakeParam};
use crate::BuildDependency;

#[derive(Debug, Default)]
pub struct Cutout {
  fix_issuers: FixIssuers,
  fix_build_meta: FixBuildMeta,
  clean_isolated_module: CleanIsolatedModule,
  has_module_graph_change: HasModuleGraphChange,
}

impl Cutout {
  pub fn cutout_artifact(
    &mut self,
    artifact: &mut MakeArtifact,
    params: Vec<MakeParam>,
  ) -> HashSet<BuildDependency> {
    let mut entry_dependencies = std::mem::take(&mut artifact.entry_dependencies);
    let mut force_build_modules = IdentifierSet::default();
    let mut force_build_deps = HashSet::default();
    let mut removed_deps = UkeySet::default();

    let module_graph = artifact.get_module_graph();

    for item in params {
      match item {
        // TODO: BuildEntry will always have one at most, remove it from params
        MakeParam::BuildEntry(deps) => {
          force_build_deps.extend(deps.difference(&entry_dependencies).map(|d| (*d, None)));
          removed_deps.extend(entry_dependencies.difference(&deps));
          entry_dependencies = deps;
        }
        MakeParam::CheckNeedBuild => {
          force_build_modules.extend(module_graph.modules().values().filter_map(|module| {
            if module.need_build() {
              Some(module.identifier())
            } else {
              None
            }
          }));
        }
        MakeParam::ModifiedFiles(files) => {
          force_build_modules.extend(module_graph.modules().values().filter_map(|module| {
            // check has dependencies modified
            if module.depends_on(&files) {
              Some(module.identifier())
            } else {
              None
            }
          }))
        }
        MakeParam::RemovedFiles(files) => {
          for module in module_graph.modules().values() {
            // check has dependencies modified
            if module.depends_on(&files) {
              // add parent module id
              force_build_modules.extend(
                module_graph
                  .get_incoming_connections(&module.identifier())
                  .filter_map(|connect| connect.original_module_identifier),
              );
              removed_deps.extend(
                module_graph
                  .get_incoming_connections(&module.identifier())
                  .map(|connect| connect.dependency_id),
              );
            }
          }
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
        .fix_build_meta
        .analyze_force_build_module(artifact, module_identifier);
      self
        .clean_isolated_module
        .analyze_force_build_module(artifact, module_identifier);
      self
        .has_module_graph_change
        .analyze_force_build_module(artifact, module_identifier);
    }

    let mut module_graph = artifact.get_module_graph_mut();
    for dep_id in removed_deps {
      // connection may have been deleted by revoke module
      if let Some(con) = module_graph.connection_by_dependency_id(&dep_id) {
        // need clean_isolated_module to check whether the module is still used by other deps
        self
          .clean_isolated_module
          .add_need_check_module(*con.module_identifier());
        module_graph.revoke_connection(&dep_id, true);
      }
    }

    // do revoke module and collect deps
    for id in force_build_modules {
      force_build_deps.extend(artifact.revoke_module(&id));
    }

    artifact.entry_dependencies = entry_dependencies;

    self.has_module_graph_change.analyze_artifact(artifact);

    force_build_deps
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let Self {
      fix_issuers,
      fix_build_meta,
      clean_isolated_module,
      has_module_graph_change,
    } = self;
    fix_issuers.fix_artifact(artifact);
    fix_build_meta.fix_artifact(artifact);
    clean_isolated_module.fix_artifact(artifact);
    has_module_graph_change.fix_artifact(artifact);
  }
}
