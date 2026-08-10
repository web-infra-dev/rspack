use std::{collections::VecDeque, mem};

use futures::Future;
use itertools::Itertools;
use rspack_collections::{IdentifierIndexMap, IdentifierMap};
use rspack_error::Result;
use rspack_util::{fx_hash::FxIndexMap, tracing_preset::TRACING_BENCH_TARGET};
use rustc_hash::FxHashMap as HashMap;
use tracing::instrument;

use crate::{
  ArtifactExt, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkGroupKind, ChunkGroupUkey,
  ChunkUkey, Compilation, DependenciesBlock, EntryDependency, EntryOptions, Filename, GroupOptions,
  Logger, ModuleDependency, ModuleIdentifier, PublicPath,
  build_chunk_graph::code_splitter::{
    CodeSplitter, DependenciesBlockIdentifier, get_active_state_of_connections,
    prepare_module_connection_map,
  },
  fast_set,
  incremental::{IncrementalPasses, Mutation},
};

#[derive(Debug, Default)]
pub struct BuildChunkGraphArtifact {
  pub chunk_by_ukey: ChunkByUkey,
  pub chunk_graph: ChunkGraph,
  pub chunk_group_by_ukey: ChunkGroupByUkey,
  pub entrypoints: FxIndexMap<String, ChunkGroupUkey>,
  pub async_entrypoints: Vec<ChunkGroupUkey>,
  pub named_chunk_groups: HashMap<String, ChunkGroupUkey>,
  pub named_chunks: HashMap<String, ChunkUkey>,
  pub(crate) code_splitter: CodeSplitter,
  pub module_idx: IdentifierMap<(u32, u32)>,
  global_include_modules: Vec<ModuleIdentifier>,
  entry_include_modules: FxIndexMap<String, Vec<ModuleIdentifier>>,
}

impl BuildChunkGraphArtifact {
  pub(crate) fn set_code_splitter(&mut self, code_splitter: CodeSplitter) {
    fast_set(&mut self.code_splitter, code_splitter);
  }

  // we can skip rebuilding chunk graph if none of modules
  // has changed its outgoings
  // we don't need to check if module has changed its incomings
  // if it changes, the incoming module changes its outgoings as well
  fn can_skip_rebuilding(&self, this_compilation: &Compilation) -> bool {
    self.can_skip_rebuilding_legacy(this_compilation)
  }

  fn can_skip_rebuilding_legacy(&self, this_compilation: &Compilation) -> bool {
    let logger = this_compilation.get_logger("rspack.Compilation.codeSplittingCache");

    if !this_compilation.entries.keys().eq(
      this_compilation
        .build_chunk_graph_artifact
        .entrypoints
        .keys(),
    ) {
      logger.log("entrypoints change detected, rebuilding chunk graph");
      return false;
    }

    let module_graph = this_compilation.get_module_graph();
    for (name, entry) in &this_compilation.entries {
      let Some(previous_entrypoint) = self
        .entrypoints
        .get(name)
        .and_then(|ukey| self.chunk_group_by_ukey.get(ukey))
      else {
        logger.log(format!(
          "entrypoint missing from cached chunk graph: {name}"
        ));
        return false;
      };
      if !previous_entrypoint
        .kind
        .get_entry_options()
        .is_some_and(|options| same_entry_options_topology(options, &entry.options))
      {
        logger.log(format!("entrypoint options change detected: {name}"));
        return false;
      }

      let current_entry_modules = this_compilation
        .global_entry
        .dependencies
        .iter()
        .chain(&entry.dependencies)
        .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
        .copied()
        .unique();
      let previous_entry_modules = self
        .chunk_graph
        .get_chunk_entry_modules_with_chunk_group_iterable(
          &previous_entrypoint.get_entrypoint_chunk(),
        )
        .keys()
        .copied();

      if !current_entry_modules.eq(previous_entry_modules) {
        logger.log(format!("entrypoint modules change detected: {name}"));
        return false;
      }

      let current_entry_requests = this_compilation
        .global_entry
        .dependencies
        .iter()
        .chain(&entry.dependencies)
        .map(|dependency| {
          module_graph
            .dependency_by_id(dependency)
            .as_any()
            .downcast_ref::<EntryDependency>()
            .map(|dependency| dependency.request())
        });
      let previous_entry_requests = previous_entrypoint
        .origins()
        .iter()
        .map(|origin| origin.request.as_deref());

      if !current_entry_requests.eq(previous_entry_requests) {
        logger.log(format!("entrypoint origins change detected: {name}"));
        return false;
      }
    }

    let (global_include_modules, entry_include_modules) =
      collect_entry_include_modules(this_compilation);
    if self.global_include_modules != global_include_modules
      || self.entry_include_modules != entry_include_modules
    {
      logger.log("entrypoint include modules change detected, rebuilding chunk graph");
      return false;
    }

    let Some(mutations) = this_compilation
      .incremental
      .mutations_read(IncrementalPasses::BUILD_MODULE_GRAPH)
    else {
      logger.log("incremental for build module graph disabled, rebuilding chunk graph");
      // if disable incremental for build module graph phase, we can't skip rebuilding
      return false;
    };

    // if we have module removal, we can't skip rebuilding
    if mutations
      .iter()
      .any(|mutation| matches!(mutation, Mutation::ModuleRemove { .. }))
    {
      logger.log("module removal detected, rebuilding chunk graph");
      return false;
    }

    let module_graph_cache = &this_compilation.module_graph_cache_artifact;
    let affected_modules = mutations.get_affected_modules_with_module_graph(module_graph);
    let previous_modules_map = &this_compilation
      .build_chunk_graph_artifact
      .code_splitter
      .block_modules_runtime_map;

    if previous_modules_map.is_empty() {
      logger.log("no cache detected, rebuilding chunk graph");
      return false;
    }

    for module in affected_modules {
      let current_blocks = module_graph
        .module_by_identifier(&module)
        .expect("should have module")
        .get_blocks();
      let previous_blocks = self
        .code_splitter
        .prepared_blocks_map
        .get(&DependenciesBlockIdentifier::Module(module))
        .map(Vec::as_slice)
        .unwrap_or_default();

      if current_blocks != previous_blocks {
        logger.log(format!("module async blocks change detected: {module}"));
        return false;
      }

      for block_id in current_blocks {
        let block = module_graph.block_by_id_expect(block_id);
        // Nested async blocks are not currently constructed, but avoid
        // reusing an incomplete topology if support is added later.
        if !block.get_blocks().is_empty() {
          logger.log(format!("nested async blocks detected: {module}"));
          return false;
        }

        let Some(previous_chunk_group) = self
          .chunk_graph
          .get_block_chunk_group(block_id, &self.chunk_group_by_ukey)
        else {
          continue;
        };
        let same_group_options = match (block.get_group_options(), &previous_chunk_group.kind) {
          (None, ChunkGroupKind::Normal { options }) => options == &Default::default(),
          (Some(GroupOptions::ChunkGroup(current)), ChunkGroupKind::Normal { options }) => {
            current == options
          }
          (Some(GroupOptions::Entrypoint(current)), ChunkGroupKind::Entrypoint { options, .. }) => {
            current == options
          }
          _ => false,
        };

        if !same_group_options {
          logger.log(format!(
            "module async block options change detected: {module}"
          ));
          return false;
        }
      }

      let mut prepared_connections_by_block = HashMap::default();
      for connection in prepare_module_connection_map(module, module_graph).unwrap_or_default() {
        prepared_connections_by_block
          .entry(connection.block)
          .or_insert_with(Vec::new)
          .push(connection);
      }

      for block in std::iter::once(DependenciesBlockIdentifier::Module(module)).chain(
        current_blocks
          .iter()
          .copied()
          .map(DependenciesBlockIdentifier::AsyncDependenciesBlock),
      ) {
        let outgoings = prepared_connections_by_block
          .remove(&block)
          .unwrap_or_default()
          .into_iter()
          .filter(|connection| {
            get_active_state_of_connections(
              &connection.connections,
              None,
              module_graph,
              module_graph_cache,
              &this_compilation
                .build_module_graph_artifact
                .side_effects_state_artifact,
              &this_compilation.exports_info_artifact,
            )
            .is_not_false()
          })
          .map(|connection| connection.module)
          .collect::<Vec<_>>();

        let mut previous_modules = IdentifierIndexMap::default();
        let mut miss_in_previous = true;
        for modules in previous_modules_map.values() {
          let Some(outgoings) = modules.get(&block) else {
            continue;
          };
          miss_in_previous = false;

          for (outgoing, state, _) in outgoings.iter() {
            // Keep false connections to preserve source order.
            previous_modules
              .entry(*outgoing)
              .and_modify(|v| {
                if state.is_not_false() {
                  *v = *state;
                }
              })
              .or_insert(*state);
          }
        }

        if miss_in_previous
          && !(matches!(block, DependenciesBlockIdentifier::Module(_))
            && outgoings.is_empty()
            && self.chunk_graph.try_get_module_chunks(&module).is_some())
        {
          logger.log("new module detected, rebuilding chunk graph");
          return false;
        }

        if !previous_modules
          .iter()
          .filter(|(_, conn_state)| conn_state.is_not_false())
          .map(|(m, _)| *m)
          .eq(outgoings)
        {
          logger.log(format!("module outgoings change detected: {module}"));
          return false;
        }
      }
    }

    true
  }

  /// Reset cached chunks back to the initial render state.
  ///
  /// webpack creates fresh `Chunk` instances for every compilation, and
  /// `Chunk.rendered` starts as `false` in the constructor. Rspack can reuse
  /// cached chunks across incremental compilations, so we need to restore the
  /// same state before running the next sealing/rendering pipeline.
  fn reset_chunk_rendered_state(&mut self) {
    for chunk in self.chunk_by_ukey.values_mut() {
      chunk.set_rendered(false);
    }
  }

  fn reset_for_rebuild(&mut self) {
    self.chunk_by_ukey = Default::default();
    self.chunk_graph = Default::default();
    self.chunk_group_by_ukey = Default::default();
    self.entrypoints.clear();
    self.async_entrypoints.clear();
    self.named_chunk_groups.clear();
    self.named_chunks.clear();
    self.set_code_splitter(Default::default());
    self.module_idx.clear();
    self.global_include_modules.clear();
    self.entry_include_modules.clear();
  }
}

fn refresh_async_chunk_group_origins(compilation: &mut Compilation) {
  let mut current_locations =
    HashMap::<(ChunkGroupUkey, ModuleIdentifier, Option<String>), VecDeque<_>>::default();

  {
    let module_graph = compilation.get_module_graph();
    let artifact = &compilation.build_chunk_graph_artifact;

    for (module_identifier, module) in module_graph.modules() {
      for block_id in module.get_blocks() {
        let Some(chunk_group) = artifact
          .chunk_graph
          .get_block_chunk_group(block_id, &artifact.chunk_group_by_ukey)
        else {
          continue;
        };
        let block = module_graph.block_by_id_expect(block_id);
        current_locations
          .entry((
            chunk_group.ukey,
            *module_identifier,
            block.request().clone(),
          ))
          .or_default()
          .push_back(block.loc());
      }
    }
  }

  for (chunk_group_ukey, chunk_group) in compilation
    .build_chunk_graph_artifact
    .chunk_group_by_ukey
    .iter_mut()
  {
    for origin in chunk_group.origins_mut() {
      let Some(module_identifier) = origin.module else {
        continue;
      };
      let Some(locations) =
        current_locations.get_mut(&(*chunk_group_ukey, module_identifier, origin.request.clone()))
      else {
        continue;
      };
      if let Some(location) = locations.pop_front() {
        origin.loc = location;
      }
    }
  }
}

fn collect_entry_include_modules(
  compilation: &Compilation,
) -> (
  Vec<ModuleIdentifier>,
  FxIndexMap<String, Vec<ModuleIdentifier>>,
) {
  let module_graph = compilation.get_module_graph();
  let mut global_includes = compilation
    .global_entry
    .include_dependencies
    .iter()
    .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
    .copied()
    .collect::<Vec<_>>();
  global_includes.sort_unstable();
  global_includes.dedup();

  let entry_includes = compilation
    .entries
    .iter()
    .filter_map(|(name, entry)| {
      if entry.include_dependencies.is_empty() {
        return None;
      }

      let mut entry_includes = entry
        .include_dependencies
        .iter()
        .filter_map(|dependency| module_graph.module_identifier_by_dependency_id(dependency))
        .copied()
        .collect::<Vec<_>>();
      entry_includes.sort_unstable();
      entry_includes.dedup();

      Some((name.clone(), entry_includes))
    })
    .collect();

  (global_includes, entry_includes)
}

fn same_entry_options_topology(previous: &EntryOptions, current: &EntryOptions) -> bool {
  let EntryOptions {
    name: previous_name,
    runtime: previous_runtime,
    chunk_loading: previous_chunk_loading,
    wasm_loading: previous_wasm_loading,
    async_chunks: previous_async_chunks,
    public_path: previous_public_path,
    base_uri: previous_base_uri,
    filename: previous_filename,
    library: previous_library,
    depend_on: previous_depend_on,
    layer: previous_layer,
  } = previous;
  let EntryOptions {
    name: current_name,
    runtime: current_runtime,
    chunk_loading: current_chunk_loading,
    wasm_loading: current_wasm_loading,
    async_chunks: current_async_chunks,
    public_path: current_public_path,
    base_uri: current_base_uri,
    filename: current_filename,
    library: current_library,
    depend_on: current_depend_on,
    layer: current_layer,
  } = current;

  previous_name == current_name
    && previous_runtime == current_runtime
    && previous_chunk_loading == current_chunk_loading
    && previous_wasm_loading == current_wasm_loading
    && previous_async_chunks == current_async_chunks
    && same_public_path_shape(previous_public_path, current_public_path)
    && previous_base_uri == current_base_uri
    && same_filename_shape(previous_filename, current_filename)
    && previous_library == current_library
    && previous_depend_on == current_depend_on
    && previous_layer == current_layer
}

fn same_public_path_shape(previous: &Option<PublicPath>, current: &Option<PublicPath>) -> bool {
  match (previous, current) {
    (Some(PublicPath::Filename(previous)), Some(PublicPath::Filename(current))) => {
      same_filename(previous, current)
    }
    _ => previous == current,
  }
}

fn same_filename_shape(previous: &Option<Filename>, current: &Option<Filename>) -> bool {
  match (previous, current) {
    (Some(previous), Some(current)) => same_filename(previous, current),
    _ => previous == current,
  }
}

fn same_filename(previous: &Filename, current: &Filename) -> bool {
  previous == current || (previous.template().is_none() && current.template().is_none())
}

fn refresh_entrypoint_options(compilation: &mut Compilation) {
  let artifact = &mut compilation.build_chunk_graph_artifact;
  let mut requires_full_chunk_assets = false;

  for (name, entry) in &compilation.entries {
    let entrypoint_ukey = *artifact
      .entrypoints
      .get(name)
      .expect("cached entrypoint should exist");
    let entrypoint = artifact
      .chunk_group_by_ukey
      .expect_get_mut(&entrypoint_ukey);
    let entrypoint_chunk = entrypoint.get_entrypoint_chunk();
    let ChunkGroupKind::Entrypoint { options, .. } = &mut entrypoint.kind else {
      unreachable!("cached entrypoint should have entrypoint options");
    };
    **options = entry.options.clone();

    let filename = entry.options.filename.clone();
    requires_full_chunk_assets |= filename
      .as_ref()
      .is_some_and(Filename::has_hash_placeholder);
    artifact
      .chunk_by_ukey
      .expect_get_mut(&entrypoint_chunk)
      .set_filename_template(filename);
  }

  if requires_full_chunk_assets
    && let Some(diagnostic) = compilation.incremental.disable_passes(
      IncrementalPasses::CHUNK_ASSET,
      "Chunk filename that dependent on full hash",
      "chunk filename that dependent on full hash is not supported in incremental compilation",
    )
    && let Some(diagnostic) = diagnostic
  {
    compilation.push_diagnostic(diagnostic);
  }
}

#[instrument(name = "Compilation:code_splitting",target=TRACING_BENCH_TARGET, skip_all)]
pub(crate) async fn use_code_splitting_cache<'a, T, F>(
  compilation: &'a mut Compilation,
  task: T,
) -> Result<()>
where
  T: Fn(&'a mut Compilation) -> F,
  F: Future<Output = Result<&'a mut Compilation>>,
{
  compilation
    .build_chunk_graph_artifact
    .reset_chunk_rendered_state();

  if !compilation.incremental.enabled() {
    task(compilation).await?;
    return Ok(());
  }

  let incremental_code_splitting = compilation
    .incremental
    .passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let no_change = incremental_code_splitting
    && compilation
      .build_chunk_graph_artifact
      .can_skip_rebuilding(compilation);

  if no_change {
    refresh_async_chunk_group_origins(compilation);
    refresh_entrypoint_options(compilation);

    let module_idx = &compilation.build_chunk_graph_artifact.module_idx;
    let module_graph = compilation
      .build_module_graph_artifact
      .get_module_graph_mut();
    for (m, (pre, post)) in module_idx.iter() {
      let mgm = module_graph.module_graph_module_by_identifier_mut(m);
      mgm.pre_order_index = Some(*pre);
      mgm.post_order_index = Some(*post);
    }

    return Ok(());
  }

  // Cache is not used, clear recovered artifact to avoid stale chunk graph data.
  compilation.build_chunk_graph_artifact.reset_for_rebuild();

  let compilation = task(compilation).await?;
  let mg = compilation.get_module_graph();
  let mut map = IdentifierMap::default();
  for (mid, mgm) in mg.module_graph_modules() {
    let (Some(pre), Some(post)) = (mgm.pre_order_index, mgm.post_order_index) else {
      continue;
    };

    map.insert(*mid, (pre, post));
  }
  compilation.build_chunk_graph_artifact.module_idx = map;
  let (global_include_modules, entry_include_modules) = collect_entry_include_modules(compilation);
  compilation
    .build_chunk_graph_artifact
    .global_include_modules = global_include_modules;
  compilation.build_chunk_graph_artifact.entry_include_modules = entry_include_modules;
  Ok(())
}

impl ArtifactExt for BuildChunkGraphArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::BUILD_CHUNK_GRAPH;
  fn should_recover(incremental: &crate::incremental::Incremental) -> bool {
    incremental.passes_enabled(IncrementalPasses::BUILD_CHUNK_GRAPH)
  }
  fn recover(_incremental: &crate::incremental::Incremental, new: &mut Self, old: &mut Self) {
    new.code_splitter = mem::take(&mut old.code_splitter);
    rayon::scope(|s| {
      s.spawn(|_| new.chunk_by_ukey.clone_from(&old.chunk_by_ukey));
      s.spawn(|_| new.chunk_graph.clone_from(&old.chunk_graph));
      s.spawn(|_| new.chunk_group_by_ukey.clone_from(&old.chunk_group_by_ukey));

      s.spawn(|_| new.async_entrypoints.clone_from(&old.async_entrypoints));
      s.spawn(|_| new.named_chunk_groups.clone_from(&old.named_chunk_groups));
      s.spawn(|_| new.named_chunks.clone_from(&old.named_chunks));
      s.spawn(|_| {
        new.entrypoints.clone_from(&old.entrypoints);
        new.module_idx.clone_from(&old.module_idx);
        new
          .global_include_modules
          .clone_from(&old.global_include_modules);
        new
          .entry_include_modules
          .clone_from(&old.entry_include_modules);
      });
    });
  }
}
