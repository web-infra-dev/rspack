use std::fmt::Display;

use itertools::Itertools;
use rustc_hash::FxHashMap as HashMap;

mod hooks;
pub use hooks::*;

mod identifier;
pub use identifier::*;

mod comment;
pub use comment::*;

mod source;
pub use source::*;

mod hash;
pub use hash::*;

mod module_rules;
pub use module_rules::*;

mod fast_actions;
pub use fast_actions::*;

mod queue;
pub use queue::*;

mod find_graph_roots;
pub use find_graph_roots::*;

mod visitor;
pub use visitor::*;

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
    map.keys().sorted().fold(String::new(), |prev, cur| {
      prev + format!(r#""{}": {},"#, cur, map.get(cur).expect("get key from map")).as_str()
    })
  )
}
