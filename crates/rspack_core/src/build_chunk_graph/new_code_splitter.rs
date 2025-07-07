use std::{
  borrow::Cow,
  hash::BuildHasherDefault,
  iter::once,
  sync::{atomic::AtomicU32, Arc},
};

use indexmap::IndexSet;
use rspack_collections::{
  DatabaseItem, IdentifierDashMap, IdentifierHasher, IdentifierIndexMap, IdentifierIndexSet,
  IdentifierMap, IdentifierSet, Ukey, UkeyMap, UkeySet,
};
use rspack_error::{error, Diagnostic, Result};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tokio::sync::mpsc::unbounded_channel;
use tracing::instrument;

use super::available_modules::AvailableModules;
use crate::{
  assign_depths,
  incremental::{IncrementalPasses, Mutation},
  merge_runtime, AsyncDependenciesBlockIdentifier, ChunkGroup, ChunkGroupKind, ChunkGroupUkey,
  ChunkLoading, Compilation, DependenciesBlock, DependencyLocation, EntryDependency, EntryRuntime,
  GroupOptions, ModuleDependency, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection,
  ModuleIdentifier, RuntimeSpec, SyntheticDependencyLocation,
};

type ModuleDeps = HashMap<
  RuntimeSpec,
  IdentifierDashMap<Arc<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)>>,
>;

static NEXT_EDGE_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Default)]
pub struct CodeSplitter {
  pub module_deps: ModuleDeps,
  pub module_ordinal: IdentifierMap<u64>,
}

#[derive(Default, Debug)]
struct FillCtx {
  pub chunk_modules: IdentifierSet,
  pub outgoings: IndexSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>,
  pub pre_order_indices: IdentifierMap<usize>,
  pub post_order_indices: IdentifierMap<usize>,
  pub module_ordinal: AvailableModules,
  pub next_pre_order_index: usize,
  pub next_post_order_index: usize,
  pub chunk_loading: bool,
  pub available_modules: Option<AvailableModules>,
  pub skipped: IdentifierSet,
}

#[derive(Debug, Clone)]
pub struct CreateChunkResult {
  pub available_modules: AvailableModules,
  pub pre_order_indices: IdentifierMap<usize>,
  pub post_order_indices: IdentifierMap<usize>,
  pub modules: IdentifierSet,
  pub modules_ordinal: AvailableModules,
  pub outgoings: IndexSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>,
  pub skipped: IdentifierSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edge {
  // entry
  Entry(String, Vec<ModuleIdentifier>, bool),

  // async entry
  // for example: `new Worker()`
  AsyncEntry(EdgeUkey, AsyncDependenciesBlockIdentifier, bool),

  // use normal `import()`
  Block(UkeySet<EdgeUkey>, AsyncDependenciesBlockIdentifier),

  // use `webpackChunkName`
  // a  b
  // \ / webpackChunkName: c.js
  //  c
  // The 2 edges from a and b is one MagicName edge
  MagicName(
    String,
    UkeySet<EdgeUkey>,
    Vec<AsyncDependenciesBlockIdentifier>,
  ),
}

impl Edge {
  fn name<'me>(&'me self, module_graph: &'me ModuleGraph) -> Option<&'me str> {
    match self {
      Edge::Entry(name, _, _) => Some(name.as_str()),
      Edge::AsyncEntry(_, ref block_id, _) => module_graph
        .block_by_id(block_id)
        .expect("should have block")
        .get_group_options()
        .and_then(|option| {
          option.name().or_else(|| {
            option
              .entry_options()
              .and_then(|option| option.name.as_deref())
          })
        }),
      Edge::Block(_, _) => None,
      Edge::MagicName(name, _, _) => Some(name.as_str()),
    }
  }

  fn block_options(&self, compilation: &Compilation) -> Option<GroupOptions> {
    let module_graph = compilation.get_module_graph();
    match self {
      Edge::Entry(entry, _, _) => {
        let entry = compilation.entries.get(entry).expect("should have entry");

        Some(GroupOptions::Entrypoint(Box::new(entry.options.clone())))
      }
      Edge::Block(_, block_id) | Edge::AsyncEntry(_, block_id, _) => {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block for block_id");
        block.get_group_options().cloned()
      }
      Edge::MagicName(_, _, blocks) => {
        let block_id = blocks[0];
        let block = module_graph
          .block_by_id(&block_id)
          .expect("should have block for block_id");
        block.get_group_options().cloned()
      }
    }
  }

  fn incoming_blocks(&self) -> Vec<AsyncDependenciesBlockIdentifier> {
    match self {
      Edge::Entry(_, _, _) => vec![],
      Edge::AsyncEntry(_, block, _) => vec![*block],
      Edge::Block(_, block) => vec![*block],
      Edge::MagicName(_, _, blocks) => blocks.clone(),
    }
  }

  fn root_modules(&self, module_graph: &ModuleGraph) -> Option<Vec<ModuleIdentifier>> {
    match self {
      Edge::Entry(_, modules, _) => Some(modules.clone()),
      Edge::AsyncEntry(_, block_id, _) => {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block");
        Some(
          block
            .get_dependencies()
            .iter()
            .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
            .copied()
            .collect(),
        )
      }
      Edge::Block(_, _) => None,
      Edge::MagicName(_, _, _) => None,
    }
  }

  fn add_parent(&mut self, parent: EdgeUkey) {
    match self {
      Edge::Entry(_, _, _) => {}
      Edge::AsyncEntry(_, _, _) => {}
      Edge::Block(from, _) => {
        from.insert(parent);
      }
      Edge::MagicName(_, from, _) => {
        from.insert(parent);
      }
    }
  }

  fn from(&self) -> Option<UkeySet<EdgeUkey>> {
    match self {
      Edge::Entry(_, _, _) => None,
      Edge::AsyncEntry(edge_ukey, _, _) => Some(once(*edge_ukey).collect()),
      Edge::Block(from, _) => Some(from.clone()),
      Edge::MagicName(_, from, _) => Some(from.clone()),
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct EdgeUkey(Ukey, std::marker::PhantomData<Edge>);

impl EdgeUkey {
  fn new() -> Self {
    Self(
      Ukey::new(NEXT_EDGE_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
      std::marker::PhantomData,
    )
  }
}

struct ScanModuleGraphResult {
  pub(crate) edges: UkeyMap<EdgeUkey, Edge>,
  pub(crate) entry_runtimes: HashMap<String, RuntimeSpec>,
  pub(crate) entry_edges: HashMap<String, EdgeUkey>,
  pub(crate) edge_parents: HashMap<EdgeUkey, HashSet<EdgeUkey>>,
  pub(crate) edge_for_block: HashMap<AsyncDependenciesBlockIdentifier, EdgeUkey>,
  pub(crate) runtime_for_edge: UkeyMap<EdgeUkey, RuntimeSpec>,
}

impl CodeSplitter {
  pub fn new(modules: impl Iterator<Item = ModuleIdentifier>) -> Self {
    let mut module_ordinal = IdentifierMap::default();
    for m in modules {
      if !module_ordinal.contains_key(&m) {
        module_ordinal.insert(m, module_ordinal.len() as u64);
      }
    }

    Self {
      module_deps: Default::default(),
      module_ordinal,
    }
  }

  #[instrument(skip_all)]
  fn scan_module_graph<'a>(
    &mut self,
    compilation: &'a mut Compilation,
  ) -> Result<ScanModuleGraphResult> {
    // determine runtime and chunkLoading
    let mut diagnostics = vec![];
    let mut entry_runtime: HashMap<String, RuntimeSpec> = HashMap::default();
    let mut entry_edges: HashMap<String, EdgeUkey> = Default::default();
    let mut runtime_for_edge: UkeyMap<EdgeUkey, RuntimeSpec> = Default::default();
    let mut edge_parents: HashMap<EdgeUkey, HashSet<EdgeUkey>> = HashMap::default();
    let mut edge_for_block: HashMap<AsyncDependenciesBlockIdentifier, EdgeUkey> =
      Default::default();
    let mut edges: UkeyMap<EdgeUkey, Edge> = Default::default();

    for entry in compilation.entries.keys() {
      let mut visited = vec![];
      if let Err(error) =
        Self::get_entry_runtime(entry, compilation, &mut entry_runtime, &mut visited)
      {
        diagnostics.push(Diagnostic::from(error));
        let tmp_runtime = once(ustr::Ustr::from(entry.as_str())).collect::<RuntimeSpec>();
        entry_runtime.insert(entry.clone(), tmp_runtime.clone());
      };
    }

    // iterate module graph to find block runtime and its parents
    // let mut blocks_with_runtime = HashMap::default();
    let mut stack: Vec<StackItem> = vec![];
    let mut visited = HashSet::default();
    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;
    let global_chunk_loading = &compilation.options.output.chunk_loading;
    let global_async_chunks = compilation.options.output.async_chunks;

    // normal `import()` or `require.ensure`
    let mut roots = HashMap::<AsyncDependenciesBlockIdentifier, EdgeUkey>::default();
    // webpackChunkName or entry
    let mut named_edges = HashMap::<String, EdgeUkey>::default();

    let global_deps = compilation.global_entry.dependencies.iter();
    let global_included_deps = compilation.global_entry.include_dependencies.iter();

    let mut next_idx = 0;
    let mut index_by_block = HashMap::<AsyncDependenciesBlockIdentifier, usize>::default();

    for (entry, entry_data) in &compilation.entries {
      let chunk_loading = !matches!(
        entry_data
          .options
          .chunk_loading
          .as_ref()
          .unwrap_or(global_chunk_loading),
        ChunkLoading::Disable
      ) && entry_data
        .options
        .async_chunks
        .unwrap_or(global_async_chunks);

      let runtime = entry_runtime
        .get(entry.as_str())
        .expect("already set runtime");

      self.module_deps.entry(runtime.clone()).or_default();

      let modules: Vec<ModuleIdentifier> = global_deps
        .clone()
        .chain(entry_data.dependencies.iter())
        .chain(global_included_deps.clone())
        .chain(entry_data.include_dependencies.iter())
        .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        .copied()
        .collect();

      let ukey = EdgeUkey::new();
      edges.insert(
        ukey,
        Edge::Entry(entry.clone(), modules.clone(), chunk_loading),
      );

      for module in &modules {
        stack.push(StackItem {
          module: *module,
          chunk_loading,
          edge: ukey,
        });
      }

      entry_edges.insert(entry.clone(), ukey);
      named_edges.insert(entry.clone(), ukey);
      runtime_for_edge.insert(ukey, runtime.clone());
    }

    struct StackItem {
      module: ModuleIdentifier,
      chunk_loading: bool,
      edge: EdgeUkey, // the edge that create current chunk
    }

    while let Some(item) = stack.pop() {
      let StackItem {
        module,
        chunk_loading,
        edge,
      } = item;

      let runtime = runtime_for_edge.get(&edge).expect("should have runtime");

      if !visited.insert((module, runtime.clone())) {
        continue;
      }

      let guard = self.outgoings_modules(&module, runtime, &module_graph, module_graph_cache);
      let (modules, blocks) = guard.value();

      let blocks = blocks.clone();
      for m in modules {
        stack.push(StackItem {
          module: *m,
          chunk_loading,
          edge: edge.clone(),
        });
      }

      for block_id in blocks {
        let runtime = runtime_for_edge.get(&edge).expect("should have runtime");

        index_by_block.entry(block_id).or_insert_with(|| {
          next_idx += 1;
          next_idx
        });

        let Some(block) = module_graph.block_by_id(&block_id) else {
          continue;
        };

        // when disable chunk loading, only async entrypoint can be created, disable normal chunk
        let entry_options = block
          .get_group_options()
          .and_then(|option| option.entry_options());
        let is_entry = entry_options.is_some();
        let should_create = chunk_loading || entry_options.is_some();
        let block = module_graph
          .block_by_id(&block_id)
          .expect("should have block");

        let child_chunk_loading = entry_options.map_or(chunk_loading, |opt| {
          !matches!(
            opt.chunk_loading.as_ref().unwrap_or(global_chunk_loading),
            ChunkLoading::Disable
          ) && opt.async_chunks.unwrap_or(global_async_chunks)
        });

        let new_edge = if should_create {
          if let Some(name) = block.get_group_options().and_then(|options| {
            options
              .name()
              .or_else(|| entry_options.and_then(|entry| entry.name.as_deref()))
          }) && let Some(prev) = named_edges.get(name)
          {
            // async entrypoint has unique runtime, do not merge runtime
            if !is_entry {
              let old_runtime = runtime_for_edge
                .get(prev)
                .expect("should have runtime info");

              let new_runtime = merge_runtime(&runtime, old_runtime);
              self.module_deps.entry(new_runtime.clone()).or_default();
              runtime_for_edge.insert(*prev, new_runtime);
            };

            match edges.get_mut(prev).expect("should have edge") {
              Edge::AsyncEntry(_, _, _) | Edge::Entry(_, _, _) => {
                if entry_options.is_some() {
                  diagnostics.push(Diagnostic::from(error!(
                    "Two entrypoints with the same name {}",
                    name
                  )));
                } else {
                  diagnostics.push(
                    Diagnostic::from(
                      error!(
                        format!("It's not allowed to load an initial chunk on demand. The chunk name \"{}\" is already used by an entrypoint.", name)
                      )
                    )
                  );
                }
              }
              Edge::MagicName(_, _from_edge, async_dependencies_block_identifiers) => {
                if async_dependencies_block_identifiers
                  .binary_search(&block_id)
                  .is_err()
                {
                  async_dependencies_block_identifiers.push(block_id);
                }
              }
              Edge::Block(_, _) => {
                unreachable!()
              }
            }

            let prev_edge = edges.get_mut(prev).expect("should have edge");
            prev_edge.add_parent(edge);

            *prev
          } else if let Some(prev) = roots.get(&block_id) {
            // already created
            if !is_entry {
              let old_runtime = runtime_for_edge
                .get(&prev)
                .expect("should have runtime info");
              let new_runtime = merge_runtime(&runtime, old_runtime);
              self.module_deps.entry(new_runtime.clone()).or_default();
              runtime_for_edge.insert(*prev, new_runtime);
            };

            let prev_edge = edges.get_mut(prev).expect("should have edge");
            prev_edge.add_parent(edge);

            *prev
          } else {
            let rt = entry_options
              .and_then(|entry_options| {
                RuntimeSpec::from_entry_options(entry_options).map(|rt| {
                  self.module_deps.entry(rt.clone()).or_default();
                  rt
                })
              })
              .unwrap_or_else(|| {
                runtime_for_edge
                  .get(&edge)
                  .expect("should have runtime for edge")
                  .clone()
              });

            let name = block.get_group_options().and_then(|options| {
              options.name().or_else(|| {
                options
                  .entry_options()
                  .and_then(|entry_options| entry_options.name.as_deref())
              })
            });
            let new_edge = if is_entry {
              let new_edge = Edge::AsyncEntry(edge, block_id, child_chunk_loading);
              let ukey = EdgeUkey::new();
              edges.insert(ukey, new_edge);
              ukey
            } else if let Some(name) = name {
              let new_edge = Edge::MagicName(
                name.to_string(),
                once(edge).collect(),
                once(block_id).collect(),
              );
              let ukey = EdgeUkey::new();
              edges.insert(ukey, new_edge);
              ukey
            } else {
              let new_edge = Edge::Block(once(edge).collect(), block_id);
              let ukey = EdgeUkey::new();
              edges.insert(ukey, new_edge);
              ukey
            };

            if let Some(name) = name {
              named_edges.insert(name.to_string(), new_edge);
            } else {
              roots.insert(block_id, new_edge);
            }

            runtime_for_edge.insert(new_edge, rt);

            new_edge
          }
        } else {
          edge
        };

        if let Some(from_edges) = edges.get(&new_edge).expect("should have from").from() {
          let parents = edge_parents.entry(new_edge).or_default();
          parents.extend(from_edges);
        };

        edge_for_block.insert(block_id, new_edge.clone());

        block
          .get_dependencies()
          .iter()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
          .for_each(|module| {
            stack.push(StackItem {
              module: *module,
              chunk_loading: child_chunk_loading,
              edge: new_edge.clone(),
            });
          });
      }
    }

    for edge in named_edges.values() {
      if let Edge::MagicName(_, _, blocks) = edges.get_mut(edge).expect("should have edge") {
        blocks.sort_by(|a, b| index_by_block.get(a).cmp(&index_by_block.get(b)));
      }
    }

    compilation.extend_diagnostics(diagnostics);

    Ok(ScanModuleGraphResult {
      edges,
      entry_edges,
      entry_runtimes: entry_runtime,
      edge_parents,
      runtime_for_edge,
      edge_for_block,
    })
  }

  fn get_entry_runtime<'a, 'b>(
    entry: &'a str,
    compilation: &'a Compilation,
    entry_runtime: &'b mut HashMap<String, RuntimeSpec>,
    visited: &'b mut Vec<&'a str>,
  ) -> Result<RuntimeSpec> {
    if visited.binary_search(&entry).is_ok() {
      return Err(error!(
        "Entrypoints '{}' and '{}' use 'dependOn' to depend on each other in a circular way.",
        visited.last().expect("has item"),
        entry
      ));
    }

    visited.push(entry);

    if let Some(runtime) = entry_runtime.get(entry) {
      return Ok(runtime.clone());
    }

    let entry_data = compilation.entries.get(entry).expect("should have entry");

    let runtime = if let Some(depend_on) = &entry_data.options.depend_on
      && !depend_on.is_empty()
    {
      if entry_data.options.runtime.is_some() {
        return Err(error!(
          "Entrypoint '{}' has 'dependOn' and 'runtime' specified",
          entry
        ));
      }
      let mut runtime: Option<RuntimeSpec> = None;
      for dep in depend_on {
        let other_runtime = Self::get_entry_runtime(dep, compilation, entry_runtime, visited)?;
        match &mut runtime {
          Some(runtime) => {
            runtime.extend(&other_runtime);
          }
          None => {
            runtime = Some(other_runtime);
          }
        }
      }
      runtime.expect("should have set")
    } else {
      RuntimeSpec::from_entry_options(&entry_data.options).expect("should have runtime")
    };

    entry_runtime.insert(entry.to_string(), runtime.clone());
    Ok(runtime)
  }

  fn get_module_ordinal(&self, m: ModuleIdentifier) -> u64 {
    *self
      .module_ordinal
      .get(&m)
      .unwrap_or_else(|| panic!("should have module ordinal: {m}"))
  }

  pub fn outgoings_modules(
    &self,
    module: &ModuleIdentifier,
    runtime: &RuntimeSpec,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Ref<ModuleIdentifier, (Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)> {
    let module_map: &dashmap::DashMap<
      rspack_collections::Identifier,
      (
        Vec<rspack_collections::Identifier>,
        Vec<AsyncDependenciesBlockIdentifier>,
      ),
      BuildHasherDefault<ustr::IdentityHasher>,
    > = self.module_deps.get(runtime).expect("should have value");

    let guard = module_map.get(module);
    if let Some(ref_value) = guard {
      return ref_value.clone();
    }

    let mut outgoings = IdentifierIndexMap::<Vec<&ModuleGraphConnection>>::default();
    let m = module_graph
      .module_by_identifier(module)
      .expect("should have module");

    m.get_dependencies()
      .iter()
      .filter(|dep_id| {
        module_graph
          .dependency_by_id(dep_id)
          .expect("should have dep")
          .as_module_dependency()
          .is_none_or(|module_dep| !module_dep.weak())
      })
      .filter_map(|dep| module_graph.connection_by_dependency_id(dep))
      .map(|conn| (conn.module_identifier(), conn))
      .for_each(|(module, conn)| outgoings.entry(*module).or_default().push(conn));

    let mut modules = IdentifierIndexSet::default();
    let mut blocks = m.get_blocks().to_vec();

    'outer: for (m, conns) in outgoings.iter() {
      for conn in conns {
        let conn_state = conn.active_state(module_graph, Some(runtime), module_graph_cache);
        match conn_state {
          crate::ConnectionState::Active(true) => {
            modules.insert(*m);
            continue 'outer;
          }
          crate::ConnectionState::TransitiveOnly => {
            let transitive = self.outgoings_modules(m, runtime, module_graph, module_graph_cache);
            let (extra_modules, extra_blocks) = transitive.as_ref();
            modules.extend(extra_modules.iter().copied());
            blocks.extend(extra_blocks.iter().copied());
          }
          crate::ConnectionState::Active(false) => {}
          crate::ConnectionState::CircularConnection => {}
        }
      }
    }

    module_map.insert(*module, Arc::new((modules.into_iter().collect(), blocks)));
    module_map.get(module).expect("have value").clone()
  }

  // insert static dependencies into a set
  fn fill_chunk_modules(
    &self,
    target_module: ModuleIdentifier,
    runtime: &RuntimeSpec,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    ctx: &mut FillCtx,
  ) {
    enum Task {
      Enter(ModuleIdentifier),
      Leave(ModuleIdentifier),
    }

    let mut stack = vec![Task::Enter(target_module)];

    while let Some(task) = stack.pop() {
      match task {
        Task::Enter(target_module) => {
          if let Some(ref available_modules) = ctx.available_modules
            && available_modules.is_module_available(self.get_module_ordinal(target_module))
          {
            ctx.skipped.insert(target_module);
            continue;
          }

          if !ctx.chunk_modules.insert(target_module) {
            continue;
          }

          let module = self.get_module_ordinal(target_module);
          if ctx.module_ordinal.is_module_available(module) {
            // we already include this module
            continue;
          } else {
            ctx.module_ordinal.add(module);
          }

          stack.push(Task::Leave(target_module));

          ctx
            .pre_order_indices
            .entry(target_module)
            .or_insert_with(|| {
              let value = ctx.next_pre_order_index;
              ctx.next_pre_order_index += 1;
              value
            });

          let guard =
            self.outgoings_modules(&target_module, runtime, module_graph, module_graph_cache);
          let (outgoing_modules, blocks) = guard.as_ref();
          let mut outgoing_modules = outgoing_modules.clone();

          if ctx.chunk_loading {
            ctx.outgoings.extend(blocks.clone());
          } else {
            let modules = blocks
              .iter()
              .filter_map(|block_id| module_graph.block_by_id(block_id))
              .flat_map(|block| {
                block.get_dependencies().iter().filter_map(|dep_id| {
                  module_graph
                    .module_identifier_by_dependency_id(dep_id)
                    .copied()
                })
              });
            outgoing_modules.extend(modules);
          }

          for m in outgoing_modules.iter().rev() {
            stack.push(Task::Enter(*m));
          }
        }
        Task::Leave(target_module) => {
          ctx
            .post_order_indices
            .insert(target_module, ctx.next_post_order_index);
          ctx.next_post_order_index += 1;
        }
      }
    }
  }

  #[tracing::instrument(skip_all)]
  fn set_order_index_and_group_index(&mut self, compilation: &mut Compilation) {
    enum Task<'compilation> {
      Group(ChunkGroupUkey, AsyncDependenciesBlockIdentifier),
      Enter((ModuleIdentifier, &'compilation RuntimeSpec)),
      Leave(ModuleIdentifier),
    }

    let mut queue = Vec::new();
    let mut queue_delay = Vec::new();
    let module_graph = compilation.get_module_graph();

    let mut chunk_group_index = 0;
    let mut pre_order_index = 0;
    let mut post_order_index = 0;

    let mut pre_order_indices = IdentifierMap::default();
    let mut post_order_indices = IdentifierMap::default();
    let mut chunk_group_indices = UkeyMap::default();

    let global_deps = compilation.global_entry.dependencies.iter();
    let global_included_deps = compilation.global_entry.include_dependencies.iter();

    for (name, entry) in &compilation.entries {
      let entrypoint_ukey = compilation
        .entrypoints
        .get(name.as_str())
        .expect("should have entrypoint");
      let entrypoint = compilation.chunk_group_by_ukey.expect_get(entrypoint_ukey);
      let entry_chunk = compilation.chunk_by_ukey.expect_get(&entrypoint.chunks[0]);
      let entry_chunk_runtime = entry_chunk.runtime();

      global_deps
        .clone()
        .chain(entry.dependencies.iter())
        .chain(global_included_deps.clone())
        .chain(entry.include_dependencies.iter())
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .for_each(|m| queue.push(Task::Enter((*m, entry_chunk_runtime))));

      chunk_group_index += 1;
      chunk_group_indices.insert(
        *compilation.entrypoints.get(name).expect("unreachable"),
        chunk_group_index,
      );
    }

    let mut visited = HashSet::default();
    let module_graph = compilation.get_module_graph();
    let module_graph_cache = &compilation.module_graph_cache_artifact;

    queue.reverse();

    while !queue.is_empty() {
      while let Some(task) = queue.pop() {
        match task {
          Task::Group(group_ukey, block_id) => {
            let module_graph = compilation.get_module_graph();
            let block = module_graph.block_by_id(&block_id);
            if let Some(block) = block {
              for dep_id in block.get_dependencies().iter().rev() {
                let Some(module) = module_graph.module_identifier_by_dependency_id(dep_id) else {
                  continue;
                };

                let entry_chunk = compilation
                  .chunk_group_by_ukey
                  .expect_get(&group_ukey)
                  .chunks[0];
                let entry_chunk = compilation.chunk_by_ukey.expect_get(&entry_chunk);
                queue.push(Task::Enter((*module, entry_chunk.runtime())));
              }
            }
            chunk_group_indices.entry(group_ukey).or_insert_with(|| {
              chunk_group_index += 1;
              chunk_group_index
            });
          }
          Task::Enter((m, runtime)) => {
            if !visited.insert((m, runtime)) {
              continue;
            }

            pre_order_indices.entry(m).or_insert_with(|| {
              let v = pre_order_index;
              pre_order_index += 1;
              queue.push(Task::Leave(m));
              v
            });

            if !self.module_deps.contains_key(runtime) {
              self.module_deps.insert(runtime.clone(), Default::default());
            }
            let guard = self.outgoings_modules(&m, runtime, &module_graph, module_graph_cache);
            let (modules, blocks) = guard.as_ref();

            for m in modules.iter().rev() {
              queue.push(Task::Enter((*m, runtime)));
            }

            for block_id in blocks {
              if let Some(chunk_group) = compilation
                .chunk_graph
                .get_block_chunk_group(block_id, &compilation.chunk_group_by_ukey)
              {
                queue_delay.push(Task::Group(chunk_group.ukey(), *block_id));
              }
            }
          }
          Task::Leave(m) => {
            post_order_indices.insert(m, post_order_index);
            post_order_index += 1;
          }
        }
      }

      if queue.is_empty() {
        queue_delay.reverse();
        queue = std::mem::take(&mut queue_delay);
      }
    }

    let mut module_graph = compilation.get_module_graph_mut();

    for (m, idx) in pre_order_indices {
      module_graph
        .module_graph_module_by_identifier_mut(&m)
        .expect("should have module")
        .pre_order_index = Some(idx);
    }
    for (m, idx) in post_order_indices {
      module_graph
        .module_graph_module_by_identifier_mut(&m)
        .expect("should have module")
        .post_order_index = Some(idx);
    }

    for (cg, index) in chunk_group_indices {
      let cg = compilation.chunk_group_by_ukey.expect_get_mut(&cg);
      cg.index = Some(index);
    }
  }

  fn create_chunk(
    &self,
    edge: &Edge,
    runtime: RuntimeSpec,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    available_modules: AvailableModules,
  ) -> CreateChunkResult {
    let (root_modules, chunk_loading) = match edge {
      Edge::Entry(_, modules, chunk_loading) => (modules.clone(), *chunk_loading),
      Edge::AsyncEntry(_, block_id, chunk_loading) => {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block");
        let modules = block
          .get_dependencies()
          .iter()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
          .copied()
          .collect::<Vec<_>>();

        (modules, *chunk_loading)
      }
      Edge::MagicName(_, _, blocks) => {
        let mut modules = vec![];
        for block_id in blocks.iter() {
          let block = module_graph
            .block_by_id(block_id)
            .expect("should have block");
          modules.extend(
            block
              .get_dependencies()
              .iter()
              .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
              .copied(),
          );
        }
        (modules, true)
      }
      Edge::Block(_, async_dependencies_block_identifier) => {
        let block = module_graph
          .block_by_id(&async_dependencies_block_identifier)
          .expect("should have block");
        let modules = block
          .get_dependencies()
          .iter()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
          .copied()
          .collect::<Vec<_>>();

        (modules, true)
      }
    };

    let mut ctx = FillCtx {
      available_modules: Some(available_modules.clone()),
      chunk_loading,
      ..Default::default()
    };

    for m in root_modules {
      self.fill_chunk_modules(m, &runtime, module_graph, module_graph_cache, &mut ctx);
    }

    CreateChunkResult {
      available_modules,
      modules: ctx.chunk_modules,
      post_order_indices: ctx.post_order_indices,
      pre_order_indices: ctx.pre_order_indices,
      modules_ordinal: ctx.module_ordinal,
      outgoings: ctx.outgoings,
      skipped: ctx.skipped,
    }
  }

  pub fn invalidate(&mut self, _affected: IdentifierSet) {
    // todo!()
  }

  fn update_chunk_result(
    &self,
    edge: &Edge,
    runtime: &RuntimeSpec,
    compilation: &Compilation,
    result: &mut CreateChunkResult,
  ) {
    let module_graph: ModuleGraph<'_> = compilation.get_module_graph();

    if matches!(edge, Edge::Entry(_, _, _)) {
      // for entry, check if we can skip more modules by depend_on
      let new_result = self.create_chunk(
        edge,
        runtime.clone(),
        &module_graph,
        &compilation.module_graph_cache_artifact,
        result.available_modules.clone(),
      );

      result.modules = new_result.modules;
      result.modules_ordinal = new_result.modules_ordinal;
      result.post_order_indices = new_result.post_order_indices;
      result.pre_order_indices = new_result.pre_order_indices;
    } else {
      // for async entry, we can check if skipped module should enter
      let chunk_loading = match edge {
        Edge::Entry(_, _, _) => unreachable!(),
        Edge::AsyncEntry(_, _, loading) => *loading,
        Edge::Block(_, _) => true,
        Edge::MagicName(_, _, _) => true,
      };
      let mut ctx = FillCtx {
        available_modules: Some(result.available_modules.clone()),
        chunk_modules: result.modules.clone(),
        module_ordinal: result.modules_ordinal.clone(),
        chunk_loading,
        outgoings: result.outgoings.clone(),
        skipped: result.skipped.clone(),
        ..Default::default()
      };

      for skipped in &result.skipped {
        if result
          .available_modules
          .is_module_available(self.get_module_ordinal(*skipped))
        {
          // still available
          continue;
        }

        ctx.skipped.remove(skipped);

        // if we reach here, means the module is not available, so we need to add it
        self.fill_chunk_modules(
          *skipped,
          runtime,
          &module_graph,
          &compilation.module_graph_cache_artifact,
          &mut ctx,
        );
      }

      result.modules = ctx.chunk_modules;
      result.modules_ordinal = ctx.module_ordinal;
      result.post_order_indices = ctx.post_order_indices;
      result.pre_order_indices = ctx.pre_order_indices;
      result.skipped = ctx.skipped;
      result.outgoings = ctx.outgoings;
    }
  }

  fn _available_modules(&self, available_modules: &AvailableModules) -> IdentifierSet {
    let mut modules = IdentifierSet::default();
    for (m, bit) in &self.module_ordinal {
      if available_modules.is_module_available(*bit) {
        modules.insert(*m);
      }
    }
    modules
  }

  #[instrument(skip_all)]
  async fn create_chunk_loop(&mut self, compilation: &mut Compilation) -> Result<()> {
    #[derive(Debug)]
    enum Task {
      Create(CreateChunk),
      Result(Option<UkeySet<EdgeUkey>>, EdgeUkey, CreateChunkResult),
      Check,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct CreateChunk {
      pub(crate) from: Option<UkeySet<EdgeUkey>>,
      pub(crate) edge: EdgeUkey,
      pub(crate) available_modules: AvailableModules,
      pub(crate) runtime: RuntimeSpec,
    }

    let (tx, mut rx) = unbounded_channel();
    let mut active = 0;

    let mut pending: HashMap<EdgeUkey, (UkeySet<EdgeUkey>, AvailableModules)> = Default::default();
    let mut executing: HashMap<EdgeUkey, AvailableModules> = Default::default();
    let mut done: HashMap<EdgeUkey, CreateChunkResult> = HashMap::default();

    let mut chunk_children: HashMap<EdgeUkey, IndexSet<EdgeUkey>> = Default::default();

    let ScanModuleGraphResult {
      entry_edges,
      edges,
      entry_runtimes,
      mut edge_parents,
      runtime_for_edge,
      edge_for_block,
    } = self.scan_module_graph(compilation)?;

    // calculate dependOn
    for (entry, data) in &compilation.entries {
      let Some(depend_on) = &data.options.depend_on else {
        continue;
      };

      let curr_edge = *entry_edges
        .get(entry.as_str())
        .expect("should have entry edge");

      for dep in depend_on {
        let Some(dep_edge) = entry_edges.get(dep) else {
          continue;
        };
        edge_parents.entry(curr_edge).or_default().insert(*dep_edge);
        chunk_children
          .entry(*dep_edge)
          .or_default()
          .insert(curr_edge);
      }
    }

    for (entry, entry_edge) in &entry_edges {
      if let Some(parents) = edge_parents.get(entry_edge)
        && !parents.is_empty()
      {
        continue;
      }

      active += 1;
      tx.send(Task::Create(CreateChunk {
        from: None,
        edge: *entry_edges.get(entry.as_str()).expect("should have edge"),
        available_modules: AvailableModules::default(),
        runtime: entry_runtimes
          .get(entry)
          .expect("should have runtime")
          .clone(),
      }))
      .expect("should send task");
    }

    loop {
      let task = rx.recv().await;
      active -= 1;
      match task {
        Some(Task::Create(create_chunk)) => {
          let CreateChunk {
            mut from,
            edge: edge_ukey,
            mut available_modules,
            runtime,
          } = create_chunk;

          let edge = edges.get(&edge_ukey).expect("should have edge");
          let is_entry = matches!(edge, Edge::Entry(_, _, _));

          if let Some(from) = from.clone()
            && let Some(parents) = edge_parents.get_mut(&edge_ukey)
          {
            for from in &from {
              parents.remove(from);
            }

            if !parents.is_empty() {
              match pending.get_mut(&edge_ukey) {
                Some((pending_parents, pending_available)) => {
                  pending_available.merge_available_modules(is_entry, &available_modules);
                  pending_parents.extend(from);
                }
                None => {
                  pending.insert(edge_ukey, (from, available_modules));
                }
              }
              active += 1;
              tx.send(Task::Check).expect("should send task");
              continue;
            }
          }

          // if we reach here, means all incoming edges should success, the from should be all incomings
          if let Some((pending_edges, pending_available_modules)) = pending.remove(&edge_ukey) {
            match &mut from {
              Some(from_edges) => {
                // we have pending edges, so we need to add them to from
                from_edges.extend(pending_edges.clone());
              }
              None => {
                // we don't have from, so we can use pending edges
                from = Some(pending_edges);
              }
            }
            available_modules.merge_available_modules(is_entry, &pending_available_modules);
          }

          if let Some(result) = done.get_mut(&edge_ukey) {
            active += 1;

            if result
              .available_modules
              .should_invalidate(is_entry, &available_modules)
            {
              // we have more accurate available modules, so we need to re-visit
              result
                .available_modules
                .merge_available_modules(is_entry, &available_modules);

              self.update_chunk_result(edge, &runtime, &compilation, result);

              tx.send(Task::Result(from.clone(), edge_ukey, result.clone()))
                .expect("should send task");
            } else {
              tx.send(Task::Check).expect("should send task");
            }
            continue;
          }

          let tx = tx.clone();
          if let Some(prev_available_modules) = executing.get(&edge_ukey) {
            available_modules.merge_available_modules(is_entry, prev_available_modules);
            executing.insert(edge_ukey, available_modules.clone());
          }

          struct Cell<T>(pub *const T);
          impl<T> Cell<T> {
            pub unsafe fn as_ref<'a>(self) -> &'a T {
              self.0.as_ref_unchecked()
            }
          }
          unsafe impl<T> Send for Cell<T> {}

          let this_ptr = Cell(self as *const CodeSplitter);
          let compilation_ptr = Cell(compilation as *const Compilation);
          let edges = Cell(&edges as *const UkeyMap<EdgeUkey, Edge>);

          active += 1;
          rayon::spawn(move || {
            // Safety:
            // we ensure captured value is available during spawn executing
            let (this, compilation, edges) =
              unsafe { (this_ptr.as_ref(), compilation_ptr.as_ref(), edges.as_ref()) };
            let mg = compilation.get_module_graph();
            let edge = edges.get(&edge_ukey).expect("should have edge");

            let res = Task::Result(
              from,
              edge_ukey,
              this.create_chunk(
                edge,
                runtime,
                &mg,
                &compilation.module_graph_cache_artifact,
                available_modules,
              ),
            );
            tx.send(res).expect("should send task");
          });
        }

        Some(Task::Result(from_edges, edge_ukey, create_result)) => {
          let edge = edges.get(&edge_ukey).expect("should have edge");

          if !done.contains_key(&edge_ukey) {
            done.insert(edge_ukey, create_result.clone());
          }

          // handle depend_on
          let mut created = UkeySet::default();
          if matches!(edge, Edge::Entry(_, _, _))
            && let Some(children) = chunk_children.get(&edge_ukey)
            && !children.is_empty()
          {
            let child_available_modules = create_result
              .available_modules
              .union(&create_result.modules_ordinal);

            for child in children {
              active += 1;
              created.insert(*child);
              tx.send(Task::Create(CreateChunk {
                from: Some(once(edge_ukey).collect()),
                edge: *child,
                available_modules: child_available_modules.clone(),
                runtime: runtime_for_edge
                  .get(child)
                  .expect("should have runtime")
                  .clone(),
              }))
              .expect("should send task");
            }
          }

          if let Some(parents) = from_edges {
            for parent in parents {
              chunk_children.entry(parent).or_default().insert(edge_ukey);
            }
          }

          if !create_result.outgoings.is_empty() {
            let child_available_modules = create_result
              .available_modules
              .union(&create_result.modules_ordinal);

            let mut has_children = false;
            for block in create_result.outgoings {
              let Some(outgoing) = edge_for_block.get(&block) else {
                continue;
              };

              let outgoing_edge = edges.get(outgoing).expect("should have item for ukey");

              if !created.insert(*outgoing) {
                continue;
              }

              has_children = true;

              let is_entry = matches!(
                outgoing_edge,
                Edge::Entry(_, _, _) | Edge::AsyncEntry(_, _, _)
              );

              active += 1;
              tx.send(Task::Create(CreateChunk {
                from: Some(once(edge_ukey).collect()),
                edge: *outgoing,
                available_modules: if is_entry {
                  AvailableModules::default()
                } else {
                  child_available_modules.clone()
                },
                runtime: runtime_for_edge
                  .get(outgoing)
                  .expect("should have runtime")
                  .clone(),
              }))
              .expect("should send task");
            }

            if !has_children {
              active += 1;
              tx.send(Task::Check).expect("should send task");
            }
          } else {
            active += 1;
            tx.send(Task::Check).expect("should send task");
          }
        }
        Some(Task::Check) => {
          // we still have tasks to be done
          if active != 0 {
            continue;
          }

          // don't have any tasks,
          if pending.is_empty() {
            break;
          }

          // if there is no tasks and we have pending task
          // there are 2 cases
          // 1.
          // there are cycle chunks, which parents can never be 0
          // 2.
          // there are dynamic imports exist in skipped modules,
          // Entry(import m) --> Home(import m as well), if m has dynamic import,
          // in the pre-scan phase we can find 2 parents for m, but actually it
          // has only 1 parent which is Entry, and skipped from Home
          let edge_ukey = pending
            .keys()
            .next()
            .expect("we have at least one item")
            .clone();

          let (pending_parents, available_modules) =
            pending.remove(&edge_ukey).expect("should have value");

          if let Some(parents) = edge_parents.get_mut(&edge_ukey) {
            // clear waiting incomings
            parents.clear();
          }

          active += 1;
          tx.send(Task::Create(CreateChunk {
            from: Some(pending_parents),
            edge: edge_ukey,
            available_modules: available_modules,
            runtime: runtime_for_edge
              .get(&edge_ukey)
              .expect("should have result")
              .clone(), // we don't have runtime here, so use default
          }))
          .expect("should send task");
        }
        None => {
          unreachable!();
        }
      }
    }

    let mut edge_to_cg = UkeyMap::<EdgeUkey, ChunkGroupUkey>::default();

    for (edge_ukey, chunk_desc) in done {
      let CreateChunkResult {
        pre_order_indices,
        post_order_indices,
        modules,
        available_modules: _,
        modules_ordinal: _,
        outgoings: _,
        skipped: _,
      } = chunk_desc;

      let edge = edges.get(&edge_ukey).expect("should have edge");

      let mg = compilation.get_module_graph();
      let name = edge.name(&mg).map(|name| name.to_string());
      let is_entry = matches!(edge, Edge::Entry(_, _, _));
      let is_async_entry = matches!(edge, Edge::AsyncEntry(_, _, _));

      let (chunk, created) = if let Some(ref name) = name {
        let (chunk, create) = Compilation::add_named_chunk(
          name.to_string(),
          &mut compilation.chunk_by_ukey,
          &mut compilation.named_chunks,
        );

        (chunk, create)
      } else {
        (Compilation::add_chunk(&mut compilation.chunk_by_ukey), true)
      };

      if created && let Some(mutations) = compilation.incremental.mutations_write() {
        mutations.add(Mutation::ChunkAdd { chunk });
      }

      let group_options = edge.block_options(compilation);

      let mut chunk_group = if is_entry || is_async_entry {
        ChunkGroup::new(ChunkGroupKind::new_entrypoint(
          !is_async_entry,
          Box::new(
            group_options
              .as_ref()
              .expect("should have entry options")
              .entry_options()
              .expect("should be entry options")
              .clone(),
          ),
        ))
      } else {
        ChunkGroup::new(ChunkGroupKind::Normal {
          options: edge
            .block_options(compilation)
            .and_then(|opt| opt.normal_options().cloned())
            .unwrap_or_default()
            .clone(),
        })
      };

      let chunk_group_ukey: ChunkGroupUkey = chunk_group.ukey();
      let runtime = runtime_for_edge
        .get(&edge_ukey)
        .expect("should have runtime")
        .clone();
      edge_to_cg.insert(edge_ukey, chunk_group_ukey);
      compilation.chunk_graph.add_chunk(chunk);
      chunk_group.connect_chunk(compilation.chunk_by_ukey.expect_get_mut(&chunk));

      chunk_group.module_pre_order_indices = pre_order_indices;
      chunk_group.module_post_order_indices = post_order_indices;

      if is_async_entry || is_entry {
        chunk_group.set_entrypoint_chunk(chunk);
      }

      if is_async_entry {
        compilation.async_entrypoints.push(chunk_group_ukey);
      }

      let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk);
      let chunk_ukey = chunk.ukey();
      chunk.set_runtime(runtime.clone());

      if let Some(name) = chunk_group.kind.name() {
        compilation
          .named_chunk_groups
          .insert(name.to_owned(), chunk_group_ukey);
      }

      if is_entry {
        let options = group_options
          .as_ref()
          .expect("should have option")
          .entry_options()
          .expect("is entry option");

        if let Some(EntryRuntime::String(ref entry_runtime)) = options.runtime {
          let (runtime_chunk_ukey, add) = Compilation::add_named_chunk(
            entry_runtime.into(),
            &mut compilation.chunk_by_ukey,
            &mut compilation.named_chunks,
          );

          if add && let Some(mutations) = compilation.incremental.mutations_write() {
            mutations.add(Mutation::ChunkAdd {
              chunk: runtime_chunk_ukey,
            });
          }

          let rt_chunk = if compilation.entries.contains_key(entry_runtime) {
            let name = name.as_ref().expect("should have name");
            compilation.push_diagnostic(Diagnostic::from(error!(
                  "Entrypoint '{name}' has a 'runtime' option which points to another entrypoint named '{entry_runtime}'.
It's not valid to use other entrypoints as runtime chunk.
Did you mean to use 'dependOn: \"{entry_runtime}\"' instead to allow using entrypoint '{name}' within the runtime of entrypoint '{entry_runtime}'? For this '{entry_runtime}' must always be loaded when '{name}' is used.
Or do you want to use the entrypoints '{name}' and '{entry_runtime}' independently on the same page with a shared runtime? In this case give them both the same value for the 'runtime' option. It must be a name not already used by an entrypoint."
                                ),
                ).with_chunk(Some(chunk_ukey.as_u32())));
            compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey)
          } else {
            let runtime_chunk = compilation
              .chunk_by_ukey
              .expect_get_mut(&runtime_chunk_ukey);
            runtime_chunk.set_prevent_integration(true);
            compilation.chunk_graph.add_chunk(runtime_chunk_ukey);
            compilation
              .chunk_by_ukey
              .expect_get_mut(&runtime_chunk_ukey)
          };
          chunk_group.unshift_chunk(rt_chunk);
          rt_chunk.set_runtime(runtime);
          rt_chunk.add_group(chunk_group_ukey);
          chunk_group.set_runtime_chunk(rt_chunk.ukey());
        } else if options.depend_on.is_none() {
          chunk_group.set_runtime_chunk(chunk_ukey);
        }

        let name = name.as_ref().expect("should have name");
        let entry_data = compilation
          .entries
          .get(name)
          .expect("should have entry data");
        let dependencies = compilation
          .global_entry
          .dependencies
          .iter()
          .chain(entry_data.dependencies.iter());
        let requests = dependencies
          .map(|dep_id| {
            let module_graph = compilation.get_module_graph();
            let dep = module_graph.dependency_by_id(dep_id);
            let mut request = None;
            if let Some(dep) = dep {
              if let Some(d) = dep.as_any().downcast_ref::<EntryDependency>() {
                request = Some(d.request().to_string());
              }
            }
            request
          })
          .collect::<Vec<_>>();
        for request in requests {
          let loc = Some(DependencyLocation::Synthetic(
            SyntheticDependencyLocation::new(name),
          ));
          chunk_group.add_origin(None, loc, request);
        }

        let mut assign_depths_map = IdentifierMap::default();
        let module_graph = compilation.get_module_graph();
        let modules = edge
          .root_modules(&module_graph)
          .expect("should have entry modules");

        assign_depths(
          &mut assign_depths_map,
          &compilation.get_module_graph(),
          modules.iter(),
        );

        let mut module_graph = compilation.get_module_graph_mut();
        for (m, depth) in assign_depths_map {
          module_graph.set_depth_if_lower(&m, depth);
        }
      } else if is_async_entry {
        chunk_group.set_runtime_chunk(chunk_ukey);
      }

      if let Some(filename) = group_options
        .as_ref()
        .and_then(|opt| opt.entry_options().and_then(|opt| opt.filename.as_ref()))
      {
        let chunk = compilation.chunk_by_ukey.expect_get_mut(&chunk_ukey);
        chunk.set_filename_template(Some(filename.clone()));

        if filename.has_hash_placeholder() && let Some(diagnostic) = compilation.incremental.disable_passes(
              IncrementalPasses::CHUNKS_RENDER,
              "Chunk filename that dependent on full hash",
              "chunk filename that dependent on full hash is not supported in incremental compilation",
            ) {
              if let Some(diagnostic) = diagnostic {
                compilation.push_diagnostic(diagnostic);
              }
              compilation.chunk_render_artifact.clear();
            }
      }

      for incoming in edge.incoming_blocks() {
        compilation
          .chunk_graph
          .connect_block_and_chunk_group(incoming, chunk_group_ukey);

        let module_graph = compilation.get_module_graph();
        if let Some(block) = module_graph.block_by_id(&incoming) {
          chunk_group.add_origin(Some(*block.parent()), block.loc(), block.request().clone());
        }
      }

      if is_entry || is_async_entry {
        let module_graph = compilation.get_module_graph();
        let entry_modules = if is_entry {
          let entry_data = compilation
            .entries
            .get(name.as_ref().expect("should have name"))
            .expect("should have entry data");

          compilation
            .global_entry
            .dependencies
            .iter()
            .chain(entry_data.dependencies.iter())
            .filter_map(|dep_id| {
              module_graph
                .module_identifier_by_dependency_id(dep_id)
                .copied()
            })
            .collect::<Vec<_>>()
        } else {
          let Edge::AsyncEntry(_, block, _) = edge else {
            unreachable!()
          };
          let block = module_graph.block_by_id(block).expect("should have block");
          block
            .get_dependencies()
            .iter()
            .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
            .copied()
            .collect::<Vec<_>>()
        };

        for m in entry_modules {
          compilation
            .chunk_graph
            .connect_chunk_and_entry_module(chunk_ukey, m, chunk_group_ukey);
        }
      }
      compilation.chunk_group_by_ukey.add(chunk_group);

      for m in modules {
        compilation
          .chunk_graph
          .connect_chunk_and_module(chunk_ukey, m);
      }
    }

    for entry in compilation.entries.keys() {
      let entrypoint_ukey = compilation
        .named_chunk_groups
        .get(entry)
        .expect("should have entrypoint");
      compilation
        .entrypoints
        .insert(entry.clone(), *entrypoint_ukey);
    }

    let mut runtime_errors = vec![];
    for name in compilation.entries.keys() {
      let options = &compilation
        .entries
        .get(name)
        .expect("should have entry")
        .options;

      if let Some(depend_on) = &options.depend_on {
        let ukey = compilation
          .entrypoints
          .get(name.as_str())
          .ok_or_else(|| error!("no entrypoints found"))?;

        let mut entry_point_runtime = None;
        let mut has_error = false;

        {
          let entry_point = compilation.chunk_group_by_ukey.expect_get(ukey);
          let entry_point_chunk = compilation
            .chunk_by_ukey
            .expect_get(&entry_point.get_entrypoint_chunk());
          let referenced_chunks =
            entry_point_chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey);

          for dep in depend_on {
            if let Some(dependency_ukey) = compilation.entrypoints.get(dep) {
              let dependency_chunk_ukey = compilation
                .chunk_group_by_ukey
                .expect_get(dependency_ukey)
                .get_entrypoint_chunk();
              if referenced_chunks.contains(&dependency_chunk_ukey) {
                runtime_errors.push(Diagnostic::from(
                error!(
                  "Entrypoints '{name}' and '{dep}' use 'dependOn' to depend on each other in a circular way."
                ),
              ).with_chunk(Some(entry_point.get_entrypoint_chunk().as_u32())));
                entry_point_runtime = Some(entry_point_chunk.ukey());
                has_error = true;
                break;
              }
            } else {
              panic!("Entry {name} depends on {dep}, but this entry was not found");
            }
          }
        }

        if has_error {
          let entry_point = compilation.chunk_group_by_ukey.expect_get_mut(ukey);
          entry_point.set_runtime_chunk(entry_point_runtime.expect("Should set runtime chunk"));
        }
      }
    }

    for (edge, children) in chunk_children {
      let chunk_group_ukey = edge_to_cg
        .get(&edge)
        .expect("should have chunk group for edge");

      for child in children {
        let child_cg = edge_to_cg.get(&child).unwrap_or_else(|| {
          let edge = edges.get(&child).expect("should have edge");
          panic!("should have chunk group for edge {:?}", edge)
        });

        let child_group = compilation.chunk_group_by_ukey.expect_get_mut(child_cg);

        let is_child_async_entrypoint =
          child_group.kind.is_entrypoint() && !child_group.is_initial();

        if !is_child_async_entrypoint {
          child_group.add_parent(*chunk_group_ukey);
        }

        let chunk_group = compilation
          .chunk_group_by_ukey
          .expect_get_mut(chunk_group_ukey);

        if is_child_async_entrypoint {
          chunk_group.add_async_entrypoint(*child_cg);
        } else {
          chunk_group.add_child(*child_cg);
        }
      }
    }

    self.set_order_index_and_group_index(compilation);

    compilation.extend_diagnostics(runtime_errors);

    Ok(())
  }
}

// main entry for code splitting
pub async fn code_split(compilation: &mut Compilation) -> Result<()> {
  // ensure every module have a cgm, webpack uses the same trick
  for m in compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect::<Vec<_>>()
  {
    compilation.chunk_graph.add_module(m);
  }

  let mutations = compilation
    .incremental
    .mutations_read(IncrementalPasses::BUILD_CHUNK_GRAPH);

  let module_graph: &ModuleGraph<'_> = &compilation.get_module_graph();

  let mut splitter = if !compilation
    .code_splitting_cache
    .new_code_splitter
    .module_ordinal
    .is_empty()
    && let Some(mutations) = mutations
  {
    let mut affected = mutations.get_affected_modules_with_module_graph(module_graph);
    let removed = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    affected.extend(removed);

    // reuse data from last computation
    let mut splitter = std::mem::take(&mut compilation.code_splitting_cache.new_code_splitter);
    splitter.invalidate(affected.into_iter().collect());
    splitter
  } else {
    CodeSplitter::new(compilation.get_module_graph().modules().keys().copied())
  };

  // fill chunks with its modules
  // splitter.create_chunks(compilation)?;
  splitter.create_chunk_loop(compilation).await?;

  compilation.code_splitting_cache.new_code_splitter = splitter;

  Ok(())
}
