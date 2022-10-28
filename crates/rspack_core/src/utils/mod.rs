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

mod hash;
pub use hash::*;

mod tree_shaking;
pub use tree_shaking::*;

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

pub fn parse_to_url(uri: &str) -> fluent_uri::Uri<String> {
  if !uri.contains(':') {
    let mut construct_string = String::with_capacity("specifier:".len() + uri.len());
    construct_string += "specifier:";
    construct_string += uri;
    fluent_uri::Uri::parse_from(construct_string).unwrap()
  } else {
    fluent_uri::Uri::parse_from(uri.to_string()).unwrap()
  }
}
