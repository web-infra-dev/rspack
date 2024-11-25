use std::hash::BuildHasherDefault;
use std::iter::once;

use bit_set::BitSet;
use dashmap::DashMap;
use indexmap::map::{IntoIter, Iter};
use num_bigint::BigInt;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use rspack_collections::{
  DatabaseItem, IdentifierHasher, IdentifierIndexMap, IdentifierIndexSet, IdentifierMap,
  IdentifierSet, ItemUkey,
};
use rspack_util::fx_hash::FxIndexSet;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  is_runtime_equal, merge_runtime, AsyncDependenciesBlockIdentifier, ChunkGroup, ChunkGroupKind,
  ChunkGroupOptions, ChunkUkey, Compilation, ConnectionState, DependenciesBlock, DependencyId,
  EntryData, EntryOptions, GroupOptions, Logger, ModuleIdentifier, RuntimeSpec,
};

type AsyncIdentifierSet = std::collections::hash_set::HashSet<
  AsyncDependenciesBlockIdentifier,
  BuildHasherDefault<IdentifierHasher>,
>;

// #[derive(Debug, Default)]
// struct ModulesRecord(IdentifierIndexMap<HashSet<DependencyId>>);

// impl ModulesRecord {
//   fn add_module_by_dependencies(&mut self, module: ModuleIdentifier) -> bool {
//     self.0.insert(module, Default::default())
//   }

//   fn remove_module(&mut self, module: ModuleIdentifier) {
//     self.0.shift_remove(&module);
//   }

//   fn iter(&self) -> Iter<ModuleIdentifier, HashSet<DependencyId>> {
//     self.0.iter()
//   }
// }

// impl IntoIterator for ModulesRecord {
//   type Item = (ModuleIdentifier, HashSet<DependencyId>);

//   type IntoIter = IntoIter<ModuleIdentifier, HashSet<DependencyId>>;

//   fn into_iter(self) -> Self::IntoIter {
//     self.0.into_iter()
//   }
// }

// impl ModulesRecord {
//   fn new(map: IdentifierIndexMap<HashSet<DependencyId>>) -> Self {
//     Self(map)
//   }
// }

#[derive(Debug)]
pub struct CodeSplitter {
  pub next_chunk_group_index: usize,
  pub _connect_count: usize,
  pub module_deps: IdentifierMap<(IdentifierIndexSet, Vec<AsyncDependenciesBlockIdentifier>)>,
  pub blocks: HashMap<AsyncDependenciesBlockIdentifier, ModuleIdentifier>,
  pub chunk_parent_modules: HashMap<ChunkUkey, HashSet<ModuleIdentifier>>,
  pub module_ordinal: IdentifierMap<u64>,
  pub chunk_modules: HashMap<ChunkUkey, BigInt>,
  pub modules: Vec<ModuleIdentifier>,
}

#[derive(Debug)]
struct ConnectionDesc {
  module_id: ModuleIdentifier,
  dep_id: Vec<DependencyId>,
}

#[derive(Debug)]
enum CreateChunkRoot {
  Entry(String, EntryData),
  Block(ModuleIdentifier, AsyncDependenciesBlockIdentifier),
}

struct FinalizeChunksResult {
  chunks: Vec<ChunkDesc>,
  chunk_parents: HashMap<usize, Vec<usize>>,
  roots: Vec<usize>,
}

// Description about how to create chunk
#[derive(Debug)]
enum ChunkDesc {
  // Name, entry_modules, Options, Modules
  Entry(Box<EntryChunkDesc>),

  // original, root, Options, Modules
  Chunk(Box<NormalChunkDesc>),
}

impl ChunkDesc {
  fn name(&self) -> Option<&str> {
    match self {
      ChunkDesc::Entry(entry) => entry.entry.as_ref().map(|s| s.as_str()),
      ChunkDesc::Chunk(chunk) => chunk
        .options
        .as_ref()
        .and_then(|opt| opt.name.as_ref().map(|s| s.as_str())),
    }
  }

  fn runtime(&self) -> Option<&RuntimeSpec> {
    match self {
      ChunkDesc::Entry(entry) => entry.runtime.as_ref(),
      ChunkDesc::Chunk(chunk) => chunk.runtime.as_ref(),
    }
  }

  fn outgoings(&self) -> impl Iterator<Item = &AsyncDependenciesBlockIdentifier> {
    match self {
      ChunkDesc::Entry(entry) => entry.outgoing_blocks.iter(),
      ChunkDesc::Chunk(chunk) => chunk.outgoing_blocks.iter(),
    }
  }

  fn modules_mut(&mut self) -> &mut IdentifierSet {
    match self {
      ChunkDesc::Entry(entry) => &mut entry.chunk_modules,
      ChunkDesc::Chunk(chunk) => &mut chunk.chunk_modules,
    }
  }

  fn modules(&self) -> &IdentifierSet {
    match self {
      ChunkDesc::Entry(entry) => &entry.chunk_modules,
      ChunkDesc::Chunk(chunk) => &chunk.chunk_modules,
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

#[derive(Debug)]
struct EntryChunkDesc {
  original: HashSet<ModuleIdentifier>,
  async_entrypoint: bool,

  options: EntryOptions,
  modules_ordinal: BigInt,
  entry: Option<String>,
  entry_modules: Vec<ModuleIdentifier>,
  chunk_modules: IdentifierSet,

  // use incoming and outgoing to track chunk relations,
  // entry has no incomings
  outgoing_blocks: AsyncIdentifierSet,
  runtime: Option<RuntimeSpec>,
}

#[derive(Debug)]
struct NormalChunkDesc {
  original: HashSet<ModuleIdentifier>,

  options: Option<ChunkGroupOptions>,
  modules_ordinal: BigInt,
  chunk_modules: IdentifierSet,

  // use incoming and outgoing to track chunk relations
  incoming_blocks: HashSet<AsyncDependenciesBlockIdentifier>,
  outgoing_blocks: AsyncIdentifierSet,
  runtime: Option<RuntimeSpec>,
}

impl CreateChunkRoot {
  fn create(&self, splitter: &CodeSplitter, compilation: &Compilation) -> Vec<ChunkDesc> {
    let module_graph: crate::ModuleGraph = compilation.get_module_graph();
    match self {
      CreateChunkRoot::Entry(entry, data) => {
        let mut entry_modules = vec![];
        let mut chunk_modules = IdentifierSet::default();
        let mut outgoing_blocks = AsyncIdentifierSet::default();
        let mut modules_ordinal = BigInt::from(0u64);

        for (dep_id, m) in data.all_dependencies().filter_map(|dep_id| {
          module_graph
            .module_identifier_by_dependency_id(dep_id)
            .map(|m| (*dep_id, m))
        }) {
          entry_modules.push(*m);
          splitter.fill_chunk_modules(
            *m,
            &mut chunk_modules,
            &mut outgoing_blocks,
            &mut modules_ordinal,
          );
        }

        vec![ChunkDesc::Entry(Box::new(EntryChunkDesc {
          async_entrypoint: true,
          original: Default::default(),
          entry: Some(entry.clone()),
          entry_modules,
          chunk_modules,
          options: data.options.clone(),
          modules_ordinal,
          outgoing_blocks,
          runtime: RuntimeSpec::from_entry_options(&data.options),
        }))]
      }
      CreateChunkRoot::Block(origin, block_id) => {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block");

        let mut chunks = vec![];
        for dep_id in block.get_dependencies() {
          let Some(m) = module_graph.module_identifier_by_dependency_id(dep_id) else {
            continue;
          };
          let mut chunk_modules = IdentifierSet::default();
          let mut outgoing_blocks = AsyncIdentifierSet::default();
          let mut modules_ordinal = BigInt::from(0u64);
          splitter.fill_chunk_modules(
            *m,
            &mut chunk_modules,
            &mut outgoing_blocks,
            &mut modules_ordinal,
          );

          if let Some(group_option) = block.get_group_options()
            && let Some(entry_options) = group_option.entry_options()
          {
            chunks.push(ChunkDesc::Entry(Box::new(EntryChunkDesc {
              async_entrypoint: true,
              original: once(*origin).collect(),
              options: entry_options.clone(),
              modules_ordinal,
              entry: entry_options.name.clone(),
              entry_modules: Default::default(),
              chunk_modules,
              outgoing_blocks,
              runtime: RuntimeSpec::from_entry_options(entry_options),
            })))
          } else {
            chunks.push(ChunkDesc::Chunk(Box::new(NormalChunkDesc {
              original: once(*origin).collect(),
              chunk_modules,
              options: block.get_group_options().map(|opt| match opt {
                GroupOptions::Entrypoint(_) => unreachable!(),
                GroupOptions::ChunkGroup(group_option) => group_option.clone(),
              }),
              modules_ordinal,

              incoming_blocks: std::iter::once(*block_id).collect(),
              outgoing_blocks,

              runtime: None,
            })));
          }
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
      next_chunk_group_index: 0,
      _connect_count: 0,
      module_deps: Default::default(),
      blocks: Default::default(),
      chunk_parent_modules: Default::default(),
      module_ordinal,
      modules,
      chunk_modules: Default::default(),
    }
  }

  fn get_module_ordinal(&self, m: ModuleIdentifier) -> u64 {
    *self
      .module_ordinal
      .get(&m)
      .unwrap_or_else(|| panic!("should have module ordinal: {}", m))
  }

  // insert static dependencies into a set
  fn fill_chunk_modules(
    &self,
    target_module: ModuleIdentifier,
    chunk_modules: &mut IdentifierSet,
    out_goings: &mut AsyncIdentifierSet,
    module_ordinal: &mut BigInt,
  ) {
    if !chunk_modules.insert(target_module) {
      return;
    }

    let module = self.get_module_ordinal(target_module);

    if module_ordinal.bit(module) {
      // we already include this module
      return;
    } else {
      module_ordinal.set_bit(module, true);
    }

    if let Some((outgoing_modules, blocks)) = self.module_deps.get(&target_module) {
      out_goings.extend(blocks.clone());
      for m in outgoing_modules.iter() {
        self.fill_chunk_modules(*m, chunk_modules, out_goings, module_ordinal);
      }
    }
  }

  fn fill_module_deps_and_blocks(&mut self, compilation: &Compilation) {
    let module_graph = compilation.get_module_graph();
    let entry = &compilation.entries.iter().next().unwrap();
    let runtime = &entry.1.options.runtime;

    let rt = runtime
      .clone()
      .map(|rt| RuntimeSpec::from_entry(&entry.0, runtime.as_ref()));

    for (module_id, m) in module_graph.modules() {
      let blocks = m.get_blocks();

      for block_id in blocks {
        self.blocks.insert(*block_id, module_id);
      }

      let mut outgoings = IdentifierIndexSet::default();
      let dep_modules = m
        .get_dependencies()
        .into_iter()
        .filter_map(|dep_id| module_graph.connection_by_dependency_id(dep_id));

      let mut module_deps = IdentifierMap::<Vec<_>>::default();
      for conn in dep_modules {
        module_deps
          .entry(*conn.module_identifier())
          .or_default()
          .push(conn);
      }

      for (m, connections) in module_deps {
        for conn in connections {
          // if one dep is active, then this module is active
          if conn.is_active(&module_graph, None) {
            outgoings.insert(m);
            break;
          }
        }
      }

      self.module_deps.insert(
        module_id,
        (outgoings, blocks.into_iter().copied().collect()),
      );
    }
  }

  fn create_chunks(&mut self, compilation: &mut Compilation) {
    let entries = &compilation.entries;
    let mut roots = vec![];

    for (name, data) in entries {
      roots.push(CreateChunkRoot::Entry(name.clone(), data.clone()));
    }

    for (block, origin) in &self.blocks {
      roots.push(CreateChunkRoot::Block(*origin, *block));
    }

    // fill chunk with modules in parallel
    let _fill_chunks = std::time::Instant::now();
    let chunks = roots
      .par_iter()
      .map(|root| root.create(self, compilation))
      .flatten()
      .collect::<Vec<_>>();
    dbg!(_fill_chunks.elapsed().as_millis());

    let _merge_chunks = std::time::Instant::now();
    let chunks = self.merge_same_chunks(chunks);
    dbg!(_merge_chunks.elapsed().as_millis());

    let _finalize = std::time::Instant::now();
    // remove available modules if could
    // determin chunk graph relations
    // determin runtime
    let finalize_result = self.finalize_chunks(chunks, &compilation);
    dbg!(_finalize.elapsed().as_millis());

    let chunks_len = finalize_result.chunks.len();
    let _add_chunk_to_compilation = std::time::Instant::now();
    let mut chunks_ukey = vec![0.into(); chunks_len];
    let mut skipped = HashSet::default();

    for (idx, chunk_desc) in finalize_result.chunks.into_iter().enumerate() {
      match chunk_desc {
        ChunkDesc::Entry(entry_desc) => {
          let box EntryChunkDesc {
            original,
            entry,
            entry_modules,
            chunk_modules,
            options,
            modules_ordinal,
            outgoing_blocks,
            runtime,
            async_entrypoint,
          } = entry_desc;

          let mut entry_chunk_ukey = if let Some(chunk_name) = &entry {
            Compilation::add_named_chunk(
              chunk_name.clone(),
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            )
            .0
          } else {
            Compilation::add_chunk(&mut compilation.chunk_by_ukey)
          };

          let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
            true,
            Box::new(options.clone()),
          ));

          let entry_chunk = compilation.chunk_by_ukey.expect_get_mut(&entry_chunk_ukey);

          if let Some(runtime) = runtime {
            entry_chunk.set_runtime(runtime);
          }

          if let Some(filename) = &options.filename {
            entry_chunk.set_filename_template(Some(filename.clone()));
          }

          if let Some(chunk_name) = entry {
            compilation
              .entrypoints
              .insert(chunk_name.clone(), entrypoint.ukey);
          }

          if let Some(name) = entrypoint.kind.name() {
            compilation
              .named_chunk_groups
              .insert(name.to_owned(), entrypoint.ukey);
          }
          entrypoint.connect_chunk(entry_chunk);
          entrypoint.set_runtime_chunk(entry_chunk_ukey);
          entrypoint.set_entry_point_chunk(entry_chunk_ukey);

          if async_entrypoint {
            compilation.async_entrypoints.push(entrypoint.ukey);
          }

          // TODO: handle chunk index and module index
          self.next_chunk_group_index += 1;
          entrypoint.next_pre_order_index = self.next_chunk_group_index;
          self.next_chunk_group_index += 1;
          entrypoint.next_post_order_index = self.next_chunk_group_index;
          self.next_chunk_group_index += 1;
          entrypoint.index = Some(self.next_chunk_group_index as u32);
          let entrypoint_ukey = entrypoint.ukey;

          compilation.chunk_group_by_ukey.add(entrypoint);
          compilation.chunk_graph.add_chunk(entry_chunk_ukey);
          compilation.async_entrypoints.push(entrypoint_ukey);

          for m in entry_modules {
            compilation.chunk_graph.connect_chunk_and_entry_module(
              entry_chunk_ukey,
              m,
              entrypoint_ukey,
            );

            self._connect_count += 1;
            compilation
              .chunk_graph
              .connect_chunk_and_module(entry_chunk_ukey, m);
          }

          // TODO: do this in parallel
          for module in chunk_modules.iter() {
            if modules_ordinal.bit(self.get_module_ordinal(*module) as u64) {
              self._connect_count += 1;
              compilation
                .chunk_graph
                .connect_chunk_and_module(entry_chunk_ukey, *module);
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
            incoming_blocks,
            outgoing_blocks,
            runtime,
          } = chunk_desc;
          let Some(runtime) = runtime else {
            skipped.insert(idx);
            continue;
          };
          let name = if let Some(option) = &options {
            option.name.as_ref()
          } else {
            None
          };

          let ukey = if let Some(name) = name {
            Compilation::add_named_chunk(
              name.clone(),
              &mut compilation.chunk_by_ukey,
              &mut compilation.named_chunks,
            )
            .0
          } else {
            Compilation::add_chunk(&mut compilation.chunk_by_ukey)
          };

          let mut group = ChunkGroup::new(crate::ChunkGroupKind::Normal {
            options: options.clone().unwrap_or_default(),
          });
          let group_ukey = group.ukey;

          let chunk = compilation.chunk_by_ukey.expect_get_mut(&ukey);
          chunk.set_runtime(runtime);
          group.connect_chunk(chunk);
          self.next_chunk_group_index += 1;
          group.next_pre_order_index = self.next_chunk_group_index;
          self.next_chunk_group_index += 1;
          group.next_post_order_index = self.next_chunk_group_index;
          self.next_chunk_group_index += 1;
          group.index = Some(self.next_chunk_group_index as u32);

          compilation.chunk_group_by_ukey.add(group);
          compilation.chunk_graph.add_chunk(ukey);

          for block in incoming_blocks {
            compilation
              .chunk_graph
              .connect_block_and_chunk_group(block, group_ukey);
          }

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
          for module in chunk_modules.iter() {
            if modules_ordinal.bit(self.get_module_ordinal(*module) as u64) {
              self._connect_count += 1;
              compilation
                .chunk_graph
                .connect_chunk_and_module(ukey, *module);
            }
          }

          chunks_ukey[idx] = ukey;
        }
      }
    }

    // connect parent and children
    for idx in 0..chunks_len {
      if skipped.contains(&idx) {
        continue;
      }
      let Some(parents) = finalize_result.chunk_parents.get(&idx) else {
        continue;
      };

      let chunk = compilation.chunk_by_ukey.expect_get(&chunks_ukey[idx]);

      let groups = chunk.groups().clone();
      let parent_groups = parents
        .iter()
        .map(|parent| {
          compilation
            .chunk_by_ukey
            .expect_get(&chunks_ukey[*parent])
            .groups()
        })
        .flatten()
        .copied();

      for group in &groups {
        let group = compilation.chunk_group_by_ukey.expect_get_mut(group);
        group.parents.extend(parent_groups.clone());
      }

      for parent in parent_groups {
        let parent = compilation.chunk_group_by_ukey.expect_get_mut(&parent);
        parent.children.extend(groups.clone());
      }
    }

    dbg!(_add_chunk_to_compilation.elapsed().as_millis());
  }

  // 1. determine parent child relationship
  // 2. remove modules that exist in all parents
  fn finalize_chunks(
    &mut self,
    mut chunks: Vec<ChunkDesc>,
    _compilation: &Compilation,
  ) -> FinalizeChunksResult {
    // map that records info about chunk to its parents
    let mut chunk_parents = HashMap::default();
    let mut chunks_by_block: HashMap<AsyncDependenciesBlockIdentifier, Vec<usize>> =
      HashMap::default();
    let mut roots = Vec::default();

    // 1st iter, find roots and analyze which block belongs to which chunk
    for (idx, chunk) in chunks.iter().enumerate() {
      if matches!(chunk, ChunkDesc::Entry(_)) {
        roots.push(idx);
      }

      for block in chunk.outgoings() {
        chunks_by_block.entry(*block).or_default().push(idx);
      }
    }

    // 2nd iter, analyze chunk relations
    for (curr_chunk_idx, chunk) in chunks.iter().enumerate() {
      chunk_parents.insert(curr_chunk_idx, Vec::default());
      if let ChunkDesc::Chunk(chunk) = chunk {
        let mut parents = HashSet::default();
        for block in &chunk.incoming_blocks {
          let Some(chunk_parents) = chunks_by_block.get(block) else {
            continue;
          };

          parents.extend(chunk_parents);
        }
        chunk_parents.insert(curr_chunk_idx, parents.into_iter().collect::<Vec<_>>());
      }
    }

    let chunks_len = chunks.len();

    // 3rd iter, merge runtime
    fn merge_chunk_runtime<'chunks>(
      curr: usize,
      chunks: &'chunks mut Vec<ChunkDesc>,
      chunk_parents: &HashMap<usize, Vec<usize>>,
      visited_runtime: &mut Vec<Option<RuntimeSpec>>,
    ) {
      if let Some(rt) = &visited_runtime[curr] {
        if is_runtime_equal(rt, &PLACE_HOLDER_RT) {
          panic!("recursive");
        }
        return;
      }

      if let ChunkDesc::Entry(entry) = &chunks[curr] {
        visited_runtime[curr] = entry.runtime.clone();
        return;
      }

      let parents = chunk_parents
        .get(&curr)
        .unwrap_or_else(|| panic!("parents already been set in above phase: chunk {:?}", curr));
      let mut runtime = None;

      static PLACE_HOLDER_RT: Lazy<RuntimeSpec> =
        Lazy::new(|| RuntimeSpec::from_iter(["&RSPACK_PLACEHOLDER_RT$".into()]));
      visited_runtime[curr] = Some(PLACE_HOLDER_RT.clone());

      for parent in parents {
        let rt = if let Some(parent_rt) = &visited_runtime[*parent] {
          parent_rt
        } else {
          merge_chunk_runtime(*parent, chunks, chunk_parents, visited_runtime);
          visited_runtime[*parent].as_ref().expect("already been set")
        };

        if let Some(runtime) = &mut runtime {
          *runtime = merge_runtime(&runtime, rt);
        } else {
          runtime = Some(rt.clone());
        }
      }

      visited_runtime[curr] = runtime.clone();
      match &mut chunks[curr] {
        ChunkDesc::Entry(_) => {
          unreachable!()
        }
        ChunkDesc::Chunk(chunk) => {
          chunk.runtime = runtime;
        }
      }
    }

    let mut visited = vec![None; chunks_len];
    for chunk in 0..chunks_len {
      merge_chunk_runtime(chunk, &mut chunks, &chunk_parents, &mut visited);
    }

    // 4rd iter, remove inactive modules for each runtime
    // do this in parallel
    // let _active_optimize = std::time::Instant::now();
    // // TODO: can we skip this optimization in dev mode or expose switch ?
    // chunks.par_iter_mut().for_each(|chunk| {
    //   // some modules are not inactive according to certain runtime
    //   let chunk_modules = chunk.modules();
    //   let runtime = chunk.runtime();
    //   let mut inactive = IdentifierSet::default();
    //   let mg = compilation.get_module_graph();
    //   for module in chunk_modules.iter() {
    //     let mut active = false;
    //     for dep in by_deps {
    //       let Some(conn) = mg.connection_by_dependency_id(dep) else {
    //         continue;
    //       };

    //       let active_state = ConnectionState::Bool(true);
    //       // let active_state = conn.active_state(&mg, runtime);
    //       if active_state.is_true() {
    //         active = true;
    //         // break;
    //       }
    //     }
    //     if !active {
    //       inactive.insert(*module);
    //     }
    //   }

    //   // remove inactive modules
    //   for module in inactive {
    //     let idx = self.get_module_ordinal(module);
    //     chunk.available_modules_mut().set_bit(idx, false);
    //     chunk.modules_mut().remove(&module);
    //   }
    // });
    // dbg!(_active_optimize.elapsed().as_millis());

    // traverse through chunk graph,remove modules that already available in all parents
    fn remove_available_modules(
      curr: usize,
      chunks: &mut Vec<ChunkDesc>,
      chunk_parents: &HashMap<usize, Vec<usize>>,
      available_modules: &mut Vec<Option<BigInt>>,
    ) {
      if available_modules.get(curr).is_none() {
        // skip if it's already visited
        return;
      }

      let Some(parents) = chunk_parents.get(&curr) else {
        // skip if it's entry chunk
        return;
      };

      // current available modules
      let mut parents_available = None;
      // insert a placeholder to avoid cyclic
      available_modules[curr] = Some(BigInt::from(0u32));

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

    let mut available_modules = vec![None; chunks_len];

    // 5rd iter, remove modules that available in parents
    let _remove_modules = std::time::Instant::now();
    for chunk in 0..chunks_len {
      remove_available_modules(chunk, &mut chunks, &chunk_parents, &mut available_modules);
    }

    dbg!(_remove_modules.elapsed().as_millis());

    FinalizeChunksResult {
      chunks,
      chunk_parents,
      roots,
    }
  }

  fn merge_same_chunks(&self, chunks: Vec<ChunkDesc>) -> Vec<ChunkDesc> {
    let mut final_chunks = vec![];
    let mut named_chunks: std::collections::HashMap<
      String,
      usize,
      BuildHasherDefault<rustc_hash::FxHasher>,
    > = HashMap::<String, usize>::default();

    for chunk in chunks {
      if let Some(name) = chunk.name() {
        if let Some(idx) = named_chunks.get(name).copied() {
          // there is existing chunk, merge them,
          // this is caused by same webpackChunkName
          let ChunkDesc::Chunk(ref mut existing_chunk) = final_chunks[idx] else {
            unreachable!()
          };

          let ChunkDesc::Chunk(chunk) = chunk else {
            unreachable!()
          };

          existing_chunk.modules_ordinal |= chunk.modules_ordinal;
          existing_chunk.incoming_blocks.extend(chunk.incoming_blocks);
          existing_chunk.outgoing_blocks.extend(chunk.outgoing_blocks);
          existing_chunk.original.extend(chunk.original);
          continue;
        } else {
          let idx = final_chunks.len();
          named_chunks.insert(name.to_string(), idx);
        }
      }
      final_chunks.push(chunk);
    }

    dbg!(final_chunks.len());
    final_chunks
  }
}

pub fn code_split(compilation: &mut Compilation) {
  let modules: Vec<ModuleIdentifier> = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect();
  let mut splitter = CodeSplitter::new(modules);

  // find all modules'deps and blocks
  let fill_module_deps = std::time::Instant::now();
  splitter.fill_module_deps_and_blocks(compilation);
  dbg!(fill_module_deps.elapsed().as_millis());

  // fill chunks with its modules
  splitter.create_chunks(compilation);

  dbg!(splitter._connect_count);

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
}
