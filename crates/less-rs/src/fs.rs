//! File system abstraction used for imports, `data-uri` and `image-size`.

use std::{
  collections::HashMap,
  io,
  path::{Path, PathBuf},
};

/// Why a URL import could not be fetched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UrlError {
  /// No HTTP support (build with the `http` feature or provide a
  /// [`FileSystem::fetch_url`] implementation).
  Unsupported,
  /// HTTP 404.
  NotFound,
  /// More than 5 redirects.
  TooManyRedirects,
  /// Any other failure (transport error or HTTP status text).
  Other(String),
}

/// The file system the compiler reads from.
pub trait FileSystem: Send + Sync {
  fn read(&self, path: &Path) -> io::Result<String>;
  fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>>;
  fn exists(&self, path: &Path) -> bool;
  /// Resolves symlinks for Node-style package imports. File systems without
  /// symlinks can keep the default identity implementation.
  fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
    Ok(path.to_path_buf())
  }
  /// Fetches an `http(s)://` import. The default supports nothing.
  fn fetch_url(&self, url: &str) -> Result<String, UrlError> {
    let _ = url;
    Err(UrlError::Unsupported)
  }
}

/// The real file system.
#[derive(Clone, Copy, Debug, Default)]
pub struct StdFs;

impl FileSystem for StdFs {
  fn read(&self, path: &Path) -> io::Result<String> {
    // `read_to_string` without its `fstat` size hint (measurably cheaper
    // for the many small files of an import tree)
    use std::io::Read;
    let mut buf = Vec::with_capacity(16 * 1024);
    std::fs::File::open(path)?.read_to_end(&mut buf)?;
    String::from_utf8(buf).map_err(|_| {
      io::Error::new(
        io::ErrorKind::InvalidData,
        "stream did not contain valid UTF-8",
      )
    })
  }
  fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
    std::fs::read(path)
  }
  fn exists(&self, path: &Path) -> bool {
    path.is_file()
  }
  fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
    dunce::canonicalize(path)
  }
  #[cfg(feature = "http")]
  fn fetch_url(&self, url: &str) -> Result<String, UrlError> {
    let agent: ureq::Agent = ureq::Agent::config_builder()
      .max_redirects(5)
      .http_status_as_error(false)
      .build()
      .into();
    match agent.get(url).call() {
      Ok(mut resp) => {
        let status = resp.status();
        if status.as_u16() == 404 {
          return Err(UrlError::NotFound);
        }
        if status.as_u16() >= 400 {
          return Err(UrlError::Other(
            status.canonical_reason().unwrap_or("").to_string(),
          ));
        }
        if status.as_u16() >= 300 {
          return Err(UrlError::TooManyRedirects);
        }
        resp
          .body_mut()
          .read_to_string()
          .map_err(|e| UrlError::Other(e.to_string()))
      }
      Err(ureq::Error::TooManyRedirects) => Err(UrlError::TooManyRedirects),
      Err(e) => Err(UrlError::Other(e.to_string())),
    }
  }
}

/// An in-memory file system (useful for tests, wasm and bundlers).
#[derive(Clone, Debug, Default)]
pub struct MemFs {
  files: HashMap<PathBuf, Vec<u8>>,
}

impl MemFs {
  pub fn new() -> MemFs {
    MemFs::default()
  }
  pub fn insert(&mut self, path: impl Into<PathBuf>, contents: impl Into<Vec<u8>>) -> &mut Self {
    self.files.insert(normalize(&path.into()), contents.into());
    self
  }
}

impl<P: Into<PathBuf>, C: Into<Vec<u8>>, const N: usize> From<[(P, C); N]> for MemFs {
  fn from(items: [(P, C); N]) -> MemFs {
    let mut fs = MemFs::new();
    for (p, c) in items {
      fs.insert(p, c);
    }
    fs
  }
}

fn normalize(p: &Path) -> PathBuf {
  let mut out = PathBuf::new();
  for c in p.components() {
    match c {
      std::path::Component::CurDir => {}
      std::path::Component::ParentDir => {
        out.pop();
      }
      other => out.push(other.as_os_str()),
    }
  }
  out
}

impl FileSystem for MemFs {
  fn read(&self, path: &Path) -> io::Result<String> {
    let b = self.read_bytes(path)?;
    String::from_utf8(b).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
  }
  fn read_bytes(&self, path: &Path) -> io::Result<Vec<u8>> {
    self.files.get(&normalize(path)).cloned().ok_or_else(|| {
      io::Error::new(
        io::ErrorKind::NotFound,
        format!("{} not found", path.display()),
      )
    })
  }
  fn exists(&self, path: &Path) -> bool {
    self.files.contains_key(&normalize(path))
  }
}
