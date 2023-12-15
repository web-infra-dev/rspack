use std::{cmp::Ordering, fmt::Display};

use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

use crate::{ChunkGroupByUkey, ChunkGroupUkey};

mod comment;
mod extract_url_and_global;
mod fast_actions;
mod find_graph_roots;
mod hash;
mod identifier;
mod import_var;
mod module_rules;
mod property_access;
mod property_name;
mod queue;
mod runtime;
mod source;
mod template;
mod to_path;
mod visitor;

pub use self::comment::*;
pub use self::extract_url_and_global::*;
pub use self::fast_actions::*;
pub use self::find_graph_roots::*;
pub use self::hash::*;
pub use self::identifier::*;
pub use self::import_var::*;
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
  let index_a = chunk_group_by_ukey
    .get(ukey_a)
    .expect("Group should exists")
    .index;
  let index_b = chunk_group_by_ukey
    .get(ukey_b)
    .expect("Group should exists")
    .index;
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
