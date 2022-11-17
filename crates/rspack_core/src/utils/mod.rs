use std::{
  path::{Component, Path},
  sync::Arc,
};

use dashmap::DashMap;
use once_cell::sync::Lazy;
use sugar_path::SugarPath;

mod hooks;
pub use hooks::*;

mod identifier;
pub use identifier::*;

mod source;
pub use source::*;

mod hash;
pub use hash::*;

mod tree_shaking;
pub use tree_shaking::*;

mod module_rules;
pub use module_rules::*;

mod snapshot;
pub use snapshot::*;

pub static PATH_START_BYTE_POS_MAP: Lazy<Arc<DashMap<String, u32>>> =
  Lazy::new(|| Arc::new(DashMap::new()));

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
    url::Url::parse(&construct_string).unwrap()
  } else {
    url::Url::parse(url).unwrap()
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
