use rspack_core::{BoxModule, ChunkGraph, Compilation, ModuleGraph, ModuleIdentifier};
use rspack_util::{
  comparators::{compare_ids, compare_numbers},
  identifier::make_paths_relative,
  number_hash::get_number_hash,
};
use std::{
  borrow::Cow,
  cmp::Ordering,
  collections::{hash_map::DefaultHasher, HashMap, HashSet},
  hash::{Hash, Hasher},
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

  compilation.module_graph.modules().for_each(|module| {
    let module_id = chunk_graph.get_module_id(&module.identifier());
    if let Some(module_id) = module_id {
      used_ids.insert(module_id.clone());
    } else {
      if filter.as_ref().map_or(true, |f| (f)(module))
        && chunk_graph.get_number_of_module_chunks(&module.identifier()) != 0
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
    return Cow::Owned(format!("_{}", s));
  }
  Cow::Borrowed(s)
}

pub fn get_long_module_name(short_name: &str, module: &BoxModule, context: &str) -> String {
  let full_name = get_full_module_name(module, context);

  format!("{}?{}", short_name, get_hash(&full_name, 4))
}

pub fn get_full_module_name(module: &BoxModule, context: &str) -> String {
  make_paths_relative(context, &module.identifier())
}

pub fn get_hash(s: impl Hash, length: usize) -> String {
  let mut hasher = DefaultHasher::default();
  s.hash(&mut hasher);
  let hash = hasher.finish();
  let mut hash_str = format!("{:x}", hash);
  if hash_str.len() > length {
    hash_str.truncate(length);
  }
  hash_str
}

// TODO: we should remove this function to crate rspack_util
pub fn compare_modules_by_identifier(a: &BoxModule, b: &BoxModule) -> std::cmp::Ordering {
  compare_ids(&a.identifier(), &b.identifier())
}

pub fn assign_names<T: Copy>(
  items: Vec<T>,
  get_short_name: impl Fn(T) -> String,
  get_long_name: impl Fn(T, &str) -> String,
  comparator: impl Fn(&T, &T) -> Ordering,
  used_ids: &mut HashSet<String>,
  mut assign_name: impl FnMut(T, String),
) -> Vec<T> {
  let mut name_to_items: HashMap<String, Vec<T>> = HashMap::default();
  for item in items {
    let name = get_short_name(item);
    name_to_items.entry(name).or_default().push(item);
  }

  let mut name_to_items2: HashMap<String, Vec<T>> = HashMap::default();

  for (name, items) in name_to_items {
    if items.len() > 1 || name.is_empty() {
      for item in items {
        let long_name = get_long_name(item, &name);
        name_to_items2.entry(long_name).or_default().push(item);
      }
    } else {
      name_to_items2.entry(name).or_default().push(items[0]);
    }
  }

  let name_to_items2_keys = name_to_items2.keys().cloned().collect::<HashSet<_>>();

  let mut unnamed_items = vec![];
  for (name, mut items) in name_to_items2 {
    if name.is_empty() {
      for item in items {
        unnamed_items.push(item)
      }
    } else if items.len() == 1 && !used_ids.contains(&name) {
      assign_name(items[0], name.clone());
      used_ids.insert(name.clone());
    } else {
      items.sort_by(&comparator);
      let mut i = 0;
      for item in items {
        let formated_name = format!("{}{}", name, i);
        while name_to_items2_keys.contains(&formated_name) && used_ids.contains(&formated_name) {
          i += 1;
        }
        assign_name(item, formated_name.clone());
        used_ids.insert(formated_name);
        i += 1;
      }
    }
  }
  unnamed_items.sort_by(comparator);
  unnamed_items
}

#[allow(clippy::too_many_arguments)]
pub fn assign_deterministic_ids<T: Copy>(
  mut items: Vec<T>,
  get_name: impl Fn(T) -> String,
  comparator: impl Fn(&T, &T) -> Ordering,
  mut assign_id: impl FnMut(T, String) -> bool,
  ranges: &[usize],
  expand_factor: usize,
  extra_space: usize,
  salt: usize,
) {
  items.sort_by(comparator);

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
    let mut id = get_number_hash(&format!("{}{}", ident, i), range).to_string();
    while !assign_id(item, id) {
      i += 1;
      id = get_number_hash(&format!("{}{}", ident, i), range).to_string();
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
    if chunk_graph.get_module_id(&module.identifier()).is_none() {
      while used_ids.contains(&next_id.to_string()) {
        next_id += 1;
      }
      chunk_graph.set_module_id(&module.identifier(), next_id.to_string());
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
