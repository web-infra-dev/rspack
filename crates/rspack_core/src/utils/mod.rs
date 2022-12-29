use std::path::{Component, Path};

use hashbrown::HashSet;
use sugar_path::SugarPath;

mod hooks;
pub use hooks::*;

mod identifier;
pub use identifier::*;

mod source;
pub use source::*;

mod hash;
pub use hash::*;

mod module_rules;
pub use module_rules::*;

mod fast_set;
pub use fast_set::*;

use crate::{ModuleGraph, ModuleIdentifier};

pub fn uri_to_chunk_name(root: &str, uri: &str) -> String {
  let path = Path::new(uri);
  let mut relatived = Path::new(path).relative(root);
  let ext = relatived
    .extension()
    .and_then(|ext| ext.to_str())
    .unwrap_or("")
    .to_string();
  relatived.set_extension("");
  let mut name = relatived
    .components()
    .filter(|com| matches!(com, Component::Normal(_)))
    .filter_map(|seg| seg.as_os_str().to_str())
    .intersperse("_")
    .fold(String::new(), |mut acc, seg| {
      acc.push_str(seg);
      acc
    });
  name.push('_');
  name.push_str(&ext);
  name
}

pub fn parse_to_url(url: &str) -> url::Url {
  if !url.contains(':') {
    let mut construct_string = String::with_capacity("specifier:".len() + url.len());
    construct_string += "specifier:";
    construct_string += url;
    url::Url::parse(&construct_string).unwrap_or_else(|_| {
      panic!(
        "Invalid specifier: {}, please use a valid specifier or a valid url",
        url
      )
    })
  } else {
    url::Url::parse(url).unwrap_or_else(|_| {
      panic!(
        "Invalid specifier: {}, please use a valid specifier or a valid url",
        url
      )
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

pub fn find_module_graph_roots(
  modules: Vec<ModuleIdentifier>,
  module_graph: &ModuleGraph,
) -> Vec<ModuleIdentifier> {
  let mut roots = vec![];
  let mut graph = petgraph::graphmap::DiGraphMap::new();
  let mut queue = modules.into_iter().collect::<Vec<_>>();
  let mut visited = HashSet::new();
  while let Some(module) = queue.pop() {
    let module = module_graph
      .module_by_identifier(&module)
      .expect("module not found");
    if visited.contains(&module.identifier()) {
      continue;
    } else {
      visited.insert(module.identifier())
    };
    let connections = module_graph.get_outgoing_connections(module);
    for connection in connections {
      if let Some(from) = &connection.original_module_identifier {
        graph.add_edge(from, &connection.module_identifier, ());
      } else {
        graph.add_node(&connection.module_identifier);
      }
      queue.push(connection.module_identifier);
    }
  }

  graph
    .nodes()
    .into_iter()
    .filter(|from| {
      graph
        .neighbors_directed(*from, petgraph::Direction::Incoming)
        .count()
        == 0
    })
    .for_each(|from| {
      roots.push(*from);
    });
  roots
}
