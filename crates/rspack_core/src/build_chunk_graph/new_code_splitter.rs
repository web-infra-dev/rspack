use std::hash::BuildHasherDefault;

use bit_set::BitSet;
use num_bigint::BigInt;
use rayon::{iter::split, prelude::*};
use rspack_collections::{DatabaseItem, IdentifierHasher, IdentifierMap, IdentifierSet, ItemUkey};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGroup, ChunkGroupKind, ChunkGroupOptions, ChunkUkey,
  Compilation, DependenciesBlock, DependencyId, EntryData, EntryOptions, GroupOptions, Logger,
  ModuleIdentifier,
};

type AsyncIdentifierSet = std::collections::hash_set::HashSet<
  AsyncDependenciesBlockIdentifier,
  BuildHasherDefault<IdentifierHasher>,
>;

#[derive(Debug)]
pub struct CodeSplitter {
  pub _connect_count: usize,
  pub module_deps: IdentifierMap<(Vec<DepRecord>, Vec<AsyncDependenciesBlockIdentifier>)>,
  pub blocks: HashMap<AsyncDependenciesBlockIdentifier, ModuleIdentifier>,
  pub chunk_parent_modules: HashMap<ChunkUkey, HashSet<ModuleIdentifier>>,
  pub module_ordinal: IdentifierMap<u64>,
  pub chunk_modules: HashMap<ChunkUkey, BigInt>,
  pub modules: Vec<ModuleIdentifier>,
}

#[derive(Debug)]
struct DepRecord {
  module_id: ModuleIdentifier,
  dep_id: DependencyId,
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
      ChunkDesc::Entry(entry) => Some(&entry.entry),
      ChunkDesc::Chunk(chunk) => chunk
        .options
        .as_ref()
        .and_then(|opt| opt.name.as_ref().map(|s| s.as_str())),
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

#[derive(Debug)]
struct EntryChunkDesc {
  entry: String,
  entry_modules: IdentifierSet,
  options: EntryOptions,
  modules_ordinal: BigInt,
  chunk_modules: IdentifierSet,
  // use incoming and outgoing to track chunk relations,
  // entry has no incomings
  outgoing_blocks: AsyncIdentifierSet,
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
}

impl CreateChunkRoot {
  fn create(&self, splitter: &CodeSplitter, compilation: &Compilation) -> Vec<ChunkDesc> {
    let module_graph: crate::ModuleGraph = compilation.get_module_graph();
    match self {
      CreateChunkRoot::Entry(entry, data) => {
        let mut entry_modules = IdentifierSet::default();
        let mut chunk_modules = IdentifierSet::default();
        let mut outgoing_blocks = AsyncIdentifierSet::default();
        let mut modules_ordinal = BigInt::from(0u64);

        for m in data
          .all_dependencies()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        {
          entry_modules.insert(*m);
          splitter.fill_dependencies(
            *m,
            &mut chunk_modules,
            &mut outgoing_blocks,
            &mut modules_ordinal,
          );
        }

        vec![ChunkDesc::Entry(Box::new(EntryChunkDesc {
          entry: entry.clone(),
          entry_modules,
          chunk_modules,
          options: data.options.clone(),
          modules_ordinal,
          outgoing_blocks,
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
          splitter.fill_dependencies(
            *m,
            &mut chunk_modules,
            &mut outgoing_blocks,
            &mut modules_ordinal,
          );
          chunks.push(ChunkDesc::Chunk(Box::new(NormalChunkDesc {
            original: std::iter::once(*origin).collect(),
            chunk_modules,
            options: block.get_group_options().map(|opt| match opt {
              GroupOptions::Entrypoint(_) => unreachable!(),
              GroupOptions::ChunkGroup(group_option) => group_option.clone(),
            }),
            modules_ordinal,

            incoming_blocks: std::iter::once(*block_id).collect(),
            outgoing_blocks,
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
  fn fill_dependencies(
    &self,
    root: ModuleIdentifier,
    set: &mut IdentifierSet,
    out_goings: &mut AsyncIdentifierSet,
    module_ordinal: &mut BigInt,
  ) {
    if !set.insert(root) {
      return;
    }

    let module = self.get_module_ordinal(root);
    if module_ordinal.bit(module) {
      return;
    }

    module_ordinal.set_bit(module, true);
    if let Some((deps, blocks)) = self.module_deps.get(&root) {
      out_goings.extend(blocks.clone());
      for dep in deps {
        self.fill_dependencies(dep.module_id, set, out_goings, module_ordinal);
      }
    }
  }

  fn fill_module_deps_and_blocks(&mut self, compilation: &Compilation) {
    let module_graph = compilation.get_module_graph();
    for (module_id, m) in module_graph.modules() {
      let blocks = m.get_blocks();

      for block_id in blocks {
        self.blocks.insert(*block_id, module_id);
      }

      let mut deps = m
        .get_dependencies()
        .into_iter()
        .filter_map(|dep_id| module_graph.connection_by_dependency_id(dep_id))
        .map(|conn| DepRecord {
          module_id: *conn.module_identifier(),
          dep_id: conn.dependency_id,
        })
        .collect::<Vec<_>>();

      deps.dedup_by_key(|rec| rec.module_id);

      self
        .module_deps
        .insert(module_id, (deps, blocks.into_iter().copied().collect()));
    }
  }

  fn create_chunks(&mut self, compilation: &mut Compilation) {
    let start = std::time::Instant::now();
    let entries = &compilation.entries;
    let mut roots = vec![];

    for (name, data) in entries {
      roots.push(CreateChunkRoot::Entry(name.clone(), data.clone()));
    }

    for (block, origin) in &self.blocks {
      roots.push(CreateChunkRoot::Block(*origin, *block));
    }

    // fill chunk with modules in parallel
    let chunks = roots
      .par_iter()
      .map(|root| root.create(self, compilation))
      .flatten()
      .collect::<Vec<_>>();

    let chunks = self.merge_same_chunks(chunks);
    dbg!(start.elapsed().as_millis());

    let remove_available = std::time::Instant::now();
    // remove available modules if could
    let finalize_result = self.finalize_chunks(chunks);
    dbg!(remove_available.elapsed().as_millis());

    for chunk_desc in finalize_result.chunks {
      match chunk_desc {
        ChunkDesc::Entry(entry_desc) => {
          let box EntryChunkDesc {
            entry,
            entry_modules,
            chunk_modules,
            options,
            modules_ordinal,
            outgoing_blocks,
          } = entry_desc;

          let mut entry_chunk_ukey = Compilation::add_named_chunk(
            entry.clone(),
            &mut compilation.chunk_by_ukey,
            &mut compilation.named_chunks,
          )
          .0;

          let mut entrypoint = ChunkGroup::new(ChunkGroupKind::new_entrypoint(
            true,
            Box::new(options.clone()),
          ));

          let entry_chunk = compilation.chunk_by_ukey.expect_get_mut(&entry_chunk_ukey);
          entrypoint.connect_chunk(entry_chunk);
          entrypoint.set_runtime_chunk(entry_chunk_ukey);
          entrypoint.set_entry_point_chunk(entry_chunk_ukey);
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

          compilation
            .named_chunk_groups
            .insert(entry, entrypoint_ukey);
        }
        ChunkDesc::Chunk(chunk_desc) => {
          let box NormalChunkDesc {
            original,
            options,
            chunk_modules,
            modules_ordinal,
            incoming_blocks,
            outgoing_blocks,
          } = chunk_desc;
          let name = if let Some(option) = &options {
            option.name.as_ref()
          } else {
            None
          };

          let chunk_ukey = if let Some(name) = name
            && let Some(chunk) = compilation.named_chunks.get(name)
          {
            // reusing
            *chunk
          } else {
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
            group.connect_chunk(chunk);

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

            ukey
          };

          self
            .chunk_parent_modules
            .entry(chunk_ukey)
            .or_default()
            .extend(original);

          // TODO: do this in parallel
          for module in chunk_modules.iter() {
            if modules_ordinal.bit(self.get_module_ordinal(*module) as u64) {
              self._connect_count += 1;
              compilation
                .chunk_graph
                .connect_chunk_and_module(chunk_ukey, *module);
            }
          }
        }
      }
    }
  }

  // 1. determine parent child relationship
  // 2. remove modules that exist in all parents
  fn finalize_chunks(&mut self, mut chunks: Vec<ChunkDesc>) -> FinalizeChunksResult {
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

    // 2rd iter, analyze chunk relations
    for (curr_chunk_idx, chunk) in chunks.iter().enumerate() {
      match chunk {
        ChunkDesc::Entry(_) => {
          chunk_parents.insert(curr_chunk_idx, Default::default());
        }
        ChunkDesc::Chunk(chunk) => {
          for block in &chunk.incoming_blocks {
            let Some(parents) = chunks_by_block.get(block) else {
              continue;
            };

            chunk_parents.insert(curr_chunk_idx, parents.clone());
          }
        }
      }
    }

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

    let chunks_len = chunks.len();
    let mut available_modules = vec![None; chunks_len];
    for chunk in 0..chunks_len {
      remove_available_modules(chunk, &mut chunks, &chunk_parents, &mut available_modules);
    }

    FinalizeChunksResult {
      chunks,
      chunk_parents,
      roots,
    }
  }

  fn link_chunks(&self, compilation: &mut Compilation) {
    for chunk in compilation
      .chunk_by_ukey
      .keys()
      .cloned()
      .collect::<Vec<_>>()
    {
      let groups = compilation
        .chunk_by_ukey
        .expect_get(&chunk)
        .groups()
        .clone();

      let Some(parent_modules) = self.chunk_parent_modules.get(&chunk) else {
        continue;
      };

      for parent_module in parent_modules {
        let chunks = compilation.chunk_graph.get_module_chunks(*parent_module);
        chunks
          .iter()
          .filter_map(|c| compilation.chunk_by_ukey.get(c))
          .map(|chunk| chunk.groups())
          .flatten()
          .for_each(|parent| {
            let parnet_group = compilation.chunk_group_by_ukey.expect_get_mut(parent);
            for group_ukey in &groups {
              parnet_group.add_child(*group_ukey);
            }
            for group_ukey in &groups {
              let group = compilation.chunk_group_by_ukey.expect_get_mut(group_ukey);
              group.add_parent(*parent);
            }
          });
      }
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
          continue;
          // TODO: origin
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
  let logger = compilation.get_logger("buildChunkGraph");

  let modules: Vec<ModuleIdentifier> = compilation
    .get_module_graph()
    .modules()
    .keys()
    .copied()
    .collect();
  let mut splitter = CodeSplitter::new(modules);

  // find all modules'deps and blocks
  let start = std::time::Instant::now();
  splitter.fill_module_deps_and_blocks(compilation);
  dbg!(start.elapsed().as_millis());

  // fill chunks with its modules
  let start = std::time::Instant::now();
  splitter.create_chunks(compilation);
  dbg!(start.elapsed().as_millis());

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
