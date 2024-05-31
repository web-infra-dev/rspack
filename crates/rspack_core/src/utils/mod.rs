use std::{cmp::Ordering, fmt::Display};

use itertools::Itertools;
use rspack_identifier::Identifier;
use rspack_util::comparators::compare_ids;
use rspack_util::comparators::compare_numbers;
use rustc_hash::FxHashMap as HashMap;

use crate::{
  BoxModule, ChunkGraph, ChunkGroupByUkey, ChunkGroupUkey, ChunkUkey, Compilation, ModuleGraph,
};

mod comment;
mod compile_boolean_matcher;
mod concatenated_module_visitor;
mod concatenation_scope;
mod extract_url_and_global;
mod fast_actions;
mod find_graph_roots;
mod hash;
mod identifier;
mod module_rules;
mod property_access;
mod property_name;
mod queue;
mod runtime;
mod source;
pub mod task_loop;
mod template;
mod to_path;
mod visitor;
pub use compile_boolean_matcher::*;
pub use concatenated_module_visitor::*;
pub use concatenation_scope::*;

pub use self::comment::*;
pub use self::extract_url_and_global::*;
pub use self::fast_actions::*;
pub use self::find_graph_roots::*;
pub use self::hash::*;
pub use self::identifier::*;
pub use self::module_rules::*;
pub use self::property_access::*;
pub use self::property_name::*;
pub use self::queue::*;
pub use self::runtime::*;
pub use self::source::*;
pub use self::template::*;
pub use self::to_path::to_path;
pub use self::visitor::*;

pub fn parse_to_url(url: &str) -> url::Url {
  if !url.contains(':') {
    let mut construct_string = String::with_capacity("specifier:".len() + url.len());
    construct_string += "specifier:";
    construct_string += url;
    url::Url::parse(&construct_string).unwrap_or_else(|_| {
      panic!("Invalid specifier: {url}, please use a valid specifier or a valid url")
    })
  } else {
    url::Url::parse(url).unwrap_or_else(|_| {
      panic!("Invalid specifier: {url}, please use a valid specifier or a valid url")
    })
  }
}

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
    0 => "".to_string(),
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

pub fn stringify_map<T: Display>(map: &HashMap<String, T>) -> String {
  format!(
    r#"{{{}}}"#,
    map
      .keys()
      .sorted_unstable()
      .fold(String::new(), |prev, cur| {
        prev + format!(r#""{}": {},"#, cur, map.get(cur).expect("get key from map")).as_str()
      })
  )
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
  let cgc_a = chunk_graph.get_chunk_graph_chunk(chunk_a_ukey);
  let cgc_b = chunk_graph.get_chunk_graph_chunk(chunk_b_ukey);
  if cgc_a.modules.len() > cgc_b.modules.len() {
    return Ordering::Less;
  }
  if cgc_a.modules.len() < cgc_b.modules.len() {
    return Ordering::Greater;
  }

  let modules_a: Vec<&BoxModule> = cgc_a
    .modules
    .iter()
    .filter_map(|module_id| module_graph.module_by_identifier(module_id))
    .collect();
  let modules_b: Vec<&BoxModule> = cgc_b
    .modules
    .iter()
    .filter_map(|module_id| module_graph.module_by_identifier(module_id))
    .collect();
  compare_module_iterables(&modules_a, &modules_b)
}
