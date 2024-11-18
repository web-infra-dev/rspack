use num_bigint::BigUint;
use rayon::{iter::split, prelude::*};
use rspack_collections::{DatabaseItem, IdentifierMap, IdentifierSet, ItemUkey};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGroup, ChunkGroupKind, ChunkGroupOptions, ChunkUkey,
  Compilation, DependenciesBlock, DependencyId, EntryData, EntryOptions, GroupOptions, Logger,
  ModuleIdentifier,
};

#[derive(Debug)]
pub struct CodeSplitter {
  pub module_deps: IdentifierMap<(Vec<DepRecord>, Vec<AsyncDependenciesBlockIdentifier>)>,
  pub blocks: HashMap<AsyncDependenciesBlockIdentifier, ModuleIdentifier>,
  pub chunk_parent_modules: HashMap<ChunkUkey, HashSet<ModuleIdentifier>>,
  pub module_ordinal: IdentifierMap<u64>,
  pub chunk_modules: HashMap<ChunkUkey, BigUint>,
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
}

#[derive(Debug)]
struct EntryChunkDesc {
  entry: String,
  entry_modules: IdentifierSet,
  modules: IdentifierSet,
  options: EntryOptions,
  modules_ordinal: BigUint,
}

#[derive(Debug)]
struct NormalChunkDesc {
  original: HashSet<ModuleIdentifier>,
  blocks: HashSet<AsyncDependenciesBlockIdentifier>,
  options: Option<ChunkGroupOptions>,
  modules: IdentifierSet,
  modules_ordinal: BigUint,
}

impl CreateChunkRoot {
  fn create(&self, splitter: &CodeSplitter, compilation: &Compilation) -> Vec<ChunkDesc> {
    let module_graph = compilation.get_module_graph();
    match self {
      CreateChunkRoot::Entry(entry, data) => {
        let mut chunk_modules = IdentifierSet::default();
        let mut entry_modules = IdentifierSet::default();
        let mut modules_ordinal = BigUint::from(0u64);
        for m in data
          .all_dependencies()
          .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
        {
          entry_modules.insert(*m);
          splitter.fill_dependencies(*m, &mut chunk_modules, &mut modules_ordinal);
        }

        vec![ChunkDesc::Entry(Box::new(EntryChunkDesc {
          entry: entry.clone(),
          entry_modules,
          options: data.options.clone(),
          modules: chunk_modules,
          modules_ordinal,
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

          let mut modules = IdentifierSet::default();
          let mut modules_ordinal = BigUint::from(0u64);
          splitter.fill_dependencies(*m, &mut modules, &mut modules_ordinal);
          chunks.push(ChunkDesc::Chunk(Box::new(NormalChunkDesc {
            original: std::iter::once(*origin).collect(),
            blocks: std::iter::once(*block_id).collect(),
            options: block.get_group_options().map(|opt| match opt {
              GroupOptions::Entrypoint(_) => unreachable!(),
              GroupOptions::ChunkGroup(group_option) => group_option.clone(),
            }),
            modules,
            modules_ordinal,
          })));
        }

        chunks
      }
    }
  }
}

impl CodeSplitter {
  pub fn new() -> Self {
    Self {
      module_deps: Default::default(),
      blocks: Default::default(),
      chunk_parent_modules: Default::default(),
      module_ordinal: Default::default(),
      chunk_modules: Default::default(),
    }
  }

  fn get_module_ordinal(&self, m: ModuleIdentifier) -> u64 {
    *self.module_ordinal.get(&m).expect("should have ordinal")
  }

  // insert static dependencies into a set
  fn fill_dependencies(
    &self,
    root: ModuleIdentifier,
    set: &mut IdentifierSet,
    module_ordinal: &mut BigUint,
  ) {
    if !set.insert(root) {
      return;
    }

    module_ordinal.set_bit(self.get_module_ordinal(root), true);
    if let Some((deps, _)) = self.module_deps.get(&root) {
      for dep in deps {
        self.fill_dependencies(dep.module_id, set, module_ordinal);
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

    let chunks = roots
      .par_iter()
      .map(|root| root.create(self, compilation))
      .flatten()
      .collect::<Vec<_>>();

    let chunks = self.merge_same_chunks(chunks);
    dbg!(start.elapsed().as_millis());

    // remove available modules if we could
    // self.optimize(compilation);

    for chunk_desc in chunks {
      match chunk_desc {
        ChunkDesc::Entry(entry_desc) => {
          let box EntryChunkDesc {
            entry,
            entry_modules,
            options,
            modules,
            modules_ordinal,
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
          }

          // TODO: do this in parallel
          for module in modules {
            compilation
              .chunk_graph
              .connect_chunk_and_module(entry_chunk_ukey, module);
          }

          compilation
            .named_chunk_groups
            .insert(entry, entrypoint_ukey);
        }
        ChunkDesc::Chunk(chunk_desc) => {
          let box NormalChunkDesc {
            original,
            blocks,
            options,
            modules,
            modules_ordinal,
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

            for block in blocks {
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
          for m in modules {
            compilation
              .chunk_graph
              .connect_chunk_and_module(chunk_ukey, m);
          }
        }
      }
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
    let mut named_chunks = HashMap::<String, usize>::default();

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

          existing_chunk.modules.extend(chunk.modules.into_iter());
          existing_chunk.modules_ordinal |= chunk.modules_ordinal;
          continue;
          // TODO: origin
        } else {
          let idx = final_chunks.len();
          named_chunks.insert(name.to_string(), idx);
        }
      }
      final_chunks.push(chunk);
    }

    final_chunks
  }
}

pub fn code_split(compilation: &mut Compilation) {
  let logger = compilation.get_logger("buildChunkGraph");
  let mut splitter = CodeSplitter::new();

  for m in compilation.get_module_graph().modules().keys() {
    splitter
      .module_ordinal
      .insert(*m, (splitter.module_ordinal.len() + 1) as u64);
  }

  // find all modules'deps and blocks
  let start = std::time::Instant::now();
  splitter.fill_module_deps_and_blocks(compilation);
  dbg!(start.elapsed().as_millis());

  // fill chunks with its modules
  let start = std::time::Instant::now();
  splitter.create_chunks(compilation);
  dbg!(start.elapsed().as_millis());

  // link chunks
  let start = logger.time("link chunks");
  splitter.link_chunks(compilation);
  dbg!(start.elapsed().as_millis());

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
