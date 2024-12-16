use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::Path;
use std::sync::Arc;

use rspack_fs::{Error, FileSystem, Result};
use rspack_paths::AssertUtf8;

pub fn get_version(
  context: &Path,
  fs: Arc<dyn FileSystem>,
  dependencies: &Vec<String>,
  salt: Vec<&String>,
) -> Result<String> {
  let mut hasher = DefaultHasher::new();
  for dep in dependencies {
    let path = context.join(dep).assert_utf8();
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
