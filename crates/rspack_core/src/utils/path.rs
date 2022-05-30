use std::path::{Component, Path};
use sugar_path::{self, PathSugar};

pub fn normalize_path(path: &str, root: &str) -> String {
  Path::new(&root)
    .relative(Path::new(&path))
    .to_string_lossy()
    .to_string()
}

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

pub fn gen_module_id(root: &str, uri: &str) -> String {
  Path::new("./")
    .join(Path::new(uri).relative(root))
    .to_str()
    .unwrap()
    .to_string()
}
