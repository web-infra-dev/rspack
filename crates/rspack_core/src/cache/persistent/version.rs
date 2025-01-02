use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
use rspack_paths::AssertUtf8;

pub fn get_version(
  fs: Arc<dyn ReadableFileSystem>,
  dependencies: &Vec<PathBuf>,
  add_salt: impl FnOnce(&mut DefaultHasher),
) -> String {
  let mut hasher = DefaultHasher::new();
  for dep in dependencies {
    let path = dep.clone().assert_utf8();
    let meta = fs
      .metadata(&path)
      .unwrap_or_else(|_| panic!("Failed to get buildDependency({path}) metadata info."));
    if !meta.is_file {
      panic!("buildDependency({path}) is not a file.");
    }
    let bytes = fs
      .read(&path)
      .unwrap_or_else(|_| panic!("Failed to read buildDependency({path}) content."));
    bytes.hash(&mut hasher);
  }
  add_salt(&mut hasher);
  hex::encode(hasher.finish().to_ne_bytes())
}
