use std::{
  borrow::Cow,
  cmp::Ordering,
  hash::{Hash, Hasher},
};

use itertools::{
  EitherOrBoth::{Both, Left, Right},
  Itertools,
};
use rspack_collections::{DatabaseItem, Identifier, UkeyMap};
use rspack_core::{
  BoxModule, Chunk, ChunkByUkey, ChunkGraph, ChunkGroupByUkey, ChunkNamedIdArtifact, ChunkUkey,
  Compilation, ExportsInfoArtifact, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier,
  ModuleIdsArtifact, compare_runtime,
};
use rspack_util::{
  comparators::{compare_ids, compare_numbers},
  identifier::make_paths_relative,
  itoa,
  number_hash::get_number_hash,
};
use rustc_hash::{FxHashSet, FxHasher};

#[allow(clippy::type_complexity)]
#[allow(clippy::collapsible_else_if)]
pub fn get_used_module_ids_and_modules(
  compilation: &Compilation,
  filter: Option<Box<dyn Fn(&BoxModule) -> bool>>,
) -> (FxHashSet<String>, Vec<ModuleIdentifier>) {
  get_used_module_ids_and_modules_with_artifact(
    compilation,
    &compilation.module_ids_artifact,
    filter,
  )
}

#[allow(clippy::type_complexity)]
#[allow(clippy::collapsible_else_if)]
pub fn get_used_module_ids_and_modules_with_artifact(
  compilation: &Compilation,
  module_ids_artifact: &ModuleIdsArtifact,
  filter: Option<Box<dyn Fn(&BoxModule) -> bool>>,
) -> (FxHashSet<String>, Vec<ModuleIdentifier>) {
  let chunk_graph = &compilation.build_chunk_graph_artifact.chunk_graph;
  let mut modules = vec![];
  let mut used_ids = FxHashSet::default();

  // TODO: currently we don't have logic of compilation.usedModuleIds
  //   if (compilation.usedModuleIds) {
  //     for (const id of compilation.usedModuleIds) {
  //         usedIds.add(id + "");
  //     }
  //   }

  compilation
    .get_module_graph()
    .modules()
    .map(|(_, module)| module)
    .filter(|m| m.need_id())
    .for_each(|module| {
      let module_id = ChunkGraph::get_module_id(module_ids_artifact, module.identifier());
      if let Some(module_id) = module_id {
        used_ids.insert(module_id.to_string());
      } else {
        if filter.as_ref().is_none_or(|f| (f)(module))
          && chunk_graph.get_number_of_module_chunks(module.identifier()) != 0
        {
          modules.push(module.identifier());
        }
      }
    });
  (used_ids, modules)
}

pub fn get_short_module_name(module: &BoxModule, context: &str) -> String {
  let lib_ident = module.lib_ident(rspack_core::LibIdentOptions { context });
  if let Some(lib_ident) = lib_ident {
    return avoid_number(&lib_ident).to_string();
  };
  let name_for_condition = module.name_for_condition();
  if let Some(name_for_condition) = name_for_condition {
    return avoid_number(&make_paths_relative(context, &name_for_condition)).to_string();
  };
  String::new()
}

fn avoid_number(s: &str) -> Cow<'_, str> {
  if s.len() > 21 {
    return Cow::Borrowed(s);
  }

  let first_char = s.chars().next();

  if let Some(first_char) = first_char {
    if first_char < '1' {
      if first_char == '-' {
        return Cow::Borrowed(s);
      };
    } else if first_char > '9' {
      return Cow::Borrowed(s);
    }
  }
  if s.chars().all(|c| c.is_ascii_digit()) {
    return Cow::Owned(format!("_{s}"));
  }
  Cow::Borrowed(s)
}

pub fn get_long_module_name(short_name: &str, module: &BoxModule, context: &str) -> String {
  let full_name = get_full_module_name(module, context);

  format!("{}?{}", short_name, get_hash(full_name, 4))
}

pub fn get_full_module_name(module: &BoxModule, context: &str) -> String {
  make_paths_relative(context, &module.identifier())
}

pub fn get_hash(s: impl Hash, length: usize) -> String {
  let mut hasher = FxHasher::default();
  s.hash(&mut hasher);
  let hash = hasher.finish();
  let mut hash_str = format!("{hash:x}");
  if hash_str.len() > length {
    hash_str.truncate(length);
  }
  hash_str
}

#[allow(clippy::too_many_arguments)]
pub fn assign_deterministic_ids<T>(
  mut items: Vec<T>,
  get_name: impl Fn(&T) -> String,
  comparator: impl FnMut(&T, &T) -> Ordering,
  mut assign_id: impl FnMut(&T, usize) -> bool,
  ranges: &[usize],
  expand_factor: usize,
  extra_space: usize,
  salt: usize,
) {
  items.sort_unstable_by(comparator);

  let optimal_range = usize::min(items.len() * 20 + extra_space, usize::MAX);
  let mut i = 0;
  debug_assert!(!ranges.is_empty());
  let mut range = ranges[i];
  while range < optimal_range {
    i += 1;
    if i < ranges.len() {
      range = usize::min(ranges[i], usize::MAX);
    } else if expand_factor != 0 {
      range = usize::min(range * expand_factor, usize::MAX);
    } else {
      break;
    }
  }

  for item in items {
    let ident = get_name(&item);
    let mut i = salt;
    let mut i_buffer = itoa::Buffer::new();
    let mut id = get_number_hash(&format!("{ident}{}", i_buffer.format(i)), range);
    while !assign_id(&item, id) {
      i += 1;
      let mut i_buffer = itoa::Buffer::new();
      id = get_number_hash(&format!("{ident}{}", i_buffer.format(i)), range);
    }
  }
}

pub fn assign_ascending_module_ids(
  used_ids: &FxHashSet<String>,
  modules: Vec<&BoxModule>,
  module_ids: &mut ModuleIdsArtifact,
) {
  let mut next_id = 0;
  let mut assign_id = |module: &BoxModule| {
    if ChunkGraph::get_module_id(module_ids, module.identifier()).is_none() {
      while used_ids.contains(&next_id.to_string()) {
        next_id += 1;
      }
      ChunkGraph::set_module_id(module_ids, module.identifier(), next_id.to_string().into());
      next_id += 1;
    }
  };
  for module in modules {
    assign_id(module);
  }
}

pub fn compare_modules_by_pre_order_index_or_identifier(
  module_graph: &ModuleGraph,
  a: &Identifier,
  b: &Identifier,
) -> std::cmp::Ordering {
  if let Some(a) = module_graph.get_pre_order_index(a)
    && let Some(b) = module_graph.get_pre_order_index(b)
  {
    compare_numbers(a, b)
  } else {
    compare_ids(a, b)
  }
}

#[allow(clippy::too_many_arguments)]
pub fn get_short_chunk_name(
  chunk: &Chunk,
  chunk_graph: &ChunkGraph,
  context: &str,
  delimiter: &str,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  named_chunk_ids_artifact: &ChunkNamedIdArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> String {
  if let Some(name) = named_chunk_ids_artifact
    .chunk_short_names
    .get(&chunk.ukey())
  {
    return name.clone();
  }

  let modules = chunk_graph
    .get_chunk_root_modules(
      &chunk.ukey(),
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    )
    .iter()
    .map(|id| {
      module_graph
        .module_by_identifier(id)
        .unwrap_or_else(|| panic!("Module not found {id}"))
    })
    .collect::<Vec<_>>();
  let short_module_names = modules
    .iter()
    .map(|module| {
      let name = get_short_module_name(module, context);
      request_to_id(&name)
    })
    .collect::<Vec<_>>();

  let mut id_name_hints = Vec::from_iter(chunk.id_name_hints().clone());
  id_name_hints.sort_unstable();

  id_name_hints.extend(short_module_names);
  let chunk_name = id_name_hints
    .iter()
    .filter(|id| !id.is_empty())
    .join(delimiter);

  shorten_long_string(chunk_name, delimiter)
}

pub fn shorten_long_string(string: String, delimiter: &str) -> String {
  if string.len() < 100 {
    string
  } else {
    format!(
      "{}{}{}",
      &string[..(100 - 6 - delimiter.len())],
      delimiter,
      get_hash(&string, 6)
    )
  }
}

#[allow(clippy::too_many_arguments)]
pub fn get_long_chunk_name(
  chunk: &Chunk,
  chunk_graph: &ChunkGraph,
  context: &str,
  delimiter: &str,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  named_chunk_ids_artifact: &ChunkNamedIdArtifact,
  exports_info_artifact: &ExportsInfoArtifact,
) -> String {
  if let Some(name) = named_chunk_ids_artifact.chunk_long_names.get(&chunk.ukey()) {
    return name.clone();
  }

  let modules = chunk_graph
    .get_chunk_root_modules(
      &chunk.ukey(),
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    )
    .iter()
    .map(|id| {
      module_graph
        .module_by_identifier(id)
        .expect("Module not found")
    })
    .collect::<Vec<_>>();

  let short_module_names = modules
    .iter()
    .map(|m| request_to_id(&get_short_module_name(m, context)))
    .collect::<Vec<_>>();

  let long_module_names = modules
    .iter()
    .map(|m| request_to_id(&get_long_module_name("", m, context)))
    .collect::<Vec<_>>();
  let mut id_name_hints = chunk.id_name_hints().iter().cloned().collect::<Vec<_>>();
  id_name_hints.sort_unstable();

  let chunk_name = {
    id_name_hints.extend(short_module_names);
    id_name_hints.extend(long_module_names);
    id_name_hints.join(delimiter)
  };

  shorten_long_string(chunk_name, delimiter)
}

pub fn get_full_chunk_name(
  chunk: &Chunk,
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  context: &str,
  exports_info_artifact: &ExportsInfoArtifact,
) -> String {
  if let Some(name) = chunk.name() {
    return name.to_owned();
  }

  let full_module_names = chunk_graph
    .get_chunk_root_modules(
      &chunk.ukey(),
      module_graph,
      module_graph_cache,
      exports_info_artifact,
    )
    .iter()
    .filter_map(|id| module_graph.module_by_identifier(id))
    .map(|module| get_full_module_name(module, context))
    .collect::<Vec<_>>();

  full_module_names.join(",")
}

pub use rspack_util::identifier::request_to_id;

pub fn get_used_chunk_ids(chunk_by_ukey: &ChunkByUkey) -> FxHashSet<String> {
  let mut used_ids = FxHashSet::default();
  for chunk in chunk_by_ukey.values() {
    if let Some(id) = chunk.id() {
      used_ids.insert(id.to_string());
    }
  }
  used_ids
}

pub fn assign_ascending_chunk_ids(chunks: &[ChunkUkey], chunk_by_ukey: &mut ChunkByUkey) {
  let used_ids = get_used_chunk_ids(chunk_by_ukey);

  let mut next_id = 0;
  if !used_ids.is_empty() {
    for chunk in chunks {
      let chunk = chunk_by_ukey.expect_get_mut(chunk);
      if chunk.id().is_none() {
        while used_ids.contains(&next_id.to_string()) {
          next_id += 1;
        }
        chunk.set_id(next_id.to_string());
        next_id += 1;
      }
    }
  } else {
    for chunk in chunks {
      let chunk = chunk_by_ukey.expect_get_mut(chunk);
      if chunk.id().is_none() {
        chunk.set_id(next_id.to_string());
        next_id += 1;
      }
    }
  }
}

fn compare_chunks_by_modules<'a>(
  chunk_graph: &ChunkGraph,
  module_ids: &'a ModuleIdsArtifact,
  a: &Chunk,
  b: &Chunk,
  ordered_chunk_modules_cache: &mut UkeyMap<ChunkUkey, Vec<Option<&'a str>>>,
) -> Ordering {
  let a_ukey = a.ukey();
  let b_ukey = b.ukey();
  let a_modules = ordered_chunk_modules_cache
    .entry(a_ukey)
    .or_insert_with(|| {
      chunk_graph
        .get_ordered_chunk_modules_identifier(&a_ukey)
        .into_iter()
        .map(|m| ChunkGraph::get_module_id(module_ids, m).map(|s| s.as_str()))
        .collect_vec()
    })
    .clone();
  let b_modules = ordered_chunk_modules_cache
    .entry(b_ukey)
    .or_insert_with(|| {
      chunk_graph
        .get_ordered_chunk_modules_identifier(&b_ukey)
        .into_iter()
        .map(|m| ChunkGraph::get_module_id(module_ids, m).map(|s| s.as_str()))
        .collect_vec()
    })
    .clone();

  a_modules
    .into_iter()
    .zip_longest(b_modules)
    .find_map(|pair| match pair {
      Both(a_module_id, b_module_id) => {
        let ordering = compare_ids(
          a_module_id.unwrap_or_default(),
          b_module_id.unwrap_or_default(),
        );
        if ordering != Ordering::Equal {
          return Some(ordering);
        }
        None
      }
      Left(_) => Some(Ordering::Greater),
      Right(_) => Some(Ordering::Less),
    })
    .unwrap_or(Ordering::Equal)
}

fn compare_chunks_by_groups(
  chunk_group_by_ukey: &ChunkGroupByUkey,
  a: &Chunk,
  b: &Chunk,
) -> Ordering {
  let a_groups: Vec<_> = a
    .get_sorted_groups_iter(chunk_group_by_ukey)
    .map(|group| chunk_group_by_ukey.expect_get(group))
    .collect();
  let a_groups_index = a_groups.iter().map(|group| group.index).collect_vec();

  let b_groups: Vec<_> = b
    .get_sorted_groups_iter(chunk_group_by_ukey)
    .map(|group| chunk_group_by_ukey.expect_get(group))
    .collect();
  let b_groups_index = b_groups.iter().map(|group| group.index).collect_vec();

  let groups_ordering = a_groups_index.cmp(&b_groups_index);

  if groups_ordering != Ordering::Equal {
    return groups_ordering;
  }

  let Some(group) = a_groups.first() else {
    return Ordering::Equal;
  };

  let a_in_children = group
    .chunks
    .iter()
    .find_position(|child| **child == a.ukey());
  let b_in_children = group
    .chunks
    .iter()
    .find_position(|child| **child == b.ukey());

  a_in_children.cmp(&b_in_children)
}

pub fn compare_chunks_natural<'a>(
  chunk_graph: &ChunkGraph,
  chunk_group_by_ukey: &ChunkGroupByUkey,
  module_ids: &'a ModuleIdsArtifact,
  a: &Chunk,
  b: &Chunk,
  ordered_chunk_modules_cache: &mut UkeyMap<ChunkUkey, Vec<Option<&'a str>>>,
) -> Ordering {
  let name_ordering = compare_ids(a.name().unwrap_or_default(), b.name().unwrap_or_default());
  if name_ordering != Ordering::Equal {
    return name_ordering;
  }

  let runtime_ordering = compare_runtime(a.runtime(), b.runtime());
  if runtime_ordering != Ordering::Equal {
    return runtime_ordering;
  }

  let modules_ordering =
    compare_chunks_by_modules(chunk_graph, module_ids, a, b, ordered_chunk_modules_cache);
  if modules_ordering != Ordering::Equal {
    return modules_ordering;
  }

  compare_chunks_by_groups(chunk_group_by_ukey, a, b)
}
