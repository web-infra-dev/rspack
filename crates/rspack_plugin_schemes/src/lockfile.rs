use std::collections::HashMap;
use std::fs;
use std::io::{self, Read, Write};
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
    for (key, value) in data.as_object().unwrap() {
      if key == "version" {
        continue;
      }
      let entry = if value.is_string() {
        LockfileEntry {
          resolved: key.clone(),
          integrity: value.as_str().unwrap().to_string(),
          content_type: String::new(),
        }
      } else {
        LockfileEntry {
          resolved: key.clone(),
          integrity: value["integrity"].as_str().unwrap().to_string(),
          content_type: value["contentType"].as_str().unwrap().to_string(),
        }
      };
      lockfile.entries.insert(key.clone(), entry);
    }
    Ok(lockfile)
  }

  pub fn to_string(&self) -> String {
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
    let content = self.to_string();
    async_fs::write(path, content).await
  }
}

pub struct LockfileCache {
  lockfile: Mutex<Option<Lockfile>>,
  snapshot: Mutex<Option<String>>, // Placeholder for the actual snapshot type
}

impl LockfileCache {
  pub fn new() -> Self {
    LockfileCache {
      lockfile: Mutex::new(None),
      snapshot: Mutex::new(None),
    }
  }

  pub async fn get_lockfile<P: AsRef<Path> + Send>(&self, path: P) -> io::Result<Lockfile> {
    let mut lockfile_guard = self.lockfile.lock().await;
    let mut snapshot_guard = self.snapshot.lock().await;

    if let Some(lockfile) = &*lockfile_guard {
      // Check snapshot validity here
      // If valid, return the cached lockfile
      return Ok((*lockfile).clone());
    }

    // Read lockfile from file
    let lockfile = Lockfile::read_from_file_async(path.as_ref()).await?;
    // Create snapshot here and store it in snapshot_guard

    *lockfile_guard = Some(lockfile.clone());
    // Store the snapshot in snapshot_guard

    Ok(lockfile)
  }
}
