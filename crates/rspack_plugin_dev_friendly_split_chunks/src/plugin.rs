use dashmap::DashMap;
use rspack_core::{Chunk, ChunkGraphChunk, ChunkUkey, Module, Plugin};
use rspack_identifier::Identifier;

/// In practice, the algorithm friendly to development/hmr of splitting chunks is doing nothing.
/// But there are number of duplicated modules in very large projects, which affects the performance of the development/hmr.
/// Currently, the plugin does following things:
/// - Split modules shared by multiple chunks into a new chunk.
#[derive(Debug, Default)]
pub struct DevFriendlySplitChunksPlugin;

impl DevFriendlySplitChunksPlugin {
  pub fn new() -> Self {
    Self
  }
}

struct SharedModule {
  module: Identifier,
  ref_chunks: Vec<ChunkUkey>,
  is_initial_loaded: bool,
}

struct ChunkInfo<'a> {
  modules: &'a [SharedModule],
}

#[async_trait::async_trait]
impl Plugin for DevFriendlySplitChunksPlugin {
  fn name(&self) -> &'static str {
    "DevFriendlySplitChunksPlugin"
  }

  async fn optimize_chunks(
    &mut self,
    _ctx: rspack_core::PluginContext,
    args: rspack_core::OptimizeChunksArgs<'_>,
  ) -> rspack_core::PluginOptimizeChunksOutput {
    use rayon::prelude::*;
    let compilation = args.compilation;

    // First we filter out all the shared modules
    let mut shared_modules = compilation
      .module_graph
      .modules()
      .values()
      .par_bridge()
      .map(|m| m.identifier())
      .filter_map(|module| {
        let chunks = compilation.chunk_graph.get_module_chunks(module);

        let is_initial_loaded = chunks.iter().any(|c| {
          c.as_ref(&compilation.chunk_by_ukey)
            .is_only_initial(&compilation.chunk_group_by_ukey)
        });

        if chunks.len() > 1 {
          Some(SharedModule {
            module,
            ref_chunks: chunks.iter().cloned().collect(),
            is_initial_loaded,
          })
        } else {
          None
        }
      })
      .collect::<Vec<_>>();

    // Filter out modules that would be loaded initially
    let mut initial_loaded_shared_modules = {
      let mut idx = 0;
      let mut initial_loaded = vec![];
      while idx < shared_modules.len() {
        let cur_module = &shared_modules[idx];
        if cur_module.is_initial_loaded {
          initial_loaded.push(shared_modules.swap_remove(idx));
        } else {
          idx += 1;
        }
      }
      initial_loaded
    };
    let mut dynamic_loaded_shared_modules = shared_modules;

    let sorter = |a: &SharedModule, b: &SharedModule| {
      // One's size is smaller, one should be put in front.
      let a_size = compilation
        .module_graph
        .module_by_identifier(&a.module)
        .map(|m| m.estimated_size(&rspack_core::SourceType::JavaScript))
        .unwrap_or_default();
      let b_size = compilation
        .module_graph
        .module_by_identifier(&b.module)
        .map(|m| m.estimated_size(&rspack_core::SourceType::JavaScript))
        .unwrap_or_default();
      let ret = a_size.total_cmp(&b_size);
      if ret != std::cmp::Ordering::Equal {
        return ret;
      }

      // If the len of ref_chunks is equal, fallback to compare module id.
      a.module.cmp(&b.module)
    };
    // Sort these modules to make the output stable
    initial_loaded_shared_modules.sort_unstable_by(sorter);
    dynamic_loaded_shared_modules.sort_unstable_by(sorter);

    // The number doesn't go through deep consideration.
    const MAX_MODULES_PER_CHUNK: usize = 2000;
    // About 3mb
    const MAX_SIZE_PER_CHUNK: f64 = 3000000.0;
    // First we group modules by MAX_MODULES_PER_CHUNK

    let split_modules = initial_loaded_shared_modules
      .par_chunks(MAX_MODULES_PER_CHUNK)
      .chain(dynamic_loaded_shared_modules.par_chunks(MAX_MODULES_PER_CHUNK))
      .flat_map(|modules| {
        let size_of_modules: f64 = modules
          .iter()
          .map(|m| {
            let module = compilation
              .module_graph
              .module_by_identifier(&m.module)
              .expect("Should have a module here");

            module.estimated_size(&rspack_core::SourceType::JavaScript)
          })
          .sum();

        if size_of_modules > MAX_SIZE_PER_CHUNK {
          // `modules` are too big, split them
          let mut remain_size_of_modules = size_of_modules;
          let mut last_end_idx = 0;
          let mut list_of_modules = vec![];
          while remain_size_of_modules > MAX_SIZE_PER_CHUNK && last_end_idx < modules.len() {
            let mut size_of_new_modules = 0.0;
            let start_idx = last_end_idx;
            while size_of_new_modules < MAX_SIZE_PER_CHUNK && last_end_idx < modules.len() {
              let module_size = (**compilation
                .module_graph
                .module_by_identifier(&modules[last_end_idx].module)
                .expect("Should have a module here"))
              .estimated_size(&rspack_core::SourceType::JavaScript);
              // about 500kb
              let pre_calculated_size_of_new_modules = size_of_new_modules + module_size;
              if pre_calculated_size_of_new_modules > MAX_SIZE_PER_CHUNK
                && size_of_new_modules != 0.0
                && module_size > 512000.0
              {
                // If the new added module is bigger than 512kb, we would skip it.
                // With this requirements, the max size a chunk is about 3.5mb
                break;
              }
              size_of_new_modules = pre_calculated_size_of_new_modules;
              remain_size_of_modules -= module_size;
              last_end_idx += 1;
            }
            list_of_modules.push(&modules[start_idx..last_end_idx])
          }

          if remain_size_of_modules <= 512000.0 {
            // If the remain_size_of_modules is smaller than 500kb,
            // we would merge remain modules to last `modules`
            let last_modules = list_of_modules.pop().unwrap_or(&[]);
            let start_idx = last_end_idx - last_modules.len();
            list_of_modules.push(&modules[start_idx..])
          } else if last_end_idx < modules.len() {
            list_of_modules.push(&modules[last_end_idx..])
          }

          list_of_modules
        } else {
          vec![modules]
        }
      });

    // Yeah. Leaky abstraction, but fast.
    let module_to_chunk_graph_module = compilation
      .chunk_graph
      .chunk_graph_module_by_module_identifier
      .iter_mut()
      .collect::<DashMap<_, _>>();

    // Yeah. Leaky abstraction, but fast.
    let mut chunk_and_cgc = split_modules
      .map(|modules| {
        let mut chunk = Chunk::new(None, None, rspack_core::ChunkKind::Normal);
        chunk
          .chunk_reasons
          .push("Split with ref count> 1".to_string());
        let mut chunk_graph_chunk = ChunkGraphChunk::new();
        chunk_graph_chunk
          .modules
          .extend(modules.iter().map(|m| m.module));

        modules.iter().for_each(|module| {
          let mut cgm = module_to_chunk_graph_module
            .get_mut(&module.module)
            .expect("mgm should exist");
          cgm.chunks.insert(chunk.ukey);
        });

        let chunk_info = ChunkInfo { modules };

        (chunk_info, chunk, chunk_graph_chunk)
      })
      .collect::<Vec<_>>();

    std::mem::drop(module_to_chunk_graph_module);

    // Split old chunks.
    chunk_and_cgc.iter_mut().for_each(|(info, chunk, _cgc)| {
      info.modules.iter().for_each(|module| {
        module.ref_chunks.iter().for_each(|old_chunk| {
          old_chunk
            .as_mut(&mut compilation.chunk_by_ukey)
            .split(chunk, &mut compilation.chunk_group_by_ukey);
        });
      });
    });

    // Add new chunks to compilation.
    chunk_and_cgc.into_iter().for_each(|(_, chunk, cgc)| {
      compilation
        .chunk_graph
        .add_chunk_wit_chunk_graph_chunk(chunk.ukey, cgc);
      compilation.chunk_by_ukey.add(chunk);
    });

    // Remove shared modules from old chunks, since they are moved to new chunks.
    initial_loaded_shared_modules
      .iter()
      .chain(dynamic_loaded_shared_modules.iter())
      .for_each(|m| {
        m.ref_chunks.iter().for_each(|old_chunk| {
          compilation
            .chunk_graph
            .disconnect_chunk_and_module(old_chunk, m.module);
        });
      });

    Ok(())
  }
}

trait EstimatedSize {
  fn estimated_size(&self, source_type: &rspack_core::SourceType) -> f64;
}

impl<T: Module + ?Sized> EstimatedSize for T {
  fn estimated_size(&self, source_type: &rspack_core::SourceType) -> f64 {
    use rspack_core::ModuleType;
    let coefficient: f64 = match self.module_type() {
      // 5.0 is a number in practice
      rspack_core::ModuleType::Jsx
      | ModuleType::JsxDynamic
      | ModuleType::JsxEsm
      | ModuleType::Tsx => 7.5,
      ModuleType::Js | ModuleType::JsDynamic => 1.5,
      _ => 1.0,
    };

    self.size(source_type) * coefficient
  }
}
