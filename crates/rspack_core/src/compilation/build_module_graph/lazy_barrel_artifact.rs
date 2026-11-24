use rspack_cacheable::{
  cacheable,
  with::{AsMap, AsOption, AsPreset, AsVec},
};
use rspack_collections::IdentifierMap;
use rspack_intern::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{DependencyId, DependencyRef, ModuleGraph, ModuleIdentifier};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ForwardId {
  All,
  Id(Atom),
  Empty,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LazyRole {
  LocalExport(Atom),
  NamedReexport(Atom),
  StarReexport,
  ReexportSideEffect,
  ImportDeclaration,
}

#[cacheable]
#[derive(Debug)]
enum ForwardedIdSet {
  All,
  IdSet(#[cacheable(with=AsVec<AsPreset>)] FxHashSet<Atom>),
}

impl ForwardedIdSet {
  fn is_empty(&self) -> bool {
    match self {
      Self::All => false,
      Self::IdSet(ids) => ids.is_empty(),
    }
  }
}

#[cacheable]
#[derive(Debug)]
enum ForwardRequest {
  Request(#[cacheable(with=AsPreset)] Atom),
  Imports,
  ImportsAnd(#[cacheable(with=AsPreset)] Atom),
}

fn request_key(dependency: &DependencyRef) -> Option<Atom> {
  let module_dependency = dependency.as_module_dependency()?;
  Some(match module_dependency.resource_identifier() {
    Some(resource_identifier) => Atom::from(resource_identifier),
    None => Atom::from(format!(
      "{}|{}",
      module_dependency.dependency_type(),
      module_dependency.request()
    )),
  })
}

fn take(
  request_to_dependencies: &mut FxHashMap<Atom, Vec<DependencyId>>,
  request_key: &Atom,
  dependencies: &mut Vec<DependencyId>,
  module_graph: &mut ModuleGraph,
) {
  let Some(group) = request_to_dependencies.remove(request_key) else {
    return;
  };
  dependencies.extend(
    group
      .into_iter()
      .filter(|dependency| module_graph.remove_lazy_dependency(dependency)),
  );
}

fn can_lazy_imports(
  forwarded_ids: Option<&ForwardedIdSet>,
  local_ids: &FxHashSet<Atom>,
  running_reexports: &[(Atom, Atom)],
) -> bool {
  let Some(ForwardedIdSet::IdSet(ids)) = forwarded_ids else {
    return false;
  };
  !ids.is_empty()
    && ids
      .iter()
      .all(|id| !local_ids.contains(id) && !running_reexports.iter().any(|(name, _)| name == id))
}

#[cacheable]
#[derive(Debug, Default)]
struct LazyDependencies {
  #[cacheable(with=AsMap<AsPreset>)]
  request_to_dependencies: FxHashMap<Atom, Vec<DependencyId>>,
  #[cacheable(with=AsMap<AsPreset>)]
  forward_id_to_request: FxHashMap<Atom, ForwardRequest>,
  #[cacheable(with=AsVec<AsPreset>)]
  import_requests: Vec<Atom>,
  #[cacheable(with=AsOption<AsVec<AsPreset>>)]
  fallback_requests: Option<Vec<Atom>>,
}

impl LazyDependencies {
  fn create(
    dependencies: &[DependencyRef],
    forwarded_ids: Option<&ForwardedIdSet>,
    lazy_imports: bool,
    module_graph: &mut ModuleGraph,
  ) -> Option<Self> {
    let mut info: Option<Self> = None;
    let mut local_ids = FxHashSet::default();
    let mut running_reexports = vec![];

    for dependency in dependencies {
      let role = match dependency.lazy_role() {
        None | Some(LazyRole::ImportDeclaration) => continue,
        Some(LazyRole::LocalExport(name)) => {
          local_ids.insert(name);
          continue;
        }
        Some(role) => role,
      };
      let Some(request_key) = request_key(dependency) else {
        continue;
      };
      let info = info.get_or_insert_default();
      info.add_lazy(request_key.clone(), *dependency.id(), module_graph);
      match role {
        LazyRole::NamedReexport(name) => {
          if dependency.get_phase().is_defer() {
            running_reexports.push((name.clone(), request_key.clone()));
          }
          info.add_forward_id(name, ForwardRequest::Request(request_key));
        }
        LazyRole::StarReexport => info.add_fallback(request_key),
        _ => {}
      }
    }

    if lazy_imports && can_lazy_imports(forwarded_ids, &local_ids, &running_reexports) {
      for dependency in dependencies {
        if dependency.lazy_role() != Some(LazyRole::ImportDeclaration) {
          continue;
        }
        let Some(request_key) = request_key(dependency) else {
          continue;
        };
        let info = info.get_or_insert_default();
        info.add_lazy(request_key.clone(), *dependency.id(), module_graph);
        if !info.import_requests.contains(&request_key) {
          info.import_requests.push(request_key);
        }
      }
    }

    let mut info = info?;
    if !info.import_requests.is_empty() {
      for (id, request_key) in running_reexports {
        info.add_forward_id(id, ForwardRequest::ImportsAnd(request_key));
      }
    }
    for id in local_ids {
      info.add_forward_id(id, ForwardRequest::Imports);
    }
    Some(info)
  }

  fn add_lazy(
    &mut self,
    request_key: Atom,
    dependency: DependencyId,
    module_graph: &mut ModuleGraph,
  ) {
    let group = self.request_to_dependencies.entry(request_key).or_default();
    if !group.contains(&dependency) {
      group.push(dependency);
    }
    module_graph.add_lazy_dependency(dependency);
  }

  fn add_forward_id(&mut self, id: Atom, request: ForwardRequest) {
    self.forward_id_to_request.insert(id, request);
  }

  fn add_fallback(&mut self, request_key: Atom) {
    let fallback_requests = self.fallback_requests.get_or_insert_default();
    if !fallback_requests.contains(&request_key) {
      fallback_requests.push(request_key);
    }
  }

  fn references_local(&self, ids: &ForwardedIdSet) -> bool {
    match ids {
      ForwardedIdSet::All => false,
      ForwardedIdSet::IdSet(ids) => ids.iter().any(|id| {
        matches!(
          self.forward_id_to_request.get(id),
          Some(ForwardRequest::Imports)
        )
      }),
    }
  }

  fn is_empty(&self) -> bool {
    self.request_to_dependencies.is_empty()
  }

  fn has_lazy_imports(&self) -> bool {
    self
      .import_requests
      .iter()
      .any(|request_key| self.request_to_dependencies.contains_key(request_key))
  }

  fn request(&mut self, ids: &ForwardedIdSet, module_graph: &mut ModuleGraph) -> Vec<DependencyId> {
    let mut dependencies = vec![];
    match ids {
      ForwardedIdSet::All => {
        for (_, group) in self.request_to_dependencies.drain() {
          dependencies.extend(
            group
              .into_iter()
              .filter(|dependency| module_graph.remove_lazy_dependency(dependency)),
          );
        }
      }
      ForwardedIdSet::IdSet(ids) => self.take_ids(ids, &mut dependencies, module_graph),
    }
    if self.request_to_dependencies.is_empty() {
      self.release();
    }
    dependencies
  }

  fn take_ids(
    &mut self,
    ids: &FxHashSet<Atom>,
    dependencies: &mut Vec<DependencyId>,
    module_graph: &mut ModuleGraph,
  ) {
    for id in ids {
      match self.forward_id_to_request.get(id) {
        Some(ForwardRequest::Request(request_key)) => take(
          &mut self.request_to_dependencies,
          request_key,
          dependencies,
          module_graph,
        ),
        Some(ForwardRequest::Imports) => {
          for request_key in &self.import_requests {
            take(
              &mut self.request_to_dependencies,
              request_key,
              dependencies,
              module_graph,
            );
          }
        }
        Some(ForwardRequest::ImportsAnd(own_request_key)) => {
          for request_key in self.import_requests.iter().chain([own_request_key]) {
            take(
              &mut self.request_to_dependencies,
              request_key,
              dependencies,
              module_graph,
            );
          }
        }
        None => {
          let Some(fallback_requests) = &self.fallback_requests else {
            continue;
          };
          for request_key in fallback_requests {
            take(
              &mut self.request_to_dependencies,
              request_key,
              dependencies,
              module_graph,
            );
          }
        }
      }
    }
  }

  fn release(&mut self) {
    self.forward_id_to_request = Default::default();
    self.import_requests = Default::default();
    self.fallback_requests = None;
  }
}

#[cacheable]
#[derive(Debug)]
pub struct LazyMakeState {
  forwarded_ids: ForwardedIdSet,
  lazy_dependencies: Option<LazyDependencies>,
}

#[derive(Debug, Default)]
pub struct ModuleToLazyMake {
  pub eager_on_local_reference: bool,
  modules: IdentifierMap<LazyMakeState>,
}

impl ModuleToLazyMake {
  pub fn classify(
    &mut self,
    module: ModuleIdentifier,
    side_effect_free: bool,
    dependencies: &[DependencyRef],
    lazy_imports: bool,
    module_graph: &mut ModuleGraph,
  ) -> bool {
    if !side_effect_free {
      self.modules.remove(&module);
      return false;
    }
    let state = self.modules.get_mut(&module);
    if let Some(state) = &state
      && matches!(state.forwarded_ids, ForwardedIdSet::All)
    {
      return false;
    }
    let Some(mut info) = LazyDependencies::create(
      dependencies,
      state.as_ref().map(|state| &state.forwarded_ids),
      lazy_imports,
      module_graph,
    ) else {
      self.modules.remove(&module);
      return false;
    };
    let Some(state) = state else {
      self.modules.insert(
        module,
        LazyMakeState {
          forwarded_ids: ForwardedIdSet::IdSet(Default::default()),
          lazy_dependencies: Some(info),
        },
      );
      return true;
    };
    if self.eager_on_local_reference && info.references_local(&state.forwarded_ids) {
      state.forwarded_ids = ForwardedIdSet::All;
    }
    info.request(&state.forwarded_ids, module_graph);
    let has_lazy_dependencies = !info.is_empty();
    state.lazy_dependencies = Some(info);
    has_lazy_dependencies
  }

  pub fn request(
    &mut self,
    module: ModuleIdentifier,
    side_effect_free: bool,
    dependencies: &[DependencyRef],
    module_graph: &mut ModuleGraph,
  ) -> Vec<DependencyId> {
    match self.modules.get(&module) {
      Some(state) if matches!(state.forwarded_ids, ForwardedIdSet::All) => return vec![],
      Some(_) => {}
      None if !side_effect_free || module_graph.module_by_identifier(&module).is_some() => {
        return vec![];
      }
      None => {}
    }
    let ids = requested_ids(dependencies);
    if ids.is_empty() {
      return vec![];
    }
    let Some(state) = self.modules.get_mut(&module) else {
      if module_graph
        .module_graph_module_by_identifier(&module)
        .is_some()
      {
        self.modules.insert(
          module,
          LazyMakeState {
            forwarded_ids: ids,
            lazy_dependencies: None,
          },
        );
      }
      return vec![];
    };
    let all = ForwardedIdSet::All;
    let ids = if self.eager_on_local_reference
      && let Some(info) = &state.lazy_dependencies
      && info.references_local(&ids)
    {
      &all
    } else {
      &ids
    };
    let Some(new_ids) = merge_forwarded_ids(&mut state.forwarded_ids, ids) else {
      return vec![];
    };
    let Some(info) = &mut state.lazy_dependencies else {
      return vec![];
    };
    info.request(&new_ids, module_graph)
  }

  pub fn get(&self, module: &ModuleIdentifier) -> Option<&LazyMakeState> {
    self.modules.get(module)
  }

  pub fn restore(&mut self, module: ModuleIdentifier, state: LazyMakeState) {
    self.modules.insert(module, state);
  }

  pub fn modules_with_lazy_imports(&self) -> impl Iterator<Item = &ModuleIdentifier> {
    self.modules.iter().filter_map(|(module, state)| {
      state
        .lazy_dependencies
        .as_ref()
        .is_some_and(LazyDependencies::has_lazy_imports)
        .then_some(module)
    })
  }

  pub fn revoke_module(&mut self, module: &ModuleIdentifier) {
    self.modules.remove(module);
  }
}

fn requested_ids(dependencies: &[DependencyRef]) -> ForwardedIdSet {
  let mut ids = FxHashSet::default();
  for dependency in dependencies {
    match dependency.forward_id() {
      ForwardId::All => return ForwardedIdSet::All,
      ForwardId::Id(id) => {
        ids.insert(id);
      }
      ForwardId::Empty => {}
    }
  }
  ForwardedIdSet::IdSet(ids)
}

fn merge_forwarded_ids(
  forwarded_ids: &mut ForwardedIdSet,
  ids: &ForwardedIdSet,
) -> Option<ForwardedIdSet> {
  let ForwardedIdSet::IdSet(forwarded) = forwarded_ids else {
    return None;
  };
  let ForwardedIdSet::IdSet(ids) = ids else {
    *forwarded_ids = ForwardedIdSet::All;
    return Some(ForwardedIdSet::All);
  };
  let new_ids = ids
    .iter()
    .filter(|id| forwarded.insert((*id).clone()))
    .cloned()
    .collect::<FxHashSet<_>>();
  (!new_ids.is_empty()).then_some(ForwardedIdSet::IdSet(new_ids))
}
