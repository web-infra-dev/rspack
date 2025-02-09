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
    let mut next_entry_dependencies = HashSet::default();
    let mut clean_entry_dependencies = false;
    let mut useless_entry_dependencies = UkeySet::default();
    let mut force_build_modules = IdentifierSet::default();
    let mut force_build_deps = HashSet::default();

    let module_graph = artifact.get_module_graph();

    for item in params {
      match item {
        MakeParam::BuildEntry(deps) => {
          next_entry_dependencies = deps;
        }
        MakeParam::BuildEntryAndClean(deps) => {
          next_entry_dependencies = deps;
          clean_entry_dependencies = true;
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
          for module in module_graph.modules().values() {
            // check has dependencies modified
            if module.depends_on(&files) {
              // add module id
              force_build_modules.insert(module.identifier());
            }
          }
        }
        MakeParam::RemovedFiles(files) => {
          for module in module_graph.modules().values() {
            // check has dependencies modified
            if module.depends_on(&files) {
              // add module id
              force_build_modules.insert(module.identifier());
              // process parent module id
              for connect in module_graph.get_incoming_connections(&module.identifier()) {
                if let Some(original_module_identifier) = connect.original_module_identifier {
                  force_build_modules.insert(original_module_identifier);
                } else {
                  useless_entry_dependencies.insert(connect.dependency_id);
                }
              }
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

    // do revoke module and collect deps
    for id in force_build_modules {
      force_build_deps.extend(artifact.revoke_module(&id));
    }

    let mut entry_dependencies = std::mem::take(&mut artifact.entry_dependencies);
    let mut module_graph = artifact.get_module_graph_mut();
    // remove useless entry dependencies
    let mut remove_entry_dependencies = HashSet::default();
    for dep_id in useless_entry_dependencies {
      if !next_entry_dependencies.contains(&dep_id) {
        remove_entry_dependencies.insert(dep_id);
      }
    }
    if clean_entry_dependencies {
      remove_entry_dependencies.extend(entry_dependencies.difference(&next_entry_dependencies));
    }
    for dep_id in remove_entry_dependencies {
      // connection may have been deleted by revoke module
      if let Some(con) = module_graph.connection_by_dependency_id(&dep_id) {
        // need clean_isolated_module to check whether the module is still used by other deps
        self
          .clean_isolated_module
          .add_need_check_module(*con.module_identifier());
      }
      module_graph.revoke_dependency(&dep_id, true);
      entry_dependencies.remove(&dep_id);
    }

    // add entry dependencies
    for dep in next_entry_dependencies
      .difference(&entry_dependencies)
      .copied()
      .collect::<Vec<_>>()
    {
      force_build_deps.insert((dep, None));
      entry_dependencies.insert(dep);
    }

    artifact.entry_dependencies = entry_dependencies;

    self.has_module_graph_change.analyze_artifact(artifact);

    // only return available force_build_deps
    let module_graph = artifact.get_module_graph();
    force_build_deps
      .into_iter()
      .filter(|(dep_id, _)| {
        let Some(dep) = module_graph.dependency_by_id(dep_id) else {
          return false;
        };
        dep.as_module_dependency().is_some() || dep.as_context_dependency().is_some()
      })
      .collect()
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
