//! The equivalent of webpack's memfs fixture, with deterministic mtimes and fault injection.
//! MemoryFS owns the entries and timestamps; only unsupported node kinds need side metadata.

use std::{
  collections::BTreeMap,
  io::{Error, ErrorKind, Read, Write},
  sync::Mutex,
  time::{Duration, SystemTime},
};

use cow_utils::CowUtils;
use rspack_fs::{FileMetadata, FilePermissions, ReadableFileSystem, Result};
use rspack_paths::{Utf8Path, Utf8PathBuf};
use vfs::{FileSystem, MemoryFS, VfsError, VfsFileType, error::VfsErrorKind};

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
  Symlink(String),
  Special,
}

#[derive(Debug)]
struct State {
  fs: MemoryFS,
  // These nodes have ordinary backing files so their existence and timestamps
  // are owned by MemoryFS too.
  nodes: BTreeMap<String, Node>,
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

fn backing_path(path: &str) -> String {
  // Keep the fixture root replaceable like any other entry. MemoryFS's own
  // root (the empty path) cannot be replaced with create_file.
  format!("/fixture{}", path.trim_end_matches('/'))
}

fn error(kind: ErrorKind, path: &str) -> rspack_fs::Error {
  Error::new(kind, format!("{kind:?}: {path}")).into()
}

fn vfs_error(err: VfsError, path: &str) -> rspack_fs::Error {
  let kind = match err.kind() {
    VfsErrorKind::FileNotFound => ErrorKind::NotFound,
    VfsErrorKind::IoError(err) => err.kind(),
    _ => ErrorKind::Other,
  };
  error(kind, path)
}

fn milliseconds(time: Option<SystemTime>) -> u64 {
  time
    .expect("fixture timestamp")
    .duration_since(SystemTime::UNIX_EPOCH)
    .expect("fixture timestamp after epoch")
    .as_millis() as u64
}

impl State {
  fn set_timestamps(&self, path: &str, mtime: u64) {
    let path = backing_path(path);
    let time = SystemTime::UNIX_EPOCH + Duration::from_millis(mtime);
    self.fs.set_creation_time(&path, time).expect("set ctime");
    self
      .fs
      .set_modification_time(&path, time)
      .expect("set mtime");
    self.fs.set_access_time(&path, time).expect("set atime");
  }

  fn touch(&mut self, path: &str) {
    // Mutations are separated by more than the maximum filesystem accuracy.
    // No wall-clock sleeps are needed to distinguish two writes.
    self.clock += 10_000;
    self.set_timestamps(path, self.clock);
  }

  fn metadata(&self, path: &str) -> Result<FileMetadata> {
    let metadata = self
      .fs
      .metadata(&backing_path(path))
      .map_err(|err| vfs_error(err, path))?;
    let node = self.nodes.get(path);
    Ok(FileMetadata {
      is_file: node.is_none() && metadata.file_type == VfsFileType::File,
      is_directory: node.is_none() && metadata.file_type == VfsFileType::Directory,
      is_symlink: matches!(node, Some(Node::Symlink(_))),
      atime_ms: milliseconds(metadata.accessed),
      mtime_ms: milliseconds(metadata.modified),
      ctime_ms: milliseconds(metadata.created),
      size: match node {
        Some(Node::Symlink(target)) => target.len() as u64,
        _ => metadata.len,
      },
    })
  }

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
        let metadata = self.metadata(&prefix)?;
        let last = index + 1 == parts.len();
        if let Some(Node::Symlink(target)) = self.nodes.get(&prefix) {
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
        } else if !last && !metadata.is_directory {
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
    let state = State {
      fs: MemoryFS::new(),
      nodes: BTreeMap::new(),
      calls: BTreeMap::new(),
      errors: BTreeMap::new(),
      clock: 10_000,
    };
    state
      .fs
      .create_dir(&backing_path("/"))
      .expect("create fixture root");
    state.set_timestamps("/", state.clock);
    Self(Mutex::new(state))
  }

  pub(super) fn mkdir(&self, path: &str) {
    let path = normalize(path);
    let mut prefix = String::new();
    for part in path.split('/').filter(|part| !part.is_empty()) {
      prefix.push('/');
      prefix.push_str(part);
      let mut state = self.0.lock().expect("filesystem lock");
      let backing = backing_path(&prefix);
      if !state.fs.exists(&backing).expect("check directory") {
        state.fs.create_dir(&backing).expect("create directory");
        state.touch(&prefix);
      }
    }
  }

  fn insert_file(&self, path: &str, content: &[u8], node: Option<Node>) {
    let path = normalize(path);
    let mut state = self.0.lock().expect("filesystem lock");
    {
      let mut file = state
        .fs
        .create_file(&backing_path(&path))
        .expect("create file");
      file.write_all(content).expect("write file");
    }
    // MemoryFS flushes on drop and stamps the write with wall-clock time.
    match node {
      Some(node) => {
        state.nodes.insert(path.clone(), node);
      }
      None => {
        state.nodes.remove(&path);
      }
    }
    state.touch(&path);
  }

  pub(super) fn write(&self, path: &str, content: impl AsRef<[u8]>) {
    let parent = path.rsplit_once('/').expect("absolute file path").0;
    self.mkdir(parent);
    self.insert_file(path, content.as_ref(), None);
  }

  pub(super) fn symlink(&self, target: &str, path: &str) {
    let parent = path.rsplit_once('/').expect("absolute symlink path").0;
    self.mkdir(parent);
    self.insert_file(path, &[], Some(Node::Symlink(target.into())));
  }

  pub(super) fn special(&self, path: &str) {
    self.write(path, "");
    let path = normalize(path);
    let mut state = self.0.lock().expect("filesystem lock");
    state.nodes.insert(path.clone(), Node::Special);
    state.touch(&path);
  }

  pub(super) fn advance_timestamps(&self, delta: u64) {
    let mut state = self.0.lock().expect("filesystem lock");
    state.clock += delta;
    let mut paths = vec!["/".to_string()];
    while let Some(path) = paths.pop() {
      let backing = backing_path(&path);
      let metadata = state.fs.metadata(&backing).expect("fixture metadata");
      state.set_timestamps(&path, milliseconds(metadata.modified) + delta);
      // Enumerate backing entries without resolving symlinks, including any
      // children retained when an existing directory was overwritten.
      for child in state.fs.read_dir(&backing).expect("fixture entries") {
        paths.push(format!("{}/{child}", path.trim_end_matches('/')));
      }
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
    state.metadata(&resolved)
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
    let metadata = state.metadata(&resolved)?;
    if metadata.is_directory {
      return Err(error(ErrorKind::IsADirectory, path.as_str()));
    }
    assert!(!metadata.is_symlink, "resolve follows the final link");
    let backing = backing_path(&resolved);
    let mut file = state
      .fs
      .open_file(&backing)
      .map_err(|err| vfs_error(err, path.as_str()))?;
    // Opening a MemoryFS file updates atime; reads in this fixture do not.
    state
      .fs
      .set_access_time(
        &backing,
        SystemTime::UNIX_EPOCH + Duration::from_millis(metadata.atime_ms),
      )
      .expect("restore atime");
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    Ok(content)
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
    match state.nodes.get(&resolved) {
      Some(Node::Symlink(target)) => Ok(Utf8PathBuf::from(target)),
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
    if !state.metadata(&resolved)?.is_directory {
      return Err(error(ErrorKind::NotADirectory, path.as_str()));
    }
    let mut entries: Vec<_> = state
      .fs
      .read_dir(&backing_path(&resolved))
      .map_err(|err| vfs_error(err, path.as_str()))?
      .collect();
    entries.sort();
    Ok(entries)
  }

  async fn permissions(&self, _path: &Utf8Path) -> Result<Option<FilePermissions>> {
    Ok(None)
  }
}
