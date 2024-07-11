use std::collections::HashMap;
use std::io::{self};
use std::path::Path;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs as async_fs;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockfileEntry {
  pub resolved: String,
  pub integrity: String,
  pub content_type: String,
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
    if data["version"] != 1 {
      return Err(format!("Unsupported lockfile version {}", data["version"]));
    }
    let mut lockfile = Lockfile::new();
    for (key, value) in data.as_object().expect("Expected JSON object") {
      if key == "version" {
        continue;
      }
      let entry = if value.is_string() {
        LockfileEntry {
          resolved: key.clone(),
          integrity: value.as_str().expect("Expected string").to_string(),
          content_type: String::new(),
        }
      } else {
        LockfileEntry {
          resolved: key.clone(),
          integrity: value["integrity"]
            .as_str()
            .expect("Expected integrity string")
            .to_string(),
          content_type: value["contentType"]
            .as_str()
            .expect("Expected contentType string")
            .to_string(),
        }
      };
      lockfile.entries.insert(key.clone(), entry);
    }
    Ok(lockfile)
  }

  pub fn to_json_string(&self) -> String {
    let mut entries: Vec<_> = self.entries.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    let mut str = String::from("{\n");
    for (key, entry) in entries {
      if entry.content_type.is_empty() {
        str.push_str(&format!("  \"{}\": \"{}\",\n", key, entry.integrity));
      } else {
        str.push_str(&format!(
          "  \"{}\": {{ \"resolved\": \"{}\", \"integrity\": \"{}\", \"contentType\": \"{}\" }},\n",
          key, entry.resolved, entry.integrity, entry.content_type
        ));
      }
    }
    str.push_str(&format!("  \"version\": {}\n}}\n", self.version));
    str
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
  lockfile: Mutex<Option<Lockfile>>,
  snapshot: Mutex<Option<String>>,
  lockfile_location: Option<String>,
}

impl LockfileCache {
  pub fn new(lockfile_location: Option<String>) -> Self {
    LockfileCache {
      lockfile: Mutex::new(None),
      snapshot: Mutex::new(None),
      lockfile_location,
    }
  }

  pub async fn get_lockfile<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Lockfile> {
    let mut lockfile_guard = self.lockfile.lock().await;
    let mut snapshot_guard = self.snapshot.lock().await;

    if let Some(lockfile) = &*lockfile_guard {
      return Ok((*lockfile).clone());
    }

    let lockfile_path = if let Some(snapshot) = &*snapshot_guard {
      path.as_ref().join(snapshot)
    } else {
      path.as_ref().join("lockfile.json")
    };

    let lockfile = if lockfile_path.exists() {
      Lockfile::read_from_file_async(&lockfile_path).await?
    } else {
      Lockfile::new()
    };

    *lockfile_guard = Some(lockfile.clone());
    *snapshot_guard = Some(lockfile_path.to_string_lossy().to_string());

    Ok(lockfile)
  }

  pub async fn save_lockfile(&self) -> io::Result<()> {
    let lockfile_guard = self.lockfile.lock().await;
    let snapshot_guard = self.snapshot.lock().await;
    if let Some(lockfile) = &*lockfile_guard {
      if let Some(lockfile_location) = &self.lockfile_location {
        if let Some(snapshot) = &*snapshot_guard {
          let path = Path::new(lockfile_location).join(snapshot);
          lockfile.write_to_file_async(path).await?;
        }
      }
    }
    Ok(())
  }
}

impl Default for LockfileCache {
  fn default() -> Self {
    LockfileCache {
      lockfile: Mutex::new(Some(Lockfile::new())),
      snapshot: Mutex::new(Some("lockfile.json".to_string())),
      lockfile_location: None,
    }
  }
}
