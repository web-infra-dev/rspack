use std::cmp::Ordering;

use rspack_collections::Identifier;
use rspack_util::comparators::{compare_ids, compare_numbers};

use crate::{
  BoxModule, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, ModuleGraph,
};
mod comment;
mod compile_boolean_matcher;
mod concatenated_module_visitor;
mod concatenation_scope;
mod extract_source_map;
mod extract_url_and_global;
mod fast_actions;
mod file_counter;
mod find_graph_roots;
mod fs_trim;
pub mod incremental_info;
pub use fs_trim::*;
mod hash;
mod identifier;
mod iterator_consumer;
mod memory_gc;
mod module_rules;
mod property_access;
mod property_name;
mod queue;
mod remove_bom;
mod runtime;
mod source;
pub mod task_loop;
mod template;
mod to_path;
pub use compile_boolean_matcher::*;
pub use concatenated_module_visitor::*;
pub use concatenation_scope::*;
pub use memory_gc::MemoryGCStorage;

pub use self::{
  comment::*,
  extract_source_map::*,
  extract_url_and_global::*,
  fast_actions::*,
  file_counter::{FileCounter, ResourceId},
  find_graph_roots::*,
  hash::*,
  identifier::*,
  iterator_consumer::{FutureConsumer, RayonConsumer, RayonFutureConsumer},
  module_rules::*,
  property_access::*,
  property_name::*,
  queue::*,
  remove_bom::*,
  runtime::*,
  source::*,
  template::*,
  to_path::to_path,
};

/// join string component in a more human readable way
/// e.g.
/// ```
/// use rspack_core::join_string_component;
/// assert_eq!(
///   "a, b and c",
///   join_string_component(vec!["a".to_string(), "b".to_string(), "c".to_string()])
/// );
/// assert_eq!(
///   "a and b",
///   join_string_component(vec!["a".to_string(), "b".to_string(),])
/// );
/// ```
pub fn join_string_component(mut components: Vec<String>) -> String {
  match components.len() {
    0 => String::new(),
    1 => std::mem::take(&mut components[0]),
    2 => {
      format!("{} and {}", components[0], components[1])
    }
    _ => {
      let prefix = &components[0..components.len() - 1];
      format!(
        "{} and {}",
        prefix.join(", "),
        components[components.len() - 1]
      )
    }
  }
}

pub fn sort_group_by_index(
  ukey_a: &ChunkGroupUkey,
  ukey_b: &ChunkGroupUkey,
  chunk_group_by_ukey: &ChunkGroupByUkey,
) -> Ordering {
  let index_a = chunk_group_by_ukey.expect_get(ukey_a).index;
  let index_b = chunk_group_by_ukey.expect_get(ukey_b).index;
  match index_a {
    None => match index_b {
      None => Ordering::Equal,
      Some(_) => Ordering::Greater,
    },
    Some(index_a) => match index_b {
      None => Ordering::Less,
      Some(index_b) => index_a.cmp(&index_b),
    },
  }
}

pub fn compare_chunk_group(
  ukey_a: &ChunkGroupUkey,
  ukey_b: &ChunkGroupUkey,
  compilation: &Compilation,
) -> Ordering {
  let chunks_a = &compilation.chunk_group_by_ukey.expect_get(ukey_a).chunks;
  let chunks_b = &compilation.chunk_group_by_ukey.expect_get(ukey_b).chunks;
  match chunks_a.len().cmp(&chunks_b.len()) {
    Ordering::Less => Ordering::Greater,
    Ordering::Greater => Ordering::Less,
    Ordering::Equal => compare_chunks_iterables(
      &compilation.chunk_graph,
      &compilation.get_module_graph(),
      chunks_a,
      chunks_b,
    ),
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

pub fn compare_modules_by_identifier(a: &BoxModule, b: &BoxModule) -> std::cmp::Ordering {
  compare_ids(&a.identifier(), &b.identifier())
}

pub fn compare_module_iterables(modules_a: &[&BoxModule], modules_b: &[&BoxModule]) -> Ordering {
  let mut a_iter = modules_a.iter();
  let mut b_iter = modules_b.iter();
  loop {
    match (a_iter.next(), b_iter.next()) {
      (None, None) => return Ordering::Equal,
      (None, Some(_)) => return Ordering::Greater,
      (Some(_), None) => return Ordering::Less,
      (Some(a_item), Some(b_item)) => {
        let res = compare_modules_by_identifier(a_item, b_item);
        if res != Ordering::Equal {
          return res;
        }
      }
    }
  }
}

pub fn compare_chunks_iterables(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  a: &[ChunkUkey],
  b: &[ChunkUkey],
) -> Ordering {
  let mut a_iter = a.iter();
  let mut b_iter = b.iter();
  loop {
    match (a_iter.next(), b_iter.next()) {
      (None, None) => return Ordering::Equal,
      (None, Some(_)) => return Ordering::Greater,
      (Some(_), None) => return Ordering::Less,
      (Some(a_item), Some(b_item)) => {
        let res = compare_chunks_with_graph(chunk_graph, module_graph, a_item, b_item);
        if res != Ordering::Equal {
          return res;
        }
      }
    }
  }
}

pub fn compare_chunks_with_graph(
  chunk_graph: &ChunkGraph,
  module_graph: &ModuleGraph,
  chunk_a_ukey: &ChunkUkey,
  chunk_b_ukey: &ChunkUkey,
) -> Ordering {
  let modules_a = chunk_graph.get_chunk_modules_identifier(chunk_a_ukey);
  let modules_b = chunk_graph.get_chunk_modules_identifier(chunk_b_ukey);
  if modules_a.len() > modules_b.len() {
    return Ordering::Less;
  }
  if modules_a.len() < modules_b.len() {
    return Ordering::Greater;
  }

  let modules_a: Vec<&BoxModule> = modules_a
    .iter()
    .filter_map(|module_id| module_graph.module_by_identifier(module_id))
    .collect();
  let modules_b: Vec<&BoxModule> = modules_b
    .iter()
    .filter_map(|module_id| module_graph.module_by_identifier(module_id))
    .collect();
  compare_module_iterables(&modules_a, &modules_b)
}

#[cfg(allocative)]
pub fn snapshot_allocative(name: &str) {
  use std::{
    path::PathBuf,
    sync::{
      LazyLock,
      atomic::{self, AtomicUsize},
    },
  };

  use rspack_util::allocative;

  static ENABLE: LazyLock<Option<PathBuf>> = LazyLock::new(|| {
    std::env::var_os("RSPACK_ALLOCATIVE_DIR")
      .map(|dir| {
        let _ = std::fs::create_dir_all(&dir);
        dir
      })
      .map(Into::into)
  });
  static COUNT: AtomicUsize = AtomicUsize::new(0);

  if let Some(dir) = ENABLE.as_deref() {
    let mut builder = allocative::FlameGraphBuilder::default();
    builder.visit_global_roots();
    let buf = builder.finish_and_write_flame_graph();
    let count = COUNT.fetch_add(1, atomic::Ordering::Relaxed);
    let path = dir.join(format!("{}-{}.allocative", count, name));
    std::fs::write(path, buf).expect("allocative write failed");
  }
}
