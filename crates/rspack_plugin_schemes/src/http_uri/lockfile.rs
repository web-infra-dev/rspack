use std::{
  collections::BTreeMap,
  io,
  path::{Path, PathBuf},
  sync::Arc,
};

use async_trait::async_trait;
use rspack_fs::{FsResultToIoResultExt, WritableFileSystem};
use rspack_paths::Utf8Path;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockfileEntry {
  // An empty value represents the URL key itself in the stored lockfile.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub resolved: String,
  #[serde(default)]
  pub integrity: String,
  #[serde(default)]
  pub content_type: String,
}

impl LockfileEntry {
  fn omit_default_resolved(&mut self, resource: &str) {
    if self.resolved == resource {
      self.resolved.clear();
    }
  }

  pub fn has_same_content(&self, other: &Self) -> bool {
    self.resolved == other.resolved
      && self.integrity == other.integrity
      && self.content_type == other.content_type
  }
}

/// Cloning snapshots the locked metadata before asynchronous resource reads, releasing the lock.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LockfileValue {
  Content(LockfileEntry),
  Tag(LockfileTag),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LockfileTag {
  Ignore,
  NoCache,
}

impl LockfileTag {
  pub fn as_str(self) -> &'static str {
    match self {
      Self::Ignore => "ignore",
      Self::NoCache => "no-cache",
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
  #[serde(flatten)]
  entries: BTreeMap<String, LockfileValue>,
  version: u8,
}

impl Lockfile {
  pub fn new() -> Self {
    Lockfile {
      version: 1,
      entries: BTreeMap::default(),
    }
  }

  pub fn parse(content: &str) -> Result<Self, String> {
    let mut lockfile: Self = serde_json::from_str(content).map_err(|e| e.to_string())?;
    if lockfile.version != 1 {
      return Err(format!("Unsupported lockfile version {}", lockfile.version));
    }

    for (key, value) in &mut lockfile.entries {
      if let LockfileValue::Content(entry) = value {
        if entry.integrity.is_empty() {
          return Err(format!(
            "Invalid HTTP lockfile entry for {key}: missing integrity"
          ));
        }
        entry.omit_default_resolved(key);
      }
    }

    Ok(lockfile)
  }

  pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
    let mut content = Vec::new();
    let mut serializer =
      serde_json::Serializer::with_formatter(&mut content, LockfileFormatter::default());
    self.serialize(&mut serializer)?;
    Ok(String::from_utf8(content).expect("JSON serialization produces valid UTF-8"))
  }

  pub fn get_entry(&self, resource: &str) -> Option<LockfileValue> {
    let mut value = self.entries.get(resource)?.clone();
    if let LockfileValue::Content(entry) = &mut value
      && entry.resolved.is_empty()
    {
      entry.resolved = resource.to_string();
    }
    Some(value)
  }

  pub fn set_entry(&mut self, resource: &str, mut value: LockfileValue) {
    if let LockfileValue::Content(entry) = &mut value {
      entry.omit_default_resolved(resource);
    }
    self.entries.insert(resource.to_string(), value);
  }
}

/// Webpack uses indented top-level keys and inline content entries.
#[derive(Default)]
struct LockfileFormatter {
  depth: usize,
}

impl serde_json::ser::Formatter for LockfileFormatter {
  fn begin_object<W: ?Sized + io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.depth += 1;
    writer.write_all(if self.depth == 1 { b"{" } else { b"{ " })
  }

  fn end_object<W: ?Sized + io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
    self.depth -= 1;
    writer.write_all(if self.depth == 0 { b"\n}\n" } else { b" }" })
  }

  fn begin_object_key<W: ?Sized + io::Write>(
    &mut self,
    writer: &mut W,
    first: bool,
  ) -> io::Result<()> {
    if self.depth == 1 {
      writer.write_all(if first { b"\n  " } else { b",\n  " })
    } else if !first {
      writer.write_all(b", ")
    } else {
      Ok(())
    }
  }

  fn begin_object_value<W: ?Sized + io::Write>(&mut self, writer: &mut W) -> io::Result<()> {
    writer.write_all(b": ")
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

    Ok(lockfile.get_entry(url))
  }

  pub async fn store_entry(&self, url: &str, entry: LockfileValue) -> io::Result<()> {
    // Keep updates and writes under the same lock so concurrent module reads cannot lose entries.
    let mut lockfile = self.lockfile.lock().await;
    lockfile.set_entry(url, entry);

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
