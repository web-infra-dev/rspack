use std::{
  collections::HashSet,
  sync::atomic::{AtomicBool, Ordering},
};

use rspack_collections::{
  Identifier, IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyMap,
  UkeySet,
};
use rspack_core::{
  ChunkUkey, Compilation, CompilationOptimizeChunks, CompilationParams, CompilerCompilation,
  Logger, Module, ModuleIdentifier, Plugin, SourceType,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_css::CssPlugin;
use rspack_regex::RspackRegex;

const MIN_CSS_CHUNK_SIZE: f64 = 30_f64 * 1024_f64;
const MAX_CSS_CHUNK_SIZE: f64 = 100_f64 * 1024_f64;

fn is_global_css(name_for_condition: &Option<Box<str>>) -> bool {
  name_for_condition.as_ref().is_some_and(|s| {
    !s.ends_with(".module.css") && !s.ends_with(".module.scss") && !s.ends_with(".module.sass")
  })
}

#[derive(Debug)]
pub struct CssChunkingPluginOptions {
  pub strict: bool,
  pub min_size: Option<f64>,
  pub max_size: Option<f64>,
  pub exclude: Option<RspackRegex>,
}

#[plugin]
#[derive(Debug)]
pub struct CssChunkingPlugin {
  once: AtomicBool,
  strict: bool,
  min_size: f64,
  max_size: f64,
  exclude: Option<RspackRegex>,
}

impl CssChunkingPlugin {
  pub fn new(options: CssChunkingPluginOptions) -> Self {
    Self::new_inner(
      AtomicBool::new(false),
      options.strict,
      options.min_size.unwrap_or(MIN_CSS_CHUNK_SIZE),
      options.max_size.unwrap_or(MAX_CSS_CHUNK_SIZE),
      options.exclude,
    )
  }
}

#[plugin_hook(CompilerCompilation for CssChunkingPlugin)]
async fn compilation(
  &self,
  _compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  self.once.store(false, Ordering::Relaxed);
  Ok(())
}

#[derive(Debug)]
struct ChunkState {
  chunk: ChunkUkey,
  modules: Vec<ModuleIdentifier>,
  requests: usize,
}

#[plugin_hook(CompilationOptimizeChunks for CssChunkingPlugin, stage = 5)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let strict = self.strict;

  if self.once.load(Ordering::Relaxed) {
    return Ok(None);
  }
  self.once.store(true, Ordering::Relaxed);

  let logger = compilation.get_logger("rspack.CssChunkingPlugin");

  let start = logger.time("collect all css modules and the execpted order of them");
  let mut chunk_states: UkeyMap<ChunkUkey, ChunkState> = Default::default();
  let mut chunk_states_by_module: IdentifierIndexMap<UkeyMap<ChunkUkey, usize>> =
    Default::default();

  // Collect all css modules in chunks and the execpted order of them
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let chunks = &compilation.build_chunk_graph_artifact.chunk_by_ukey;
  let module_graph = compilation.get_module_graph();

  for (chunk_ukey, chunk) in chunks.iter() {
    if let Some(name) = chunk.name()
      && let Some(exclude) = &self.exclude
      && exclude.test(name)
    {
      continue;
    }

    let modules: Vec<&dyn Module> = chunk_graph
      .get_chunk_modules(chunk_ukey, module_graph)
      .into_iter()
      .filter(|module| {
        module.source_types(module_graph).iter().any(|t| match t {
          SourceType::Css => true,
          SourceType::CssImport => true,
          SourceType::Custom(str) => str == "css/mini-extract",
          _ => false,
        })
      })
      .map(|module| module.as_ref())
      .collect();
    if modules.is_empty() {
      continue;
    }
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let (ordered_modules, _) = CssPlugin::get_modules_in_order(chunk, modules, compilation);
    let mut module_identifiers: Vec<ModuleIdentifier> = Vec::with_capacity(ordered_modules.len());
    for (i, module) in ordered_modules.iter().enumerate() {
      let module_identifier = module.identifier();
      module_identifiers.push(module_identifier);

      match chunk_states_by_module.entry(module_identifier) {
        indexmap::map::Entry::Occupied(mut occupied_entry) => {
          let module_chunk_states = occupied_entry.get_mut();
          module_chunk_states.insert(*chunk_ukey, i);
        }
        indexmap::map::Entry::Vacant(vacant_entry) => {
          let mut module_chunk_states = UkeyMap::default();
          module_chunk_states.insert(*chunk_ukey, i);
          vacant_entry.insert(module_chunk_states);
        }
      };
    }
    let requests = module_identifiers.len();
    let chunk_state = ChunkState {
      chunk: *chunk_ukey,
      modules: module_identifiers,
      requests,
    };
    chunk_states.insert(*chunk_ukey, chunk_state);
  }

  let module_infos: IdentifierMap<(f64, Option<Box<str>>)> = {
    let module_graph = compilation.get_module_graph();
    let mut result = IdentifierMap::default();
    for module_identifier in chunk_states_by_module.keys() {
      #[allow(clippy::unwrap_used)]
      let module = module_graph
        .module_by_identifier(module_identifier)
        .unwrap();
      let size = module.size(None, None);
      result.insert(*module_identifier, (size, module.name_for_condition()));
    }
    result
  };
  logger.time_end(start);

  // Sort modules by their index sum
  let start = logger.time("sort modules by their index sum");
  let mut ordered_modules: Vec<(ModuleIdentifier, usize)> = chunk_states_by_module
    .iter()
    .map(|(module_identifier, module_states)| {
      let sum = module_states.values().sum();
      (*module_identifier, sum)
    })
    .collect();
  ordered_modules.sort_by_key(|&(_module, sum)| sum);
  let mut remaining_modules: IdentifierIndexSet = ordered_modules
    .into_iter()
    .map(|(module_identifier, _)| module_identifier)
    .collect();
  logger.time_end(start);

  // In loose mode we guess the dependents of modules from the order
  // assuming that when a module is a dependency of another module
  // it will always appear before it in every chunk.
  let mut all_dependents: IdentifierMap<HashSet<ModuleIdentifier>> = IdentifierMap::default();
  if !self.strict {
    let start = logger.time("guess the dependents of modules from the order");
    for b in &remaining_modules {
      let mut dependents = HashSet::new();
      'outer: for a in &remaining_modules {
        if a == b {
          continue;
        }
        let a_states = &chunk_states_by_module[a];
        let b_states = &chunk_states_by_module[b];
        // check if a depends on b
        for (chunk_ukey, ia) in a_states {
          match b_states.get(chunk_ukey) {
            // If a would depend on b, it would be included in that chunk group too
            None => continue 'outer,
            // If a would depend on b, b would be before a in order
            Some(&ib) if ib > *ia => continue 'outer,
            _ => {}
          }
        }
        dependents.insert(*a);
      }
      if !dependents.is_empty() {
        all_dependents.insert(*b, dependents);
      }
    }
    logger.time_end(start);
  }

  // Stores the new chunk for every module
  let mut new_chunks_by_module: IdentifierMap<ChunkUkey> = IdentifierMap::default();

  // Process through all modules
  let start = logger.time("process through all modules");
  loop {
    let Some(start_module_identifier) = remaining_modules.iter().next().copied() else {
      break;
    };
    remaining_modules.shift_remove(&start_module_identifier);

    #[allow(clippy::unwrap_used)]
    let mut global_css_mode = is_global_css(&module_infos[&start_module_identifier].1);

    // The current position of processing in all selected chunks
    #[allow(clippy::unwrap_used)]
    let all_chunk_states = chunk_states_by_module
      .get(&start_module_identifier)
      .unwrap();

    // The list of modules that goes into the new chunk
    let mut new_chunk_modules = IdentifierSet::default();
    new_chunk_modules.insert(start_module_identifier);

    // The current size of the new chunk
    #[allow(clippy::unwrap_used)]
    let mut current_size = module_infos[&start_module_identifier].0;

    // A pool of potential modules where the next module is selected from.
    // It's filled from the next module of the selected modules in every chunk.
    // It also keeps some metadata to improve performance [size, chunkStates].
    let mut potential_next_modules: IdentifierIndexMap<f64> = Default::default();
    for (chunk_ukey, i) in all_chunk_states {
      #[allow(clippy::unwrap_used)]
      let chunk_state = &chunk_states[chunk_ukey];
      if let Some(next_module_identifier) = chunk_state.modules.get(i + 1)
        && remaining_modules.contains(next_module_identifier)
      {
        #[allow(clippy::unwrap_used)]
        let next_module_size = module_infos[next_module_identifier].0;
        potential_next_modules.insert(*next_module_identifier, next_module_size);
      }
    }

    // Try to add modules to the chunk until a break condition is met
    let mut cont = true;
    while cont {
      cont = false;

      // We try to select a module that reduces request count and
      // has the highest number of requests
      #[allow(clippy::unwrap_used)]
      let all_chunk_states = chunk_states_by_module
        .get(&start_module_identifier)
        .unwrap();
      let mut ordered_potential_next_modules: Vec<(Identifier, f64, usize)> =
        potential_next_modules
          .iter()
          .map(|(next_module_identifier, size)| {
            #[allow(clippy::unwrap_used)]
            let next_chunk_states = chunk_states_by_module.get(next_module_identifier).unwrap();
            let mut max_requests = 0;
            for next_chunk_ukey in next_chunk_states.keys() {
              // There is always some overlap
              if all_chunk_states.contains_key(next_chunk_ukey) {
                #[allow(clippy::unwrap_used)]
                let chunk_state = &chunk_states[next_chunk_ukey];
                max_requests = max_requests.max(chunk_state.requests);
              }
            }
            (*next_module_identifier, *size, max_requests)
          })
          .collect();
      ordered_potential_next_modules.sort_by(|a, b| b.2.cmp(&a.2).then_with(|| a.0.cmp(&b.0)));

      // Try every potential module
      'outer: for (next_module_identifier, size, _) in ordered_potential_next_modules {
        if current_size + size > self.max_size {
          // Chunk would be too large
          continue;
        }
        #[allow(clippy::unwrap_used)]
        let next_chunk_states = chunk_states_by_module
          .get(&next_module_identifier)
          .cloned()
          .unwrap();
        if !strict {
          // In loose mode we only check if the dependencies are not violated
          if let Some(deps) = all_dependents.get(&next_module_identifier) {
            let new_chunk_modules_ref = &new_chunk_modules;
            if deps.iter().any(|d| new_chunk_modules_ref.contains(d)) {
              continue;
            }
          }
        } else {
          // In strict mode we check that none of the order in any chunk is changed by adding the module
          for (chunk_ukey, i) in &next_chunk_states {
            match all_chunk_states.get(chunk_ukey) {
              None => {
                // New chunk group, can add it, but should we?
                // We only add that if below min size
                if current_size >= self.min_size {
                  continue 'outer;
                }
              }
              Some(&prev_idx) if prev_idx + 1 == *i => {}
              _ => continue 'outer,
            }
          }
        }

        // Global CSS must not leak into unrelated chunks
        #[allow(clippy::unwrap_used)]
        let is_global = is_global_css(&module_infos[&next_module_identifier].1);
        if is_global && global_css_mode && all_chunk_states.len() != next_chunk_states.len() {
          // Fast check: chunk groups need to be identical
          continue;
        }
        if global_css_mode
          && next_chunk_states
            .keys()
            .any(|cs| !all_chunk_states.contains_key(cs))
        {
          continue;
        }
        if is_global
          && all_chunk_states
            .keys()
            .any(|cs| !next_chunk_states.contains_key(cs))
        {
          continue;
        }
        potential_next_modules.shift_remove(&next_module_identifier);
        current_size += size;
        if is_global {
          global_css_mode = true;
        }
        #[allow(clippy::unwrap_used)]
        let all_chunk_states = chunk_states_by_module
          .get_mut(&start_module_identifier)
          .unwrap();
        for (chunk_ukey, i) in next_chunk_states {
          #[allow(clippy::unwrap_used)]
          let chunk_state = chunk_states.get_mut(&chunk_ukey).unwrap();
          if all_chunk_states.contains_key(&chunk_ukey) {
            // This reduces the request count of the chunk group
            chunk_state.requests -= 1;
          }
          all_chunk_states.insert(chunk_ukey, i);
          if let Some(next_module_identifier) = chunk_state.modules.get(i + 1)
            && remaining_modules.contains(next_module_identifier)
            && !new_chunk_modules.contains(next_module_identifier)
          {
            #[allow(clippy::unwrap_used)]
            let next_module_size = module_infos[next_module_identifier].0;
            potential_next_modules.insert(*next_module_identifier, next_module_size);
          }
        }
        new_chunk_modules.insert(next_module_identifier);
        cont = true;
        break;
      }
    }
    let new_chunk_ukey =
      Compilation::add_chunk(&mut compilation.build_chunk_graph_artifact.chunk_by_ukey);
    #[allow(clippy::unwrap_used)]
    let new_chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .get_mut(&new_chunk_ukey)
      .unwrap();
    new_chunk.prevent_integration();
    new_chunk.add_id_name_hints("css".to_string());
    let chunk_graph = &mut compilation.build_chunk_graph_artifact.chunk_graph;
    for module_identifier in &new_chunk_modules {
      remaining_modules.shift_remove(module_identifier);
      chunk_graph.connect_chunk_and_module(new_chunk_ukey, *module_identifier);
      new_chunks_by_module.insert(*module_identifier, new_chunk_ukey);
    }
  }
  logger.time_end(start);

  let start = logger.time("apply split chunks");
  let chunk_graph = &mut compilation.build_chunk_graph_artifact.chunk_graph;
  for chunk_state in chunk_states.values() {
    let mut chunks: UkeySet<ChunkUkey> = UkeySet::default();
    for module_identifier in &chunk_state.modules {
      if let Some(new_chunk_ukey) = new_chunks_by_module.get(module_identifier) {
        chunk_graph.disconnect_chunk_and_module(&chunk_state.chunk, *module_identifier);
        if chunks.contains(new_chunk_ukey) {
          continue;
        }
        chunks.insert(*new_chunk_ukey);
        let chunk_by_ukey = &mut compilation.build_chunk_graph_artifact.chunk_by_ukey;
        let [chunk, new_chunk] = chunk_by_ukey.get_many_mut([&chunk_state.chunk, new_chunk_ukey]);
        #[allow(clippy::unwrap_used)]
        chunk.unwrap().split(
          new_chunk.unwrap(),
          &mut compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
        );
      }
    }
  }
  logger.time_end(start);

  Ok(None)
}

impl Plugin for CssChunkingPlugin {
  fn name(&self) -> &'static str {
    "rspack.CssChunkingPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));

    Ok(())
  }
}
