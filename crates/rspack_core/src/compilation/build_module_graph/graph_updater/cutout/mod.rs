mod fix_build_meta;
mod fix_issuers;

use rspack_collections::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;

use self::{fix_build_meta::FixBuildMeta, fix_issuers::FixIssuers};
use super::{BuildModuleGraphArtifact, UpdateParam};
use crate::{BuildDependency, Compilation, ResourceId, internal};

/// Cutout module graph.
///
/// This toolkit can remove useless module and dependency through `UpdateParam` and
/// do some post-processing on module graph like clean up isolated module.
#[derive(Debug, Default)]
pub struct Cutout {
  fix_issuers: FixIssuers,
  fix_build_meta: FixBuildMeta,
}

impl Cutout {
  /// Cutout artifact, the first step to incrementally update MakeArtifact.
  ///
  /// This step will remove useless module and dependency through `UpdateParam` and return
  /// the dependencyId and original module identifier of breaking point in the module graph.
  /// If we have a module graph like "A -> B -> C -> D", and the modules to remove are C and D,
  /// it will return the dependency of B->C.
  pub fn cutout_artifact(
    &mut self,
    compilation: &Compilation,
    artifact: &mut BuildModuleGraphArtifact,
    params: Vec<UpdateParam>,
  ) -> HashSet<BuildDependency> {
    // the entry dependencies after update module graph
    let mut next_entry_dependencies = HashSet::default();
    // whether to clean up useless entry dependencies
    let mut clean_entry_dependencies = false;
    let mut force_build_modules = IdentifierSet::default();
    let mut force_build_deps = HashSet::default();

    let module_graph = artifact.get_module_graph();

    for item in params {
      match item {
        UpdateParam::BuildEntry(deps) => {
          next_entry_dependencies = deps;
        }
        UpdateParam::BuildEntryAndClean(deps) => {
          next_entry_dependencies = deps;
          clean_entry_dependencies = true;
        }
        UpdateParam::CheckNeedBuild => {
          force_build_modules.extend(module_graph.modules().filter_map(|(_, module)| {
            if module.need_build(&compilation.value_cache_versions) {
              Some(module.identifier())
            } else {
              None
            }
          }));
        }
        UpdateParam::ModifiedFiles(files) | UpdateParam::RemovedFiles(files) => {
          for file in files {
            for resource_ids in [
              artifact.file_dependencies.related_resource_ids(&file),
              artifact.context_dependencies.related_resource_ids(&file),
              artifact.missing_dependencies.related_resource_ids(&file),
            ]
            .into_iter()
            .flatten()
            {
              for resource_id in resource_ids {
                match resource_id {
                  ResourceId::Module(mid) => {
                    force_build_modules.insert(*mid);
                  }
                  ResourceId::Dependency(dep_id) => {
                    force_build_deps.insert(*dep_id);
                  }
                }
              }
            }
          }
        }
        UpdateParam::ForceBuildModules(modules) => {
          force_build_modules.extend(modules);
        }
      };
    }

    // analyze force_build_module and force_build_deps
    self
      .fix_issuers
      .analyze_force_build_modules(artifact, &force_build_modules);
    self
      .fix_issuers
      .analyze_force_build_dependencies(artifact, &force_build_deps);
    self
      .fix_build_meta
      .analyze_force_build_modules(artifact, &force_build_modules);

    let mut build_deps = HashSet::default();

    // do revoke dependencies and collect deps
    for dep_id in force_build_deps {
      build_deps.extend(artifact.revoke_dependency(&dep_id, false));
    }

    // do revoke module and collect deps
    for id in force_build_modules {
      build_deps.extend(artifact.revoke_module(&id));
    }

    let mut entry_dependencies = std::mem::take(&mut artifact.entry_dependencies);
    // remove useless entry dependencies
    if clean_entry_dependencies {
      let module_graph = artifact.get_module_graph();
      let mut remove_entry_dependencies = vec![];
      for dep_id in entry_dependencies.difference(&next_entry_dependencies) {
        // connection may have been deleted by revoke module
        if let Some(con) = module_graph.connection_by_dependency_id(dep_id) {
          // need check if issuer still works
          self
            .fix_issuers
            .add_need_check_module(*con.module_identifier());
        }
        remove_entry_dependencies.push(*dep_id);
      }

      for dep_id in remove_entry_dependencies {
        artifact.revoke_dependency(&dep_id, true);
        entry_dependencies.remove(&dep_id);
      }
    }
    // add entry dependencies
    for dep in next_entry_dependencies.iter() {
      if !entry_dependencies.contains(dep) {
        build_deps.insert((*dep, None));
        entry_dependencies.insert(*dep);
      }
    }
    artifact.entry_dependencies = entry_dependencies;

    // only return available build_deps
    let module_graph = artifact.get_module_graph();
    build_deps
      .into_iter()
      .filter(|(dep_id, _)| {
        let Some(dep) = internal::try_dependency_by_id(module_graph, dep_id) else {
          return false;
        };
        dep.as_module_dependency().is_some() || dep.as_context_dependency().is_some()
      })
      .collect()
  }

  /// Fix artifact, the last step to incrementally update MakeArtifact.
  pub fn fix_artifact(self, artifact: &mut BuildModuleGraphArtifact) {
    let Self {
      fix_issuers,
      fix_build_meta,
    } = self;
    fix_issuers.fix_artifact(artifact);
    fix_build_meta.fix_artifact(artifact);
  }
}
