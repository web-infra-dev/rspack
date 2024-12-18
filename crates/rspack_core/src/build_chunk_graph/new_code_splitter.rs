use std::borrow::Cow;
use std::hash::BuildHasherDefault;
use std::iter::once;
use std::sync::Arc;

use dashmap::{mapref::one::Ref, DashMap};
use indexmap::IndexSet;
use num_bigint::BigInt;
use rayon::prelude::*;
use rspack_collections::{
  DatabaseItem, IdentifierDashMap, IdentifierDashSet, IdentifierHasher, IdentifierIndexMap,
  IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyMap,
};
use rspack_error::Result;
use rspack_error::{error, Diagnostic};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::incremental::{IncrementalPasses, Mutation};
use crate::{
  assign_depths, merge_runtime, AsyncDependenciesBlockIdentifier, ChunkGroup, ChunkGroupKind,
  ChunkGroupOptions, ChunkGroupUkey, ChunkLoading, ChunkUkey, Compilation, DependenciesBlock,
  DependencyLocation, EntryData, EntryDependency, EntryOptions, EntryRuntime, GroupOptions,
  ModuleDependency, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, RuntimeSpec,
  SyntheticDependencyLocation,
};

type ModuleDeps = HashMap<
  RuntimeSpec,
  IdentifierDashMap<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)>,
>;

#[derive(Debug)]
pub struct CodeSplitter {
  pub _connect_count: usize,

  pub module_deps: ModuleDeps,
  pub module_deps_without_runtime:
    IdentifierDashMap<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)>,

  // blocks that maps a block to its origin module
  pub blocks: HashMap<AsyncDependenciesBlockIdentifier, ModuleIdentifier>,
  pub chunk_parent_modules: HashMap<ChunkUkey, HashSet<ModuleIdentifier>>,
  pub module_ordinal: IdentifierMap<u64>,
}

#[derive(Debug)]
enum CreateChunkRoot {
  Entry(String, EntryData, Option<RuntimeSpec>),
  Block(
    ModuleIdentifier,
    AsyncDependenciesBlockIdentifier,
    Option<RuntimeSpec>,
  ),
}

struct FinalizeChunksResult {
  chunks: Vec<ChunkDesc>,
  chunk_children: Vec<Vec<usize>>,
}

// Description about how to create chunk
#[derive(Debug, Clone)]
enum ChunkDesc {
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

  fn as_entry_mut(&mut self) -> Option<&mut EntryChunkDesc> {
    match self {
      ChunkDesc::Entry(entry) => Some(entry),
      ChunkDesc::Chunk(_) => None,
    }
  }

  fn outgoings(&self) -> impl Iterator<Item = &AsyncDependenciesBlockIdentifier> {
    match self {
      ChunkDesc::Entry(entry) => entry.outgoing_blocks.iter(),
      ChunkDesc::Chunk(chunk) => chunk.outgoing_blocks.iter(),
    }
  }

  fn available_modules(&self) -> &BigInt {
    match self {
      ChunkDesc::Entry(entry) => &entry.modules_ordinal,
      ChunkDesc::Chunk(chunk) => &chunk.modules_ordinal,
    }
  }

  fn available_modules_mut(&mut self) -> &mut BigInt {
    match self {
      ChunkDesc::Entry(entry) => &mut entry.modules_ordinal,
      ChunkDesc::Chunk(chunk) => &mut chunk.modules_ordinal,
    }
  }
}

#[derive(Debug, Clone)]
struct EntryChunkDesc {
  initial: bool,

  options: EntryOptions,
  modules_ordinal: BigInt,
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
struct NormalChunkDesc {
  original: HashSet<ModuleIdentifier>,

  options: Option<ChunkGroupOptions>,
  modules_ordinal: BigInt,
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
  pub module_ordinal: BigInt,
  pub next_pre_order_index: usize,
  pub next_post_order_index: usize,
}

impl CreateChunkRoot {
  fn entry_name(&self) -> &str {
    match self {
      CreateChunkRoot::Entry(name, _, _) => name,
      CreateChunkRoot::Block(_, _, _) => unreachable!(),
    }
  }

  pub(crate) fn get_runtime(&self) -> Option<&RuntimeSpec> {
    match self {
      CreateChunkRoot::Entry(_, _, rt) => rt.as_ref(),
      CreateChunkRoot::Block(_, _, rt) => rt.as_ref(),
    }
  }

  fn set_runtime(&mut self, runtime: RuntimeSpec) {
    match self {
      CreateChunkRoot::Entry(_, _, rt) => *rt = Some(runtime),
      CreateChunkRoot::Block(_, _, rt) => *rt = Some(runtime),
    }
  }

  fn create(self, splitter: &CodeSplitter, compilation: &Compilation) -> Vec<ChunkDesc> {
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

        for m in deps
          .clone()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        {
          entry_modules.push(*m);
        }

        for m in deps
          .chain(
            data
              .include_dependencies
              .iter()
              .chain(compilation.global_entry.include_dependencies.iter()),
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
          modules_ordinal: ctx.module_ordinal,
          incoming_blocks: Default::default(),
          outgoing_blocks: ctx.out_goings,
          runtime,
        }))]
      }
      CreateChunkRoot::Block(origin, block_id, runtime) => {
        let block = module_graph
          .block_by_id(&block_id)
          .expect("should have block");

        let mut chunks = vec![];
        let mut ctx = FillCtx::default();

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
            entry: entry_options.name.clone(),
            entry_modules: entry_modules.iter().copied().collect(),
            chunk_modules: ctx.chunk_modules,
            outgoing_blocks: ctx.out_goings,
            incoming_blocks: std::iter::once(block_id).collect(),
            pre_order_indices: ctx.pre_order_indices,
            post_order_indices: ctx.post_order_indices,
            runtime: runtime.clone(),
          })))
        } else {
          chunks.push(ChunkDesc::Chunk(Box::new(NormalChunkDesc {
            original: once(origin).collect(),
            chunk_modules: ctx.chunk_modules,
            options: block.get_group_options().map(|opt| match opt {
              GroupOptions::Entrypoint(_) => unreachable!(),
              GroupOptions::ChunkGroup(group_option) => group_option.clone(),
            }),
            modules_ordinal: ctx.module_ordinal,
            pre_order_indices: ctx.pre_order_indices,
            post_order_indices: ctx.post_order_indices,

            incoming_blocks: std::iter::once(block_id).collect(),
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
  pub fn new(modules: Vec<ModuleIdentifier>) -> Self {
    let mut module_ordinal = IdentifierMap::default();
    for (idx, m) in modules.iter().enumerate() {
      module_ordinal.insert(*m, idx as u64);
    }

    Self {
      _connect_count: 0,
      module_deps: Default::default(),
      module_deps_without_runtime: Default::default(),
      blocks: Default::default(),
      chunk_parent_modules: Default::default(),
      module_ordinal,
    }
  }

  fn analyze_module_graph(
    &mut self,
    compilation: &mut Compilation,
  ) -> Result<Vec<CreateChunkRoot>> {
    let visit_module_graph = std::time::Instant::now();

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
        let tmp_runtime = RuntimeSpec::from_iter(std::iter::once(Arc::from(entry.clone())));
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

      compilation
        .global_entry
        .all_dependencies()
        .chain(entry_data.all_dependencies())
        .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
        .for_each(|m| stack.push((*m, Cow::Borrowed(runtime), chunk_loading)));
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
            roots.insert(
              block_id,
              CreateChunkRoot::Block(
                *block.parent(),
                block_id,
                Some(runtime.clone().into_owned()),
              ),
            );
            runtime.clone()
          }
        } else {
          runtime.clone()
        };

        let block = module_graph
          .block_by_id(&block_id)
          .expect("should have block");

        let child_chunk_loading = entry_options
          .map(|opt| {
            !matches!(opt.chunk_loading, Some(ChunkLoading::Disable))
              && matches!(opt.async_chunks, Some(true))
          })
          .unwrap_or(chunk_loading);

        block
          .get_dependencies()
          .iter()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
          .for_each(|module| {
            stack.push((*module, child_runtime.clone(), child_chunk_loading));
          });
      }
    }

    dbg!(visit_module_graph.elapsed().as_millis());
    compilation.extend_diagnostics(diagnostics);
    entries.extend(roots.into_values());

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
        "Entrypoints '{entry}' and '{}' use 'dependOn' to depend on each other in a circular way.",
        visited.last().expect("has item")
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
      .unwrap_or_else(|| panic!("should have module ordinal: {}", m))
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
          .dependency_by_id(&dep_id)
          .expect("should have dep")
          .as_module_dependency()
          .map(|module_dep| !module_dep.weak())
          .unwrap_or(true)
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
    if !ctx.chunk_modules.insert(target_module) {
      return;
    }

    let module = self.get_module_ordinal(target_module);

    if ctx.module_ordinal.bit(module) {
      // we already include this module
      return;
    } else {
      ctx.module_ordinal.set_bit(module, true);
    }

    if !ctx.pre_order_indices.contains_key(&target_module) {
      ctx
        .pre_order_indices
        .insert(target_module, ctx.next_pre_order_index);
      ctx.next_pre_order_index += 1;
    }

    let guard = self.outgoings_modules(&target_module, runtime, module_graph);
    let (_, (outgoing_modules, blocks)) = guard.pair();

    ctx.out_goings.extend(blocks.clone());
    let outgoing_modules = outgoing_modules.clone();
    drop(guard);

    for m in outgoing_modules.iter() {
      self.fill_chunk_modules(*m, runtime, module_graph, ctx);
    }

    ctx
      .post_order_indices
      .insert(target_module, ctx.next_post_order_index);
    ctx.next_post_order_index += 1;
  }

  fn create_chunks(&mut self, compilation: &mut Compilation) -> Result<()> {
    let entries = &compilation.entries;

    let mut error_roots: Vec<(usize, Diagnostic)> = vec![];
    let mut errors = vec![];
    let mut deps: Vec<(&String, Vec<String>)> = vec![];
    let mut name_to_idx: std::collections::HashMap<&String, usize, rustc_hash::FxBuildHasher> =
      HashMap::default();
    // let mut roots: Vec<CreateChunkRoot> = vec![];
    // for (idx, (name, data)) in entries.iter().enumerate() {
    //   name_to_idx.insert(name, idx);
    //   let runtime = if let Some(depend_on) = &data.options.depend_on {
    //     deps.push((name, depend_on.clone()));
    //     None
    //   } else {
    //     Some(RuntimeSpec::from_entry_options(&data.options).expect("should have runtime"))
    //   };

    //   // set runtime later
    //   roots.push(CreateChunkRoot::Entry(name.clone(), data.clone(), runtime));
    // }

    // let mut entry_to_deps = HashMap::default();
    // for (entry, deps) in deps {
    //   entry_to_deps.insert(
    //     entry.as_str(),
    //     deps
    //       .into_iter()
    //       .map(|dep| *name_to_idx.get(&dep).expect("should have idx"))
    //       .collect::<Vec<_>>(),
    //   );
    // }

    // for (entry, _) in entries.iter() {
    //   let curr = *name_to_idx.get(entry).expect("unreachable");
    //   if roots[curr].get_runtime().is_some() {
    //     // already set
    //     continue;
    //   }
    //   let mut visited = Default::default();
    //   if let Err(dep) =
    //     set_entry_runtime_and_depend_on(curr, &mut roots, &entry_to_deps, &mut visited, &mut vec![])
    //   {
    //     let dep_name = roots[dep].entry_name();
    //     let dep_rt = RuntimeSpec::from_entry(dep_name, None);
    //     error_roots.push((curr, Diagnostic::from(error!("Entrypoints '{entry}' and '{dep_name}' use 'dependOn' to depend on each other in a circular way."))));
    //     roots[dep].set_runtime(dep_rt);
    //     roots[curr].set_runtime(RuntimeSpec::from_entry(entry, None));
    //   }
    // }

    // let mut entry_module_runtime = IdentifierMap::<RuntimeSpec>::default();
    // let module_graph: ModuleGraph = compilation.get_module_graph();
    // for root in &roots {
    //   if let CreateChunkRoot::Entry(_name, data, runtime) = root {
    //     let runtime = runtime
    //       .as_ref()
    //       .expect("should have runtime after calculated depend on");
    //     for dep_id in compilation
    //       .global_entry
    //       .all_dependencies()
    //       .chain(data.all_dependencies())
    //     {
    //       let Some(module) = module_graph.module_identifier_by_dependency_id(dep_id) else {
    //         continue;
    //       };
    //       match entry_module_runtime.entry(*module) {
    //         std::collections::hash_map::Entry::Occupied(mut existing) => {
    //           let new_runtime = merge_runtime(existing.get(), &runtime);
    //           existing.insert(new_runtime);
    //         }
    //         std::collections::hash_map::Entry::Vacant(vacant) => {
    //           vacant.insert(runtime.clone());
    //         }
    //       }
    //     }
    //   }
    // }

    // let module_graph = compilation.get_module_graph();
    // let module_cache = DashMap::default();

    // roots.extend(
    //   self
    //     .blocks
    //     .par_iter()
    //     .map(|(block_id, origin)| {
    //       let visited = IdentifierDashSet::default();
    //       let block = module_graph
    //         .block_by_id(block_id)
    //         .expect("should have block");
    //       let runtime = if let Some(group_options) = block.get_group_options()
    //         && let Some(entry_options) = group_options.entry_options()
    //       {
    //         RuntimeSpec::from_entry_options(entry_options).or_else(|| {
    //           determine_runtime(
    //             *origin,
    //             &module_graph,
    //             &entry_module_runtime,
    //             &module_cache,
    //             &visited,
    //           )
    //         })
    //       } else {
    //         determine_runtime(
    //           *origin,
    //           &module_graph,
    //           &entry_module_runtime,
    //           &module_cache,
    //           &visited,
    //         )
    //       };

    //       if runtime.is_none() {
    //         dbg!(block.identifier());
    //         panic!()
    //       }
    //       CreateChunkRoot::Block(*origin, *block_id, runtime)
    //     })
    //     .collect::<Vec<_>>(),
    // );

    // for root in &roots {
    //   if let Some(runtime) = root.get_runtime() {
    //     self.module_deps.insert(runtime.clone(), Default::default());
    //   }
    // }

    let roots = self.analyze_module_graph(compilation)?;
    // fill chunk with modules in parallel
    let chunks = roots
      .into_par_iter()
      .map(|root| root.create(self, compilation))
      .flatten()
      .collect::<Vec<_>>();
    let chunks = self.merge_same_chunks(chunks, &compilation)?;

    // remove available modules if could
    // determin chunk graph relations
    // determin runtime
    let finalize_result = self.finalize_chunk_desc(chunks, compilation);

    let chunks_len = finalize_result.chunks.len();
    let mut chunks_ukey = vec![0.into(); chunks_len];
    let mut skipped = vec![false; chunks_len];

    let mut entries_depend_on = vec![];
    let mut skipped_modules = 0;
    for (idx, chunk_desc) in finalize_result.chunks.into_iter().enumerate() {
      match chunk_desc {
        ChunkDesc::Entry(entry_desc) => {
          let box EntryChunkDesc {
            entry,
            entry_modules,
            chunk_modules,
            options,
            modules_ordinal,
            incoming_blocks,
            outgoing_blocks: _,
            initial,
            pre_order_indices,
            post_order_indices,
            runtime,
          } = entry_desc;

          let entry_chunk_ukey = if let Some(chunk_name) = &options.name {
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
          entrypoint.set_entry_point_chunk(entry_chunk_ukey);

          if initial {
            compilation
              .entrypoints
              .insert(entry.clone().expect("entry has name"), entrypoint.ukey);
          } else {
            compilation.async_entrypoints.push(entrypoint.ukey);
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

            let mut assign_depths_map = IdentifierMap::default();
            assign_depths(
              &mut assign_depths_map,
              &compilation.get_module_graph(),
              entry_modules.iter(),
            );

            let mut module_graph = compilation.get_module_graph_mut();
            for (m, depth) in assign_depths_map {
              module_graph.set_depth(m, depth)
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
            if modules_ordinal.bit(self.get_module_ordinal(*module)) {
              self._connect_count += 1;
              compilation
                .chunk_graph
                .connect_chunk_and_module(entry_chunk_ukey, *module);
            } else {
              skipped_modules += 1;
            }
          }

          chunks_ukey[idx] = entry_chunk_ukey;
        }
        ChunkDesc::Chunk(chunk_desc) => {
          let box NormalChunkDesc {
            original,
            options,
            chunk_modules,
            modules_ordinal,
            pre_order_indices,
            post_order_indices,
            incoming_blocks,
            outgoing_blocks: _,
            runtime,
          } = chunk_desc;

          let modules = chunk_modules
            .into_iter()
            .filter(|m| {
              if modules_ordinal.bit(self.get_module_ordinal(*m)) {
                true
              } else {
                skipped_modules += 1;
                false
              }
            })
            .collect::<Vec<_>>();
          if modules.is_empty() {
            // skip if this is empty chunk group
            skipped[idx] = true;
            continue;
          }

          let name = if let Some(option) = &options {
            option.name.as_ref()
          } else {
            None
          };

          let ukey = if let Some(name) = name {
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

          self
            .chunk_parent_modules
            .entry(ukey)
            .or_default()
            .extend(original);

          // TODO: do this in parallel
          for module in modules {
            self._connect_count += 1;
            compilation
              .chunk_graph
              .connect_chunk_and_module(ukey, module);
          }

          chunks_ukey[idx] = ukey;
        }
      }
    }

    for (entry, depend_on) in entries_depend_on {
      let entrypoint_ukey = *compilation.entrypoints.get(&entry).expect("unreachable");
      let entrypoint = compilation
        .chunk_group_by_ukey
        .get(&entrypoint_ukey)
        .expect("unreachable");
      let entry_point_chunk = compilation
        .chunk_by_ukey
        .expect_get(&entrypoint.get_entry_point_chunk());
      let entry_point_chunk_ukey = entry_point_chunk.ukey();
      let referenced_chunks =
        entry_point_chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey);

      let mut parents = vec![];

      for dep in depend_on {
        if let Some(dep_entrypoint_ukey) = compilation.entrypoints.get(&dep) {
          let dep_entrypoint = compilation
            .chunk_group_by_ukey
            .expect_get_mut(dep_entrypoint_ukey);
          let dependency_chunk_ukey = dep_entrypoint.get_entry_point_chunk();
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
      if skipped[idx] {
        continue;
      }

      let Some(children) = finalize_result.chunk_children.get(idx) else {
        continue;
      };

      let children = children.iter().filter(|child| !skipped[**child]);

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
          .filter(|cg| !compilation.async_entrypoints.contains(cg));
        let group = compilation.chunk_group_by_ukey.expect_get_mut(group);
        group.children.extend(children);
      }

      for child_ukey in children_groups {
        if compilation.async_entrypoints.contains(&child_ukey) {
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
    set_order_index_and_group_index(compilation);

    for (idx, err) in error_roots {
      errors.push(err.with_chunk(Some(chunks_ukey[idx].as_u32())));
    }
    compilation.extend_diagnostics(errors);

    Ok(())
  }

  // 1. determine parent child relationship
  // 2. remove modules that exist in all parents
  fn finalize_chunk_desc(
    &mut self,
    mut chunks: Vec<ChunkDesc>,
    _compilation: &Compilation,
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

    // one block.dep only point to a unique chunk
    // use this to a chunk is initialized because of which block.dep
    let mut chunks_origin_block = HashMap::default();

    // 1st iter, find roots and analyze which block belongs to which chunk
    // note that async entrypoint has no parents
    for (idx, chunk) in chunks.iter().enumerate() {
      let incoming_blocks = match chunk {
        ChunkDesc::Entry(entry) => &entry.incoming_blocks,
        ChunkDesc::Chunk(chunk) => &chunk.incoming_blocks,
      };
      for block_and_dep in incoming_blocks {
        if chunks_origin_block.insert(*block_and_dep, idx).is_some() {
          unreachable!()
        }
      }

      for block in chunk.outgoings() {
        chunks_by_block.entry(*block).or_default().push(idx);
      }
    }

    // 2nd iter, analyze chunk relations
    for chunk in chunks.iter() {
      if let ChunkDesc::Chunk(chunk) = chunk {
        let mut parents = HashSet::default();
        for block in &chunk.incoming_blocks {
          let Some(chunk_parents) = chunks_by_block.get(block) else {
            continue;
          };

          parents.extend(chunk_parents);
        }
        chunk_parents.push(parents.into_iter().collect::<Vec<_>>());
      } else {
        chunk_parents.push(Vec::default());
      }
    }

    for chunk in chunks.iter() {
      chunk_children.push(
        chunk
          .outgoings()
          .filter_map(|outgoing_block| chunks_origin_block.get(outgoing_block).copied())
          .collect(),
      );
    }

    // traverse through chunk graph,remove modules that already available in all parents
    let mut available_modules = vec![None; chunks_len];

    // 4rd iter, remove modules that is available in parents
    let remove = std::time::Instant::now();
    for chunk in 0..chunks_len {
      remove_available_modules(chunk, &mut chunks, &chunk_parents, &mut available_modules);
    }
    dbg!(remove.elapsed().as_millis());

    FinalizeChunksResult {
      chunks,
      chunk_children,
    }
  }

  // merge chunks that has the same name on them
  fn merge_same_chunks(
    &mut self,
    chunks: Vec<ChunkDesc>,
    compilation: &Compilation,
  ) -> Result<Vec<ChunkDesc>> {
    let mut final_chunks: Vec<ChunkDesc> = vec![];
    let mut named_chunks = HashMap::<String, usize>::default();

    for chunk in chunks {
      if let Some(name) = chunk.name() {
        if let Some(idx) = named_chunks.get(name).copied() {
          // there is existing chunk, merge them,
          // this is caused by same webpackChunkName
          merge_chunk(chunk, &mut final_chunks[idx])?;
          continue;
        } else {
          let idx = final_chunks.len();
          named_chunks.insert(name.to_string(), idx);
        }
      }
      final_chunks.push(chunk);
    }

    Ok(final_chunks)
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
        (Some(existing), Some(other)) => *existing = merge_runtime(&existing, &other),
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
    (ChunkDesc::Chunk(existing_chunk), ChunkDesc::Chunk(chunk)) => {
      existing_chunk.modules_ordinal |= chunk.modules_ordinal;
      existing_chunk.chunk_modules.extend(chunk.chunk_modules);
      existing_chunk.incoming_blocks.extend(chunk.incoming_blocks);
      existing_chunk.outgoing_blocks.extend(chunk.outgoing_blocks);
      existing_chunk.original.extend(chunk.original);
      match (&mut existing_chunk.runtime, chunk.runtime) {
        (None, None) => {}
        (None, Some(other)) => existing_chunk.runtime = Some(other),
        (Some(_), None) => {}
        (Some(existing), Some(other)) => *existing = merge_runtime(&existing, &other),
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
    (ChunkDesc::Entry(entry), ChunkDesc::Chunk(_)) => {
      return Err(error!(format!("Cannot add an async entrypoint with the name '{}', because there is already an chunk group with this name", entry.entry.as_ref().expect("already has name"))));
    }
    (ChunkDesc::Chunk(_), ChunkDesc::Entry(entry)) => {
      return Err(error!(format!("Cannot add an async entrypoint with the name '{}', because there is already an chunk group with this name", entry.entry.as_ref().expect("already has name"))));
    }
  };

  Ok(())
}

fn merge_modules_to_entry(chunk: &NormalChunkDesc, existing_chunk: &mut EntryChunkDesc) {
  existing_chunk.modules_ordinal |= &chunk.modules_ordinal;
  existing_chunk
    .chunk_modules
    .extend(chunk.chunk_modules.iter().copied());
}

pub fn code_split(compilation: &mut Compilation) -> Result<()> {
  let modules: Vec<ModuleIdentifier> = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect();
  let mut splitter = CodeSplitter::new(modules);

  let module_graph = compilation.get_module_graph();

  // find all modules'deps and blocks
  for m in module_graph.modules().values() {
    let id = m.identifier();
    for block_id in m.get_blocks() {
      let Some(block) = module_graph.block_by_id(block_id) else {
        continue;
      };
      if let Some(GroupOptions::Entrypoint(entry_options)) = block.get_group_options()
        && let Some(chunk_loading) = &entry_options.chunk_loading
        && matches!(chunk_loading, ChunkLoading::Disable)
      {
        continue;
      }

      splitter.blocks.insert(*block_id, id);
    }
  }

  // fill chunks with its modules
  splitter.create_chunks(compilation)?;

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

fn determine_runtime(
  current: ModuleIdentifier,
  module_graph: &ModuleGraph,
  entry_module_runtime: &IdentifierMap<RuntimeSpec>,
  module_cache: &DashMap<ModuleIdentifier, Option<RuntimeSpec>>,
  visited: &IdentifierDashSet,
) -> Option<RuntimeSpec> {
  let mut result_runtime: Option<RuntimeSpec> = None;

  if let Some(result) = module_cache.get(&current) {
    return result.value().clone();
  }

  let mut cycle_deps = false;

  for incoming in module_graph.get_incoming_connections(&current) {
    // parent is block and has runtime
    if let Some(block_id) = module_graph.get_parent_block(&incoming.dependency_id)
      && let Some(block) = module_graph.block_by_id(block_id)
      && let Some(group_options) = block.get_group_options()
      && let Some(entry_options) = group_options.entry_options()
      && let Some(runtime) = RuntimeSpec::from_entry_options(entry_options)
    {
      result_runtime = if let Some(result_runtime) = result_runtime {
        Some(merge_runtime(&result_runtime, &runtime))
      } else {
        Some(runtime)
      }
    } else {
      let Some(parent) = incoming.original_module_identifier else {
        // entry module
        if let Some(rt) = entry_module_runtime.get(&current) {
          result_runtime = match result_runtime {
            Some(result_rt) => Some(merge_runtime(&result_rt, &rt)),
            None => Some(rt.clone()),
          }
        }
        continue;
      };
      if visited.insert(parent) {
        if let Some(rt) = determine_runtime(
          parent,
          module_graph,
          entry_module_runtime,
          module_cache,
          visited,
        ) {
          result_runtime = match result_runtime {
            Some(result_rt) => Some(merge_runtime(&result_rt, &rt)),
            None => Some(rt),
          }
        }
      } else {
        cycle_deps = true;
      }
    }
  }

  if !cycle_deps {
    // have cycle
    module_cache.insert(current, result_runtime.clone());
  }

  result_runtime
}

fn remove_available_modules(
  curr: usize,
  chunks: &mut Vec<ChunkDesc>,
  chunk_parents: &Vec<Vec<usize>>,
  available_modules: &mut Vec<Option<BigInt>>,
) {
  if available_modules[curr].is_some() {
    // skip if it's already visited
    return;
  }

  let Some(parents) = chunk_parents.get(curr) else {
    unreachable!()
  };

  // current available modules
  let mut parents_available = None;
  // insert a placeholder to avoid cyclic
  available_modules[curr] = Some(chunks[curr].available_modules().clone());

  let parents = parents.clone();
  for parent in parents {
    let parent_available_modules = if let Some(finished) = &available_modules[parent] {
      finished.clone()
    } else {
      remove_available_modules(parent, chunks, chunk_parents, available_modules);
      available_modules[parent]
        .clone()
        .expect("already calculated")
    };

    if let Some(ref mut parents_available) = parents_available {
      *parents_available &= parent_available_modules
    } else {
      parents_available = Some(parent_available_modules);
    }
  }

  let chunk = &mut chunks[curr];
  if let Some(parents_available) = &parents_available {
    // a: parents:       0110
    // b: self:          1100
    // c: parents & self 0100
    // remove c in b
    // self:    1000
    let chunk_modules = chunk.available_modules().clone();
    let available = parents_available & &chunk_modules;
    let available_bitwise_not = &available ^ BigInt::from(-1);
    let result = chunk_modules & available_bitwise_not;
    available_modules[curr] = Some(parents_available | &result);
    *chunk.available_modules_mut() = result;
  } else {
    available_modules[curr] = Some(chunk.available_modules().clone());
  }
}

fn set_order_index_and_group_index(compilation: &mut Compilation) {
  enum Task {
    Group(ChunkGroupUkey, AsyncDependenciesBlockIdentifier),
    Enter(ModuleIdentifier),
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

  for (name, entry) in &compilation.entries {
    compilation
      .global_entry
      .all_dependencies()
      .chain(entry.all_dependencies())
      .filter_map(|dep| module_graph.module_identifier_by_dependency_id(dep))
      .for_each(|m| queue.push(Task::Enter(*m)));

    chunk_group_index += 1;
    chunk_group_indices.insert(
      *compilation.entrypoints.get(name).expect("unreachable"),
      chunk_group_index,
    );
  }

  let mut visited = IdentifierSet::default();
  let module_graph = compilation.get_module_graph();

  queue.reverse();

  while !queue.is_empty() {
    while let Some(task) = queue.pop() {
      match task {
        Task::Group(group_ukey, block_id) => {
          let module_graph = compilation.get_module_graph();
          let block = module_graph.block_by_id(&block_id);
          if let Some(block) = block {
            for dep_id in block.get_dependencies() {
              let Some(module) = module_graph.module_identifier_by_dependency_id(dep_id) else {
                continue;
              };
              queue.push(Task::Enter(*module));
            }
          }
          if !chunk_group_indices.contains_key(&group_ukey) {
            chunk_group_index += 1;
            chunk_group_indices.insert(group_ukey, chunk_group_index);
          }
        }
        Task::Enter(m) => {
          if !visited.insert(m) {
            continue;
          }

          pre_order_indices.insert(m, pre_order_index);
          pre_order_index += 1;
          queue.push(Task::Leave(m));

          let module = module_graph.module_by_identifier(&m).expect("unreachable");
          module
            .get_dependencies()
            .iter()
            .filter_map(|dep_id| module_graph.connection_by_dependency_id(dep_id))
            .map(|conn| conn.module_identifier())
            .rev()
            .for_each(|m| {
              queue.push(Task::Enter(*m));
            });

          for block_id in module.get_blocks() {
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

fn set_entry_runtime_and_depend_on(
  curr: usize,
  roots: &mut Vec<CreateChunkRoot>,
  entry_to_deps: &HashMap<&str, Vec<usize>>,
  visited: &mut BigInt,
  dep_chain: &mut Vec<usize>,
) -> std::result::Result<(), usize> {
  if visited.bit(curr as u64) {
    return Err(dep_chain.last().copied().unwrap_or(curr));
  }

  dep_chain.push(curr);
  visited.set_bit(curr as u64, true);
  let curr_root = &roots[curr];
  if let Some(deps) = entry_to_deps.get(curr_root.entry_name()) {
    let mut runtime = None;
    for dep in deps {
      let dep_root = &roots[*dep];
      let rt = if let Some(rt) = dep_root.get_runtime() {
        rt
      } else {
        set_entry_runtime_and_depend_on(*dep, roots, entry_to_deps, visited, dep_chain)?;
        roots[*dep].get_runtime().expect("already set")
      };

      match &mut runtime {
        Some(runtime) => *runtime = merge_runtime(runtime, rt),
        None => runtime = Some(rt.clone()),
      }
    }

    let curr_root = &mut roots[curr];
    curr_root.set_runtime(
      runtime.unwrap_or_else(|| panic!("not runtime for entry: {}", curr_root.entry_name())),
    );
  }
  Ok(())
}
