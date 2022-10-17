use std::collections::{HashMap, HashSet};

// use crate::{
//     BundleOptions, Chunk, ChunkGraph, ChunkIdAlgo, ChunkKind, JsModuleKind, ModuleGraphContainer,
// };

use rspack_error::Result;
use tracing::instrument;

use crate::{
  uri_to_chunk_name, ChunkGroup, ChunkGroupKind, ChunkGroupUkey, ChunkKind, ChunkUkey, Compilation,
  Dependency,
};

struct EntryData {
  name: String,
  module_uri: String,
  dependencies: Vec<Dependency>,
}
#[instrument()]
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
  split_point_modules: HashSet<String>,
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
      split_point_modules: Default::default(),
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
            module_uri: module.uri.clone(),
            name: name.to_string(),
            dependencies: vec![dep.clone()],
          })
      })
      .collect::<Vec<_>>();

    let mut input_entrypoints_and_modules: HashMap<ChunkGroupUkey, Vec<String>> = HashMap::new();

    for EntryData {
      name,
      module_uri,
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
      compilation
        .chunk_graph
        .split_point_module_uri_to_chunk_ukey
        .insert(module_uri.clone(), chunk.ukey);

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
        self.queue = std::mem::take(&mut self.queue_delayed);
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

    {
      let mut module = self
        .compilation
        .module_graph
        .module_by_uri_mut(&item.module_uri)
        .expect("No module found");

      if module.pre_order_index.is_none() {
        module.pre_order_index = Some(self.next_free_module_pre_order_index);
        self.next_free_module_pre_order_index += 1;
      }
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

    let mut module = self
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
      let is_already_split_module = self.split_point_modules.contains(&dyn_dep_mgm.uri);
      if is_already_split_module {
        continue;
      } else {
        self.split_point_modules.insert(dyn_dep_mgm.uri.clone());
      }

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

      self
        .compilation
        .chunk_graph
        .split_point_module_uri_to_chunk_ukey
        .insert(dyn_dep_mgm.uri.clone(), chunk.ukey);

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
