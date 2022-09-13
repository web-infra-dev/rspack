use std::collections::{HashMap, HashSet};

// use crate::{
//     BundleOptions, Chunk, ChunkGraph, ChunkIdAlgo, ChunkKind, JsModuleKind, ModuleGraphContainer,
// };

use rspack_error::Result;

use crate::{
  uri_to_chunk_name, ChunkGroup, ChunkGroupKind, ChunkGroupUkey, ChunkKind, ChunkUkey, Compilation,
  Dependency,
};

// pub fn code_splitting2(compilation: &mut Compilation) {
//   // let code_splitting_options = &bundle_options.code_splitting;
//   let is_enable_code_splitting = true;
//   let is_reuse_existing_chunk = true;

//   let module_graph = &compilation.module_graph;

//   let mut id_generator = ChunkIdGenerator {
//     id_count: 0,
//     module_graph,
//     root: compilation.options.context.as_str(),
//     chunk_id_algo: ChunkIdAlgo::Named,
//   };

//   let mut chunk_ref_by_entry_module_uri: HashMap<&str, ChunkUkey> = HashMap::new();
//   let mut chunk_relation_graph2 = petgraph::graphmap::DiGraphMap::<ChunkUkey, ()>::new();

//   let mut chunk_entries = compilation
//     .entry_dependencies()
//     .values()
//     .filter_map(|dep| module_graph.module_by_dependency(dep))
//     .map(|module| module.uri.as_str())
//     .collect::<Vec<_>>();

//   let chunk_by_ref = &mut compilation.chunk_by_ukey;

//   let chunk_graph = &mut compilation.chunk_graph;

//   // First we need to create entry chunk.
//   for entry in &chunk_entries {
//     let chunk_id = id_generator.gen_id(entry);
//     let chunk = Chunk::new(None, chunk_id.clone(), ChunkKind::Entry);

//     chunk_ref_by_entry_module_uri.insert(*entry, chunk.ukey);
//     // chunk_graph.add_chunk(&chunk);
//     chunk_by_ref.insert(chunk.ukey, chunk);
//   }

//   if is_enable_code_splitting {
//     module_graph.modules().for_each(|module| {
//       module
//         .dynamic_depended_modules(module_graph)
//         .into_iter()
//         .for_each(|dyn_dep_module| {
//           chunk_ref_by_entry_module_uri
//             .entry(dyn_dep_module.uri.as_str())
//             .or_insert_with_key(|mod_uri| {
//               chunk_entries.push(*mod_uri);

//               let chunk_id = id_generator.gen_id(mod_uri);
//               let chunk = Chunk::new(None, chunk_id, ChunkKind::Normal);
//               let chunk_ref = chunk.ukey;
//               // chunk_graph.add_chunk(&chunk);
//               chunk_by_ref.insert(chunk.ukey, chunk);
//               chunk_ref
//             });
//         });
//     });
//   }

//   // Now, we have all chunks and need place right modules into chunks.
//   // We iterate through all chunks, and place modules that depended(directed or non-directed) by the chunk to the map below.
//   // Without bundle splitting, a module can be placed into multiple chunks based on its usage:

//   // E.g. (Code Splitting enabled)
//   //                  (dynamic import)
//   // a.js(entrypoint)------------------>b.js------->c.js
//   //        |
//   //        |
//   //        +------>c.js
//   // In this case, two chunks will be generated, chunk entires are `a.js` (Chunk A) and `b.js` (Chunk B),
//   // and module `c.js` will be placed into both of them.

//   let mut mod_to_chunk_ref: HashMap<&str, HashSet<ChunkUkey>> = Default::default();

//   for entry in &chunk_entries {
//     let chunk_ref = chunk_ref_by_entry_module_uri[*entry];
//     let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
//     let mut visited = HashSet::new();
//     while let Some(module_uri) = queue.pop_front() {
//       let module = module_graph
//         .module_by_uri(module_uri)
//         .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
//       if !visited.contains(module_uri) {
//         visited.insert(module_uri);
//         mod_to_chunk_ref
//           .entry(module_uri)
//           .or_default()
//           .insert(chunk_ref);
//         // let chunk = &mut chunk_graph[chunk_id];
//         // chunk.module_ids.push(module_uri.to_string());
//         module
//           .depended_modules(module_graph)
//           .into_iter()
//           .for_each(|dep_module| queue.push_back(&dep_module.uri));
//         if !is_enable_code_splitting {
//           module
//             .dynamic_depended_modules(module_graph)
//             .into_iter()
//             .for_each(|module| queue.push_back(&module.uri));
//         }
//       } else {
//         // TODO: detect circle import
//       }
//     }
//   }

//   // Now, we have the relationship between modules and chunks.
//   // We create directed graph from starting chunk to another.

//   // For the example above, we have the following graph:
//   // Chunk A(entrypoint: a.js) -> Chunk B(entrypoint: b.js)

//   module_graph.modules().for_each(|each_mod| {
//     each_mod
//       .depended_modules(module_graph)
//       .into_iter()
//       .for_each(|dep_mod| {
//         if let Some(dep_mod_chunk_ref) = chunk_ref_by_entry_module_uri.get(dep_mod.uri.as_str()) {
//           mod_to_chunk_ref[each_mod.uri.as_str()]
//             .iter()
//             .filter(|each_chunk_ref| *each_chunk_ref != dep_mod_chunk_ref)
//             .for_each(|each_chunk_ref| {
//               chunk_relation_graph2.add_edge(*each_chunk_ref, *dep_mod_chunk_ref, ());
//             });
//         }
//       });
//     each_mod
//       .dynamic_depended_modules(module_graph)
//       .into_iter()
//       .for_each(|dep_mod| {
//         if let Some(chunk_id) = chunk_ref_by_entry_module_uri.get(dep_mod.uri.as_str()) {
//           mod_to_chunk_ref[each_mod.uri.as_str()]
//             .iter()
//             .filter(|each_chunk_id| *each_chunk_id != chunk_id)
//             .for_each(|each_chunk_id| {
//               chunk_relation_graph2.add_edge(*each_chunk_id, *chunk_id, ());
//             });
//         }
//       });
//   });

//   // println!("chunk graph {:?}", Dot::new(&chunk_graph));

//   for entry in &chunk_entries {
//     let mut queue = [*entry].into_iter().collect::<VecDeque<_>>();
//     let mut visited = HashSet::new();
//     while let Some(module_uri) = queue.pop_front() {
//       let module = module_graph
//         .module_by_uri(module_uri)
//         .unwrap_or_else(|| panic!("no entry found for key {:?}", module_uri));
//       if !visited.contains(module_uri) {
//         visited.insert(module_uri);

//         let belong_to_chunks = &mod_to_chunk_ref[module_uri];
//         // println!(
//         //   "[module {:?}]: belong to chunks {:?}",
//         //   module_uri, belong_to_chunks
//         // );
//         belong_to_chunks
//           .iter()
//           .filter(|id_of_chunk_to_place_module| {
//             if is_reuse_existing_chunk {
//               // We only want to have chunks that have no superiors.
//               // If both chunk A and B have the same module, we only want to place module into the uppermost chunk based on the relationship between A and B.
//               let has_superior = belong_to_chunks.iter().any(|maybe_superior_chunk| {
//                 chunk_relation_graph2
//                   .contains_edge(*maybe_superior_chunk, **id_of_chunk_to_place_module)
//               });
//               !has_superior
//             } else {
//               true
//             }
//           })
//           .for_each(|id_of_chunk_to_place_module| {
//             let chunk_to_place_module = chunk_by_ref
//               .get_mut(id_of_chunk_to_place_module)
//               .expect("Failed to get chunk by id");
//             chunk_to_place_module
//               .module_uris
//               .insert(module_uri.to_string());
//           });

//         module
//           .depended_modules(module_graph)
//           .into_iter()
//           .for_each(|dep_module| queue.push_back(&dep_module.uri));
//         if !is_enable_code_splitting {
//           module
//             .dynamic_depended_modules(module_graph)
//             .into_iter()
//             .for_each(|module| queue.push_back(&module.uri));
//         }
//       } else {
//         // TODO: detect circle import
//       }
//     }
//   }

//   // if bundle_options.optimization.remove_empty_chunks {
//   if true {
//     let empty_chunk_id_to_be_removed = chunk_by_ref
//       .values()
//       .filter(|chunk| chunk.module_uris.is_empty())
//       .map(|chunk| chunk.ukey)
//       .collect::<Vec<_>>();

//     empty_chunk_id_to_be_removed.iter().for_each(|chunk_ref| {
//       chunk_by_ref.remove(chunk_ref);
//     });
//   }
// }

struct EntryData {
  name: String,
  _module_uri: String,
  dependencies: Vec<Dependency>,
}

pub fn code_splitting(compilation: &mut Compilation) -> Result<()> {
  CodeSplitter::new(compilation).split()?;
  Ok(())
}

struct CodeSplitter<'me> {
  compilation: &'me mut Compilation,
  next_free_module_pre_order_index: usize,
  next_free_module_post_order_index: usize,
  queue: Vec<QueueItem>,
  queue_delayed: Vec<QueueItem>,
  chunk_relation_graph: petgraph::graphmap::DiGraphMap<ChunkUkey, ()>,
}

impl<'me> CodeSplitter<'me> {
  pub fn new(compilation: &'me mut Compilation) -> Self {
    CodeSplitter {
      compilation,
      next_free_module_pre_order_index: 0,
      next_free_module_post_order_index: 0,
      queue: Default::default(),
      queue_delayed: Default::default(),
      chunk_relation_graph: Default::default(),
    }
  }

  fn prepare_input_entrypoints_and_modules(
    &mut self,
  ) -> Result<HashMap<ChunkGroupUkey, Vec<String>>> {
    let compilation = &mut self.compilation;
    let module_graph = &compilation.module_graph;

    let entries = compilation
      .entry_dependencies()
      .iter()
      .filter_map(|(name, dep)| {
        module_graph
          .module_by_dependency(dep)
          .map(|module| EntryData {
            _module_uri: module.uri.clone(),
            name: name.to_string(),
            dependencies: vec![dep.clone()],
          })
      })
      .collect::<Vec<_>>();

    let mut input_entrypoints_and_modules: HashMap<ChunkGroupUkey, Vec<String>> = HashMap::new();

    for EntryData {
      name,
      _module_uri: _,
      dependencies,
    } in &entries
    {
      let chunk = Compilation::add_chunk(
        &mut compilation.chunk_by_ukey,
        Some(name.to_string()),
        name.to_string(),
        ChunkKind::Entry,
      );

      compilation.chunk_graph.add_chunk(chunk.ukey);

      let mut entrypoint = ChunkGroup::new(ChunkGroupKind::Entrypoint);

      entrypoint.connect_chunk(chunk);

      compilation
        .named_chunk_groups
        .insert(name.to_string(), entrypoint.ukey);

      compilation
        .entrypoints
        .insert(name.to_string(), entrypoint.ukey);

      let entrypoint = {
        let ukey = entrypoint.ukey;
        compilation.chunk_group_by_ukey.insert(ukey, entrypoint);

        compilation
          .chunk_group_by_ukey
          .get(&ukey)
          .ok_or_else(|| anyhow::format_err!("no chunk group found"))?
      };

      let mut entry_modules_uri = HashSet::new();

      for dep in dependencies {
        let module = module_graph
          .module_by_dependency(dep)
          .ok_or_else(|| anyhow::format_err!("no module found"))?;
        compilation.chunk_graph.add_module(module.uri.clone());

        input_entrypoints_and_modules
          .entry(entrypoint.ukey)
          .or_default()
          .push(module.uri.clone());

        compilation.chunk_graph.connect_chunk_and_entry_module(
          chunk.ukey,
          module.uri.clone(),
          entrypoint.ukey,
        );
        entry_modules_uri.insert(module.uri.clone());
      }
    }
    Ok(input_entrypoints_and_modules)
  }

  pub fn split(mut self) -> Result<()> {
    let input_entrypoints_and_modules = self.prepare_input_entrypoints_and_modules()?;

    for (chunk_group, modules) in input_entrypoints_and_modules {
      let chunk_group = self
        .compilation
        .chunk_group_by_ukey
        .get(&chunk_group)
        .ok_or_else(|| anyhow::format_err!("no chunk group found"))?;
      // We could assume that the chunk group is an entrypoint and must have one chunk, which is entry chunk.
      // TODO: we need a better and safe way to ensure this.
      let chunk = chunk_group.chunks[0];
      for module in modules {
        self.queue.push(QueueItem {
          action: QueueAction::AddAndEnter,
          chunk,
          chunk_group: chunk_group.ukey,
          module_uri: module,
        });
      }
    }
    self.queue.reverse();

    tracing::debug!("--- process_queue start ---");
    while !self.queue.is_empty() || !self.queue_delayed.is_empty() {
      self.process_queue();
      if self.queue.is_empty() {
        self.queue = self.queue_delayed;
        self.queue_delayed = vec![];
      }
    }
    tracing::debug!("--- process_queue end ---");

    // Optmize to remove duplicated module which is safe

    let mut modules_to_be_removed_in_chunk = HashMap::new() as HashMap<ChunkUkey, HashSet<String>>;

    for chunk in self.compilation.chunk_by_ukey.values() {
      for module in self
        .compilation
        .chunk_graph
        .get_chunk_modules(&chunk.ukey, &self.compilation.module_graph)
      {
        let belong_to_chunks = self
          .compilation
          .chunk_graph
          .get_modules_chunks(&module.uri)
          .clone();

        let has_superior = belong_to_chunks.iter().any(|maybe_superior_chunk| {
          self
            .chunk_relation_graph
            .contains_edge(chunk.ukey, *maybe_superior_chunk)
        });

        if has_superior {
          modules_to_be_removed_in_chunk
            .entry(chunk.ukey)
            .or_default()
            .insert(module.uri.clone());
        }

        tracing::debug!(
          "module {} in chunk {:?} has_superior {:?}",
          module.uri,
          chunk.id,
          has_superior
        );
      }
    }

    for (chunk, modules) in modules_to_be_removed_in_chunk {
      for module in modules {
        self
          .compilation
          .chunk_graph
          .disconnect_chunk_and_module(&chunk, &module);
      }
    }

    Ok(())
  }

  fn process_queue(&mut self) {
    tracing::debug!("process_queue");
    while let Some(queue_item) = self.queue.pop() {
      match queue_item.action {
        QueueAction::AddAndEnter => self.add_and_enter_module(&queue_item),
        QueueAction::_Enter => self.enter_module(&queue_item),
        QueueAction::_ProcessModule => self.process_module(&queue_item),
        QueueAction::Leave => self.leave_module(&queue_item),
      }
    }
  }

  fn add_and_enter_module(&mut self, item: &QueueItem) {
    tracing::debug!("add_and_enter_module {:?}", item);
    if self
      .compilation
      .chunk_graph
      .is_module_in_chunk(&item.module_uri, item.chunk)
    {
      return;
    }

    self
      .compilation
      .chunk_graph
      .add_module(item.module_uri.clone());

    self
      .compilation
      .chunk_graph
      .connect_chunk_and_module(item.chunk, item.module_uri.clone());
    self.enter_module(item)
  }

  fn enter_module(&mut self, item: &QueueItem) {
    tracing::debug!("enter_module {:?}", item);
    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item.chunk_group)
      .expect("chunk group not found");

    if chunk_group
      .module_pre_order_indices
      .get(&item.module_uri)
      .is_none()
    {
      chunk_group
        .module_pre_order_indices
        .insert(item.module_uri.clone(), chunk_group.next_pre_order_index);
      chunk_group.next_pre_order_index += 1;
    }

    let module = self
      .compilation
      .module_graph
      .module_by_uri_mut(&item.module_uri)
      .expect("no module found");

    if module.pre_order_index.is_none() {
      module.pre_order_index = Some(self.next_free_module_pre_order_index);
      self.next_free_module_pre_order_index += 1;
    }

    self.queue.push(QueueItem {
      action: QueueAction::Leave,
      ..item.clone()
    });
    self.process_module(item)
  }

  fn leave_module(&mut self, item: &QueueItem) {
    tracing::debug!("leave_module {:?}", item);
    let chunk_group = self
      .compilation
      .chunk_group_by_ukey
      .get_mut(&item.chunk_group)
      .expect("chunk group not found");

    if chunk_group
      .module_post_order_indices
      .get(&item.module_uri)
      .is_none()
    {
      chunk_group
        .module_post_order_indices
        .insert(item.module_uri.clone(), chunk_group.next_post_order_index);
      chunk_group.next_post_order_index += 1;
    }

    let module = self
      .compilation
      .module_graph
      .module_by_uri_mut(&item.module_uri)
      .expect("no module found");

    if module.post_order_index.is_none() {
      module.post_order_index = Some(self.next_free_module_post_order_index);
      self.next_free_module_post_order_index += 1;
    }
  }

  fn process_module(&mut self, item: &QueueItem) {
    tracing::debug!("process_module {:?}", item);
    let mgm = self
      .compilation
      .module_graph
      .module_by_uri(&item.module_uri)
      .expect("no module found");

    for dep_mgm in mgm
      .depended_modules(&self.compilation.module_graph)
      .into_iter()
      .rev()
    {
      self.queue.push(QueueItem {
        action: QueueAction::AddAndEnter,
        chunk: item.chunk,
        chunk_group: item.chunk_group,
        module_uri: dep_mgm.uri.clone(),
      });
    }

    for dyn_dep_mgm in mgm
      .dynamic_depended_modules(&self.compilation.module_graph)
      .into_iter()
      .rev()
    {
      let chunk = Compilation::add_chunk(
        &mut self.compilation.chunk_by_ukey,
        None,
        uri_to_chunk_name(self.compilation.options.context.as_str(), &dyn_dep_mgm.uri),
        ChunkKind::Normal,
      );
      self.compilation.chunk_graph.add_chunk(chunk.ukey);
      self
        .chunk_relation_graph
        .add_edge(chunk.ukey, item.chunk, ());

      let mut chunk_group = ChunkGroup::new(ChunkGroupKind::Entrypoint);

      chunk_group.connect_chunk(chunk);

      let chunk_group = {
        let ukey = chunk_group.ukey;
        self
          .compilation
          .chunk_group_by_ukey
          .insert(ukey, chunk_group);

        self.compilation.chunk_group_by_ukey.get(&ukey).unwrap()
      };

      self.queue_delayed.push(QueueItem {
        action: QueueAction::AddAndEnter,
        chunk: chunk.ukey,
        chunk_group: chunk_group.ukey,
        module_uri: dyn_dep_mgm.uri.clone(),
      });
    }
  }
}

#[derive(Debug, Clone)]
struct QueueItem {
  action: QueueAction,
  chunk_group: ChunkGroupUkey,
  chunk: ChunkUkey,
  module_uri: String,
}

#[derive(Debug, Clone)]
enum QueueAction {
  AddAndEnter,
  _Enter,
  _ProcessModule,
  Leave,
}

// struct chunkGroupInfoMap {}
