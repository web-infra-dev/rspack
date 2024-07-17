use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockfileEntry {
  pub resolved: String,
  pub integrity: String,
  pub content_type: String,
  pub valid_until: u64,
  pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Lockfile {
  version: u8,
  entries: HashMap<String, LockfileEntry>,
}

impl Lockfile {
  pub fn new() -> Self {
    Lockfile {
      version: 1,
      entries: HashMap::new(),
    }
  }

  pub fn parse(content: &str) -> Result<Self, String> {
    let data: serde_json::Value = serde_json::from_str(content).map_err(|e| e.to_string())?;

    let version = data.get("version").and_then(|v| v.as_u64()).unwrap_or(1);

    if version != 1 {
      return Err(format!("Unsupported lockfile version {}", version));
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

  pub fn to_json_string(&self) -> String {
    let json = serde_json::json!({
        "version": self.version,
        "entries": self.entries
    });
    serde_json::to_string_pretty(&json).unwrap()
  }

  pub fn get_entry(&self, resource: &str) -> Option<&LockfileEntry> {
    self.entries.get(resource)
  }

  pub fn entries_mut(&mut self) -> &mut HashMap<String, LockfileEntry> {
    &mut self.entries
  }
}

#[async_trait]
pub trait LockfileAsync {
  async fn read_from_file_async<P: AsRef<Path> + Send>(path: P) -> io::Result<Lockfile>;
  async fn write_to_file_async<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<()>;
}

#[async_trait]
impl LockfileAsync for Lockfile {
  async fn read_from_file_async<P: AsRef<Path> + Send>(path: P) -> io::Result<Lockfile> {
    let content = async_fs::read_to_string(path).await?;
    Lockfile::parse(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
  }

  async fn write_to_file_async<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<()> {
    let content = self.to_json_string();
    async_fs::write(path, content).await
  }
}

#[derive(Debug)]
pub struct LockfileCache {
  lockfile: Arc<Mutex<Lockfile>>,
  lockfile_path: Option<PathBuf>,
}

impl LockfileCache {
  pub fn new(lockfile_path: Option<PathBuf>) -> Self {
    LockfileCache {
      lockfile: Arc::new(Mutex::new(Lockfile::new())),
      lockfile_path,
    }
  }

  pub async fn get_lockfile(&self) -> io::Result<Arc<Mutex<Lockfile>>> {
    let mut lockfile = self.lockfile.lock().await;

    if let Some(lockfile_path) = &self.lockfile_path {
      if lockfile_path.exists() {
        match Lockfile::read_from_file_async(lockfile_path).await {
          Ok(lf) => {
            *lockfile = lf;
          }
          Err(_) => {}
        }
      }
    }

    Ok(self.lockfile.clone())
  }

  pub async fn save_lockfile(&self) -> io::Result<()> {
    let lockfile = self.lockfile.lock().await;

    if let Some(lockfile_path) = &self.lockfile_path {
      if let Some(parent) = lockfile_path.parent() {
        async_fs::create_dir_all(parent).await?;
      }
      let content = lockfile.to_json_string();
      async_fs::write(lockfile_path, &content).await?;
    }

    Ok(())
  }
}
impl Default for LockfileCache {
  fn default() -> Self {
    LockfileCache {
      lockfile: Arc::new(Mutex::new(Lockfile::new())),
      lockfile_path: None,
    }
  }
}
