use std::{collections::BTreeMap, time::SystemTime};

use rspack_fs::{Error, FileMetadata, Result};
use rspack_hash::{HashDigest, HashFunction, RspackHasher};
use rspack_paths::Utf8Path;
use serde::{Deserialize, Serialize};
use ustr::{Ustr, UstrMap};

use crate::virtual_modules::VirtualFileStore;

struct FileNode {
  ctime: u64,
  mtime: u64,
  content: Vec<u8>,
  content_hash: String,
}

impl FileNode {
  fn restore_metadata(&mut self, snapshot: &MetadataSnapshot) {
    if let MetadataSnapshot::File {
      ctime,
      mtime,
      content_hash,
    } = snapshot
    {
      if &self.content_hash == content_hash {
        self.ctime = *ctime;
        self.mtime = *mtime;
      } else {
        // Distinguish writes within the same millisecond, including a clock
        // adjustment between compilers. The hash is never used as a time.
        self.mtime = self.mtime.max(mtime.saturating_add(1));
        self.ctime = self.mtime;
      }
    }
  }
}

struct DirectoryNode {
  ctime: u64,
  mtime: u64,
  children: UstrMap<TrieNode>,
}

enum TrieNode {
  File(FileNode),
  Directory(DirectoryNode),
}

// Only metadata is persisted. A previous build must never supply virtual inputs
// that the current compiler did not create.
#[derive(Serialize, Deserialize)]
enum MetadataSnapshot {
  File {
    ctime: u64,
    mtime: u64,
    content_hash: String,
  },
  Directory {
    ctime: u64,
    mtime: u64,
    children: BTreeMap<String, MetadataSnapshot>,
  },
}

impl MetadataSnapshot {
  fn get_node(&self, path: &Utf8Path) -> Option<&Self> {
    let mut current = self;
    for part in path.iter().skip(1) {
      current = match current {
        Self::File { .. } => return None,
        Self::Directory { children, .. } => children.get(part)?,
      };
    }
    Some(current)
  }
}

impl TrieNode {
  fn new_dir(time: u64) -> Self {
    Self::Directory(DirectoryNode {
      ctime: time,
      mtime: time,
      children: UstrMap::default(),
    })
  }

  fn snapshot_metadata(&self) -> MetadataSnapshot {
    match self {
      Self::File(file) => MetadataSnapshot::File {
        ctime: file.ctime,
        mtime: file.mtime,
        content_hash: file.content_hash.clone(),
      },
      Self::Directory(dir) => MetadataSnapshot::Directory {
        ctime: dir.ctime,
        mtime: dir.mtime,
        children: dir
          .children
          .iter()
          .map(|(name, child)| (name.to_string(), child.snapshot_metadata()))
          .collect(),
      },
    }
  }

  fn restore_metadata(&mut self, snapshot: &MetadataSnapshot) {
    match (self, snapshot) {
      (Self::File(file), snapshot) => file.restore_metadata(snapshot),
      (
        Self::Directory(dir),
        MetadataSnapshot::Directory {
          ctime,
          mtime,
          children,
        },
      ) => {
        let same_entries = dir.children.len() == children.len()
          && dir.children.iter().all(|(name, child)| {
            matches!(
              (child, children.get(name.as_str())),
              (Self::File(_), Some(MetadataSnapshot::File { .. }))
                | (Self::Directory(_), Some(MetadataSnapshot::Directory { .. }))
            )
          });
        if same_entries {
          dir.ctime = *ctime;
          dir.mtime = *mtime;
        } else {
          dir.mtime = dir.mtime.max(mtime.saturating_add(1));
          dir.ctime = dir.mtime;
        }
        for (name, child) in &mut dir.children {
          if let Some(saved) = children.get(name.as_str()) {
            child.restore_metadata(saved);
          }
        }
      }
      _ => {}
    }
  }

  fn generate_file_metadata(&self) -> FileMetadata {
    match self {
      TrieNode::File(file_node) => FileMetadata {
        is_file: true,
        is_directory: false,
        is_symlink: false,
        atime_ms: file_node.mtime,
        mtime_ms: file_node.mtime,
        ctime_ms: file_node.ctime,
        size: file_node.content.len() as u64,
      },
      TrieNode::Directory(directory_node) => FileMetadata {
        is_file: false,
        is_directory: true,
        is_symlink: false,
        atime_ms: directory_node.mtime,
        mtime_ms: directory_node.mtime,
        ctime_ms: directory_node.ctime,
        size: 0,
      },
    }
  }

  fn get_node(&self, path: &Utf8Path) -> Option<&TrieNode> {
    let mut current = self;
    for part in path.iter().skip(1) {
      current = match current {
        TrieNode::File(_) => return None,
        TrieNode::Directory(directory_node) => directory_node.children.get(&Ustr::from(part))?,
      }
    }
    Some(current)
  }

  fn insert(&mut self, path: &Utf8Path, file: FileNode) -> std::result::Result<(), &str> {
    let mut parent = self;
    let parts: Vec<&str> = path.iter().skip(1).collect();
    for part in parts[..parts.len() - 1].iter() {
      parent = match parent {
        TrieNode::File(_) => return Err("ancestor directory is a file"),
        TrieNode::Directory(directory_node) => {
          let name = Ustr::from(part);
          if !directory_node.children.contains_key(&name) {
            directory_node.mtime = directory_node.mtime.saturating_add(1).max(file.mtime);
            directory_node.ctime = directory_node.mtime;
          }
          directory_node
            .children
            .entry(name)
            .or_insert_with(|| TrieNode::new_dir(file.ctime))
        }
      }
    }
    if let Some(name) = parts.last() {
      match parent {
        TrieNode::File(_) => return Err("ancestor directory is a file"),
        TrieNode::Directory(directory_node) => {
          let name = Ustr::from(name);
          if let Some(node) = directory_node.children.get_mut(&name) {
            if let TrieNode::Directory(_) = node {
              return Err("target path is a directory");
            } else {
              *node = TrieNode::File(file)
            }
          } else {
            directory_node.mtime = directory_node.mtime.saturating_add(1).max(file.mtime);
            directory_node.ctime = directory_node.mtime;
            directory_node.children.insert(name, TrieNode::File(file));
          }
        }
      }
    }
    Ok(())
  }
}

pub struct TrieVirtualFileStore {
  inner: TrieNode,
  metadata_restored: bool,
  // Dynamic inputs may first be supplied by a compilation hook. Keep metadata
  // for their first write, without materializing any previously cached input.
  restored_metadata: Option<MetadataSnapshot>,
}

impl TrieVirtualFileStore {
  pub fn new() -> Self {
    Self {
      inner: TrieNode::new_dir(0),
      metadata_restored: false,
      restored_metadata: None,
    }
  }
}

impl VirtualFileStore for TrieVirtualFileStore {
  fn write_file(&mut self, path: &Utf8Path, content: Vec<u8>) {
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .expect("failed to get now")
      .as_millis() as u64;
    let mtime = if let Some(TrieNode::File(previous)) = self.inner.get_node(path) {
      if previous.content == content {
        return;
      }
      now.max(previous.mtime.saturating_add(1))
    } else {
      now
    };
    let mut hasher = RspackHasher::new(&HashFunction::SHA256);
    hasher.write(&content);
    let mut file = FileNode {
      ctime: mtime,
      mtime,
      content,
      content_hash: hasher.digest(&HashDigest::Hex).encoded().to_owned(),
    };
    if self.inner.get_node(path).is_none()
      && let Some(snapshot) = self
        .restored_metadata
        .as_ref()
        .and_then(|saved| saved.get_node(path))
    {
      file.restore_metadata(snapshot);
    }
    let _ = self.inner.insert(path, file);
  }

  fn snapshot_metadata(&self) -> Vec<u8> {
    serde_json::to_vec(&self.inner.snapshot_metadata())
      .expect("virtual filesystem metadata should be serializable")
  }

  fn restore_metadata(&mut self, snapshot: Option<&[u8]>) -> Result<()> {
    if self.metadata_restored {
      return Ok(());
    }
    if let Some(snapshot) = snapshot {
      let snapshot = serde_json::from_slice(snapshot)
        .map_err(|error| Error::new(std::io::ErrorKind::InvalidData, &error.to_string()))?;
      self.inner.restore_metadata(&snapshot);
      self.restored_metadata = Some(snapshot);
    }
    self.metadata_restored = true;
    Ok(())
  }

  fn get_file_content(&self, path: &Utf8Path) -> Option<&Vec<u8>> {
    match self.inner.get_node(path)? {
      TrieNode::File(file_node) => Some(&file_node.content),
      TrieNode::Directory(_) => None,
    }
  }

  fn get_file_metadata(&self, path: &Utf8Path) -> Option<FileMetadata> {
    self
      .inner
      .get_node(path)
      .map(|node| node.generate_file_metadata())
  }

  fn contains(&self, path: &Utf8Path) -> bool {
    self.inner.get_node(path).is_some()
  }

  fn read_dir(&self, path: &Utf8Path) -> Option<Vec<String>> {
    self.inner.get_node(path).and_then(|node| match node {
      TrieNode::File(_) => None,
      TrieNode::Directory(directory_node) => Some(
        directory_node
          .children
          .keys()
          .map(|s| s.to_string())
          .collect(),
      ),
    })
  }
}
