use std::collections::VecDeque;

use rspack_collections::{IdentifierMap, IdentifierSet};

use super::super::MakeArtifact;
use crate::{ModuleGraph, ModuleIdentifier, ModuleIssuer};

enum IsIssuerResult {
  Ok,
  Cycle(IdentifierSet),
}
enum CalcIssuerResult {
  Ok(ModuleIssuer),
  Cycle(IdentifierMap<IdentifierSet>),
}

#[derive(Debug, Default)]
struct IssuerHelper {
  available_module: IdentifierSet,
}
impl IssuerHelper {
  fn is_issuer(&mut self, mg: &ModuleGraph, mid: &ModuleIdentifier) -> IsIssuerResult {
    let mut checked_modules = IdentifierSet::default();
    let mut current_module = *mid;
    loop {
      if self.available_module.contains(&current_module) {
        return IsIssuerResult::Ok;
      }
      if checked_modules.contains(&current_module) {
        return IsIssuerResult::Cycle(checked_modules);
      }
      checked_modules.insert(current_module);
      let mgm = mg
        .module_graph_module_by_identifier(&current_module)
        .expect("should mgm exist");
      let Some(parent) = mgm.issuer().identifier() else {
        self.available_module.extend(checked_modules);
        // Issuer is entry
        return IsIssuerResult::Ok;
      };
      current_module = *parent;
    }
  }

  fn calc_issuer(
    &mut self,
    mg: &ModuleGraph,
    parents: Vec<Option<ModuleIdentifier>>,
  ) -> CalcIssuerResult {
    let mut cycle_paths = IdentifierMap::default();
    for item in parents {
      let Some(mid) = item else {
        return CalcIssuerResult::Ok(ModuleIssuer::None);
      };
      if cycle_paths.contains_key(&mid) {
        continue;
      }
      match self.is_issuer(mg, &mid) {
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

#[derive(Debug, Default)]
pub struct FixIssuers {
  force_build_module_issuers: IdentifierMap<ModuleIssuer>,
  need_check_modules: IdentifierSet,
}

impl FixIssuers {
  pub fn analyze_force_build_module(
    &mut self,
    artifact: &MakeArtifact,
    module_identifier: &ModuleIdentifier,
  ) {
    let module_graph = artifact.get_module_graph();
    let mgm = module_graph
      .module_graph_module_by_identifier(module_identifier)
      .expect("should have module graph module");
    self
      .force_build_module_issuers
      .insert(*module_identifier, mgm.issuer().clone());
    self.need_check_modules.insert(*module_identifier);

    // analyze child module
    // if child module issuer is current module,
    // add child module to self.need_check_module_id
    for child_dep_id in mgm.outgoing_connections() {
      let child_mid = module_graph
        .module_identifier_by_dependency_id(child_dep_id)
        .expect("should module exist");
      let child_module_issuer = module_graph
        .module_graph_module_by_identifier(child_mid)
        .expect("should have module graph module")
        .issuer();
      if let ModuleIssuer::Some(i) = child_module_issuer
        && i == module_identifier
      {
        self.need_check_modules.insert(*child_mid);
      }
    }
  }

  pub fn add_need_check_module(&mut self, module_identifier: ModuleIdentifier) {
    self.need_check_modules.insert(module_identifier);
  }

  fn apply_force_build_module_issuer(
    self,
    artifact: &mut MakeArtifact,
  ) -> IdentifierMap<Vec<Option<ModuleIdentifier>>> {
    let Self {
      mut force_build_module_issuers,
      need_check_modules,
    } = self;
    let mut module_graph = artifact.get_module_graph_mut();
    need_check_modules
      .into_iter()
      .filter_map(|mid| {
        let Some(mgm) = module_graph.module_graph_module_by_identifier_mut(&mid) else {
          // no mgm means the module has been removed, ignored.
          return None;
        };
        if let Some(origin_issuer) = force_build_module_issuers.remove(&mid) {
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

  // set first parent as issuer
  fn try_set_first_issuer(
    artifact: &mut MakeArtifact,
    need_update_issuer_modules: IdentifierMap<Vec<Option<ModuleIdentifier>>>,
  ) -> IdentifierMap<Vec<Option<ModuleIdentifier>>> {
    let mut queue = VecDeque::with_capacity(need_update_issuer_modules.len());
    for (mid, parents) in need_update_issuer_modules {
      queue.push_back((mid, parents));
    }

    let mut module_graph = artifact.get_module_graph_mut();
    let mut revoke_module = IdentifierSet::default();
    let mut need_check_available_modules = IdentifierMap::default();
    loop {
      let Some((mid, parents)) = queue.pop_front() else {
        break;
      };
      let Some(first_parent) = parents.first() else {
        // clean
        for con in module_graph.get_outgoing_connections(&mid) {
          let mgm = module_graph
            .module_graph_module_by_identifier(con.module_identifier())
            .expect("should mgm exist");
          if matches!(mgm.issuer(), ModuleIssuer::Some(x) if x == &mid) {
            let parents = mgm
              .incoming_connections()
              .iter()
              .filter_map(|dep_id| {
                let conn = module_graph
                  .connection_by_dependency_id(dep_id)
                  .expect("should have connection");
                // mid will be revoked, filter those parents
                if conn.original_module_identifier != Some(mid) {
                  Some(conn.original_module_identifier)
                } else {
                  None
                }
              })
              .collect();
            queue.push_back((mgm.module_identifier, parents));
          }
        }
        revoke_module.insert(mid);
        continue;
      };
      let mgm = module_graph
        .module_graph_module_by_identifier_mut(&mid)
        .expect("should mgm exist");
      mgm.set_issuer(match first_parent {
        Some(id) => ModuleIssuer::Some(*id),
        None => ModuleIssuer::None,
      });
      need_check_available_modules.insert(mid, parents);
    }
    for mid in revoke_module {
      artifact.revoke_module(&mid);
    }
    need_check_available_modules
  }

  fn apply_available_issuer(
    artifact: &mut MakeArtifact,
    helper: &mut IssuerHelper,
    need_check_available_modules: IdentifierMap<Vec<Option<ModuleIdentifier>>>,
  ) -> IdentifierMap<IdentifierSet> {
    let mut module_graph = artifact.get_module_graph_mut();
    let mut need_clean_cycle_modules: IdentifierMap<IdentifierMap<IdentifierSet>> =
      IdentifierMap::default();
    for (mid, parents) in need_check_available_modules {
      match helper.calc_issuer(&module_graph, parents) {
        CalcIssuerResult::Ok(issuer) => {
          let mgm = module_graph
            .module_graph_module_by_identifier_mut(&mid)
            .expect("should have mgm");
          mgm.set_issuer(issuer);
          // check cycled items
          // the parent which cycle_paths contains current module can be set as issuer.
          need_clean_cycle_modules.retain(|id, paths| {
            for (issuer, cycle_paths) in paths {
              if cycle_paths.contains(&mid) {
                let mgm = module_graph
                  .module_graph_module_by_identifier_mut(id)
                  .expect("should have mgm");
                mgm.set_issuer(ModuleIssuer::Some(*issuer));
                return false;
              }
            }
            true
          });
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

  fn clean_cycle_module(
    artifact: &mut MakeArtifact,
    helper: &mut IssuerHelper,
    clean_modules: IdentifierMap<IdentifierSet>,
  ) {
    let mut revoke_module = IdentifierSet::default();
    let mut module_graph = artifact.get_module_graph_mut();
    for (mid, paths) in clean_modules {
      if revoke_module.contains(&mid) {
        continue;
      }

      let mut useless_module = IdentifierSet::default();
      let mut queue = VecDeque::new();
      queue.push_back((mid, paths));
      loop {
        let Some((current, parents)) = queue.pop_front() else {
          break;
        };
        useless_module.insert(current);

        for mid in parents {
          if useless_module.contains(&mid) {
            continue;
          }
          let parents: Vec<_> = module_graph
            .get_incoming_connections(&mid)
            .map(|item| item.original_module_identifier)
            .collect();

          match helper.calc_issuer(&module_graph, parents) {
            CalcIssuerResult::Ok(issuer) => {
              let mgm = module_graph
                .module_graph_module_by_identifier_mut(&mid)
                .expect("should have mgm");
              mgm.set_issuer(issuer);
              // TODO fix child issuer
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
                    let mgm = module_graph
                      .module_graph_module_by_identifier_mut(&child_mid)
                      .expect("should have mgm");
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

    for mid in revoke_module {
      artifact.revoke_module(&mid);
    }
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let need_update_issuer_modules = self.apply_force_build_module_issuer(artifact);
    if need_update_issuer_modules.is_empty() {
      return;
    }

    let need_check_available_modules =
      Self::try_set_first_issuer(artifact, need_update_issuer_modules);
    if need_check_available_modules.is_empty() {
      return;
    }

    let mut helper = IssuerHelper::default();
    let need_clean_cycle_modules =
      Self::apply_available_issuer(artifact, &mut helper, need_check_available_modules);
    if need_clean_cycle_modules.is_empty() {
      return;
    }

    Self::clean_cycle_module(artifact, &mut helper, need_clean_cycle_modules);
  }
}
