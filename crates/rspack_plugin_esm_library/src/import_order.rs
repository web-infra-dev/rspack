use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_core::{
  ChunkUkey, Compilation, ConnectionState, Dependency, DependencyType, ImportAttributes,
  ImportPhase, ModuleIdentifier, RuntimeSpec,
};
use rspack_util::fx_hash::FxIndexMap;

use crate::chunk_link::{ChunkLinkContext, RawImportSource};

struct StaticDependency<'a> {
  dependency: &'a dyn Dependency,
  target: ModuleIdentifier,
  state: ConnectionState,
  source_order: i32,
  range_start: u32,
}

enum StaticImport<'a> {
  Dependency(StaticDependency<'a>),
  Native(i32, &'a RawImportSource),
}

enum ImportTask<'a> {
  Module(ModuleIdentifier, Option<&'a ImportAttributes>, bool),
  Native(&'a RawImportSource),
}

impl StaticImport<'_> {
  fn position(&self) -> (i32, u32) {
    match self {
      Self::Dependency(dependency) => (dependency.source_order, dependency.range_start),
      Self::Native(source_order, _) => (*source_order, 0),
    }
  }
}

fn is_static_evaluation_dependency(dependency: &dyn Dependency) -> bool {
  matches!(
    dependency.dependency_type(),
    DependencyType::EsmImport
      | DependencyType::EsmImportSpecifier
      | DependencyType::EsmExportImport
      | DependencyType::EsmExportImportedSpecifier
  ) && dependency.get_phase() == ImportPhase::Evaluation
}

fn static_dependencies<'a>(
  compilation: &'a Compilation,
  module: ModuleIdentifier,
  runtime: &RuntimeSpec,
) -> Vec<StaticDependency<'a>> {
  let module_graph = compilation.get_module_graph();
  module_graph
    .get_ordered_outgoing_connections(&module)
    .filter_map(|connection| {
      let dependency = module_graph.dependency_by_id(&connection.dependency_id);
      if !is_static_evaluation_dependency(dependency) {
        return None;
      }

      let source_order = dependency.source_order()?;
      let state = connection.active_state(
        module_graph,
        Some(runtime),
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &compilation.exports_info_artifact,
      );
      if state.is_false() {
        return None;
      }

      Some(StaticDependency {
        dependency,
        target: *connection.module_identifier(),
        state,
        source_order,
        range_start: dependency.range().map_or(u32::MAX, |range| range.start),
      })
    })
    .collect()
}

struct ImportOrder<'a> {
  compilation: &'a Compilation,
  chunk: ChunkUkey,
  runtime: &'a RuntimeSpec,
  requests: &'a FxIndexMap<RawImportSource, String>,
  native_requests: &'a IdentifierMap<Vec<(i32, RawImportSource)>>,
  positions: Vec<Option<usize>>,
  next_position: usize,
  visited: IdentifierSet,
}

impl<'a> ImportOrder<'a> {
  fn record(&mut self, request: RawImportSource) {
    if let Some(index) = self.requests.get_index_of(&request)
      && self.positions[index].is_none()
    {
      self.positions[index] = Some(self.next_position);
      self.next_position += 1;
    }
  }

  fn visit(
    &mut self,
    module: ModuleIdentifier,
    attributes: Option<&'a ImportAttributes>,
    transitive_only: bool,
  ) -> bool {
    let compilation = self.compilation;
    let module_graph = compilation.get_module_graph();
    let native_requests = self.native_requests;
    let requests = self.requests;
    let mut pending = vec![ImportTask::Module(module, attributes, transitive_only)];
    while let Some(task) = pending.pop() {
      let (module, attributes, transitive_only) = match task {
        ImportTask::Module(module, attributes, transitive_only) => {
          (module, attributes, transitive_only)
        }
        ImportTask::Native(request) => {
          self.record(request.clone());
          continue;
        }
      };
      let Some(target) = module_graph.module_by_identifier(&module) else {
        return false;
      };
      if target.as_external_module().is_some_and(|external| {
        external.resolve_external_type() != "module"
          || external.get_import_phase() != ImportPhase::Evaluation
      }) {
        return false;
      }

      if !transitive_only {
        if let Some(external) = target.as_external_module() {
          let attributes = attributes.map(|attributes| {
            format!(
              " with {}",
              serde_json::to_string(attributes).expect("import attributes are string pairs")
            )
          });
          let request =
            RawImportSource::Source((external.get_request().primary().to_string(), attributes));
          if requests.contains_key(&request) {
            self.record(request);
            continue;
          }
        }

        let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
        if chunk_graph.get_number_of_module_chunks(module) == 0 {
          continue;
        }
        let chunks = chunk_graph.get_module_chunks(module);
        if !chunks.contains(&self.chunk) {
          // A shared module can have multiple copies. Only use a chunk boundary
          // when the emitted requests identify the copy used by this chunk.
          let mut requested_chunks = chunks
            .iter()
            .copied()
            .filter(|chunk| requests.contains_key(&RawImportSource::Chunk(*chunk)));
          let requested_chunk = requested_chunks.next();
          if requested_chunks.next().is_none()
            && let Some(chunk) = requested_chunk
          {
            self.record(RawImportSource::Chunk(chunk));
          }
          continue;
        }
      }

      if !self.visited.insert(module) {
        continue;
      }
      if module_graph
        .get_ordered_outgoing_connections(&module)
        .any(|connection| {
          !is_static_evaluation_dependency(module_graph.dependency_by_id(&connection.dependency_id))
        })
      {
        return false;
      }

      let mut imports = static_dependencies(compilation, module, self.runtime)
        .into_iter()
        .map(StaticImport::Dependency)
        .collect::<Vec<_>>();
      if let Some(native_requests) = native_requests.get(&module) {
        imports.extend(
          native_requests
            .iter()
            .map(|(position, request)| StaticImport::Native(*position, request)),
        );
      }
      imports.sort_by_key(StaticImport::position);
      pending.extend(imports.into_iter().rev().map(|import| match import {
        StaticImport::Dependency(dependency) => ImportTask::Module(
          dependency.target,
          dependency.dependency.get_attributes(),
          dependency.state == ConnectionState::TransitiveOnly,
        ),
        StaticImport::Native(_, request) => ImportTask::Native(request),
      }));
    }
    true
  }
}

fn has_only_static_evaluation_dependencies(compilation: &Compilation, chunk: ChunkUkey) -> bool {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  chunk_graph
    .get_chunk_modules_identifier(&chunk)
    .iter()
    .all(|module| {
      module_graph
        .module_by_identifier(module)
        .is_some_and(|module| {
          module.as_external_module().is_none_or(|external| {
            external.resolve_external_type() == "module"
              && external.get_import_phase() == ImportPhase::Evaluation
          })
        })
        && module_graph
          .get_ordered_outgoing_connections(module)
          .all(|connection| {
            is_static_evaluation_dependency(
              module_graph.dependency_by_id(&connection.dependency_id),
            )
          })
    })
}

fn can_reorder_chunk(compilation: &Compilation, chunk: ChunkUkey) -> bool {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let modules = chunk_graph.get_chunk_modules_identifier(&chunk);
  let mut roots = chunk_graph
    .get_chunk_entry_modules(&chunk)
    .into_iter()
    .collect::<IdentifierSet>();

  for module in modules {
    let Some(target) = module_graph.module_by_identifier(module) else {
      return false;
    };
    if target.as_external_module().is_some() {
      return false;
    }

    for connection in module_graph.get_ordered_outgoing_connections(module) {
      let dependency = module_graph.dependency_by_id(&connection.dependency_id);
      if !modules.contains(connection.module_identifier())
        || !is_static_evaluation_dependency(dependency)
      {
        return false;
      }
    }

    for connection in module_graph.get_incoming_connections(module) {
      if connection.original_module_identifier.is_none_or(|origin| {
        chunk_graph.get_number_of_module_chunks(origin) == 0
          || chunk_graph
            .get_module_chunks(origin)
            .iter()
            .any(|origin_chunk| *origin_chunk != chunk)
      }) {
        roots.insert(*module);
      }
    }
  }

  // Only a closed region with a unique entry can move as one request. Other
  // chunks may already combine source modules with interleaved dependencies,
  // or participate in a cycle whose first entry determines evaluation order.
  // Retain the original rendering order for chunks with these boundaries.
  // Check every dependency kind, including inactive connections: native
  // re-export fragments can survive their graph connections being disabled.
  // This eligibility check does not make these dependencies eager.
  roots.len() == 1
}

fn entry_roots(compilation: &Compilation, chunk: ChunkUkey) -> Option<Vec<ModuleIdentifier>> {
  let module_graph = compilation.get_module_graph();
  let artifact = &compilation.build_chunk_graph_artifact;
  let mut roots = Vec::new();
  let mut entrypoint_context = None;

  // Included modules are available to the runtime without being execution
  // roots. Optimizations can move them between chunks, so do not derive an
  // execution order from entry metadata in a compilation containing includes.
  if !compilation.global_entry.include_dependencies.is_empty()
    || compilation
      .entries
      .values()
      .any(|entry| !entry.include_dependencies.is_empty())
  {
    return None;
  }

  for (name, entrypoint) in &artifact.entrypoints {
    if artifact
      .chunk_group_by_ukey
      .expect_get(entrypoint)
      .get_entrypoint_chunk()
      != chunk
    {
      continue;
    }
    if entrypoint_context.replace(*entrypoint).is_some() {
      return None;
    }

    // The first item of an array entry can live in another chunk. Keep its
    // position instead of restricting the roots to modules owned by this chunk.
    roots.extend(
      compilation
        .global_entry
        .dependencies
        .iter()
        .chain(&compilation.entries[name].dependencies)
        .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
        .copied(),
    );
  }

  for entrypoint in &artifact.async_entrypoints {
    let group = artifact.chunk_group_by_ukey.expect_get(entrypoint);
    if group.get_entrypoint_chunk() != chunk {
      continue;
    }
    if entrypoint_context.replace(*entrypoint).is_some() {
      return None;
    }
    // Async entry blocks schedule their own modules and global entries through
    // a stack, unlike ordinary entries. Once their roots have been split across
    // chunks, the final group order cannot recover that execution sequence.
    if !compilation.global_entry.dependencies.is_empty() {
      return None;
    }
    for entry_chunk in &group.chunks {
      roots.extend(
        artifact
          .chunk_graph
          .get_chunk_entry_modules_with_chunk_group_iterable(entry_chunk)
          .iter()
          .filter(|(_, module_entrypoint)| *module_entrypoint == entrypoint)
          .map(|(module, _)| *module),
      );
    }
    // A named async entrypoint can be shared by several originating blocks.
    // Only a configured ordinary entry supplies an ordered array of roots.
    if roots.len() != 1 {
      return None;
    }
  }

  if let Some(entrypoint) = entrypoint_context
    && artifact
      .chunk_graph
      .get_chunk_entry_modules_with_chunk_group_iterable(&chunk)
      .values()
      .any(|module_entrypoint| *module_entrypoint != entrypoint)
  {
    return None;
  }

  if roots.is_empty() {
    roots.extend(artifact.chunk_graph.get_chunk_entry_modules(&chunk));
    // Optimizations can place independent entry modules in a shared chunk.
    // Their insertion order does not establish which entry will execute first.
    if roots.len() > 1 {
      return None;
    }
  }
  Some(roots)
}

fn incoming_roots(
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  runtime: &RuntimeSpec,
) -> Option<IdentifierSet> {
  let module_graph = compilation.get_module_graph();
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let mut roots = IdentifierSet::default();

  for module in chunk_link
    .hoisted_modules
    .iter()
    .chain(&chunk_link.decl_modules)
  {
    for connection in module_graph.get_incoming_connections(module) {
      let Some(origin) = connection.original_module_identifier else {
        continue;
      };
      if chunk_graph.get_number_of_module_chunks(origin) == 0
        || !chunk_graph
          .get_module_chunks(origin)
          .iter()
          .any(|chunk| *chunk != chunk_link.chunk)
      {
        continue;
      }

      if !connection.is_target_active(
        module_graph,
        Some(runtime),
        &compilation.module_graph_cache_artifact,
        &compilation
          .build_module_graph_artifact
          .side_effects_state_artifact,
        &compilation.exports_info_artifact,
      ) {
        continue;
      }
      let dependency = module_graph.dependency_by_id(&connection.dependency_id);
      // A normal dynamic import identifies an entry into the loaded chunk; it
      // does not make its target an eager dependency of the importing module.
      if !(is_static_evaluation_dependency(dependency)
        || matches!(dependency.dependency_type(), DependencyType::DynamicImport)
          && dependency.get_phase() == ImportPhase::Evaluation)
      {
        return None;
      }
      roots.insert(*module);
    }
  }
  Some(roots)
}

fn local_roots(
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  runtime: &RuntimeSpec,
) -> Vec<ModuleIdentifier> {
  let module_graph = compilation.get_module_graph();
  let local_modules = chunk_link
    .hoisted_modules
    .iter()
    .copied()
    .collect::<IdentifierSet>();
  let mut referenced = IdentifierSet::default();

  for module in &chunk_link.hoisted_modules {
    let mut pending = vec![*module];
    let mut visited = IdentifierSet::default();
    while let Some(module) = pending.pop() {
      if !visited.insert(module) {
        continue;
      }
      for dependency in static_dependencies(compilation, module, runtime) {
        if dependency.state == ConnectionState::TransitiveOnly {
          pending.push(dependency.target);
        } else if local_modules.contains(&dependency.target) {
          referenced.insert(dependency.target);
        }
      }
    }
  }

  // A component without a root (a cycle with no known entry) is deliberately
  // left unmapped; choosing an arbitrary cycle root could change its order.
  chunk_link
    .hoisted_modules
    .iter()
    .filter(|module| {
      !referenced.contains(*module)
        && module_graph
          .module_by_identifier(module)
          .is_some_and(|module| module.as_external_module().is_none())
    })
    .copied()
    .collect()
}

/// Returns a complete request order only when static evaluation dependencies
/// explain every request. Otherwise the renderer must retain its original
/// stages: an opaque dependency can evaluate another request before that
/// request's directly observed declaration.
pub(crate) fn ordered_imports(
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  requests: &FxIndexMap<RawImportSource, String>,
  native_requests: &IdentifierMap<Vec<(i32, RawImportSource)>>,
) -> Option<Vec<RawImportSource>> {
  if requests.len() < 2 {
    return None;
  }
  // A direct static declaration can share its request with an earlier opaque
  // dependency. Mapping the declaration alone would not establish the first
  // evaluation of that request, so require the whole chunk to be understood.
  if !has_only_static_evaluation_dependencies(compilation, chunk_link.chunk) {
    return None;
  }
  if requests.keys().any(|request| {
    matches!(request, RawImportSource::Chunk(chunk) if !can_reorder_chunk(compilation, *chunk))
  }) {
    return None;
  }

  let chunk = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&chunk_link.chunk);
  let runtime = chunk.runtime();
  let mut roots = entry_roots(compilation, chunk_link.chunk)?;
  if roots.is_empty() {
    roots.extend(incoming_roots(compilation, chunk_link, runtime)?);
    if roots.is_empty() {
      roots = local_roots(compilation, chunk_link, runtime);
    }
    // Several modules of a shared chunk can be entered in different orders by
    // its importers. Without a unique entry, retain the existing output order.
    if roots.len() != 1 {
      return None;
    }
  }
  let mut order = ImportOrder {
    compilation,
    chunk: chunk_link.chunk,
    runtime,
    requests,
    native_requests,
    positions: vec![None; requests.len()],
    next_position: 0,
    visited: IdentifierSet::default(),
  };
  for root in roots {
    if !order.visit(root, None, false) {
      return None;
    }
  }

  if order.positions.iter().any(Option::is_none) {
    return None;
  }
  let mut indices = (0..requests.len()).collect::<Vec<_>>();
  indices.sort_unstable_by_key(|index| order.positions[*index]);
  Some(
    indices
      .into_iter()
      .map(|index| {
        requests
          .get_index(index)
          .expect("request indices remain a permutation")
          .0
          .clone()
      })
      .collect(),
  )
}
