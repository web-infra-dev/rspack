use std::collections::VecDeque;

use rspack_collections::{IdentifierMap, IdentifierSet};
use rustc_hash::FxHashSet as HashSet;

use super::BuildModuleGraphArtifact;
use crate::{
  DependencyId, ModuleGraph, ModuleIdentifier, ModuleIssuer,
  internal::try_get_module_graph_module_mut_by_identifier,
};

/// Result of IssuerHelper.is_issuer.
enum IsIssuerResult {
  /// Input module id can be set as issuer.
  Ok,
  /// Input module id set as issuer will cause issuer cycle, return the cycle paths.
  Cycle(IdentifierSet),
}

/// Result of IssuerHelper.calc_issuer.
enum CalcIssuerResult {
  /// Return available issuer.
  Ok(ModuleIssuer),
  /// All inputs can not be set as issuer,
  /// return the cycle paths of each input.
  Cycle(IdentifierMap<IdentifierSet>),
}

/// A toolkit for calculate issuer.
#[derive(Debug, Default)]
struct IssuerHelper {
  /// Cache the module which can set as issuer.
  available_module: IdentifierSet,
}
impl IssuerHelper {
  /// Check if `check_mid` can be set as issuer of `base_mid`.
  ///
  /// This method will search upward for the issuer until
  /// If current module issuer is ModuleIssuer::None, it means `check_mid` can find the entry by loop issuer, return IsIssuerResult::Ok.
  /// If current module issuer has been checked, it means `check_mid` is a cycle issuer, return IsIssuerResult::Cycle with cycle path.
  ///
  /// NOTE: Before call this function, you should ensure that the issuer of all module are be valid (issuer in incoming connections).
  fn is_issuer(
    &mut self,
    mg: &ModuleGraph,
    base_mid: &ModuleIdentifier,
    check_mid: &ModuleIdentifier,
  ) -> IsIssuerResult {
    let mut checked_modules = IdentifierSet::default();
    let mut current_module = *check_mid;
    loop {
      if self.available_module.contains(&current_module) {
        self.available_module.extend(checked_modules);
        return IsIssuerResult::Ok;
      }
      if checked_modules.contains(&current_module) || &current_module == base_mid {
        return IsIssuerResult::Cycle(checked_modules);
      }
      checked_modules.insert(current_module);
      let mgm = mg
        .module_graph_module_by_identifier(&current_module)
        .expect("should mgm exist");
      let Some(parent) = mgm.issuer().identifier() else {
        // mgm.issuer is entry
        self.available_module.extend(checked_modules);
        return IsIssuerResult::Ok;
      };
      current_module = *parent;
    }
  }

  /// Calculate the issuer of `base_mid` from `parents`.
  ///
  /// If a module in `parents` can be set as issuer, return CalcIssuerResult::Ok with ModuleIssuer.
  /// If none of the module in `parents` match, return CalcIssuerResult::Cycle with each module and its cycle paths.
  ///
  /// NOTE: Before call this function, you should ensure that the issuer of all module are valid (issuer in incoming connections).
  fn calc_issuer(
    &mut self,
    mg: &ModuleGraph,
    base_mid: &ModuleIdentifier,
    parents: Vec<Option<ModuleIdentifier>>,
  ) -> CalcIssuerResult {
    let mut cycle_paths = IdentifierMap::default();
    for item in parents {
      let Some(mid) = item else {
        // item is none means issuer can set as ModuleIssuer::None.
        return CalcIssuerResult::Ok(ModuleIssuer::None);
      };
      // `parents` is not deduplicated, skipping those duplicate calculations.
      if cycle_paths.contains_key(&mid) {
        continue;
      }
      match self.is_issuer(mg, base_mid, &mid) {
        IsIssuerResult::Ok => {
          return CalcIssuerResult::Ok(ModuleIssuer::Some(mid));
        }
        IsIssuerResult::Cycle(checked_paths) => {
          cycle_paths.insert(mid, checked_paths);
        }
      }
    }
    CalcIssuerResult::Cycle(cycle_paths)
  }
}

/// A toolkit for cutout to fix module graph issuers and clean isolated modules.
#[derive(Debug, Default)]
pub(super) struct FixIssuers {
  /// Collect issuer of force_build_module.
  force_build_module_issuers: IdentifierMap<ModuleIssuer>,
  /// Collect the module whose issuer need to be checked for availability.
  need_check_modules: IdentifierSet,
}

impl FixIssuers {
  /// Analyze force_build_modules.
  ///
  /// This function will
  /// 1. save the issuer of force_build_module to self.force_build_module_issuers.
  /// 2. add force_build_module and the child module whose issuer is this force_build_module to self.need_check_modules.
  pub(super) fn analyze_force_build_modules(
    &mut self,
    artifact: &BuildModuleGraphArtifact,
    ids: &IdentifierSet,
  ) {
    let module_graph = artifact.get_module_graph();
    for module_identifier in ids {
      let mgm = module_graph
        .module_graph_module_by_identifier(module_identifier)
        .expect("should have module graph module");
      self
        .force_build_module_issuers
        .insert(*module_identifier, mgm.issuer().clone());
      self.need_check_modules.insert(*module_identifier);

      // analyze child module
      // if child module issuer is current module,
      // add child module to self.need_check_modules
      for child_dep_id in mgm.outgoing_connections() {
        let child_mid = module_graph
          .module_identifier_by_dependency_id(child_dep_id)
          .expect("should module exist");
        let Some(child_mgm) = module_graph.module_graph_module_by_identifier(child_mid) else {
          // peresistent cache recovery module graph will lose some module and mgm.
          // TODO replace to .expect() after all modules are cacheable.
          self.need_check_modules.insert(*child_mid);
          continue;
        };

        let child_module_issuer = child_mgm.issuer();
        if let ModuleIssuer::Some(i) = child_module_issuer
          && i == module_identifier
        {
          self.need_check_modules.insert(*child_mid);
        }
      }
    }
  }

  /// Analyze force_build_dependencies.
  ///
  /// If the target module issuer is the same as the current dependency original module,
  /// this function will add the dependency target module to self.need_check_modules
  pub(super) fn analyze_force_build_dependencies(
    &mut self,
    artifact: &BuildModuleGraphArtifact,
    ids: &HashSet<DependencyId>,
  ) {
    let module_graph = artifact.get_module_graph();
    for dep_id in ids {
      let Some(connection) = module_graph.connection_by_dependency_id(dep_id) else {
        // only ModuleDependency/ContextDependency has connection
        continue;
      };

      let Some(origin) = &connection.original_module_identifier else {
        // no original module means entry
        continue;
      };
      let Some(mgm) =
        module_graph.module_graph_module_by_identifier(connection.module_identifier())
      else {
        // TODO: change to expect when persistent cache supports all modules
        continue;
      };

      if let ModuleIssuer::Some(issuer) = mgm.issuer()
        && issuer == origin
      {
        self.need_check_modules.insert(mgm.module_identifier);
      }
    }
  }

  /// Add module to self.need_check_modules.
  pub(super) fn add_need_check_module(&mut self, module_identifier: ModuleIdentifier) {
    self.need_check_modules.insert(module_identifier);
  }

  /// The 1st step of self.fix_artifact, apply `self.force_build_module_issuers` to module graph.
  /// Returns the modules and their parents whose issuer not in the incoming connections.
  ///
  /// This function will traverse the `self.need_check_modules`
  /// 1. remove not exist module.
  /// 2. if current module is in `self.force_build_module_issuers`, set the issuer.
  /// 3. remove the module whose issuer in mgm.incoming_connection.
  /// 4. return the module with invalid issuer and its parents.
  fn apply_force_build_module_issuer(
    self,
    artifact: &mut BuildModuleGraphArtifact,
  ) -> IdentifierMap<Vec<Option<ModuleIdentifier>>> {
    let Self {
      mut force_build_module_issuers,
      need_check_modules,
    } = self;
    let module_graph = &mut artifact.module_graph;
    need_check_modules
      .into_iter()
      .filter_map(|mid| {
        let Some(mgm) = try_get_module_graph_module_mut_by_identifier(module_graph, &mid) else {
          // no mgm means the module has been removed, ignored.
          return None;
        };
        if let Some(origin_issuer) = force_build_module_issuers.remove(&mid) {
          artifact.issuer_update_modules.insert(mgm.module_identifier);
          mgm.set_issuer(origin_issuer);
        }

        let incoming_connections: Vec<_> = mgm.incoming_connections().iter().copied().collect();
        let issuer_identifier = mgm.issuer().identifier().copied();
        let mut parents = Vec::with_capacity(incoming_connections.len());
        for dep_id in incoming_connections {
          let conn = module_graph
            .connection_by_dependency_id(&dep_id)
            .expect("should have connection");
          if conn.original_module_identifier == issuer_identifier {
            // current issuer is a incoming connection, skip it.
            return None;
          }
          parents.push(conn.original_module_identifier);
        }
        Some((mid, parents))
      })
      .collect()
  }

  /// The 2nd step of self.fix_artifact, try to set first mgm.incoming_connection as issuer.
  /// Returns the modules and their parents whose issuer has been set.
  ///
  /// This function will traverse the `need_update_issuer_modules` returned in previous step
  /// 1. if a module has no parent module, revoke the module and add its child modules
  ///    whose issuer is the current module to `need_update_issuer_modules`.
  /// 2. set the first parent module as issuer.
  /// 3. return these modules and their parents.
  ///
  /// After this step, the issuer of all module are valid and we can use IssuerHelper in next steps.
  fn try_set_first_incoming(
    artifact: &mut BuildModuleGraphArtifact,
    need_update_issuer_modules: IdentifierMap<Vec<Option<ModuleIdentifier>>>,
  ) -> IdentifierMap<Vec<Option<ModuleIdentifier>>> {
    let mut queue = VecDeque::with_capacity(need_update_issuer_modules.len());
    for (mid, parents) in need_update_issuer_modules {
      queue.push_back((mid, parents));
    }

    let module_graph = &mut artifact.module_graph;
    let mut revoke_module = IdentifierSet::default();
    let mut need_check_available_modules = IdentifierMap::default();
    loop {
      let Some((mid, mut parents)) = queue.pop_front() else {
        break;
      };

      // remove revoke_module from parents
      parents.retain(|item| {
        let Some(parent_mid) = item else {
          return true;
        };
        !revoke_module.contains(parent_mid)
      });
      let Some(first_parent) = parents.first() else {
        // no first parent, isolated module.
        // revoke current module and add child module whose issuer is current module to queue.
        let mut child_modules = IdentifierMap::default();
        for child_con in module_graph.get_outgoing_connections(&mid) {
          let child_mid = child_con.module_identifier();
          if child_modules.contains_key(child_mid) {
            continue;
          }

          let child_mgm = module_graph
            .module_graph_module_by_identifier(child_mid)
            .expect("should mgm exist");
          if matches!(child_mgm.issuer(), ModuleIssuer::Some(x) if x == &mid) {
            let child_module_parents = child_mgm
              .incoming_connections()
              .iter()
              .map(|dep_id| {
                let conn = module_graph
                  .connection_by_dependency_id(dep_id)
                  .expect("should have connection");
                conn.original_module_identifier
              })
              .collect();
            child_modules.insert(*child_mid, child_module_parents);
          }
        }
        // add child modules to queue
        queue.extend(child_modules);
        revoke_module.insert(mid);
        continue;
      };
      let mgm = module_graph.module_graph_module_by_identifier_mut(&mid);
      artifact.issuer_update_modules.insert(mgm.module_identifier);
      mgm.set_issuer(match first_parent {
        Some(id) => ModuleIssuer::Some(*id),
        None => ModuleIssuer::None,
      });
      need_check_available_modules.insert(mid, parents);
    }

    // make sure return value does not contain any module in revoke_module.
    need_check_available_modules.retain(|k, v| {
      if revoke_module.contains(k) {
        return false;
      }

      v.retain(|mid| {
        let Some(mid) = mid else {
          return true;
        };
        !revoke_module.contains(mid)
      });
      true
    });
    for mid in revoke_module {
      artifact.revoke_module(&mid);
    }
    need_check_available_modules
  }

  /// The 3rd step of self.fix_artifact, set available issuer from parents.
  /// Returns the modules and their parents whose parents are cycled.
  ///
  /// This function will traverse the `need_check_available_modules` returned in previous step
  /// 1. call IssuerHelper.calc_issuer to get a issuer
  /// 2. if all parents are cycled, save mid, parents, cycle_paths to `need_clean_cycle_modules`.
  /// 3. if a parent can be set as issuer, set it.
  ///    - any module set issuer success should check if the current module affects modules in `need_clean_cycle_modules`.
  /// 4. return need_clean_cycle_modules but remove cycle_paths info.
  fn set_available_issuer(
    artifact: &mut BuildModuleGraphArtifact,
    helper: &mut IssuerHelper,
    need_check_available_modules: IdentifierMap<Vec<Option<ModuleIdentifier>>>,
  ) -> IdentifierMap<IdentifierSet> {
    let module_graph = &mut artifact.module_graph;
    let mut need_clean_cycle_modules: IdentifierMap<IdentifierMap<IdentifierSet>> =
      IdentifierMap::default();
    for (mid, parents) in need_check_available_modules {
      match helper.calc_issuer(module_graph, &mid, parents) {
        CalcIssuerResult::Ok(issuer) => {
          let mgm = module_graph.module_graph_module_by_identifier_mut(&mid);
          artifact.issuer_update_modules.insert(mgm.module_identifier);
          mgm.set_issuer(issuer);
          // check cycled modules.
          // the parent which cycle_paths contains current module can be set as issuer.
          let mut queue = VecDeque::with_capacity(1);
          queue.push_back(mid);
          loop {
            let Some(current_id) = queue.pop_front() else {
              break;
            };
            need_clean_cycle_modules.retain(|id, paths| {
              for (issuer, cycle_paths) in paths {
                if cycle_paths.contains(&current_id) {
                  let mgm = module_graph.module_graph_module_by_identifier_mut(id);
                  artifact.issuer_update_modules.insert(mgm.module_identifier);
                  mgm.set_issuer(ModuleIssuer::Some(*issuer));
                  // this module issuer has been update, add module to queue to recheck need_clean_cycle_modules
                  queue.push_back(*id);
                  return false;
                }
              }
              true
            });
          }
        }
        CalcIssuerResult::Cycle(paths) => {
          need_clean_cycle_modules.insert(mid, paths);
        }
      }
    }
    need_clean_cycle_modules
      .into_iter()
      .map(|(id, parents)| (id, parents.into_keys().collect()))
      .collect()
  }

  /// The 4th step of self.fix_artifact, clean cycled modules.
  /// Returns the modules and their parents whose issuer has been removed.
  ///
  /// This function will traverse the `clean_modules` returned in previous step,
  /// recursively check whether the incoming_connections of all parent modules of the current module can be used as the issuer,
  /// - if not, delete all checked modules.
  /// - if a module can be modified to be an available issuer, then starting from the current module,
  ///   re-update the issuers of all checked modules.
  fn clean_cycle_module(
    artifact: &mut BuildModuleGraphArtifact,
    helper: &mut IssuerHelper,
    clean_modules: IdentifierMap<IdentifierSet>,
  ) -> IdentifierMap<Vec<Option<ModuleIdentifier>>> {
    let mut revoke_module = IdentifierSet::default();
    let module_graph = &mut artifact.module_graph;
    for (mid, paths) in clean_modules {
      if revoke_module.contains(&mid) {
        continue;
      }

      let mut useless_module = IdentifierSet::default();
      let mut queue = VecDeque::new();
      queue.push_back((mid, paths));
      loop {
        let Some((current, current_parents)) = queue.pop_front() else {
          break;
        };
        useless_module.insert(current);

        for mid in current_parents {
          if useless_module.contains(&mid) {
            continue;
          }
          let parents: Vec<_> = module_graph
            .get_incoming_connections(&mid)
            .map(|item| item.original_module_identifier)
            .collect();

          match helper.calc_issuer(module_graph, &mid, parents) {
            CalcIssuerResult::Ok(issuer) => {
              let mgm = module_graph.module_graph_module_by_identifier_mut(&mid);
              artifact.issuer_update_modules.insert(mgm.module_identifier);
              mgm.set_issuer(issuer);
              // fix child issuer
              let mut fixing_queue = VecDeque::new();
              fixing_queue.push_back(mid);
              loop {
                if useless_module.is_empty() {
                  break;
                }
                let Some(mid) = fixing_queue.pop_back() else {
                  break;
                };
                let child_modules: Vec<_> = module_graph
                  .get_outgoing_connections(&mid)
                  .map(|conn| *conn.module_identifier())
                  .collect();
                for child_mid in child_modules {
                  if useless_module.remove(&child_mid) {
                    let mgm = module_graph.module_graph_module_by_identifier_mut(&child_mid);
                    artifact.issuer_update_modules.insert(mgm.module_identifier);
                    mgm.set_issuer(ModuleIssuer::Some(mid));
                    fixing_queue.push_back(child_mid);
                  }
                }
              }
            }
            CalcIssuerResult::Cycle(paths) => {
              queue.push_back((mid, paths.into_keys().collect()));
            }
          }
        }
      }
      revoke_module.extend(useless_module);
    }

    // calculate need_update_issuer_modules by revoke module
    let mut need_update_issuer_modules = IdentifierMap::default();
    for mid in &revoke_module {
      for child_con in module_graph.get_outgoing_connections(mid) {
        let child_mid = child_con.module_identifier();
        if need_update_issuer_modules.contains_key(child_mid) {
          continue;
        }
        if revoke_module.contains(child_mid) {
          continue;
        }
        let child_mgm = module_graph
          .module_graph_module_by_identifier(child_mid)
          .expect("should mgm exist");
        if matches!(child_mgm.issuer(), ModuleIssuer::Some(x) if x == mid) {
          let child_module_parents = child_mgm
            .incoming_connections()
            .iter()
            .filter_map(|dep_id| {
              let origin_module_identifier = module_graph
                .connection_by_dependency_id(dep_id)
                .expect("should have connection")
                .original_module_identifier;
              if let Some(mid) = origin_module_identifier
                && revoke_module.contains(&mid)
              {
                return None;
              }
              Some(origin_module_identifier)
            })
            .collect();
          need_update_issuer_modules.insert(*child_mid, child_module_parents);
        }
      }
    }
    for mid in revoke_module {
      artifact.revoke_module(&mid);
    }

    need_update_issuer_modules
  }

  /// fix artifact module graph issuers
  pub(super) fn fix_artifact(self, artifact: &mut BuildModuleGraphArtifact) {
    let mut need_update_issuer_modules = self.apply_force_build_module_issuer(artifact);

    let mut helper = IssuerHelper::default();
    loop {
      if need_update_issuer_modules.is_empty() {
        return;
      }
      let need_check_available_modules =
        Self::try_set_first_incoming(artifact, need_update_issuer_modules);
      if need_check_available_modules.is_empty() {
        return;
      }

      let need_clean_cycle_modules =
        Self::set_available_issuer(artifact, &mut helper, need_check_available_modules);
      if need_clean_cycle_modules.is_empty() {
        return;
      }

      need_update_issuer_modules =
        Self::clean_cycle_module(artifact, &mut helper, need_clean_cycle_modules);
    }
  }
}
