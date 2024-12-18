use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::sync::Arc;

use rspack_fs::{Error, FileSystem, Result};
use rspack_paths::AssertUtf8;

pub fn get_version(
  fs: Arc<dyn FileSystem>,
  dependencies: &Vec<PathBuf>,
  salt: Vec<&str>,
) -> Result<String> {
  let mut hasher = DefaultHasher::new();
  for dep in dependencies {
    let path = dep.clone().assert_utf8();
    let meta = fs.metadata(&path)?;
    if !meta.is_file {
      return Err(Error::Io(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!("{path:?} is not a file"),
      )));
    }
    let bytes = fs.read(&path)?;
    bytes.hash(&mut hasher);
  }
  salt.hash(&mut hasher);
  Ok(hex::encode(hasher.finish().to_ne_bytes()))
}
