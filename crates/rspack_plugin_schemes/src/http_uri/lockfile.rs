use std::{
  io,
  path::{Path, PathBuf},
  sync::Arc,
};

use async_trait::async_trait;
use rspack_fs::{FsResultToIoResultExt, WritableFileSystem};
use rspack_paths::Utf8Path;
use rspack_util::fx_hash::FxHashMap;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct LockfileEntry {
  pub resolved: String,
  pub integrity: String,
  pub content_type: String,
}

impl LockfileEntry {
  pub fn has_same_content(&self, other: &Self) -> bool {
    self.resolved == other.resolved
      && self.integrity == other.integrity
      && self.content_type == other.content_type
  }
}

/// Cloning snapshots the locked metadata before asynchronous resource reads, releasing the lock.
#[derive(Debug, Clone)]
pub enum LockfileValue {
  Content(LockfileEntry),
  Tag(String),
}

#[derive(Debug, Clone)]
pub struct Lockfile {
  version: u8,
  entries: FxHashMap<String, LockfileValue>,
}

impl Lockfile {
  pub fn new() -> Self {
    Lockfile {
      version: 1,
      entries: FxHashMap::default(),
    }
  }

  pub fn parse(content: &str) -> Result<Self, String> {
    let data: serde_json::Value = serde_json::from_str(content).map_err(|e| e.to_string())?;

    let entries = data
      .as_object()
      .ok_or_else(|| "Invalid HTTP lockfile: expected an object".to_string())?;
    let version = data.get("version").unwrap_or(&serde_json::Value::Null);
    if version.as_u64() != Some(1) {
      return Err(format!("Unsupported lockfile version {version}"));
    }

    let mut lockfile = Lockfile::new();

    for (key, value) in entries {
      if key == "version" {
        continue;
      }
      if let Some(tag @ ("ignore" | "no-cache")) = value.as_str() {
        lockfile
          .entries
          .insert(key.clone(), LockfileValue::Tag(tag.to_string()));
        continue;
      }
      let entry = LockfileEntry {
        resolved: value
          .get("resolved")
          .and_then(|v| v.as_str())
          .unwrap_or(key)
          .to_string(),
        integrity: value
          .get("integrity")
          .and_then(|v| v.as_str())
          .filter(|integrity| !integrity.is_empty())
          .ok_or_else(|| format!("Invalid HTTP lockfile entry for {key}: missing integrity"))?
          .to_string(),
        content_type: value
          .get("contentType")
          .and_then(|v| v.as_str())
          .unwrap_or("")
          .to_string(),
      };
      lockfile
        .entries
        .insert(key.clone(), LockfileValue::Content(entry));
    }

    Ok(lockfile)
  }

  pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
    // Match webpack's stable, URL-sorted lockfile representation.
    let mut content = String::from("{\n");
    let mut entries: Vec<_> = self.entries.iter().collect();
    entries.sort_unstable_by_key(|(key, _)| *key);
    for (key, value) in entries {
      content.push_str("  ");
      content.push_str(&serde_json::to_string(key)?);
      content.push_str(": ");
      match value {
        LockfileValue::Tag(tag) => content.push_str(&serde_json::to_string(tag)?),
        LockfileValue::Content(entry) => {
          content.push_str("{ ");
          if entry.resolved != *key {
            content.push_str("\"resolved\": ");
            content.push_str(&serde_json::to_string(&entry.resolved)?);
            content.push_str(", ");
          }
          content.push_str("\"integrity\": ");
          content.push_str(&serde_json::to_string(&entry.integrity)?);
          content.push_str(", \"contentType\": ");
          content.push_str(&serde_json::to_string(&entry.content_type)?);
          content.push_str(" }");
        }
      }
      content.push_str(",\n");
    }
    content.push_str(&format!("  \"version\": {}\n}}\n", self.version));
    Ok(content)
  }

  pub fn get_entry(&self, resource: &str) -> Option<&LockfileValue> {
    self.entries.get(resource)
  }
}

#[async_trait]
#[allow(dead_code)]
pub trait LockfileAsync {
  async fn read_from_file_async<
    P: AsRef<Path> + Send,
    F: WritableFileSystem + Send + Sync + ?Sized,
  >(
    path: P,
    filesystem: &F,
  ) -> io::Result<Lockfile>;
  async fn write_to_file_async<
    P: AsRef<Path> + Send,
    F: WritableFileSystem + Send + Sync + ?Sized,
  >(
    &self,
    path: P,
    filesystem: &F,
  ) -> io::Result<()>;
}

#[async_trait]
impl LockfileAsync for Lockfile {
  async fn read_from_file_async<
    P: AsRef<Path> + Send,
    F: WritableFileSystem + Send + Sync + ?Sized,
  >(
    path: P,
    filesystem: &F,
  ) -> io::Result<Lockfile> {
    let utf8_path = Utf8Path::from_path(path.as_ref())
      .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8 path"))?;
    let content = filesystem.read_file(utf8_path).await.to_io_result()?;
    let content_str =
      String::from_utf8(content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Lockfile::parse(&content_str).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
  }

  async fn write_to_file_async<P: AsRef<Path> + Send, F: WritableFileSystem + ?Sized>(
    &self,
    path: P,
    filesystem: &F,
  ) -> io::Result<()> {
    let utf8_path = Utf8Path::from_path(path.as_ref())
      .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8 path"))?;
    let content = self
      .to_json_string()
      .map_err(|e| io::Error::other(e.to_string()))?;
    filesystem
      .write(utf8_path, content.as_bytes())
      .await
      .map_err(|e| io::Error::other(e.to_string()))?;
    Ok(())
  }
}

#[derive(Debug)]
pub struct LockfileCache {
  lockfile: Mutex<Lockfile>,
  lockfile_path: Option<PathBuf>,
  filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
}

impl LockfileCache {
  pub fn new(
    lockfile_path: Option<PathBuf>,
    filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
  ) -> Self {
    LockfileCache {
      lockfile: Mutex::new(Lockfile::new()),
      lockfile_path,
      filesystem,
    }
  }

  pub async fn get_entry(&self, url: &str) -> io::Result<Option<LockfileValue>> {
    let mut lockfile = self.lockfile.lock().await;

    if let Some(lockfile_path) = &self.lockfile_path {
      match Lockfile::read_from_file_async(lockfile_path, self.filesystem.as_ref()).await {
        Ok(lf) => {
          *lockfile = lf;
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
          *lockfile = Lockfile::new();
        }
        Err(e) => return Err(e),
      }
    }

    Ok(lockfile.get_entry(url).cloned())
  }

  pub async fn store_entry(&self, url: &str, entry: LockfileValue) -> io::Result<()> {
    // Keep updates and writes under the same lock so concurrent module reads cannot lose entries.
    let mut lockfile = self.lockfile.lock().await;
    lockfile.entries.insert(url.to_string(), entry);

    if let Some(lockfile_path) = &self.lockfile_path {
      if let Some(parent) = lockfile_path.parent() {
        let utf8_parent = Utf8Path::from_path(parent)
          .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8 path"))?;
        self
          .filesystem
          .create_dir_all(utf8_parent)
          .await
          .map_err(|e| io::Error::other(e.to_string()))?;
      }
      let content = lockfile
        .to_json_string()
        .map_err(|e| io::Error::other(e.to_string()))?;
      let utf8_lockfile_path = Utf8Path::from_path(lockfile_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid UTF-8 path"))?;
      self
        .filesystem
        .write(utf8_lockfile_path, content.as_bytes())
        .await
        .map_err(|e| io::Error::other(e.to_string()))?;
    }

    Ok(())
  }
}
