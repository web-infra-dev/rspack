use napi::bindgen_prelude::{Int32Array, Uint8Array, Uint32Array};
use napi_derive::napi;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rspack_core::{
  Compilation, ConnectionState, ExportsInfoArtifact, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleIdentifier, SideEffectsStateArtifact, internal,
};
use rustc_hash::FxHashMap;

/// The edge is known to be inactive for the requested runtime.
#[napi]
pub const EDGE_STATE_INACTIVE: u8 = 0;
/// The edge is known to be active for the requested runtime.
#[napi]
pub const EDGE_STATE_ACTIVE: u8 = 1;
/// `ConnectionState::CircularConnection`.
#[napi]
pub const EDGE_STATE_CIRCULAR: u8 = 2;
/// `ConnectionState::TransitiveOnly`.
#[napi]
pub const EDGE_STATE_TRANSITIVE_ONLY: u8 = 3;

/// Sentinel used in `edgeTargets` when a connection exists but its target
/// module is no longer present in the graph. This mirrors the `null` that
/// `ModuleGraphConnection.module` returns for the same connection, and keeps
/// edge index `k` of module `m` aligned with
/// `getOutgoingConnectionsInOrder(m)[k]`.
#[napi]
pub const NO_MODULE: u32 = u32::MAX;

/// A compact, ordered description of the module graph.
///
/// Consumers that need every module and every outgoing connection (import
/// linting, dependency reports, custom graph analysis) otherwise walk the
/// graph through per-module and per-connection getters. Each of those getters
/// crosses the JavaScript boundary, allocates a wrapper object and re-acquires
/// the artifacts it reads. This snapshot answers the same questions in a
/// single call, computed on the native side.
///
/// Modules appear in `Compilation.modules` order, so a consumer that iterates
/// the snapshot observes the same order it observes today. Outgoing edges
/// appear in `getOutgoingConnectionsInOrder` order.
#[napi(object)]
pub struct JsModuleGraphSnapshot {
  /// `Module.identifier()` per module index.
  pub identifiers: Vec<String>,
  /// `Module.nameForCondition()` per module index, `None` when the module has
  /// none. Query strings are stripped by `nameForCondition` itself; the value
  /// is passed through unchanged.
  pub name_for_conditions: Vec<Option<String>>,
  /// `Module.layer` per module index.
  pub layers: Vec<Option<String>>,
  /// Module indices reachable from `Compilation.entries[*].dependencies`, in
  /// entry order then dependency order. Entry dependencies without a
  /// connection or without a module are omitted, matching a consumer that
  /// skips `getConnection(dep)?.module == null`.
  pub entry_modules: Uint32Array,
  /// CSR row offsets, length `identifiers.len() + 1`. The outgoing edges of
  /// module `m` are the range `edgeOffsets[m]..edgeOffsets[m + 1]`.
  pub edge_offsets: Uint32Array,
  /// Target module index per edge, or `NO_MODULE` (`0xffffffff`) when the
  /// connection has no target module.
  pub edge_targets: Uint32Array,
  /// Index into `requests` per edge, or `-1` when the dependency is not a
  /// module dependency and therefore has no request.
  pub edge_requests: Int32Array,
  /// One of the `EDGE_STATE_*` values per edge.
  pub edge_states: Uint8Array,
  /// Deduplicated dependency request strings.
  pub requests: Vec<String>,
  /// `false` when the exports info artifact was unavailable, in which case
  /// every edge state is reported as active. This reproduces the fallback in
  /// `ModuleGraphConnection.getActiveState`, and lets a consumer detect that
  /// it queried the graph at a phase where active state is not yet decided
  /// instead of silently treating unresolved edges as live.
  pub exports_info_available: bool,
}

struct ModuleEdges {
  targets: Vec<u32>,
  requests: Vec<i32>,
  states: Vec<u8>,
}

fn to_edge_state(state: ConnectionState) -> u8 {
  match state {
    ConnectionState::Active(true) => EDGE_STATE_ACTIVE,
    ConnectionState::Active(false) => EDGE_STATE_INACTIVE,
    ConnectionState::CircularConnection => EDGE_STATE_CIRCULAR,
    ConnectionState::TransitiveOnly => EDGE_STATE_TRANSITIVE_ONLY,
  }
}

fn collect_module_edges(
  module_identifier: &ModuleIdentifier,
  module_graph: &ModuleGraph,
  module_indices: &FxHashMap<ModuleIdentifier, u32>,
  request_indices: &FxHashMap<&str, i32>,
  active_state_inputs: Option<(
    &ModuleGraphCacheArtifact,
    &SideEffectsStateArtifact,
    &ExportsInfoArtifact,
  )>,
) -> ModuleEdges {
  let mut edges = ModuleEdges {
    targets: Vec::new(),
    requests: Vec::new(),
    states: Vec::new(),
  };

  for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
    let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
      continue;
    };

    edges.targets.push(
      module_indices
        .get(connection.module_identifier())
        .copied()
        .unwrap_or(NO_MODULE),
    );

    edges.requests.push(
      internal::try_dependency_by_id(module_graph, dependency_id)
        .and_then(|dependency| dependency.as_module_dependency())
        .and_then(|dependency| request_indices.get(dependency.request()).copied())
        .unwrap_or(-1),
    );

    edges.states.push(match active_state_inputs {
      Some((module_graph_cache, side_effects_state, exports_info)) => {
        to_edge_state(connection.active_state(
          module_graph,
          None,
          module_graph_cache,
          side_effects_state,
          exports_info,
        ))
      }
      None => EDGE_STATE_ACTIVE,
    });
  }

  edges
}

/// Collect every module request once so that `requests` can be deduplicated.
/// The application graph that motivated this API classified 1.29 million
/// edges over 2481 distinct request strings, so the table is small compared
/// to the edge count.
fn collect_requests<'a>(
  module_identifiers: &[ModuleIdentifier],
  module_graph: &'a ModuleGraph,
) -> (Vec<String>, FxHashMap<&'a str, i32>) {
  let mut request_indices: FxHashMap<&'a str, i32> = FxHashMap::default();
  let mut requests: Vec<String> = Vec::new();

  for module_identifier in module_identifiers {
    for dependency_id in module_graph.get_outgoing_deps_in_order(module_identifier) {
      let Some(request) = internal::try_dependency_by_id(module_graph, dependency_id)
        .and_then(|dependency| dependency.as_module_dependency())
        .map(|dependency| dependency.request())
      else {
        continue;
      };
      if request_indices.contains_key(request) {
        continue;
      }
      request_indices.insert(request, requests.len() as i32);
      requests.push(request.to_string());
    }
  }

  (requests, request_indices)
}

fn collect_entry_modules(
  compilation: &Compilation,
  module_graph: &ModuleGraph,
  module_indices: &FxHashMap<ModuleIdentifier, u32>,
) -> Vec<u32> {
  let mut entry_modules = Vec::new();

  for entry_data in compilation.entries.values() {
    for dependency_id in entry_data.dependencies.iter() {
      let Some(connection) = module_graph.connection_by_dependency_id(dependency_id) else {
        continue;
      };
      let Some(index) = module_indices.get(connection.module_identifier()) else {
        continue;
      };
      entry_modules.push(*index);
    }
  }

  entry_modules
}

pub fn build_module_graph_snapshot(compilation: &Compilation) -> JsModuleGraphSnapshot {
  let module_graph = compilation.get_module_graph();

  let module_identifiers: Vec<ModuleIdentifier> =
    module_graph.modules_keys().copied().collect::<Vec<_>>();
  let module_indices: FxHashMap<ModuleIdentifier, u32> = module_identifiers
    .iter()
    .enumerate()
    .map(|(index, identifier)| (*identifier, index as u32))
    .collect();

  // `active_state` needs the exports info artifact to evaluate conditional
  // connections. When it is unavailable the per-connection getter reports
  // every connection as active, and so do we, rather than reporting a
  // different graph from a different phase.
  let exports_info_artifact = compilation.exports_info_artifact.try_read();
  let default_module_graph_cache: ModuleGraphCacheArtifact = Default::default();
  let module_graph_cache = compilation
    .module_graph_cache_artifact
    .try_read()
    .unwrap_or(&default_module_graph_cache);
  let side_effects_state_artifact = &compilation
    .build_module_graph_artifact
    .side_effects_state_artifact;
  let active_state_inputs = exports_info_artifact.map(|exports_info| {
    (
      module_graph_cache,
      side_effects_state_artifact,
      exports_info,
    )
  });

  let (requests, request_indices) = collect_requests(&module_identifiers, module_graph);

  let per_module: Vec<ModuleEdges> = module_identifiers
    .par_iter()
    .map(|module_identifier| {
      collect_module_edges(
        module_identifier,
        module_graph,
        &module_indices,
        &request_indices,
        active_state_inputs,
      )
    })
    .collect();

  let edge_count: usize = per_module.iter().map(|edges| edges.targets.len()).sum();
  let mut edge_offsets: Vec<u32> = Vec::with_capacity(module_identifiers.len() + 1);
  let mut edge_targets: Vec<u32> = Vec::with_capacity(edge_count);
  let mut edge_requests: Vec<i32> = Vec::with_capacity(edge_count);
  let mut edge_states: Vec<u8> = Vec::with_capacity(edge_count);

  edge_offsets.push(0);
  for edges in per_module {
    edge_targets.extend_from_slice(&edges.targets);
    edge_requests.extend_from_slice(&edges.requests);
    edge_states.extend_from_slice(&edges.states);
    edge_offsets.push(edge_targets.len() as u32);
  }

  let descriptors: Vec<(String, Option<String>, Option<String>)> = module_identifiers
    .par_iter()
    .map(|module_identifier| {
      let module = module_graph.module_by_identifier(module_identifier);
      (
        module_identifier.to_string(),
        module.and_then(|module| module.name_for_condition().map(|name| name.to_string())),
        module.and_then(|module| module.get_layer().cloned()),
      )
    })
    .collect();

  let mut identifiers = Vec::with_capacity(descriptors.len());
  let mut name_for_conditions = Vec::with_capacity(descriptors.len());
  let mut layers = Vec::with_capacity(descriptors.len());
  for (identifier, name_for_condition, layer) in descriptors {
    identifiers.push(identifier);
    name_for_conditions.push(name_for_condition);
    layers.push(layer);
  }

  let entry_modules = collect_entry_modules(compilation, module_graph, &module_indices);

  JsModuleGraphSnapshot {
    identifiers,
    name_for_conditions,
    layers,
    entry_modules: Uint32Array::new(entry_modules),
    edge_offsets: Uint32Array::new(edge_offsets),
    edge_targets: Uint32Array::new(edge_targets),
    edge_requests: Int32Array::new(edge_requests),
    edge_states: Uint8Array::new(edge_states),
    requests,
    exports_info_available: active_state_inputs.is_some(),
  }
}
