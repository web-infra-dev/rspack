use std::{collections::HashMap, time::SystemTime};

use rspack_fs::FileMetadata;
use rspack_paths::Utf8Path;
use ustr::Ustr;

use crate::virtual_modules::VirtualFileStore;

struct FileNode {
  ctime: u64,
  mtime: u64,
  content: Vec<u8>,
}

struct DirectoryNode {
  ctime: u64,
  mtime: u64,
  children: HashMap<Ustr, TrieNode>,
}

enum TrieNode {
  File(FileNode),
  Directory(DirectoryNode),
}

impl TrieNode {
  fn new_dir(time: u64) -> Self {
    Self::Directory(DirectoryNode {
      ctime: time,
      mtime: time,
      children: HashMap::default(),
    })
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

  fn insert(&mut self, path: &Utf8Path, file: FileNode) -> Result<(), &str> {
    let mut parent = self;

    let parts: Vec<&str> = path.iter().skip(1).collect();

    for part in parts[..parts.len() - 1].iter() {
      parent = match parent {
        TrieNode::File(_) => return Err("ancestor directory is a file"),
        TrieNode::Directory(directory_node) => directory_node
          .children
          .entry(Ustr::from(part))
          .or_insert_with(|| TrieNode::new_dir(file.ctime)),
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
}

impl TrieVirtualFileStore {
  pub fn new() -> Self {
    Self {
      inner: TrieNode::new_dir(0),
    }
  }
}

impl VirtualFileStore for TrieVirtualFileStore {
  fn write_file(&mut self, path: &Utf8Path, content: Vec<u8>) {
    let now = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .expect("failed to get now")
      .as_millis() as u64;
    let file = FileNode {
      ctime: now,
      mtime: now,
      content,
    };
    let _ = self.inner.insert(path, file);
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
