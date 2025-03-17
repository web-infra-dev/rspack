use std::borrow::Cow;
use std::hash::BuildHasherDefault;
use std::iter::once;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use dashmap::mapref::one::Ref;
use indexmap::IndexSet;
use num_bigint::BigUint;
use rayon::prelude::*;
use rspack_collections::{
  DatabaseItem, IdentifierDashMap, IdentifierHasher, IdentifierIndexMap, IdentifierIndexSet,
  IdentifierMap, IdentifierSet, Ukey, UkeyMap,
};
use rspack_error::Result;
use rspack_error::{error, Diagnostic};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::remove_available_modules::remove_available_modules;
use crate::incremental::{IncrementalPasses, Mutation};
use crate::{
  assign_depths, merge_runtime, AsyncDependenciesBlockIdentifier, Chunk, ChunkGroup,
  ChunkGroupKind, ChunkGroupOptions, ChunkGroupUkey, ChunkLoading, ChunkUkey, Compilation,
  DependenciesBlock, DependencyLocation, EntryData, EntryDependency, EntryOptions, EntryRuntime,
  GroupOptions, ModuleDependency, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
  RuntimeSpec, SyntheticDependencyLocation,
};

type ModuleDeps = HashMap<
  RuntimeSpec,
  IdentifierDashMap<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)>,
>;

static NEXT_CACHE_UKEY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct CacheUkey(Ukey, std::marker::PhantomData<CacheableChunkItem>);

impl CacheUkey {
  fn new() -> Self {
    Self(
      Ukey::new(NEXT_CACHE_UKEY.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
      std::marker::PhantomData,
    )
  }
}

#[derive(Debug, Clone)]
pub struct CacheableChunkItem {
  pub cache_ukey: CacheUkey,
  pub chunk_desc: ChunkDesc,
}

#[derive(Debug, Default)]
pub struct CodeSplitter {
  cache_chunk_desc: HashMap<CreateChunkRoot, Vec<CacheableChunkItem>>,
  cache_chunks: UkeyMap<CacheUkey, Chunk>,

  pub module_deps: ModuleDeps,
  pub module_deps_without_runtime:
    IdentifierDashMap<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)>,
  pub module_ordinal: IdentifierMap<u64>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum CreateChunkRoot {
  Entry(String, EntryData, Option<RuntimeSpec>),
  Block(AsyncDependenciesBlockIdentifier, Option<RuntimeSpec>),
}

struct FinalizeChunksResult {
  chunks: Vec<(bool, CacheableChunkItem)>,
  chunk_children: Vec<Vec<usize>>,
}

// Description about how to create chunk
#[derive(Debug, Clone)]
pub enum ChunkDesc {
  // Name, entry_modules, Options, Modules
  Entry(Box<EntryChunkDesc>),

  // original, root, Options, Modules
  Chunk(Box<NormalChunkDesc>),
}

impl ChunkDesc {
  fn name(&self) -> Option<&str> {
    match self {
      ChunkDesc::Entry(entry) => entry.entry.as_deref(),
      ChunkDesc::Chunk(chunk) => chunk.options.as_ref().and_then(|opt| opt.name.as_deref()),
    }
  }

  fn outgoings(&self) -> impl Iterator<Item = &AsyncDependenciesBlockIdentifier> {
    match self {
      ChunkDesc::Entry(entry) => entry.outgoing_blocks.iter(),
      ChunkDesc::Chunk(chunk) => chunk.outgoing_blocks.iter(),
    }
  }

  pub(crate) fn chunk_modules_ordinal(&self) -> &BigUint {
    match self {
      ChunkDesc::Entry(entry) => &entry.modules_ordinal,
      ChunkDesc::Chunk(chunk) => &chunk.modules_ordinal,
    }
  }

  pub(crate) fn available_modules_mut(&mut self) -> &mut BigUint {
    match self {
      ChunkDesc::Entry(entry) => &mut entry.available_modules,
      ChunkDesc::Chunk(chunk) => &mut chunk.available_modules,
    }
  }
}

#[derive(Debug, Clone)]
pub struct EntryChunkDesc {
  pub initial: bool,

  options: EntryOptions,
  modules_ordinal: BigUint,
  available_modules: BigUint,
  entry: Option<String>,
  entry_modules: Vec<ModuleIdentifier>,
  chunk_modules: IdentifierSet,

  pre_order_indices: IdentifierMap<usize>,
  post_order_indices: IdentifierMap<usize>,

  // use incoming and outgoing to track chunk relations,
  // entry has no incomings
  outgoing_blocks: IndexSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>,
  incoming_blocks: HashSet<AsyncDependenciesBlockIdentifier>,

  runtime: Option<RuntimeSpec>,
}

#[derive(Debug, Clone)]
pub struct NormalChunkDesc {
  options: Option<ChunkGroupOptions>,
  modules_ordinal: BigUint,
  available_modules: BigUint,
  chunk_modules: IdentifierSet,

  pre_order_indices: IdentifierMap<usize>,
  post_order_indices: IdentifierMap<usize>,

  // use incoming and outgoing to track chunk relations
  incoming_blocks: HashSet<AsyncDependenciesBlockIdentifier>,
  outgoing_blocks: IndexSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>,
  runtime: Option<RuntimeSpec>,
}

#[derive(Default, Debug)]
struct FillCtx {
  pub chunk_modules: IdentifierSet,
  pub out_goings: IndexSet<AsyncDependenciesBlockIdentifier, BuildHasherDefault<IdentifierHasher>>,
  pub pre_order_indices: IdentifierMap<usize>,
  pub post_order_indices: IdentifierMap<usize>,
  pub module_ordinal: BigUint,
  pub next_pre_order_index: usize,
  pub next_post_order_index: usize,
  pub chunk_loading: bool,
}

impl CreateChunkRoot {
  pub(crate) fn get_runtime(&self) -> Option<&RuntimeSpec> {
    match self {
      CreateChunkRoot::Entry(_, _, rt) => rt.as_ref(),
      CreateChunkRoot::Block(_, rt) => rt.as_ref(),
    }
  }

  fn set_runtime(&mut self, runtime: RuntimeSpec) {
    match self {
      CreateChunkRoot::Entry(_, _, rt) => *rt = Some(runtime),
      CreateChunkRoot::Block(_, rt) => *rt = Some(runtime),
    }
  }

  fn create(&self, splitter: &CodeSplitter, compilation: &Compilation) -> Vec<ChunkDesc> {
    let module_graph = compilation.get_module_graph();

    match self {
      CreateChunkRoot::Entry(entry, data, runtime) => {
        let mut entry_modules = vec![];

        let deps = compilation
          .global_entry
          .dependencies
          .iter()
          .chain(data.dependencies.iter());

        let mut ctx = FillCtx::default();
        let chunk_loading = data
          .options
          .chunk_loading
          .as_ref()
          .unwrap_or(&compilation.options.output.chunk_loading);
        let async_chunks = data
          .options
          .async_chunks
          .unwrap_or(compilation.options.output.async_chunks);
        ctx.chunk_loading = !matches!(chunk_loading, ChunkLoading::Disable) && async_chunks;

        for m in deps
          .clone()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        {
          entry_modules.push(*m);
        }

        for m in deps
          .chain(
            compilation
              .global_entry
              .include_dependencies
              .iter()
              .chain(data.include_dependencies.iter()),
          )
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        {
          splitter.fill_chunk_modules(*m, runtime.as_ref(), &module_graph, &mut ctx);
        }

        vec![ChunkDesc::Entry(Box::new(EntryChunkDesc {
          initial: true,
          entry: Some(entry.clone()),
          entry_modules,
          chunk_modules: ctx.chunk_modules,
          pre_order_indices: ctx.pre_order_indices,
          post_order_indices: ctx.post_order_indices,
          options: data.options.clone(),
          available_modules: BigUint::ZERO,
          modules_ordinal: ctx.module_ordinal,
          incoming_blocks: Default::default(),
          outgoing_blocks: ctx.out_goings,
          runtime: runtime.clone(),
        }))]
      }
      CreateChunkRoot::Block(block_id, runtime) => {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block");

        let mut chunks = vec![];
        let mut ctx = FillCtx {
          chunk_loading: if let Some(entry_option) = block
            .get_group_options()
            .and_then(|opt| opt.entry_options())
          {
            let chunk_loading = entry_option
              .chunk_loading
              .as_ref()
              .unwrap_or(&compilation.options.output.chunk_loading);
            let async_chunks = entry_option
              .async_chunks
              .unwrap_or(compilation.options.output.async_chunks);

            !matches!(chunk_loading, ChunkLoading::Disable) && async_chunks
          } else {
            true
          },
          ..Default::default()
        };

        for dep_id in block.get_dependencies() {
          let Some(m) = module_graph.module_identifier_by_dependency_id(dep_id) else {
            continue;
          };

          splitter.fill_chunk_modules(*m, runtime.as_ref(), &module_graph, &mut ctx);
        }

        if let Some(group_option) = block.get_group_options()
          && let Some(entry_options) = group_option.entry_options()
        {
          let mut entry_modules = IdentifierSet::default();
          for dep_id in block.get_dependencies() {
            let Some(m) = module_graph.module_identifier_by_dependency_id(dep_id) else {
              continue;
            };
            entry_modules.insert(*m);
          }

          chunks.push(ChunkDesc::Entry(Box::new(EntryChunkDesc {
            initial: false,
            options: entry_options.clone(),
            modules_ordinal: ctx.module_ordinal,
            available_modules: BigUint::ZERO,
            entry: entry_options.name.clone(),
            entry_modules: entry_modules.iter().copied().collect(),
            chunk_modules: ctx.chunk_modules,
            outgoing_blocks: ctx.out_goings,
            incoming_blocks: once(*block_id).collect(),
            pre_order_indices: ctx.pre_order_indices,
            post_order_indices: ctx.post_order_indices,
            runtime: runtime.clone(),
          })))
        } else {
          chunks.push(ChunkDesc::Chunk(Box::new(NormalChunkDesc {
            chunk_modules: ctx.chunk_modules,
            options: block.get_group_options().map(|opt| match opt {
              GroupOptions::Entrypoint(_) => unreachable!(),
              GroupOptions::ChunkGroup(group_option) => group_option.clone(),
            }),
            available_modules: BigUint::ZERO,
            modules_ordinal: ctx.module_ordinal,
            pre_order_indices: ctx.pre_order_indices,
            post_order_indices: ctx.post_order_indices,

            incoming_blocks: once(*block_id).collect(),
            outgoing_blocks: ctx.out_goings,

            runtime: runtime.clone(),
          })));
        }

        chunks
      }
    }
  }
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
      cache_chunk_desc: Default::default(),
      cache_chunks: Default::default(),
      module_deps: Default::default(),
      module_deps_without_runtime: Default::default(),
      module_ordinal,
    }
  }

  // modify the module ordinal for changed_modules
  pub fn invalidate(&mut self, changed_modules: impl Iterator<Item = ModuleIdentifier>) {
    let module_ordinal = &mut self.module_ordinal;

    for module in changed_modules {
      // add ordinal for new modules
      if !module_ordinal.contains_key(&module) {
        module_ordinal.insert(module, module_ordinal.len() as u64);
      }

      // refresh module traversal result in the last compilation
      for map in self.module_deps.values() {
        map.remove(&module);
      }
      self.module_deps_without_runtime.remove(&module);

      // remove chunks that contains changed module
      self.cache_chunk_desc.retain(|_, chunks| {
        // if this module is not in this chunk
        chunks.iter().all(|item| match &item.chunk_desc {
          ChunkDesc::Entry(entry_chunk_desc) => !entry_chunk_desc.chunk_modules.contains(&module),
          ChunkDesc::Chunk(normal_chunk_desc) => !normal_chunk_desc.chunk_modules.contains(&module),
        })
      });
    }
  }

  fn analyze_module_graph(
    &mut self,
    compilation: &mut Compilation,
  ) -> Result<Vec<CreateChunkRoot>> {
    // determine runtime and chunkLoading
    let mut entry_runtime: std::collections::HashMap<&str, RuntimeSpec, rustc_hash::FxBuildHasher> =
      HashMap::default();
    let mut diagnostics = vec![];
    for entry in compilation.entries.keys() {
      let mut visited = vec![];
      if let Err(error) =
        Self::get_entry_runtime(entry, compilation, &mut entry_runtime, &mut visited)
      {
        diagnostics.push(Diagnostic::from(error));
        let tmp_runtime = once(Arc::from(entry.clone())).collect::<RuntimeSpec>();
        entry_runtime.insert(entry, tmp_runtime.clone());
      };
    }

    // iterate module graph to find block runtime and its parents
    // let mut blocks_with_runtime = HashMap::default();
    let mut stack = vec![];
    let mut visited = HashSet::default();
    let module_graph = compilation.get_module_graph();
    let global_chunk_loading = &compilation.options.output.chunk_loading;
    let mut roots = HashMap::<AsyncDependenciesBlockIdentifier, CreateChunkRoot>::default();
    let mut entries = vec![];

    let global_deps = compilation.global_entry.dependencies.iter();
    let global_included_deps = compilation.global_entry.include_dependencies.iter();

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
        .unwrap_or(compilation.options.output.async_chunks);
      let runtime = entry_runtime
        .get(entry.as_str())
        .expect("already set runtime");

      self.module_deps.entry(runtime.clone()).or_default();

      entries.push(CreateChunkRoot::Entry(
        entry.clone(),
        entry_data.clone(),
        Some(runtime.clone()),
      ));

      global_deps
        .clone()
        .chain(entry_data.dependencies.iter())
        .chain(global_included_deps.clone())
        .chain(entry_data.include_dependencies.iter())
        .for_each(|dep_id| {
          if let Some(m) = module_graph.module_identifier_by_dependency_id(dep_id) {
            stack.push((*m, Cow::Borrowed(runtime), chunk_loading));
          }
        });
    }

    while let Some((module, runtime, chunk_loading)) = stack.pop() {
      if !visited.insert((module, runtime.clone())) {
        continue;
      }

      let guard = self.outgoings_modules(&module, Some(runtime.as_ref()), &module_graph);
      let (modules, blocks) = guard.value();
      let blocks = blocks.clone();
      for m in modules {
        stack.push((*m, runtime.clone(), chunk_loading));
      }
      drop(guard);

      for block_id in blocks {
        let Some(block) = module_graph.block_by_id(&block_id) else {
          continue;
        };

        // when disable chunk loading, only async entrypoint can be created, disable normal chunk
        let entry_options = block
          .get_group_options()
          .and_then(|option| option.entry_options());
        let should_create = chunk_loading || entry_options.is_some();
        let child_runtime = if should_create {
          if let Some(root) = roots.get_mut(&block_id) {
            let old_runtime = root.get_runtime().expect("already set runtime");
            let new_runtime = merge_runtime(&runtime, old_runtime);
            self.module_deps.entry(new_runtime.clone()).or_default();
            root.set_runtime(new_runtime.clone());
            Cow::Owned(new_runtime)
          } else {
            let rt = if let Some(entry_options) = entry_options {
              RuntimeSpec::from_entry_options(entry_options)
                .map(|rt| {
                  self.module_deps.entry(rt.clone()).or_default();
                  Cow::Owned(rt)
                })
                .unwrap_or(runtime.clone())
            } else {
              runtime.clone()
            };
            roots.insert(
              block_id,
              CreateChunkRoot::Block(block_id, Some(rt.clone().into_owned())),
            );
            rt.clone()
          }
        } else {
          runtime.clone()
        };

        let block = module_graph
          .block_by_id(&block_id)
          .expect("should have block");

        let child_chunk_loading = entry_options.map_or(chunk_loading, |opt| {
          !matches!(
            opt.chunk_loading.as_ref().unwrap_or(global_chunk_loading),
            ChunkLoading::Disable
          ) && opt
            .async_chunks
            .unwrap_or(compilation.options.output.async_chunks)
        });

        block
          .get_dependencies()
          .iter()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
          .for_each(|module| {
            stack.push((*module, child_runtime.clone(), child_chunk_loading));
          });
      }
    }

    compilation.extend_diagnostics(diagnostics);
    entries.extend(roots.into_values());
    // dbg!(visit_module_graph.elapsed().as_millis());

    Ok(entries)
  }

  fn get_entry_runtime<'a, 'b>(
    entry: &'a str,
    compilation: &'a Compilation,
    entry_runtime: &'b mut HashMap<&'a str, RuntimeSpec>,
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
      let mut runtime = None;
      for dep in depend_on {
        let other_runtime = Self::get_entry_runtime(dep, compilation, entry_runtime, visited)?;
        match &mut runtime {
          Some(runtime) => {
            *runtime = merge_runtime(runtime, &other_runtime);
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

    entry_runtime.insert(entry, runtime.clone());
    Ok(runtime)
  }

  fn get_module_ordinal(&self, m: ModuleIdentifier) -> u64 {
    *self
      .module_ordinal
      .get(&m)
      .unwrap_or_else(|| panic!("should have module ordinal: {m}"))
  }

  fn outgoings_modules(
    &self,
    module: &ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
  ) -> Ref<ModuleIdentifier, (Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)> {
    let module_map = if let Some(runtime) = runtime {
      self.module_deps.get(runtime).expect("should have value")
    } else {
      &self.module_deps_without_runtime
    };

    let guard = module_map.get(module);

    if let Some(ref_value) = guard {
      return ref_value;
    }

    drop(guard);

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
          .map_or(true, |module_dep| !module_dep.weak())
      })
      .filter_map(|dep| module_graph.connection_by_dependency_id(dep))
      .map(|conn| (conn.module_identifier(), conn))
      .for_each(|(module, conn)| outgoings.entry(*module).or_default().push(conn));

    let mut modules = IdentifierIndexSet::default();
    let mut blocks = m.get_blocks().to_vec();

    'outer: for (m, conns) in outgoings.iter() {
      for conn in conns {
        let conn_state = conn.active_state(module_graph, runtime);
        match conn_state {
          crate::ConnectionState::Bool(true) => {
            modules.insert(*m);
            continue 'outer;
          }
          crate::ConnectionState::TransitiveOnly => {
            let transitive = self.outgoings_modules(m, runtime, module_graph);
            let (extra_modules, extra_blocks) = transitive.value();
            modules.extend(extra_modules.iter().copied());
            blocks.extend(extra_blocks.iter().copied());
          }
          crate::ConnectionState::Bool(false) => {}
          crate::ConnectionState::CircularConnection => {}
        }
      }
    }

    module_map.insert(*module, (modules.into_iter().collect(), blocks));
    module_map.get(module).expect("have value")
  }

  // insert static dependencies into a set
  fn fill_chunk_modules(
    &self,
    target_module: ModuleIdentifier,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
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
          if !ctx.chunk_modules.insert(target_module) {
            continue;
          }
          let module = self.get_module_ordinal(target_module);

          if ctx.module_ordinal.bit(module) {
            // we already include this module
            continue;
          } else {
            ctx.module_ordinal.set_bit(module, true);
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

          let guard = self.outgoings_modules(&target_module, runtime, module_graph);
          let (_, (outgoing_modules, blocks)) = guard.pair();
          let mut outgoing_modules = outgoing_modules.clone();

          if ctx.chunk_loading {
            ctx.out_goings.extend(blocks.clone());
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

          drop(guard);

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
      global_deps
        .clone()
        .chain(entry.dependencies.iter())
        .chain(global_included_deps.clone())
        .chain(entry.include_dependencies.iter())
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .for_each(|m| {
          let entrypoint_ukey = compilation
            .entrypoints
            .get(name.as_str())
            .expect("should have entrypoint");
          let entrypoint = compilation.chunk_group_by_ukey.expect_get(entrypoint_ukey);
          let entry_chunk = compilation.chunk_by_ukey.expect_get(&entrypoint.chunks[0]);
          let entry_chunk_runtime = entry_chunk.runtime();
          queue.push(Task::Enter((*m, entry_chunk_runtime)))
        });

      chunk_group_index += 1;
      chunk_group_indices.insert(
        *compilation.entrypoints.get(name).expect("unreachable"),
        chunk_group_index,
      );
    }

    let mut visited = HashSet::default();
    let module_graph = compilation.get_module_graph();

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
            let guard = self.outgoings_modules(&m, Some(runtime), &module_graph);
            let (modules, blocks) = guard.value();

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

  fn reuse_chunk(mut chunk: Chunk, compilation: &mut Compilation) -> ChunkUkey {
    chunk.clear_groups();
    chunk.set_prevent_integration(false);

    let ukey = chunk.ukey();
    if let Some(name) = chunk.name() {
      compilation.named_chunks.insert(name.to_owned(), ukey);
    }
    compilation.chunk_by_ukey.add(chunk);
    ukey
  }

  fn create_chunks(&mut self, compilation: &mut Compilation) -> Result<()> {
    let mut errors = vec![];

    let mut roots = self.analyze_module_graph(compilation)?;

    let enable_incremental: bool = compilation
      .incremental
      .can_read_mutations(IncrementalPasses::BUILD_CHUNK_GRAPH);

    let reused = if enable_incremental {
      // lets see if any of them are from cache
      roots
        .par_iter()
        .map(|root| self.cache_chunk_desc.contains_key(root))
        .collect()
    } else {
      vec![false; roots.len()]
    };

    // fill chunk with modules in parallel
    let mut root_chunks = roots
      .par_iter()
      .enumerate()
      .map(|(idx, root)| {
        if reused[idx] {
          // we can reuse result from last computation
          // because inner modules have no changes at all
          self
            .cache_chunk_desc
            .get(root)
            .expect("should have value")
            .clone()
        } else {
          root
            .create(self, compilation)
            .into_iter()
            .map(|chunk| CacheableChunkItem {
              cache_ukey: CacheUkey::new(),
              chunk_desc: chunk,
            })
            .collect()
        }
      })
      .collect::<Vec<_>>();

    if enable_incremental {
      // flush cache from previous compilation
      self.cache_chunk_desc.clear();

      // insert new cache
      root_chunks.reverse();
      for chunks in &root_chunks {
        let root = roots.pop().expect("should have root");
        self.cache_chunk_desc.insert(root, chunks.clone());
      }
    }

    let chunks = root_chunks
      .into_iter()
      .enumerate()
      .flat_map(|(idx, caches)| {
        caches
          .into_iter()
          .map(|cache| (reused[idx], cache))
          .collect::<Vec<_>>()
      })
      .collect::<Vec<_>>();

    let (chunks, merge_errors) = self.merge_same_chunks(chunks);

    errors.extend(merge_errors);

    // determine chunk graph relations and
    // remove available modules if could
    let finalize_result = self.finalize_chunk_desc(chunks, compilation);

    let chunks_len = finalize_result.chunks.len();
    let mut chunks_ukey = vec![0.into(); chunks_len];

    let mut entries_depend_on = vec![];
    let mut async_entrypoints = HashSet::default();
    let mut skipped = HashSet::<usize>::default();

    for (idx, (reuse, cache)) in finalize_result.chunks.into_iter().enumerate() {
      let chunk_desc = cache.chunk_desc;
      match chunk_desc {
        ChunkDesc::Entry(entry_desc) => {
          let box EntryChunkDesc {
            entry,
            entry_modules,
            chunk_modules,
            options,
            available_modules,
            incoming_blocks,
            outgoing_blocks: _,
            initial,
            pre_order_indices,
            post_order_indices,
            runtime,
            ..
          } = entry_desc;

          let entry_chunk_ukey =
            if reuse && let Some(chunk) = self.cache_chunks.remove(&cache.cache_ukey) {
              Self::reuse_chunk(chunk, compilation)
            } else if let Some(chunk_name) = &options.name {
              let (ukey, add) = Compilation::add_named_chunk(
                chunk_name.clone(),
                &mut compilation.chunk_by_ukey,
                &mut compilation.named_chunks,
              );
              if add && let Some(mutations) = compilation.incremental.mutations_write() {
                mutations.add(Mutation::ChunkAdd { chunk: ukey });
              };
              ukey
            } else {
              let ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
              if let Some(mutations) = compilation.incremental.mutations_write() {
                mutations.add(Mutation::ChunkAdd { chunk: ukey });
              };
              ukey
            };

          let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
            initial,
            Box::new(options.clone()),
          ));

          entrypoint.module_pre_order_indices = pre_order_indices;
          entrypoint.module_post_order_indices = post_order_indices;

          let entry_chunk = compilation.chunk_by_ukey.expect_get_mut(&entry_chunk_ukey);
          entrypoint.set_entrypoint_chunk(entry_chunk_ukey);

          if initial {
            compilation
              .entrypoints
              .insert(entry.clone().expect("entry has name"), entrypoint.ukey);
          } else {
            compilation.async_entrypoints.push(entrypoint.ukey);
            async_entrypoints.insert(entrypoint.ukey);
          }

          let runtime = runtime.clone().expect("should have runtime");
          entry_chunk.set_runtime(runtime.clone());

          if let Some(filename) = &options.filename {
            entry_chunk.set_filename_template(Some(filename.clone()));
          }

          entrypoint.connect_chunk(entry_chunk);

          if let Some(name) = entrypoint.kind.name() {
            compilation
              .named_chunk_groups
              .insert(name.to_owned(), entrypoint.ukey);
          }

          if initial {
            if let Some(EntryRuntime::String(entry_runtime)) = &options.runtime {
              let (runtime_chunk_ukey, add) = Compilation::add_named_chunk(
                entry_runtime.into(),
                &mut compilation.chunk_by_ukey,
                &mut compilation.named_chunks,
              );

              let rt_chunk = if !add && compilation.entries.contains_key(entry_runtime) {
                let name = entry.as_ref().expect("should have name");
                errors.push(Diagnostic::from(error!(
                  "Entrypoint '{name}' has a 'runtime' option which points to another entrypoint named '{entry_runtime}'.
It's not valid to use other entrypoints as runtime chunk.
Did you mean to use 'dependOn: \"{entry_runtime}\"' instead to allow using entrypoint '{name}' within the runtime of entrypoint '{entry_runtime}'? For this '{entry_runtime}' must always be loaded when '{name}' is used.
Or do you want to use the entrypoints '{name}' and '{entry_runtime}' independently on the same page with a shared runtime? In this case give them both the same value for the 'runtime' option. It must be a name not already used by an entrypoint."
                                ),
                ).with_chunk(Some(entry_chunk_ukey.as_u32())));
                compilation.chunk_by_ukey.expect_get_mut(&entry_chunk_ukey)
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
              entrypoint.unshift_chunk(rt_chunk);
              rt_chunk.set_runtime(runtime);
              rt_chunk.add_group(entrypoint.ukey);
              entrypoint.set_runtime_chunk(rt_chunk.ukey());
            } else if let Some(depend_on) = &options.depend_on {
              entries_depend_on.push((
                entry
                  .clone()
                  .expect("entry has dependOn but does not have name, this is unreachable"),
                depend_on.clone(),
              ));
            } else {
              entrypoint.set_runtime_chunk(entry_chunk_ukey);
            }
          } else {
            entrypoint.set_runtime_chunk(entry_chunk_ukey);
          }

          let entrypoint_ukey = entrypoint.ukey;

          if initial {
            let name = entry.as_ref().expect("should have name");
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
              entrypoint.add_origin(None, loc, request);
            }

            if initial {
              let mut assign_depths_map = IdentifierMap::default();
              assign_depths(
                &mut assign_depths_map,
                &compilation.get_module_graph(),
                entry_modules.iter(),
              );
              let mut module_graph = compilation.get_module_graph_mut();
              for (m, depth) in assign_depths_map {
                module_graph.set_depth_if_lower(&m, depth);
              }
            }
          }

          for incoming in incoming_blocks {
            compilation
              .chunk_graph
              .connect_block_and_chunk_group(incoming, entrypoint_ukey);

            let module_graph = compilation.get_module_graph();
            if let Some(block) = module_graph.block_by_id(&incoming) {
              entrypoint.add_origin(Some(*block.parent()), block.loc(), block.request().clone());
            }
          }

          compilation.chunk_group_by_ukey.add(entrypoint);
          compilation.chunk_graph.add_chunk(entry_chunk_ukey);

          for m in entry_modules {
            compilation.chunk_graph.connect_chunk_and_entry_module(
              entry_chunk_ukey,
              m,
              entrypoint_ukey,
            );

            compilation
              .chunk_graph
              .connect_chunk_and_module(entry_chunk_ukey, m);
          }

          // TODO: do this in parallel
          for module in chunk_modules.iter() {
            if !available_modules.bit(self.get_module_ordinal(*module)) {
              compilation
                .chunk_graph
                .connect_chunk_and_module(entry_chunk_ukey, *module);
            }
          }

          chunks_ukey[idx] = entry_chunk_ukey;
          self.cache_chunks.insert(
            cache.cache_ukey,
            compilation
              .chunk_by_ukey
              .expect_get(&entry_chunk_ukey)
              .clone(),
          );
        }
        ChunkDesc::Chunk(chunk_desc) => {
          let box NormalChunkDesc {
            options,
            chunk_modules,
            available_modules,
            pre_order_indices,
            post_order_indices,
            incoming_blocks,
            runtime,
            ..
          } = chunk_desc;

          let modules = chunk_modules
            .into_iter()
            .filter(|m| !available_modules.bit(self.get_module_ordinal(*m)))
            .collect::<Vec<_>>();
          if modules.is_empty() {
            // ignore empty chunk
            skipped.insert(idx);
            // continue;
          }

          let name = if let Some(option) = &options {
            option.name.as_ref()
          } else {
            None
          };

          let ukey = if reuse && let Some(chunk) = self.cache_chunks.remove(&cache.cache_ukey) {
            Self::reuse_chunk(chunk, compilation)
          } else if let Some(name) = name {
            let (ukey, add) = Compilation::add_named_chunk(
              name.clone(),
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            );
            if add && let Some(mutations) = compilation.incremental.mutations_write() {
              mutations.add(Mutation::ChunkAdd { chunk: ukey });
            }
            ukey
          } else {
            let ukey = Compilation::add_chunk(&mut compilation.chunk_by_ukey);
            if let Some(mutations) = compilation.incremental.mutations_write() {
              mutations.add(Mutation::ChunkAdd { chunk: ukey });
            }
            ukey
          };

          let mut group = ChunkGroup::new(crate::ChunkGroupKind::Normal {
            options: options.clone().unwrap_or_default(),
          });
          group.module_pre_order_indices = pre_order_indices;
          group.module_post_order_indices = post_order_indices;
          let group_ukey = group.ukey;

          let chunk = compilation.chunk_by_ukey.expect_get_mut(&ukey);
          group.connect_chunk(chunk);
          chunk.set_runtime(runtime.expect("should have runtime"));

          for block in incoming_blocks {
            compilation
              .chunk_graph
              .connect_block_and_chunk_group(block, group_ukey);

            let module_graph = compilation.get_module_graph();
            if let Some(block) = module_graph.block_by_id(&block) {
              group.add_origin(Some(*block.parent()), block.loc(), block.request().clone());
            }
          }

          compilation.chunk_group_by_ukey.add(group);
          compilation.chunk_graph.add_chunk(ukey);

          if let Some(name) = name {
            compilation
              .named_chunk_groups
              .insert(name.clone(), group_ukey);
          }

          chunks_ukey[idx] = ukey;
          self.cache_chunks.insert(
            cache.cache_ukey,
            compilation.chunk_by_ukey.expect_get(&ukey).clone(),
          );
          // TODO: do this in parallel
          for module in modules {
            compilation
              .chunk_graph
              .connect_chunk_and_module(ukey, module);
          }
        }
      }
    }

    // the left cache is unused, mark them as removed
    if let Some(mutations) = compilation.incremental.mutations_write() {
      self.cache_chunks.retain(|_, chunk| {
        let should_remove = !compilation.chunk_by_ukey.contains(&chunk.ukey());

        if should_remove {
          mutations.add(Mutation::ChunkRemove {
            chunk: chunk.ukey(),
          });
        }

        !should_remove
      });
    }

    for (entry, depend_on) in entries_depend_on {
      let entrypoint_ukey = *compilation.entrypoints.get(&entry).expect("unreachable");
      let entrypoint = compilation
        .chunk_group_by_ukey
        .get(&entrypoint_ukey)
        .expect("unreachable");
      let entry_point_chunk = compilation
        .chunk_by_ukey
        .expect_get(&entrypoint.get_entrypoint_chunk());
      let entry_point_chunk_ukey = entry_point_chunk.ukey();
      let referenced_chunks =
        entry_point_chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey);

      let mut parents = vec![];

      for dep in depend_on {
        if let Some(dep_entrypoint_ukey) = compilation.entrypoints.get(&dep) {
          let dep_entrypoint = compilation
            .chunk_group_by_ukey
            .expect_get_mut(dep_entrypoint_ukey);
          let dependency_chunk_ukey = dep_entrypoint.get_entrypoint_chunk();
          if referenced_chunks.contains(&dependency_chunk_ukey) {
            // cycle dep
            compilation
              .chunk_group_by_ukey
              .get_mut(&entrypoint_ukey)
              .expect("unreachable")
              .set_runtime_chunk(entry_point_chunk_ukey);
            break;
          }

          dep_entrypoint.add_child(entrypoint_ukey);
          parents.push(dep_entrypoint_ukey);
        } else {
          errors.push(Diagnostic::from(error!("entry {dep} not found")));
        }
      }

      let entrypoint = compilation
        .chunk_group_by_ukey
        .expect_get_mut(&entrypoint_ukey);
      for parent in parents {
        entrypoint.add_parent(*parent);
      }
    }

    // connect parent and children
    for idx in 0..chunks_len {
      if skipped.contains(&idx) {
        continue;
      }

      let Some(children) = finalize_result.chunk_children.get(idx) else {
        continue;
      };
      let children = children.iter().filter(|idx| !skipped.contains(idx));

      let chunk = compilation.chunk_by_ukey.expect_get(&chunks_ukey[idx]);

      let groups = chunk.groups().clone();
      let children_groups = children
        .flat_map(|parent| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunks_ukey[*parent])
            .groups()
        })
        .copied();

      for group in &groups {
        let children = children_groups
          .clone()
          .filter(|cg| !async_entrypoints.contains(cg));
        let group = compilation.chunk_group_by_ukey.expect_get_mut(group);
        group.children.extend(children);
      }

      for child_ukey in children_groups {
        if async_entrypoints.contains(&child_ukey) {
          for parent in &groups {
            let parent = compilation.chunk_group_by_ukey.expect_get_mut(parent);
            parent.add_async_entrypoint(child_ukey);
          }
        } else {
          let child = compilation.chunk_group_by_ukey.expect_get_mut(&child_ukey);
          child.parents.extend(groups.clone());
        }
      }
    }
    self.set_order_index_and_group_index(compilation);

    compilation.extend_diagnostics(errors);

    Ok(())
  }

  // 1. determine parent child relationship
  // 2. remove modules that exist in all parents
  fn finalize_chunk_desc(
    &mut self,
    mut chunks: Vec<(bool, CacheableChunkItem)>,
    compilation: &Compilation,
  ) -> FinalizeChunksResult {
    let chunks_len = chunks.len();

    // map that records info about chunk to its parents
    // this is useful when calculate removeAvailableModules, as it needs calculate based on parents
    let mut chunk_parents = Vec::with_capacity(chunks_len);

    // this is useful when determine chunk index, the order index of chunk is deterministic,
    // we use chunk outgoing blocks order to ensure that
    let mut chunk_children = Vec::<Vec<usize>>::with_capacity(chunks_len);

    // which chunk owns this block identifier
    let mut chunks_by_block: HashMap<AsyncDependenciesBlockIdentifier, Vec<usize>> =
      HashMap::default();

    let mut idx_by_name: HashMap<&str, usize> = Default::default();

    // one block.dep only point to a unique chunk
    // use this to a chunk is initialized because of which block.dep
    let mut chunks_origin_block = HashMap::default();

    let mut roots = vec![];

    // 1st iter, find roots and analyze which block belongs to which chunk
    // note that async entrypoint has no parents
    for (idx, (_, cache)) in chunks.iter().enumerate() {
      let chunk = &cache.chunk_desc;
      if let Some(name) = chunk.name() {
        idx_by_name.insert(name, idx);
      }
      let incoming_blocks = match chunk {
        ChunkDesc::Entry(entry) => {
          roots.push(idx);
          &entry.incoming_blocks
        }
        ChunkDesc::Chunk(chunk) => &chunk.incoming_blocks,
      };
      for block in incoming_blocks {
        if chunks_origin_block.insert(*block, idx).is_some() {
          unreachable!()
        }
      }

      for block in chunk.outgoings() {
        chunks_by_block.entry(*block).or_default().push(idx);
      }
    }

    // 2nd iter, analyze chunk relations
    for (_, cache) in chunks.iter() {
      match &cache.chunk_desc {
        ChunkDesc::Entry(entry) => {
          if let Some(depend_on) = &entry.options.depend_on {
            let depend_on_parents = depend_on
              .iter()
              .map(|dep| idx_by_name.get(dep.as_str()).expect("unreachable"))
              .copied()
              .collect();
            chunk_parents.push(depend_on_parents);
          } else {
            chunk_parents.push(Vec::default());
          }
        }
        ChunkDesc::Chunk(chunk) => {
          let mut parents = HashSet::default();
          for block in &chunk.incoming_blocks {
            let Some(chunk_parents) = chunks_by_block.get(block) else {
              continue;
            };

            parents.extend(chunk_parents);
          }
          chunk_parents.push(parents.into_iter().collect::<Vec<_>>());
        }
      }
    }

    for (_, cache) in chunks.iter() {
      chunk_children.push(
        cache
          .chunk_desc
          .outgoings()
          .filter_map(|outgoing_block| chunks_origin_block.get(outgoing_block).copied())
          .collect(),
      );
    }

    if compilation.options.optimization.remove_available_modules {
      /*
            4rd iter, remove modules that is available in parents
            remove modules that are already exist in parent chunk

            if we meet cycle like following:

                  ┌────┐      ┌────┐
      module x <- │ e1 │      │ e2 │ -> module x
                  └──┬─┘      └─┬──┘
                     │          │
                     v          v
                  ┌───┐       ┌───┐
                  │ a ├──────>│ b │
                  └───┘       └─┬─┘
                    ^           │
                    │  ┌─────┐  │
                    └──┤  c  │<─┘
                       └─────┘
                          |
                          v
                      module x (this should not be included in any chunk, as it already exist in all parents)
          */
      remove_available_modules(&mut chunks, &roots, &chunk_parents, &chunk_children);
    }

    FinalizeChunksResult {
      chunks,
      chunk_children,
    }
  }

  // merge chunks that has the same name on them
  fn merge_same_chunks(
    &mut self,
    chunks: Vec<(bool, CacheableChunkItem)>,
  ) -> (Vec<(bool, CacheableChunkItem)>, Vec<Diagnostic>) {
    let mut final_chunks: Vec<(bool, CacheableChunkItem)> = vec![];
    let mut named_chunks = HashMap::<String, usize>::default();
    let mut errors = vec![];

    for (reuse, cache) in chunks {
      if let Some(name) = cache.chunk_desc.name() {
        if let Some(idx) = named_chunks.get(name).copied() {
          // there is existing chunk, merge them,
          // this is caused by same webpackChunkName
          if let Err(err) = merge_chunk(cache.chunk_desc, &mut final_chunks[idx].1.chunk_desc) {
            errors.push(Diagnostic::from(err));
          }
          let (reuse, _) = &mut final_chunks[idx];
          // if the chunk has multiple parents this chunk cannot be reused
          *reuse = false;
          continue;
        } else {
          let idx = final_chunks.len();
          named_chunks.insert(name.to_string(), idx);
        }
      }
      final_chunks.push((reuse, cache));
    }

    (final_chunks, errors)
  }
}

fn merge_chunk(chunk: ChunkDesc, existing_chunk: &mut ChunkDesc) -> Result<()> {
  match (existing_chunk, chunk) {
    (ChunkDesc::Entry(existing_chunk), ChunkDesc::Entry(chunk)) => {
      existing_chunk.modules_ordinal |= chunk.modules_ordinal;
      existing_chunk.chunk_modules.extend(chunk.chunk_modules);
      existing_chunk.incoming_blocks.extend(chunk.incoming_blocks);
      existing_chunk.outgoing_blocks.extend(chunk.outgoing_blocks);
      match (&mut existing_chunk.runtime, chunk.runtime) {
        (None, None) => {}
        (None, Some(other)) => existing_chunk.runtime = Some(other),
        (Some(_), None) => {}
        (Some(existing), Some(other)) => *existing = merge_runtime(existing, &other),
      }
    }
    (ChunkDesc::Chunk(existing_chunk), ChunkDesc::Chunk(chunk)) => {
      existing_chunk.modules_ordinal |= chunk.modules_ordinal;
      existing_chunk.chunk_modules.extend(chunk.chunk_modules);
      existing_chunk.incoming_blocks.extend(chunk.incoming_blocks);
      existing_chunk.outgoing_blocks.extend(chunk.outgoing_blocks);
      match (&mut existing_chunk.runtime, chunk.runtime) {
        (None, None) => {}
        (None, Some(other)) => existing_chunk.runtime = Some(other),
        (Some(_), None) => {}
        (Some(existing), Some(other)) => *existing = merge_runtime(existing, &other),
      }

      match (&mut existing_chunk.options, chunk.options) {
        (None, None) => {}
        (Some(_), None) => {}
        (None, Some(options)) => {
          existing_chunk.options = Some(options);
        }
        (Some(existing), Some(other)) => {
          match (&mut existing.prefetch_order, other.prefetch_order) {
            (None, None) => {}
            (Some(_), None) => {}
            (None, Some(other)) => existing.prefetch_order = Some(other),
            (Some(existing), Some(other)) => {
              *existing = std::cmp::max(*existing, other);
            }
          }

          match (&mut existing.preload_order, other.preload_order) {
            (None, None) => {}
            (Some(_), None) => {}
            (None, Some(other)) => existing.preload_order = Some(other),
            (Some(existing), Some(other)) => {
              *existing = std::cmp::max(*existing, other);
            }
          }

          match (&mut existing.fetch_priority, other.fetch_priority) {
            (None, None) => {}
            (Some(_), None) => {}
            (None, Some(other)) => existing.fetch_priority = Some(other),
            (Some(existing), Some(other)) => {
              *existing = std::cmp::max(*existing, other);
            }
          }
        }
      }

      // for (module, idx) in &mut existing_chunk.pre_order_indices {
      //   if let Some(other_idx) = chunk.pre_order_indices.get(module) {
      //     *idx = std::cmp::min(*idx, *other_idx);
      //   }
      // }
      // for (module, idx) in &mut existing_chunk.post_order_indices {
      //   if let Some(other_idx) = chunk.post_order_indices.get(module) {
      //     *idx = std::cmp::min(*idx, *other_idx);
      //   }
      // }
    }
    (ChunkDesc::Entry(entry), ChunkDesc::Chunk(chunk)) => {
      entry.modules_ordinal |= chunk.modules_ordinal;
      entry.chunk_modules.extend(chunk.chunk_modules);
      entry.incoming_blocks.extend(chunk.incoming_blocks);
      entry.outgoing_blocks.extend(chunk.outgoing_blocks);
      return Err(error!(
        format!("It's not allowed to load an initial chunk on demand. The chunk name \"{}\" is already used by an entrypoint.", entry.entry.as_ref().expect("already has name"))
      ));
    }
    (ChunkDesc::Chunk(existing), ChunkDesc::Entry(entry)) => {
      existing.modules_ordinal |= entry.modules_ordinal;
      existing.chunk_modules.extend(entry.chunk_modules);
      existing.incoming_blocks.extend(entry.incoming_blocks);
      existing.outgoing_blocks.extend(entry.outgoing_blocks);
      return Err(error!(
        format!("It's not allowed to load an initial chunk on demand. The chunk name \"{}\" is already used by an entrypoint.", entry.entry.as_ref().expect("already has name"))
      ));
    }
  };

  Ok(())
}

// main entry for code splitting
pub fn code_split(compilation: &mut Compilation) -> Result<()> {
  let mutations = compilation
    .incremental
    .mutations_read(IncrementalPasses::BUILD_CHUNK_GRAPH);
  let enable_incremental = mutations.is_some();

  let module_graph: &ModuleGraph<'_> = &compilation.get_module_graph();

  let mut splitter = if let Some(mutations) = mutations {
    let mut affected = mutations.get_affected_modules_with_module_graph(module_graph);
    let removed = mutations.iter().filter_map(|mutation| match mutation {
      Mutation::ModuleRemove { module } => Some(*module),
      _ => None,
    });
    affected.extend(removed);

    // reuse data from last computation
    let mut splitter = std::mem::take(&mut compilation.code_splitting_cache.new_code_splitter);
    splitter.invalidate(affected.into_iter());
    splitter
  } else {
    CodeSplitter::new(compilation.get_module_graph().modules().keys().copied())
  };

  // fill chunks with its modules
  splitter.create_chunks(compilation)?;

  if enable_incremental {
    // if enable incremental, store data
    compilation.code_splitting_cache.new_code_splitter = splitter;
  }

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

  Ok(())
}
