use std::{borrow::Cow, hash::BuildHasherDefault, iter::once, sync::Arc};

use crossbeam_channel::Sender;
use indexmap::IndexSet;
use num_bigint::BigUint;
use rayon::prelude::*;
use rspack_collections::{
  IdentifierDashMap, IdentifierHasher, IdentifierIndexMap, IdentifierIndexSet, IdentifierMap,
  IdentifierSet,
};
use rspack_error::{Diagnostic, Result, error};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use tracing::instrument;

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, Chunk, ChunkLoading, Compilation,
  DependenciesBlock, ModuleGraph, ModuleGraphCacheArtifact, ModuleGraphConnection,
  ModuleIdentifier, RuntimeSpec, build_chunk_graph::available_modules::AvailableModules,
  merge_runtime,
};

#[derive(Debug, Clone)]
enum CreateChunk {
  /**
  Initial entry chunk
  */
  Entry(String, RuntimeSpec),

  /**
  Named chunk
  created by import(/*webpackChunkName*/ './foo')
  */
  NamedBlock(String, AsyncDependenciesBlockIdentifier, RuntimeSpec),

  /**
  Normal chunk
  */
  Block(AsyncDependenciesBlockIdentifier, RuntimeSpec),

  /**
  Async Entry
  new Worker()
  */
  AsyncEntry(AsyncDependenciesBlockIdentifier, RuntimeSpec),
}

impl CreateChunk {
  fn expect_block(&self) -> AsyncDependenciesBlockIdentifier {
    match self {
      CreateChunk::Block(block, _) => *block,
      CreateChunk::AsyncEntry(block, _) => *block,
      _ => unreachable!(),
    }
  }

  fn get_name(&self) -> Option<&str> {
    match self {
      CreateChunk::Entry(name, _) => Some(name),
      CreateChunk::NamedBlock(name, _, _) => Some(name),
      CreateChunk::Block(_, _) => None,
      CreateChunk::AsyncEntry(_, _) => None,
    }
  }

  fn get_runtime(&self) -> &RuntimeSpec {
    match self {
      CreateChunk::Entry(_, runtime_spec) => runtime_spec,
      CreateChunk::Block(_, runtime_spec) => runtime_spec,
      CreateChunk::NamedBlock(_, _, runtime_spec) => runtime_spec,
      CreateChunk::AsyncEntry(_, runtime_spec) => runtime_spec,
    }
  }

  fn set_runtime(&mut self, runtime: RuntimeSpec) {
    match self {
      CreateChunk::Entry(_, runtime_spec) => *runtime_spec = runtime,
      CreateChunk::Block(_, runtime_spec) => *runtime_spec = runtime,
      CreateChunk::NamedBlock(_, _, runtime_spec) => *runtime_spec = runtime,
      CreateChunk::AsyncEntry(_, runtime_spec) => *runtime_spec = runtime,
    }
  }
}

type ModuleDeps = HashMap<
  RuntimeSpec,
  IdentifierDashMap<Arc<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)>>,
>;

#[derive(Debug)]
pub(crate) struct CodeSplitting {
  /**
    record map of module outgoings
  */
  pub module_deps: ModuleDeps,

  module_ordinals: IdentifierMap<u64>,
}

struct ChunkDesc {
  name: Option<String>,

  modules: IdentifierSet,
  mask: AvailableModules,
  runtime: RuntimeSpec,
  min_available_modules: AvailableModules,
  pre_order_indices: IdentifierIndexMap<usize>,
  post_order_indices: IdentifierIndexMap<usize>,

  outgoing_blocks: HashSet<AsyncDependenciesBlockIdentifier>,

  parents: HashSet<ChunkInitializer>,
  children: HashSet<ChunkInitializer>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ChunkInitializer {
  None,
  Named(String),
  Block(AsyncDependenciesBlockIdentifier),
}

enum Task {
  CreateChunk(ChunkInitializer, CreateChunk, AvailableModules),
  ChunkCreated(ChunkInitializer, CreateChunk, ChunkDesc),
  Poll,
}

impl CodeSplitting {
  pub fn new() -> Self {
    Self {
      module_deps: HashMap::default(),
      module_ordinals: Default::default(),
    }
  }

  pub fn get_module_ordinal(&self, module: ModuleIdentifier) -> u64 {
    *self
      .module_ordinals
      .get(&module)
      .expect("should have ordinal for module")
  }

  fn create_chunk(
    &self,
    create_chunk: CreateChunk,
    available_modules: AvailableModules,
    compilation: &Compilation,
  ) -> ChunkDesc {
    let name = create_chunk.get_name().map(|s| s.to_string());
    let runtime = create_chunk.get_runtime().clone();

    self.outgoings_modules(module, &runtime, module_graph, module_graph_cache);

    ChunkDesc {
      name,
      modules: (),
      mask: (),
      runtime,
      min_available_modules: (),
      pre_order_indices: (),
      post_order_indices: (),
      outgoing_blocks: (),

      parents: Default::default(),
      children: Default::default(),
    }
  }

  fn get_chunk<'map>(
    &self,
    id: &ChunkInitializer,
    named_chunks: &'map HashMap<String, ChunkDesc>,
    chunks: &'map HashMap<AsyncDependenciesBlockIdentifier, ChunkDesc>,
  ) -> Option<&'map ChunkDesc> {
    match id {
      ChunkInitializer::None => None,
      ChunkInitializer::Named(name) => named_chunks.get(name),
      ChunkInitializer::Block(block) => chunks.get(block),
    }
  }

  fn get_chunk_mut<'map>(
    &self,
    id: &ChunkInitializer,
    named_chunks: &'map mut HashMap<String, ChunkDesc>,
    chunks: &'map mut HashMap<AsyncDependenciesBlockIdentifier, ChunkDesc>,
  ) -> Option<&'map mut ChunkDesc> {
    match id {
      ChunkInitializer::None => None,
      ChunkInitializer::Named(name) => named_chunks.get_mut(name),
      ChunkInitializer::Block(block) => chunks.get_mut(block),
    }
  }

  fn split(&mut self, compilation: &mut Compilation) -> Result<()> {
    rayon::scope(|scope| {
      let (tx, rx) = crossbeam_channel::unbounded::<Task>();
      let mut named_chunks = HashMap::<String, ChunkDesc>::default();
      let mut chunks = HashMap::<AsyncDependenciesBlockIdentifier, ChunkDesc>::default();
      let mut active = 0;

      loop {
        match rx.recv().expect("should not have error") {
          Task::CreateChunk(parent, create_chunk, available_modules) => {
            // re-assign to escape rustc borrow_ck
            let this = &self;
            let compilation = &compilation;
            let tx = tx.clone();
            active += 1;
            scope.spawn(move |_| {
              let chunk = this.create_chunk(create_chunk.clone(), available_modules, compilation);
              tx.send(Task::ChunkCreated(parent, create_chunk, chunk))
                .expect("should not have error");
            });
          }
          Task::ChunkCreated(parent, create_chunk, chunk) => {
            active -= 1;
            let module_graph = compilation.get_module_graph();

            // store result
            let existing = match &create_chunk {
              CreateChunk::Entry(name, _) => named_chunks.get_mut(name),
              CreateChunk::NamedBlock(name, _, _) => named_chunks.get_mut(name),
              CreateChunk::Block(block, _) => chunks.get_mut(block),
              CreateChunk::AsyncEntry(block, _) => chunks.get_mut(block),
            };

            let (chunk, resulting_availbale_modules) = if let Some(existing) = existing {
              existing.modules.extend(chunk.modules);
              existing.mask.union(&chunk.mask);
              existing.outgoing_blocks.extend(chunk.outgoing_blocks);
              existing.runtime = merge_runtime(&existing.runtime, &chunk.runtime);
              existing.min_available_modules = existing
                .min_available_modules
                .intersect(&chunk.min_available_modules);

              let resulting_available = existing.min_available_modules.union(&existing.mask);
              (existing, resulting_available)
            } else {
              let resulting_available_modules = chunk.min_available_modules.union(&chunk.mask);

              let name = chunk.name.clone();
              if let Some(ref name) = name {
                named_chunks.insert(name.to_string(), chunk);
                (
                  named_chunks.get_mut(name).expect("just inserted"),
                  resulting_available_modules,
                )
              } else {
                chunks.insert(create_chunk.expect_block(), chunk);
                (
                  chunks
                    .get_mut(&create_chunk.expect_block())
                    .expect("just inserted"),
                  resulting_available_modules,
                )
              }
            };

            let chunk_initializer = if let Some(name) = &chunk.name {
              ChunkInitializer::Named(name.clone())
            } else {
              ChunkInitializer::Block(create_chunk.expect_block())
            };

            // connect to parent
            chunk.parents.insert(parent.clone());

            match parent {
              ChunkInitializer::None => {}
              ChunkInitializer::Named(name) => {
                let parent = named_chunks.get_mut(&name).expect("should have chunk");
                parent.children.insert(chunk_initializer.clone());
              }
              ChunkInitializer::Block(block) => {
                let parent = chunks.get_mut(&block).expect("should have chunk");
                parent.children.insert(chunk_initializer.clone());
              }
            };

            let chunk = self
              .get_chunk(&chunk_initializer, &named_chunks, &chunks)
              .expect("should have chunk");

            // process dependencies
            for block_id in &chunk.outgoing_blocks {
              let block = module_graph.block_by_id_expect(block_id);
              let block_options = block.get_group_options();
              let entry_options = block_options.and_then(|opt| opt.entry_options());
              let name = block_options.and_then(|opt| {
                opt
                  .name()
                  .or_else(|| opt.entry_options().and_then(|entry| entry.name.as_deref()))
              });

              active += 1;
              tx.send(Task::CreateChunk(
                chunk_initializer.clone(),
                if entry_options.is_some() {
                  CreateChunk::AsyncEntry(*block_id, chunk.runtime.clone())
                } else if let Some(name) = name {
                  CreateChunk::NamedBlock(name.to_string(), *block_id, chunk.runtime.clone())
                } else {
                  CreateChunk::Block(*block_id, chunk.runtime.clone())
                },
                resulting_availbale_modules.clone(),
              ));
            }

            tx.send(Task::Poll).expect("should not have error");
          }
          Task::Poll => {
            if active == 0 {
              break;
            }
          }
        }
      }
    });

    Ok(())
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

    entry_runtime.insert(entry, runtime.clone());
    Ok(runtime)
  }

  /**
  Find module outgoings
  */
  #[instrument(skip_all)]
  pub fn outgoings_modules(
    &self,
    module: &ModuleIdentifier,
    runtime: &RuntimeSpec,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> Arc<(Vec<ModuleIdentifier>, Vec<AsyncDependenciesBlockIdentifier>)> {
    let module_map = self.module_deps.get(runtime).expect("should have value");

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

  // /**
  // Parallel walking module graph
  // 1. find all possible chunk incomings
  // 2. initialize all module outgoings
  // */
  // #[instrument(skip_all)]
  // fn analyze_module_graph(&mut self, compilation: &mut Compilation) -> Result<Vec<CreateChunk>> {
  //   // determine runtime and chunkLoading
  //   let mut entry_runtime = HashMap::default();
  //   let mut diagnostics = vec![];
  //   for entry in compilation.entries.keys() {
  //     let mut visited = vec![];
  //     if let Err(error) =
  //       Self::get_entry_runtime(entry, compilation, &mut entry_runtime, &mut visited)
  //     {
  //       diagnostics.push(Diagnostic::from(error));
  //       let tmp_runtime = once(ustr::Ustr::from(entry.as_str())).collect::<RuntimeSpec>();
  //       entry_runtime.insert(entry, tmp_runtime.clone());
  //     };
  //   }

  //   // iterate module graph to find block runtime and its parents
  //   // let mut blocks_with_runtime = HashMap::default();
  //   let mut batch = vec![];
  //   let mut visited = HashSet::default();
  //   let module_graph = compilation.get_module_graph();
  //   let module_graph_cache = &compilation.module_graph_cache_artifact;
  //   let global_chunk_loading = &compilation.options.output.chunk_loading;
  //   let mut chunks = HashMap::<AsyncDependenciesBlockIdentifier, CreateChunk>::default();
  //   let mut named_chunks = HashMap::<String, CreateChunk>::default();

  //   let global_deps = compilation.global_entry.dependencies.iter();
  //   let global_included_deps = compilation.global_entry.include_dependencies.iter();

  //   let mut next_idx = 0;
  //   let mut index_by_block = HashMap::<AsyncDependenciesBlockIdentifier, usize>::default();

  //   for (entry, entry_data) in &compilation.entries {
  //     let chunk_loading = !matches!(
  //       entry_data
  //         .options
  //         .chunk_loading
  //         .as_ref()
  //         .unwrap_or(global_chunk_loading),
  //       ChunkLoading::Disable
  //     ) && entry_data
  //       .options
  //       .async_chunks
  //       .unwrap_or(compilation.options.output.async_chunks);
  //     let runtime = entry_runtime
  //       .get(entry.as_str())
  //       .expect("already set runtime");

  //     self.module_deps.entry(runtime.clone()).or_default();

  //     global_deps
  //       .clone()
  //       .chain(entry_data.dependencies.iter())
  //       .chain(global_included_deps.clone())
  //       .chain(entry_data.include_dependencies.iter())
  //       .for_each(|dep_id| {
  //         if let Some(m) = module_graph.module_identifier_by_dependency_id(dep_id) {
  //           batch.push((*m, Cow::Borrowed(runtime), chunk_loading));
  //         }
  //       });

  //     named_chunks.insert(
  //       entry.clone(),
  //       CreateChunk::Entry(entry.clone(), runtime.clone()),
  //     );
  //   }

  //   loop {
  //     let tasks = std::mem::take(&mut batch);

  //     let mut new_tasks = Vec::new();
  //     for (module, runtime, chunk_loading) in tasks {
  //       if visited.insert((module, runtime.clone())) {
  //         new_tasks.push((module, runtime, chunk_loading));
  //       }
  //     }
  //     new_tasks.reverse();

  //     let tasks = new_tasks;

  //     let outgoings = tasks
  //       .par_iter()
  //       .map(|(_module, runtime, _)| {
  //         self.outgoings_modules(_module, runtime.as_ref(), &module_graph, module_graph_cache)
  //       })
  //       .collect::<Vec<_>>();

  //     for ((_, runtime, chunk_loading), outgoings) in tasks.into_iter().zip(outgoings.into_iter()) {
  //       let (modules, blocks) = outgoings.as_ref();
  //       let blocks = blocks.clone();
  //       for m in modules {
  //         batch.push((*m, runtime.clone(), chunk_loading));
  //       }

  //       for block_id in blocks {
  //         index_by_block.entry(block_id).or_insert_with(|| {
  //           next_idx += 1;
  //           next_idx
  //         });

  //         let Some(block) = module_graph.block_by_id(&block_id) else {
  //           continue;
  //         };

  //         // when disable chunk loading, only async entrypoint can be created, disable normal chunk
  //         let entry_options = block
  //           .get_group_options()
  //           .and_then(|option| option.entry_options());
  //         let is_entry = entry_options.is_some();
  //         let should_create = chunk_loading || entry_options.is_some();
  //         let block = module_graph
  //           .block_by_id(&block_id)
  //           .expect("should have block");

  //         let child_chunk_loading = entry_options.map_or(chunk_loading, |opt| {
  //           !matches!(
  //             opt.chunk_loading.as_ref().unwrap_or(global_chunk_loading),
  //             ChunkLoading::Disable
  //           ) && opt
  //             .async_chunks
  //             .unwrap_or(compilation.options.output.async_chunks)
  //         });
  //         let child_runtime = if should_create {
  //           if let Some(name) = block.get_group_options().and_then(|options| {
  //             options
  //               .name()
  //               .or_else(|| entry_options.and_then(|entry| entry.name.as_deref()))
  //           }) && let Some(chunk) = named_chunks.get_mut(name)
  //           {
  //             // already created with name, let old_runtime = root.get_runtime();
  //             let old_runtime = chunk.get_runtime();
  //             let new_runtime = if is_entry {
  //               // async entrypoint has unique runtime, do not merge runtime
  //               old_runtime.clone()
  //             } else {
  //               let new_runtime = merge_runtime(&runtime, old_runtime);
  //               self.module_deps.entry(new_runtime.clone()).or_default();
  //               chunk.set_runtime(new_runtime.clone());
  //               new_runtime
  //             };

  //             match chunk {
  //               CreateChunk::Entry(_, _) => {
  //                 if entry_options.is_some() {
  //                   diagnostics.push(Diagnostic::from(error!(
  //                     "Two entrypoints with the same name {}",
  //                     name
  //                   )));
  //                 } else {
  //                   diagnostics.push(
  //                     Diagnostic::from(
  //                       error!(
  //                         format!("It's not allowed to load an initial chunk on demand. The chunk name \"{}\" is already used by an entrypoint.", name)
  //                       )
  //                     )
  //                 );
  //                 }
  //               }
  //               CreateChunk::AsyncEntry(existing_block_id, _) => {
  //                 if *existing_block_id != block_id {
  //                   diagnostics.push(Diagnostic::from(error!(
  //                     "Two async entrypoints with the same name {}",
  //                     name
  //                   )));
  //                 }
  //               }
  //               CreateChunk::Block(async_dependencies_block_identifiers, runtime) => {
  //                 if !async_dependencies_block_identifiers.contains(&block_id) {
  //                   async_dependencies_block_identifiers.insert(block_id);
  //                 }
  //                 // should re-visit all children bringing new runtime
  //                 async_dependencies_block_identifiers
  //                   .iter()
  //                   .filter(|id| **id != block_id)
  //                   .for_each(|root_block| {
  //                     let root_block = module_graph
  //                       .block_by_id(root_block)
  //                       .expect("should have block");
  //                     root_block
  //                       .get_dependencies()
  //                       .iter()
  //                       .filter_map(|dep_id| {
  //                         module_graph.module_identifier_by_dependency_id(dep_id)
  //                       })
  //                       .for_each(|m| {
  //                         batch.push((*m, Cow::Owned(new_runtime.clone()), child_chunk_loading));
  //                       });
  //                   });
  //               }
  //             }

  //             Cow::Owned(new_runtime)
  //           } else if let Some(chunk) = chunks.get_mut(&block_id) {
  //             // already created
  //             let old_runtime = chunk.get_runtime();
  //             let new_runtime = if is_entry {
  //               // async entrypoint has unique runtime, do not merge runtime
  //               old_runtime.clone()
  //             } else {
  //               let new_runtime = merge_runtime(&runtime, old_runtime);
  //               self.module_deps.entry(new_runtime.clone()).or_default();
  //               chunk.set_runtime(new_runtime.clone());
  //               new_runtime
  //             };
  //             Cow::Owned(new_runtime)
  //           } else {
  //             let rt = if let Some(entry_options) = entry_options {
  //               RuntimeSpec::from_entry_options(entry_options)
  //                 .map(|rt| {
  //                   self.module_deps.entry(rt.clone()).or_default();
  //                   Cow::Owned(rt)
  //                 })
  //                 .unwrap_or(runtime.clone())
  //             } else {
  //               runtime.clone()
  //             };

  //             if let Some(name) = block.get_group_options().and_then(|options| {
  //               options.name().or_else(|| {
  //                 options
  //                   .entry_options()
  //                   .and_then(|entry_options| entry_options.name.as_deref())
  //               })
  //             }) {
  //               named_chunks.insert(
  //                 name.to_string(),
  //                 CreateChunk::AsyncEntry(block_id, rt.clone().into_owned()),
  //               );
  //             } else {
  //               chunks.insert(
  //                 block_id,
  //                 CreateChunk::Block(once(block_id).collect(), rt.clone().into_owned()),
  //               );
  //             }
  //             rt.clone()
  //           }
  //         } else {
  //           runtime.clone()
  //         };

  //         block
  //           .get_dependencies()
  //           .iter()
  //           .filter_map(|dep_id| module_graph.module_identifier_by_dependency_id(dep_id))
  //           .for_each(|module| {
  //             batch.push((*module, child_runtime.clone(), child_chunk_loading));
  //           });
  //       }
  //     }

  //     if batch.is_empty() {
  //       break;
  //     }
  //   }

  //   compilation.extend_diagnostics(diagnostics);

  //   for root in named_chunks.values_mut() {
  //     if let CreateChunk::Block(blocks, _) = root {
  //       if blocks.len() <= 1 {
  //         continue;
  //       }

  //       blocks.sort_by(|a, b| {
  //         let a_index = index_by_block.get(a).expect("should have index");
  //         let b_index = index_by_block.get(b).expect("should have index");

  //         a_index.cmp(b_index)
  //       });
  //     }
  //   }

  //   Ok(
  //     chunks
  //       .into_values()
  //       .chain(named_chunks.into_values())
  //       .collect(),
  //   )
  // }
}
