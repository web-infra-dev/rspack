use std::{
  collections::HashMap,
  io,
  path::{Path, PathBuf},
  sync::Arc,
};

use async_trait::async_trait;
use rspack_fs::WritableFileSystem;
use rspack_paths::Utf8Path;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct LockfileEntry {
  pub resolved: String,
  pub integrity: String,
  pub content_type: String,
  pub valid_until: u64,
  pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub(super) struct Lockfile {
  version: u8,
  entries: HashMap<String, LockfileEntry>,
}

impl Lockfile {
  pub(super) fn new() -> Self {
    Lockfile {
      version: 1,
      entries: HashMap::new(),
    }
  }

  pub(super) fn parse(content: &str) -> Result<Self, String> {
    let data: serde_json::Value = serde_json::from_str(content).map_err(|e| e.to_string())?;

    let version = data.get("version").and_then(|v| v.as_u64()).unwrap_or(1);

    if version != 1 {
      return Err(format!("Unsupported lockfile version {version}"));
    }

    let mut lockfile = Lockfile::new();

    if let Some(entries) = data.get("entries").and_then(|e| e.as_object()) {
      for (key, value) in entries {
        let entry = if value.is_string() {
          LockfileEntry {
            resolved: key.clone(),
            integrity: value.as_str().expect("Expected string").to_string(),
            content_type: String::new(),
            valid_until: 0,
            etag: None,
          }
        } else {
          LockfileEntry {
            resolved: key.clone(),
            integrity: value
              .get("integrity")
              .and_then(|v| v.as_str())
              .unwrap_or("")
              .to_string(),
            content_type: value
              .get("content_type")
              .and_then(|v| v.as_str())
              .unwrap_or("")
              .to_string(),
            valid_until: value
              .get("valid_until")
              .and_then(|v| v.as_u64())
              .unwrap_or(0),
            etag: value.get("etag").and_then(|v| v.as_str()).map(String::from),
          }
        };
        lockfile.entries.insert(key.clone(), entry);
      }
    }

    Ok(lockfile)
  }

  pub(super) fn to_json_string(&self) -> Result<String, serde_json::Error> {
    let json = serde_json::json!({
        "version": self.version,
        "entries": self.entries
    });
    serde_json::to_string_pretty(&json)
  }

  pub(super) fn get_entry(&self, resource: &str) -> Option<&LockfileEntry> {
    self.entries.get(resource)
  }

  pub(super) fn entries_mut(&mut self) -> &mut HashMap<String, LockfileEntry> {
    &mut self.entries
  }
}

#[async_trait]
#[allow(dead_code)]
pub(super) trait LockfileAsync {
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
    let content = filesystem
      .read_file(utf8_path)
      .await
      .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{e:?}")))?;
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
pub(super) struct LockfileCache {
  lockfile: Arc<Mutex<Lockfile>>,
  lockfile_path: Option<PathBuf>,
  filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
}

impl LockfileCache {
  pub(super) fn new(
    lockfile_path: Option<PathBuf>,
    filesystem: Arc<dyn WritableFileSystem + Send + Sync>,
  ) -> Self {
    LockfileCache {
      lockfile: Arc::new(Mutex::new(Lockfile::new())),
      lockfile_path,
      filesystem,
    }
  }

  pub(super) async fn get_lockfile(&self) -> io::Result<Arc<Mutex<Lockfile>>> {
    let mut lockfile = self.lockfile.lock().await;

    if let Some(lockfile_path) = &self.lockfile_path {
      match Lockfile::read_from_file_async(lockfile_path, self.filesystem.as_ref()).await {
        Ok(lf) => {
          *lockfile = lf;
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
          // File doesn't exist, use the default empty lockfile
        }
        Err(_e) => {
          // Error reading lockfile
        }
      }
    }

    Ok(self.lockfile.clone())
  }

  pub(super) async fn save_lockfile(&self) -> io::Result<()> {
    let lockfile = self.lockfile.lock().await;

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
