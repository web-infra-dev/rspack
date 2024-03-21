use std::{
  borrow::Cow,
  cmp::Ordering,
  collections::{hash_map::DefaultHasher, HashMap, HashSet},
  hash::{Hash, Hasher},
};

use itertools::{
  EitherOrBoth::{Both, Left, Right},
  Itertools,
};
use once_cell::sync::Lazy;
use rayon::prelude::*;
use regex::Regex;
use rspack_core::{
  compare_runtime, BoxModule, Chunk, ChunkGraph, ChunkUkey, Compilation, ModuleGraph,
  ModuleIdentifier,
};
use rspack_util::{
  comparators::{compare_ids, compare_numbers},
  identifier::make_paths_relative,
  number_hash::get_number_hash,
};

#[allow(clippy::type_complexity)]
#[allow(clippy::collapsible_else_if)]
pub fn get_used_module_ids_and_modules(
  compilation: &Compilation,
  filter: Option<Box<dyn Fn(&BoxModule) -> bool>>,
) -> (HashSet<String>, Vec<ModuleIdentifier>) {
  let chunk_graph = &compilation.chunk_graph;
  let mut modules = vec![];
  let mut used_ids = HashSet::new();

  // TODO: currently we don't have logic of compilation.usedModuleIds
  //   if (compilation.usedModuleIds) {
  //     for (const id of compilation.usedModuleIds) {
  //         usedIds.add(id + "");
  //     }
  //   }

  compilation
    .get_module_graph()
    .modules()
    .values()
    .for_each(|module| {
      let module_id = chunk_graph.get_module_id(module.identifier());
      if let Some(module_id) = module_id {
        used_ids.insert(module_id.clone());
      } else {
        if filter.as_ref().map_or(true, |f| (f)(module))
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
  "".to_string()
}

fn avoid_number(s: &str) -> Cow<str> {
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
  let mut hasher = DefaultHasher::default();
  s.hash(&mut hasher);
  let hash = hasher.finish();
  let mut hash_str = format!("{hash:x}");
  if hash_str.len() > length {
    hash_str.truncate(length);
  }
  hash_str
}

// pub fn assign_names<T: Copy>(
//   items: Vec<T>,
//   get_short_name: impl Fn(T) -> String,
//   get_long_name: impl Fn(T, &str) -> String,
//   comparator: impl Fn(&T, &T) -> Ordering,
//   used_ids: &mut HashSet<String>,
//   mut assign_name: impl FnMut(T, String),
// ) -> Vec<T> {
//   let mut name_to_items: HashMap<String, Vec<T>> = HashMap::default();
//   for item in items {
//     let name = get_short_name(item);
//     name_to_items.entry(name).or_default().push(item);
//   }
//
//   let mut name_to_items2: HashMap<String, Vec<T>> = HashMap::default();
//
//   for (name, items) in name_to_items {
//     if items.len() > 1 || name.is_empty() {
//       for item in items {
//         let long_name = get_long_name(item, &name);
//         name_to_items2.entry(long_name).or_default().push(item);
//       }
//     } else {
//       name_to_items2.entry(name).or_default().push(items[0]);
//     }
//   }
//
//   let name_to_items2_keys = name_to_items2.keys().cloned().collect::<HashSet<_>>();
//
//   let mut unnamed_items = vec![];
//   for (name, mut items) in name_to_items2 {
//     if name.is_empty() {
//       for item in items {
//         unnamed_items.push(item)
//       }
//     } else if items.len() == 1 && !used_ids.contains(&name) {
//       assign_name(items[0], name.clone());
//       used_ids.insert(name.clone());
//     } else {
//       items.sort_unstable_by(&comparator);
//       let mut i = 0;
//       for item in items {
//         let formatted_name = format!("{name}{i}");
//         while name_to_items2_keys.contains(&formatted_name) && used_ids.contains(&formatted_name) {
//           i += 1;
//         }
//         assign_name(item, formatted_name.clone());
//         used_ids.insert(formatted_name);
//         i += 1;
//       }
//     }
//   }
//   unnamed_items.sort_unstable_by(comparator);
//   unnamed_items
// }

pub fn assign_names_par<T: Copy + Send>(
  items: Vec<T>,
  get_short_name: impl Fn(T) -> String + std::marker::Sync,
  get_long_name: impl Fn(T, &str) -> String + std::marker::Sync,
  comparator: impl Fn(&T, &T) -> Ordering,
  used_ids: &mut HashSet<String>,
  mut assign_name: impl FnMut(T, String),
) -> Vec<T> {
  let item_name_pair = items
    .into_par_iter()
    .map(|item| {
      let name = get_short_name(item);
      (item, name)
    })
    .collect::<Vec<(T, String)>>();
  let mut name_to_items: HashMap<String, Vec<T>> = HashMap::default();
  let mut invalid_and_repeat_names: HashSet<String> = HashSet::default();
  invalid_and_repeat_names.insert(String::from(""));
  for (item, name) in item_name_pair {
    let items = name_to_items.entry(name.clone()).or_default();
    items.push(item);
    if items.len() > 1 {
      invalid_and_repeat_names.insert(name);
    }
  }

  let item_name_pair = invalid_and_repeat_names
    .iter()
    .flat_map(|name| {
      let mut res = vec![];
      for item in name_to_items.remove(name).unwrap_or_default() {
        res.push((name.clone(), item));
      }
      res
    })
    .par_bridge()
    .map(|(name, item)| {
      let long_name = get_long_name(item, name.as_str());
      (item, long_name)
    })
    .collect::<Vec<(T, String)>>();
  for (item, name) in item_name_pair {
    name_to_items.entry(name).or_default().push(item);
  }

  let name_to_items_keys = name_to_items.keys().cloned().collect::<HashSet<_>>();

  let mut unnamed_items = vec![];
  for (name, mut items) in name_to_items {
    if name.is_empty() {
      for item in items {
        unnamed_items.push(item)
      }
    } else if items.len() == 1 && !used_ids.contains(&name) {
      assign_name(items[0], name.clone());
      used_ids.insert(name.clone());
    } else {
      items.sort_unstable_by(&comparator);
      let mut i = 0;
      for item in items {
        let formatted_name = format!("{name}{i}");
        while name_to_items_keys.contains(&formatted_name) && used_ids.contains(&formatted_name) {
          i += 1;
        }
        assign_name(item, formatted_name.clone());
        used_ids.insert(formatted_name);
        i += 1;
      }
    }
  }
  unnamed_items.sort_unstable_by(comparator);
  unnamed_items
}

#[allow(clippy::too_many_arguments)]
pub fn assign_deterministic_ids<T: Copy>(
  mut items: Vec<T>,
  get_name: impl Fn(T) -> String,
  comparator: impl Fn(&T, &T) -> Ordering,
  mut assign_id: impl FnMut(T, usize) -> bool,
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
    let ident = get_name(item);
    let mut i = salt;
    let mut id = get_number_hash(&format!("{ident}{i}"), range);
    while !assign_id(item, id) {
      i += 1;
      id = get_number_hash(&format!("{ident}{i}"), range);
    }
  }
}

pub fn assign_ascending_module_ids(
  used_ids: &HashSet<String>,
  modules: Vec<&BoxModule>,
  chunk_graph: &mut ChunkGraph,
) {
  let mut next_id = 0;
  let mut assign_id = |module: &BoxModule| {
    if chunk_graph.get_module_id(module.identifier()).is_none() {
      while used_ids.contains(&next_id.to_string()) {
        next_id += 1;
      }
      chunk_graph.set_module_id(module.identifier(), next_id.to_string());
      next_id += 1;
    }
  };
  for module in modules {
    assign_id(module);
  }
}

pub fn compare_modules_by_pre_order_index_or_identifier(
  module_graph: &ModuleGraph,
  a: &BoxModule,
  b: &BoxModule,
) -> Ordering {
  let cmp = compare_numbers(
    module_graph
      .get_pre_order_index(&a.identifier())
      .unwrap_or_default(),
    module_graph
      .get_pre_order_index(&b.identifier())
      .unwrap_or_default(),
  );
  if cmp == Ordering::Equal {
    compare_ids(&a.identifier(), &b.identifier())
  } else {
    cmp
  }
}

pub fn get_short_chunk_name(
  chunk: &Chunk,
  chunk_graph: &ChunkGraph,
  context: &str,
  delimiter: &str,
  module_graph: &ModuleGraph,
) -> String {
  let modules = chunk_graph
    .get_chunk_root_modules(&chunk.ukey, module_graph)
    .iter()
    .map(|id| {
      module_graph
        .module_by_identifier(id)
        .expect("Module not found")
    })
    .collect::<Vec<_>>();
  let short_module_names = modules
    .iter()
    .map(|module| {
      let name = get_short_module_name(module, context);
      request_to_id(&name)
    })
    .collect::<Vec<_>>();

  let mut id_name_hints = Vec::from_iter(chunk.id_name_hints.clone());
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

pub fn get_long_chunk_name(
  chunk: &Chunk,
  chunk_graph: &ChunkGraph,
  context: &str,
  delimiter: &str,
  module_graph: &ModuleGraph,
) -> String {
  let modules = chunk_graph
    .get_chunk_root_modules(&chunk.ukey, module_graph)
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
  let mut id_name_hints = chunk.id_name_hints.iter().cloned().collect::<Vec<_>>();
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
  context: &str,
) -> String {
  if let Some(name) = &chunk.name {
    return name.to_owned();
  }

  let full_module_names = chunk_graph
    .get_chunk_root_modules(&chunk.ukey, module_graph)
    .iter()
    .map(|id| {
      module_graph
        .module_by_identifier(id)
        .expect("Module not found")
    })
    .map(|module| get_full_module_name(module, context))
    .collect::<Vec<_>>();

  full_module_names.join(",")
}

static REGEX1: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\.\.?/)+").expect("Invalid regex"));
static REGEX2: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"(^[.-]|[^a-zA-Z0-9_-])+").expect("Invalid regex"));

pub fn request_to_id(request: &str) -> String {
  REGEX2
    .replace_all(&REGEX1.replace(request, ""), "_")
    .to_string()
}

pub fn get_used_chunk_ids(compilation: &Compilation) -> HashSet<String> {
  let mut used_ids = compilation
    .used_chunk_ids
    .iter()
    .cloned()
    .collect::<HashSet<_>>();
  for chunk in compilation.chunk_by_ukey.values() {
    if let Some(id) = &chunk.id {
      used_ids.insert(id.clone());
    }
  }
  used_ids
}

pub fn assign_ascending_chunk_ids(chunks: &[ChunkUkey], compilation: &mut Compilation) {
  let used_ids = get_used_chunk_ids(compilation);

  let mut next_id = 0;
  if !used_ids.is_empty() {
    for chunk in chunks {
      let chunk = compilation.chunk_by_ukey.expect_get_mut(chunk);
      if chunk.id.is_none() {
        while used_ids.contains(&next_id.to_string()) {
          next_id += 1;
        }
        chunk.id = Some(next_id.to_string());
        chunk.ids = vec![next_id.to_string()];
        next_id += 1;
      }
    }
  } else {
    for chunk in chunks {
      let chunk = compilation.chunk_by_ukey.expect_get_mut(chunk);
      if chunk.id.is_none() {
        chunk.id = Some(next_id.to_string());
        chunk.ids = vec![next_id.to_string()];
        next_id += 1;
      }
    }
  }
}

fn compare_chunks_by_modules(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  a: &Chunk,
  b: &Chunk,
) -> Ordering {
  let a_modules = chunk_graph.get_ordered_chunk_modules(&a.ukey, module_graph);
  let b_modules = chunk_graph.get_ordered_chunk_modules(&b.ukey, module_graph);

  a_modules
    .into_iter()
    .zip_longest(b_modules)
    .find_map(|pair| match pair {
      Both(a_module, b_module) => {
        let a_module_id = chunk_graph.get_module_id(a_module.identifier());
        let b_module_id = chunk_graph.get_module_id(b_module.identifier());
        let ordering = compare_ids(
          &a_module_id.clone().unwrap_or_default(),
          &b_module_id.clone().unwrap_or_default(),
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

pub fn compare_chunks_natural(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  a: &Chunk,
  b: &Chunk,
) -> Ordering {
  let name_ordering = compare_ids(
    &a.name.clone().unwrap_or_default(),
    &b.name.clone().unwrap_or_default(),
  );
  if name_ordering != Ordering::Equal {
    return name_ordering;
  }

  let runtime_ordering = compare_runtime(&a.runtime, &b.runtime);
  if runtime_ordering != Ordering::Equal {
    return runtime_ordering;
  }

  compare_chunks_by_modules(chunk_graph, module_graph, a, b)
}
