pub mod log;
use std::path::{Component, Path};

use sugar_path::PathSugar;

mod hooks;
pub use hooks::*;

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
