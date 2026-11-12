//! The equivalent of webpack's memfs fixture, with deterministic mtimes and fault injection.
//! Kept local to these tests because rspack_fs::MemoryFileSystem has no symlink/utimes support.

use std::{
  collections::BTreeMap,
  io::{Error, ErrorKind},
  sync::Mutex,
};

use cow_utils::CowUtils;
use rspack_fs::{FileMetadata, FilePermissions, ReadableFileSystem, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Operation {
  Read,
  Stat,
  Lstat,
  ReadLink,
  Canonicalize,
  ReadDir,
}

#[derive(Debug)]
enum Node {
  Directory,
  File(Vec<u8>),
  Symlink(String),
  Special(Vec<u8>),
}

#[derive(Debug)]
struct Entry {
  node: Node,
  mtime: u64,
}

impl Entry {
  fn metadata(&self) -> FileMetadata {
    FileMetadata {
      is_file: matches!(self.node, Node::File(_)),
      is_directory: matches!(self.node, Node::Directory),
      is_symlink: matches!(self.node, Node::Symlink(_)),
      atime_ms: self.mtime,
      mtime_ms: self.mtime,
      ctime_ms: self.mtime,
      size: match &self.node {
        Node::File(data) | Node::Special(data) => data.len() as u64,
        Node::Symlink(target) => target.len() as u64,
        Node::Directory => 0,
      },
    }
  }
}

#[derive(Debug)]
struct State {
  entries: BTreeMap<String, Entry>,
  calls: BTreeMap<(Operation, String), usize>,
  errors: BTreeMap<(Operation, String), (ErrorKind, usize)>,
  clock: u64,
}

fn normalize(path: &str) -> String {
  let path = path.cow_replace('\\', "/");
  let mut parts = Vec::new();
  for part in path.split('/') {
    match part {
      "" | "." => {}
      ".." => {
        parts.pop();
      }
      _ => parts.push(part),
    }
  }
  format!("/{}", parts.join("/"))
}

fn error(kind: ErrorKind, path: &str) -> rspack_fs::Error {
  Error::new(kind, format!("{kind:?}: {path}")).into()
}

impl State {
  fn record(&mut self, operation: Operation, path: &str) -> Result<()> {
    let key = (operation, normalize(path));
    *self.calls.entry(key.clone()).or_default() += 1;
    if let Some((kind, remaining)) = self.errors.get_mut(&key)
      && *remaining > 0
    {
      *remaining -= 1;
      return Err(error(*kind, path));
    }
    Ok(())
  }

  fn resolve(&self, path: &str, follow_final: bool) -> Result<String> {
    let mut path = normalize(path);
    // Like a real filesystem, reject a chain that cannot be resolved. Directory
    // graph cycles are different: each individual symlink still resolves normally.
    for _ in 0..40 {
      let parts: Vec<_> = path.split('/').filter(|part| !part.is_empty()).collect();
      let mut prefix = String::new();
      let mut redirected = None;
      for (index, part) in parts.iter().enumerate() {
        prefix.push('/');
        prefix.push_str(part);
        let entry = self
          .entries
          .get(&prefix)
          .ok_or_else(|| error(ErrorKind::NotFound, &prefix))?;
        let last = index + 1 == parts.len();
        if let Node::Symlink(target) = &entry.node {
          if !last || follow_final {
            let parent = prefix.rsplit_once('/').expect("absolute path").0;
            let target = if target.starts_with('/') {
              target.clone()
            } else {
              format!("{parent}/{target}")
            };
            redirected = Some(normalize(&format!(
              "{target}/{}",
              parts[index + 1..].join("/")
            )));
            break;
          }
        } else if !last && !matches!(entry.node, Node::Directory) {
          return Err(error(ErrorKind::NotADirectory, &prefix));
        }
      }
      match redirected {
        Some(target) => path = target,
        None => return Ok(path),
      }
    }
    Err(error(ErrorKind::Other, "too many symbolic links"))
  }
}

#[derive(Debug)]
pub(super) struct TestFileSystem(Mutex<State>);

impl TestFileSystem {
  pub(super) fn new() -> Self {
    Self(Mutex::new(State {
      entries: BTreeMap::from([(
        "/".into(),
        Entry {
          node: Node::Directory,
          mtime: 10_000,
        },
      )]),
      calls: BTreeMap::new(),
      errors: BTreeMap::new(),
      clock: 10_000,
    }))
  }

  fn insert(&self, path: &str, node: Node) {
    let path = normalize(path);
    let mut state = self.0.lock().expect("filesystem lock");
    // Mutations are separated by more than the maximum filesystem accuracy.
    // No wall-clock sleeps are needed to distinguish two writes.
    state.clock += 10_000;
    let mtime = state.clock;
    state.entries.insert(path, Entry { node, mtime });
  }

  pub(super) fn mkdir(&self, path: &str) {
    let path = normalize(path);
    let mut prefix = String::new();
    for part in path.split('/').filter(|part| !part.is_empty()) {
      prefix.push('/');
      prefix.push_str(part);
      if !self
        .0
        .lock()
        .expect("filesystem lock")
        .entries
        .contains_key(&prefix)
      {
        self.insert(&prefix, Node::Directory);
      }
    }
  }

  pub(super) fn write(&self, path: &str, content: impl AsRef<[u8]>) {
    let parent = path.rsplit_once('/').expect("absolute file path").0;
    self.mkdir(parent);
    self.insert(path, Node::File(content.as_ref().to_vec()));
  }

  pub(super) fn symlink(&self, target: &str, path: &str) {
    let parent = path.rsplit_once('/').expect("absolute symlink path").0;
    self.mkdir(parent);
    self.insert(path, Node::Symlink(target.into()));
  }

  pub(super) fn special(&self, path: &str) {
    self.write(path, "");
    self.insert(path, Node::Special(Vec::new()));
  }

  pub(super) fn advance_timestamps(&self, delta: u64) {
    let mut state = self.0.lock().expect("filesystem lock");
    state.clock += delta;
    for entry in state.entries.values_mut() {
      entry.mtime += delta;
    }
  }

  pub(super) fn fail(&self, operation: Operation, path: &str, kind: ErrorKind, times: usize) {
    self
      .0
      .lock()
      .expect("filesystem lock")
      .errors
      .insert((operation, normalize(path)), (kind, times));
  }

  pub(super) fn calls(&self) -> BTreeMap<(Operation, String), usize> {
    self.0.lock().expect("filesystem lock").calls.clone()
  }

  fn stat(&self, path: &Utf8Path, follow_final: bool) -> Result<FileMetadata> {
    let mut state = self.0.lock().expect("filesystem lock");
    state.record(
      if follow_final {
        Operation::Stat
      } else {
        Operation::Lstat
      },
      path.as_str(),
    )?;
    let resolved = state.resolve(path.as_str(), follow_final)?;
    Ok(
      state
        .entries
        .get(&resolved)
        .expect("resolved entry")
        .metadata(),
    )
  }
}

#[async_trait::async_trait]
impl ReadableFileSystem for TestFileSystem {
  async fn read(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    tokio::task::yield_now().await;
    self.read_sync(path)
  }

  fn read_sync(&self, path: &Utf8Path) -> Result<Vec<u8>> {
    let mut state = self.0.lock().expect("filesystem lock");
    state.record(Operation::Read, path.as_str())?;
    let resolved = state.resolve(path.as_str(), true)?;
    match &state.entries.get(&resolved).expect("resolved entry").node {
      Node::File(data) | Node::Special(data) => Ok(data.clone()),
      Node::Directory => Err(error(ErrorKind::IsADirectory, path.as_str())),
      Node::Symlink(_) => unreachable!("resolve follows the final link"),
    }
  }

  async fn metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    tokio::task::yield_now().await;
    self.metadata_sync(path)
  }

  fn metadata_sync(&self, path: &Utf8Path) -> Result<FileMetadata> {
    self.stat(path, true)
  }

  async fn symlink_metadata(&self, path: &Utf8Path) -> Result<FileMetadata> {
    tokio::task::yield_now().await;
    self.stat(path, false)
  }

  async fn read_link(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    tokio::task::yield_now().await;
    let mut state = self.0.lock().expect("filesystem lock");
    state.record(Operation::ReadLink, path.as_str())?;
    let resolved = state.resolve(path.as_str(), false)?;
    match &state.entries.get(&resolved).expect("resolved entry").node {
      Node::Symlink(target) => Ok(Utf8PathBuf::from(target)),
      _ => Err(error(ErrorKind::InvalidInput, path.as_str())),
    }
  }

  async fn canonicalize(&self, path: &Utf8Path) -> Result<Utf8PathBuf> {
    tokio::task::yield_now().await;
    let mut state = self.0.lock().expect("filesystem lock");
    state.record(Operation::Canonicalize, path.as_str())?;
    state.resolve(path.as_str(), true).map(Utf8PathBuf::from)
  }

  async fn read_dir(&self, path: &Utf8Path) -> Result<Vec<String>> {
    tokio::task::yield_now().await;
    self.read_dir_sync(path)
  }

  fn read_dir_sync(&self, path: &Utf8Path) -> Result<Vec<String>> {
    let mut state = self.0.lock().expect("filesystem lock");
    state.record(Operation::ReadDir, path.as_str())?;
    let resolved = state.resolve(path.as_str(), true)?;
    if !matches!(
      state.entries.get(&resolved).expect("resolved entry").node,
      Node::Directory
    ) {
      return Err(error(ErrorKind::NotADirectory, path.as_str()));
    }
    let prefix = format!("{}/", resolved.trim_end_matches('/'));
    Ok(
      state
        .entries
        .keys()
        .filter_map(|entry| {
          let rest = entry.strip_prefix(&prefix)?;
          (!rest.is_empty() && !rest.contains('/')).then(|| rest.to_string())
        })
        .collect(),
    )
  }

  async fn permissions(&self, _path: &Utf8Path) -> Result<Option<FilePermissions>> {
    Ok(None)
  }
}
