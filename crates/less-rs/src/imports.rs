//! Import resolution and processing: ports of `import-manager.js`,
//! `less-node/file-manager.js`, `visitors/import-visitor.js` and
//! `visitors/import-sequencer.js` (synchronous mode).

use std::{collections::VecDeque, path::Path};

use cow_utils::CowUtils;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::{
  error::{LResult, LessError},
  eval::EvalCtx,
  options::RewriteUrls,
  session::{FileInfo, Preparsed, ReadCache, Session},
  syntax::parser::Parser,
  tree::*,
};

// ---------------------------------------------------------------------------
// path helpers (node's `path` module, posix flavour)
// ---------------------------------------------------------------------------

/// `path.normalize` (posix).
pub(crate) fn path_normalize(p: &str) -> String {
  if p.is_empty() {
    return ".".to_string();
  }
  // already normal: no empty, `.` or `..` segments
  if !p.contains("//")
    && !p.contains("/./")
    && !p.contains("/../")
    && !p.starts_with("./")
    && !p.starts_with("../")
    && !p.ends_with("/.")
    && !p.ends_with("/..")
    && p != "."
    && p != ".."
  {
    return p.to_string();
  }
  let absolute = p.starts_with('/');
  let trailing = p.ends_with('/');
  let mut out: smallvec::SmallVec<[&str; 16]> = smallvec::SmallVec::new();
  for seg in p.split('/') {
    match seg {
      "" | "." => {}
      ".." => {
        if !out.is_empty() && *out.last().expect("path segments are nonempty") != ".." {
          out.pop();
        } else if !absolute {
          out.push("..");
        }
      }
      s => out.push(s),
    }
  }
  let mut s = out.join("/");
  if absolute {
    s.insert(0, '/');
  }
  if s.is_empty() {
    s = if absolute { "/".into() } else { ".".into() };
  }
  if trailing && !s.ends_with('/') {
    s.push('/');
  }
  s
}

/// `path.join(a, b)` (posix).
pub(crate) fn path_join(a: &str, b: &str) -> String {
  let joined = match (a.is_empty(), b.is_empty()) {
    (true, true) => return ".".into(),
    (true, false) => b.to_string(),
    (false, true) => a.to_string(),
    (false, false) => format!("{}/{}", a, b),
  };
  path_normalize(&joined)
}

/// `AbstractFileManager.getPath`
pub(crate) fn get_path(filename: &str) -> String {
  let mut f = filename;
  if let Some(j) = f.rfind('?')
    && j > 0
  {
    f = &f[..j];
  }
  let j = f.rfind('/').or_else(|| f.rfind('\\'));
  match j {
    Some(j) => f[..j + 1].to_string(),
    None => String::new(),
  }
}

/// `isPathAbsolute`: `/^(?:[a-z-]+:|\/|\\|#)/i`
pub(crate) fn is_path_absolute(f: &str) -> bool {
  let b = f.as_bytes();
  if matches!(b.first(), Some(b'/' | b'\\' | b'#')) {
    return true;
  }
  let n = b
    .iter()
    .take_while(|c| c.is_ascii_alphabetic() || **c == b'-')
    .count();
  n > 0 && b.get(n) == Some(&b':')
}

/// `tryAppendExtension`: `/(\.[a-z]*$)|([?;].*)$/`
fn try_append_extension(path: &str, ext: &str) -> String {
  let b = path.as_bytes();
  // `\.[a-z]*$`
  if let Some(dot) = path.rfind('.')
    && b[dot + 1..].iter().all(|c| c.is_ascii_lowercase())
  {
    return path.to_string();
  }
  // `[?;].*$`
  if path.contains('?') || path.contains(';') {
    return path.to_string();
  }
  format!("{}{}", path, ext)
}

struct UrlParts {
  host_part: Option<String>,
  directories: Vec<String>,
  raw_path: String,
  filename: String,
}

/// `extractUrlParts(url)`: `rawPath + filename` (without query or hash),
/// borrowed when there is nothing to rewrite.
fn url_path(url: &str) -> std::borrow::Cow<'_, str> {
  let b = url.as_bytes();
  let mut i = 0;
  let proto_len = b
    .iter()
    .take_while(|c| c.is_ascii_alphabetic() || **c == b'-')
    .count();
  let mut j = if proto_len > 0 && b.get(proto_len) == Some(&b':') {
    proto_len + 1
  } else {
    0
  };
  let mut host = false;
  if b[j..].starts_with(b"//") {
    j += 2;
    let rest = &url[j..];
    if let Some(p) = rest.find(['/', '?', '#'])
      && rest.as_bytes()[p] == b'/'
    {
      host = true;
      i = j + p + 1;
    }
  }
  if !host && matches!(b.first(), Some(b'/' | b'\\')) {
    i = 1;
  }
  let rest = &url[i..];
  let end = i + rest.find(['?', '#']).unwrap_or(rest.len());
  if url[i..end].contains('\\') {
    // the host part stays as written, directories use `/`
    std::borrow::Cow::Owned(format!(
      "{}{}",
      &url[..i],
      url[i..end].cow_replace('\\', "/")
    ))
  } else {
    std::borrow::Cow::Borrowed(&url[..end])
  }
}

/// `extractUrlParts(url)`
fn extract_url_parts(url: &str) -> UrlParts {
  // ^((?:[a-z-]+:)?\/{2}(?:[^/?#]*\/)|([/\\]))?((?:[^/\\?#]*[/\\])*)([^/\\?#]*)([#?].*)?$
  let b = url.as_bytes();
  let mut i = 0;
  let mut host: Option<String> = None;
  // protocol host
  let proto_len = b
    .iter()
    .take_while(|c| c.is_ascii_alphabetic() || **c == b'-')
    .count();
  let mut j = if proto_len > 0 && b.get(proto_len) == Some(&b':') {
    proto_len + 1
  } else {
    0
  };
  if b[j..].starts_with(b"//") {
    j += 2;
    let rest = &url[j..];
    if let Some(p) = rest.find(['/', '?', '#'])
      && rest.as_bytes()[p] == b'/'
    {
      host = Some(url[..j + p + 1].to_string());
      i = j + p + 1;
    }
  }
  if host.is_none() && matches!(b.first(), Some(b'/' | b'\\')) {
    host = Some(url[..1].to_string());
    i = 1;
  }
  let rest = &url[i..];
  let end_q = rest.find(['?', '#']).unwrap_or(rest.len());
  let path_part = &rest[..end_q];
  let last_sep = path_part.rfind(['/', '\\']);
  let (dirs, filename) = match last_sep {
    Some(p) => (&path_part[..p + 1], &path_part[p + 1..]),
    None => ("", path_part),
  };
  let mut raw_dirs: Vec<String> = Vec::new();
  let mut directories: Vec<String> = Vec::new();
  if !dirs.is_empty() {
    raw_dirs = dirs
      .cow_replace('\\', "/")
      .split('/')
      .map(|s| s.to_string())
      .collect();
    for d in &raw_dirs {
      if d == ".." {
        directories.pop();
      } else if d != "." {
        directories.push(d.clone());
      }
    }
  }
  let raw_path = format!("{}{}", host.clone().unwrap_or_default(), raw_dirs.join("/"));
  UrlParts {
    host_part: host,
    directories,
    raw_path,
    filename: filename.to_string(),
  }
}

/// `pathDiff(url, baseUrl)`
pub(crate) fn path_diff(url: &str, base_url: &str) -> String {
  let u = extract_url_parts(url);
  let b = extract_url_parts(base_url);
  if u.host_part != b.host_part {
    return String::new();
  }
  let max = b.directories.len().max(u.directories.len());
  let mut i = 0;
  while i < max {
    if b.directories.get(i) != u.directories.get(i) {
      break;
    }
    i += 1;
  }
  let base_rest = &b.directories[i.min(b.directories.len())..];
  let url_rest = &u.directories[i.min(u.directories.len())..];
  let mut diff = String::new();
  for _ in 0..base_rest.len().saturating_sub(1) {
    diff.push_str("../");
  }
  for d in url_rest.iter().take(url_rest.len().saturating_sub(1)) {
    diff.push_str(d);
    diff.push('/');
  }
  diff
}

// ---------------------------------------------------------------------------
// file manager
// ---------------------------------------------------------------------------

/// Resolves imports using a host application's module resolver.
///
/// The returned filename is read through the compiler's [`crate::FileSystem`]
/// and becomes the base for nested imports, URL rewriting and source maps.
pub trait ImportResolver: Send + Sync {
  fn resolve(&self, filename: &str, current_directory: &str) -> std::io::Result<String>;

  /// Try Less's native file lookup first, falling back to this resolver.
  /// Return false to resolve the request exclusively through the host.
  fn use_less_resolver(&self, _filename: &str) -> bool {
    true
  }
}

pub(crate) struct Loaded<'a> {
  pub filename: String,
  pub contents: std::borrow::Cow<'a, str>,
}

/// Node-style module resolution for `paths[i] === '.'`.
fn require_resolve(s: &Session<'_>, name: &str) -> Option<String> {
  if name.starts_with('.') || name.starts_with('/') {
    return None;
  }
  let mut dir = std::env::current_dir().ok()?;
  loop {
    let cand = dir.join("node_modules").join(name);
    if s.fs.exists(&cand) {
      // Node's require.resolve returns the real filename. Keeping a pnpm
      // symlink here loses the package's own node_modules for nested imports.
      return s
        .fs
        .canonicalize(&cand)
        .ok()
        .map(|path| path.to_string_lossy().cow_replace('\\', "/").into_owned());
    }
    if !dir.pop() {
      return None;
    }
  }
}

/// `/^(?:https?:)?\/\//i`
pub(crate) fn is_url(s: &str) -> bool {
  let b = s.as_bytes();
  let starts = |p: &[u8]| b.len() >= p.len() && b[..p.len()].eq_ignore_ascii_case(p);
  starts(b"//") || starts(b"http://") || starts(b"https://")
}

/// `url.resolve(base, rel)` for the cases imports produce.
fn url_resolve(base: &str, rel: &str) -> String {
  if is_url(rel) {
    return rel.to_string();
  }
  let scheme_end = base.find("//").map(|p| p + 2).unwrap_or(0);
  let host_end = base[scheme_end..]
    .find('/')
    .map(|p| p + scheme_end)
    .unwrap_or(base.len());
  let origin = &base[..host_end];
  let joined = if let Some(r) = rel.strip_prefix('/') {
    format!("/{}", r)
  } else {
    let path = &base[host_end..];
    let dir = &path[..path.rfind('/').map(|p| p + 1).unwrap_or(0)];
    format!("{}{}", if dir.is_empty() { "/" } else { dir }, rel)
  };
  format!("{}{}", origin, path_normalize(&joined))
}

/// `UrlFileManager.loadFile`
fn load_url<'a>(
  s: &Session<'a>,
  filename: &str,
  current_directory: &str,
) -> Result<Loaded<'a>, Box<LessError>> {
  use crate::fs::UrlError;
  let url = if is_url(filename) {
    filename.to_string()
  } else {
    url_resolve(current_directory, filename)
  };
  let request = if url.contains('?') {
    url.clone()
  } else {
    format!("{}?", url)
  };
  match s.fs.fetch_url(&request) {
    Ok(contents) => {
      if contents.is_empty() {
        s.warn(
          format!("Warning: Empty body (HTTP 200) returned by \"{}\"", url),
          None,
        );
      }
      Ok(Loaded {
        filename: url,
        contents: contents.into(),
      })
    }
    Err(UrlError::Unsupported) => Err(LessError::new(
      "File",
      "optional dependency 'needle' required to import over http(s)\n",
    )),
    Err(UrlError::NotFound) => Err(LessError::new(
      "File",
      format!("resource '{}' was not found\n", url),
    )),
    Err(UrlError::TooManyRedirects) => Err(LessError::new(
      "File",
      format!("resource '{}' caused too many redirects", url),
    )),
    Err(UrlError::Other(m)) => Err(LessError::new(
      "File",
      format!("resource '{}' gave this Error:\n  {}\n", url, m),
    )),
  }
}

/// `FileManager.loadFileSync`
pub(crate) fn load_file<'a>(
  s: &Session<'a>,
  filename: &str,
  current_directory: &str,
  ext: Option<&str>,
) -> Result<Loaded<'a>, Box<LessError>> {
  if is_url(filename) || is_url(current_directory) {
    return load_url(s, filename, current_directory);
  }
  let Some(resolver) = s.import_resolver else {
    return load_native_file(s, filename, current_directory, ext);
  };
  let native_error = if resolver.use_less_resolver(filename) {
    match load_native_file(s, filename, current_directory, ext) {
      Ok(loaded) => return Ok(loaded),
      Err(error) => Some(error),
    }
  } else {
    None
  };
  let resolved = resolver
    .resolve(filename, current_directory)
    .map_err(|error| {
      let message = match native_error {
        Some(native) => format!("Less resolver error:\n{}\n\n{}", native.message, error),
        None => error.to_string(),
      };
      LessError::new("File", message)
    })?;
  // Do not reinterpret the resolved filename with include paths or append an
  // extension. Its identity must also be retained for nested imports and maps.
  let contents = read_once(s.fs, &s.read_cache, s.a, &resolved).ok_or_else(|| {
    LessError::new(
      "File",
      format!("Could not read resolved import '{}'", resolved),
    )
  })?;
  Ok(Loaded {
    filename: resolved,
    contents: contents.into(),
  })
}

fn load_native_file<'a>(
  s: &Session<'a>,
  filename: &str,
  current_directory: &str,
  ext: Option<&str>,
) -> Result<Loaded<'a>, Box<LessError>> {
  let is_absolute = is_path_absolute(filename);
  let prefix = filename.as_bytes().first().copied();
  let explicit = prefix == Some(b'.') || prefix == Some(b'/');
  let mut paths: Vec<String> = if is_absolute {
    vec![String::new()]
  } else {
    vec![current_directory.to_string()]
  };
  for p in &s.opts.include_paths {
    paths.push(p.clone());
  }
  if !is_absolute && !paths.iter().any(|p| p == ".") {
    paths.push(".".into());
  }
  let url_path = url_path(filename);
  let mut tried: Vec<String> = Vec::new();
  for p in &paths {
    let mut full = url_path.to_string();
    if !p.is_empty() {
      if let Some(rest) = p.strip_prefix('#') {
        full = format!("{}{}", rest, full);
      } else {
        full = path_join(p, &full);
      }
    }
    let mut is_node_module = false;
    let try_with_extension = |full: &mut String, is_nm: &mut bool, tried: &mut Vec<String>| {
      let ext_filename = match ext {
        Some(e) => try_append_extension(full, e),
        None => full.clone(),
      };
      if ext_filename != *full && !explicit && p == "." {
        match require_resolve(s, &ext_filename) {
          Some(r) => {
            *full = r;
            *is_nm = true;
          }
          None => {
            tried.push(format!("npm://{}", ext_filename));
            *full = ext_filename;
          }
        }
      } else {
        *full = ext_filename;
      }
    };
    if !explicit && p == "." {
      match require_resolve(s, &full) {
        Some(r) => {
          full = r;
          is_node_module = true;
        }
        None => {
          tried.push(format!("npm://{}", full));
          try_with_extension(&mut full, &mut is_node_module, &mut tried);
        }
      }
    } else {
      try_with_extension(&mut full, &mut is_node_module, &mut tried);
    }
    let contents = match cached_read(&s.read_cache, &full) {
      Some(c) => c,
      None => {
        // being read on a parser thread?
        let ahead = s
          .preparsed
          .borrow()
          .get(&full)
          .and_then(|q| q.front().map(|pp| pp.slot.clone()));
        if let Some(slot) = &ahead {
          help_while_running(s, slot, true);
        }
        match ahead.and_then(|slot| slot.contents()) {
          Some(c) => record_read(&s.read_cache, &full, c),
          None => read_once(s.fs, &s.read_cache, s.a, &full),
        }
      }
    };
    match contents {
      Some(contents) => {
        return Ok(Loaded {
          filename: full,
          contents: contents.into(),
        });
      }
      None => {
        tried.push(if is_node_module {
          format!("npm://{}", full)
        } else {
          full
        });
      }
    }
  }
  Err(LessError::new(
    "File",
    format!("'{}' wasn't found. Tried - {}", filename, tried.join(",")),
  ))
}

/// A file's parse result, as produced on a parser thread.
pub(crate) type ParseResult<'a> = (
  Result<&'a Ruleset<'a>, Box<LessError>>,
  Vec<(String, Option<&'static str>)>,
);

/// Values that cross between the main and the parser threads. Sound because
/// each worker arena is allocated in by its own thread only (the main thread
/// neither allocates in it nor grows vectors created there during the import
/// phase), results are handed over through a mutex, and all workers are
/// joined before evaluation starts.
struct AssertSend<T>(T);
unsafe impl<T> Send for AssertSend<T> {}

/// What a parser thread produced for one file.
pub(crate) struct Ahead<'a> {
  /// The file as read (`None`: not readable).
  contents: Option<&'a str>,
  /// The text `load` would parse (BOM removed, line endings normalized).
  text: Option<&'a str>,
  parsed: Option<ParseResult<'a>>,
}

enum SlotState<'a> {
  Pending,
  Running,
  /// Read, being parsed.
  Read(Option<&'a str>),
  Done(AssertSend<Ahead<'a>>),
  /// Taken over by the main thread before a worker started it.
  Stolen,
}

/// Where a parser thread puts one file's result.
pub(crate) struct Slot<'a> {
  state: std::sync::Mutex<SlotState<'a>>,
  cv: std::sync::Condvar,
}

impl Default for Slot<'_> {
  fn default() -> Self {
    Slot {
      state: std::sync::Mutex::new(SlotState::Pending),
      cv: std::sync::Condvar::new(),
    }
  }
}

impl<'a> Slot<'a> {
  /// A job no worker has started is dropped.
  fn cancel(&self) {
    let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    if matches!(*g, SlotState::Pending) {
      *g = SlotState::Stolen;
    }
  }
  /// Called by a worker: whether to do this job.
  fn start(&self) -> bool {
    let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    if matches!(*g, SlotState::Pending) {
      *g = SlotState::Running;
      true
    } else {
      false
    }
  }
  fn put(&self, r: Ahead<'a>) {
    let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    *g = SlotState::Done(AssertSend(r));
    self.cv.notify_all();
  }
  fn put_read(&self, contents: Option<&'a str>) {
    let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    *g = SlotState::Read(contents);
    self.cv.notify_all();
  }
  /// Waits briefly without sleeping (hand-offs are short and frequent),
  /// then on the condition variable.
  fn wait<'g>(
    &'g self,
    g: std::sync::MutexGuard<'g, SlotState<'a>>,
    spun: &mut u32,
  ) -> std::sync::MutexGuard<'g, SlotState<'a>> {
    if *spun < 2000 {
      *spun += 1;
      drop(g);
      for _ in 0..32 {
        std::hint::spin_loop();
      }
      self.state.lock().unwrap_or_else(|e| e.into_inner())
    } else {
      self.cv.wait(g).unwrap_or_else(|e| e.into_inner())
    }
  }
  /// Waits for a running job; a job no worker has started is stolen
  /// (`None`: the caller does the work itself).
  fn with<T>(&self, f: impl FnOnce(&mut Ahead<'a>) -> T) -> Option<T> {
    let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    let mut spun = 0;
    loop {
      match &mut *g {
        SlotState::Pending => {
          *g = SlotState::Stolen;
          return None;
        }
        SlotState::Stolen => return None,
        SlotState::Done(r) => return Some(f(&mut r.0)),
        SlotState::Running | SlotState::Read(_) => g = self.wait(g, &mut spun),
      }
    }
  }
  /// Whether a worker is still reading (`contents`) or parsing.
  fn running(&self, contents: bool) -> bool {
    let g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    match &*g {
      SlotState::Running => true,
      SlotState::Read(_) => !contents,
      _ => false,
    }
  }

  /// The file contents, unless the job was stolen.
  pub(crate) fn contents(&self) -> Option<Option<&'a str>> {
    let mut g = self.state.lock().unwrap_or_else(|e| e.into_inner());
    let mut spun = 0;
    loop {
      match &mut *g {
        SlotState::Pending => {
          *g = SlotState::Stolen;
          return None;
        }
        SlotState::Stolen => return None,
        SlotState::Read(c) => return Some(*c),
        SlotState::Done(r) => return Some(r.0.contents),
        SlotState::Running => g = self.wait(g, &mut spun),
      }
    }
  }
  /// The parsed text and result (`None` when stolen or not parsed).
  pub(crate) fn take_parsed(&self) -> Option<(Option<&'a str>, ParseResult<'a>)> {
    self
      .with(|r| r.parsed.take().map(|p| (r.text, p)))
      .flatten()
  }
}

/// The parser threads' job queues.
struct Pool<'a> {
  senders: Vec<std::sync::mpsc::Sender<ParseJob<'a>>>,
  next: std::cell::Cell<usize>,
}

impl<'a> Pool<'a> {
  fn send(&self, job: ParseJob<'a>) -> Result<(), ()> {
    if self.senders.is_empty() {
      return Err(());
    }
    let k = self.next.get();
    self.next.set((k + 1) % self.senders.len());
    self.senders[k].send(job).map_err(|_| ())
  }
}

pub(crate) struct ParseJob<'a> {
  slot: std::sync::Arc<Slot<'a>>,
  /// The file to read, unless `contents` is known.
  path: String,
  contents: Option<&'a str>,
  name: &'a str,
  base: u32,
  max_len: usize,
}

/// The position range reserved for a file read ahead (bigger files are not
/// parsed ahead).
const AHEAD_RANGE: usize = 1 << 20;

/// While the main thread waits for `slot`, it does queued jobs no worker
/// has started (in its own arena).
fn help_while_running<'a>(s: &Session<'a>, slot: &Slot<'a>, contents: bool) {
  while slot.running(contents) {
    let job = s.ahead_queue.borrow_mut().pop_front();
    let Some((path, base, max_len, other)) = job else {
      return;
    };
    if !other.start() {
      continue;
    }
    let read = read_once(s.fs, &s.read_cache, s.a, &path);
    other.put_read(read);
    let text = read.map(|c| import_text(s.a, c));
    let parsed = match text {
      Some(t) if t.len() <= max_len => {
        let mut p = Parser::detached(s.a, s.opts, s.a.str(&path), base, t);
        let root = p.parse_root();
        Some((root, std::mem::take(&mut p.warnings)))
      }
      _ => None,
    };
    other.put(Ahead {
      contents: read,
      text,
      parsed,
    });
  }
}

fn cached_read<'a>(reads: &ReadCache<'a>, path: &str) -> Option<Option<&'a str>> {
  reads
    .lock()
    .unwrap_or_else(|e| e.into_inner())
    .get(path)
    .copied()
    .flatten()
}

/// Reads a file once per compilation (see `Session::read_cache`): a file
/// being read on another thread is waited for (reads always finish).
fn read_once<'a>(
  fs: &dyn crate::fs::FileSystem,
  reads: &ReadCache<'a>,
  a: Alloc<'a>,
  path: &str,
) -> Option<&'a str> {
  loop {
    let mut g = reads.lock().unwrap_or_else(|e| e.into_inner());
    match g.get(path) {
      Some(Some(c)) => return *c,
      Some(None) => {
        drop(g);
        std::thread::yield_now();
      }
      None => {
        g.insert(path.to_string(), None);
        drop(g);
        let c = fs
          .read(Path::new(path))
          .ok()
          .map(|t| &*a.bump.alloc_str(&t));
        reads
          .lock()
          .unwrap_or_else(|e| e.into_inner())
          .insert(path.to_string(), Some(c));
        return c;
      }
    }
  }
}

/// Records contents read elsewhere (a parser thread's slot).
fn record_read<'a>(reads: &ReadCache<'a>, path: &str, c: Option<&'a str>) -> Option<&'a str> {
  let mut g = reads.lock().unwrap_or_else(|e| e.into_inner());
  match g.get(path) {
    Some(Some(prev)) => *prev,
    _ => {
      g.insert(path.to_string(), Some(c));
      c
    }
  }
}

/// `load`'s text for file contents: BOM removed, line endings normalized.
fn import_text<'a>(a: Alloc<'a>, contents: &'a str) -> &'a str {
  let body = contents.strip_prefix('\u{feff}').unwrap_or(contents);
  if body.contains('\r') {
    a.str(&body.cow_replace("\r\n", "\n").cow_replace('\r', "\n"))
  } else {
    body
  }
}

/// The import phase: `ImportVisitor` on this thread, with parser threads
/// (one per worker arena) parsing imported files ahead while it runs.
pub(crate) fn run_imports<'a>(
  s: &Session<'a>,
  root: &'a Ruleset<'a>,
  filename: &str,
) -> LResult<()> {
  let mut iv = ImportVisitor::new(s, filename);
  let mut imports = Vec::new();
  collect_imports(Node::Ruleset(root), &mut imports, 0);
  // a few imports are loaded sooner than parser threads would start; the
  // root's imports may be index files of many more, so those count too
  // (their texts are read now instead of later: `read_once` keeps them)
  let many_imports = imports.len() >= 4 || {
    let mut n = imports.len();
    for imp in &imports {
      let (Some(path), false) = (
        imp.get_path(),
        imp.options.inline || imp.options.is_plugin || is_variable_import(imp),
      ) else {
        continue;
      };
      let dir = s
        .pos_file_info(imp.pos)
        .map(|i| i.current_directory)
        .unwrap_or("");
      if is_url(path) || is_url(dir) {
        continue;
      }
      if let Some(text) =
        first_candidate(s, path, dir, ".less").and_then(|f| read_once(s.fs, &s.read_cache, s.a, &f))
      {
        n += import_statements(text);
      }
      if n >= 4 {
        break;
      }
    }
    n >= 4
  };
  if s.worker_bumps.is_empty() || !many_imports || !s.functions.pre.is_empty() {
    return iv.run(root);
  }
  let opts: &crate::options::Options = s.opts;
  let fs: &dyn crate::fs::FileSystem = s.fs;
  let reads: &ReadCache<'a> = &s.read_cache;
  std::thread::scope(|scope| {
    let mut senders = Vec::new();
    for bump in s.worker_bumps {
      let bump = AssertSend(bump as *const bumpalo::Bump);
      // one channel per worker (jobs are dealt round-robin): no lock
      // to contend on while workers poll
      let (tx, rx) = std::sync::mpsc::channel::<ParseJob<'a>>();
      let spawned = std::thread::Builder::new()
        .stack_size(8 << 20)
        .spawn_scoped(scope, move || {
          let bump = bump;
          // SAFETY: see `AssertSend`.
          let a = Alloc {
            bump: unsafe { &*bump.0 },
          };
          loop {
            // poll briefly before sleeping: jobs come in quick bursts
            let mut job = Err(std::sync::mpsc::RecvError);
            let mut polled = false;
            for _ in 0..400 {
              match rx.try_recv() {
                Ok(j) => {
                  job = Ok(j);
                  polled = true;
                  break;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                  polled = true;
                  break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                  for _ in 0..32 {
                    std::hint::spin_loop();
                  }
                }
              }
            }
            if !polled {
              job = rx.recv();
            }
            let Ok(job) = job else { break };
            if !job.slot.start() {
              continue;
            }
            let contents = match job.contents {
              Some(c) => Some(c),
              None => read_once(fs, reads, a, &job.path),
            };
            job.slot.put_read(contents);
            let text = contents.map(|c| import_text(a, c));
            let parsed = match text {
              Some(t) if t.len() <= job.max_len => {
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                  let mut p = Parser::detached(a, opts, job.name, job.base, t);
                  let root = p.parse_root();
                  (root, std::mem::take(&mut p.warnings))
                }))
                .ok()
              }
              _ => None,
            };
            job.slot.put(Ahead {
              contents,
              text,
              parsed,
            });
          }
        });
      if spawned.is_err() {
        break;
      }
      senders.push(tx);
    }
    iv.pool = Some(Pool {
      senders,
      next: std::cell::Cell::new(0),
    });
    let r = iv.run(root);
    // jobs no worker has started are no longer needed (speculative ones,
    // or files every import of which was served otherwise): cancel them so
    // the workers wind down instead of finishing their queues
    for (_, _, _, slot) in s.ahead_queue.borrow_mut().drain(..) {
      slot.cancel();
    }
    // closing the channel ends the workers; `scope` joins them
    iv.pool = None;
    r
  })
}

/// A rough count of `@import` statements in a text (not `@import-x`
/// variables, nor mentions inside comments that start a line with `//`).
fn import_statements(text: &str) -> usize {
  let b = text.as_bytes();
  memchr::memmem::find_iter(b, b"@import")
    .filter(|&i| {
      let next = b.get(i + 7).copied();
      let line = &b[b[..i]
        .iter()
        .rposition(|&c| c == b'\n')
        .map_or(0, |p| p + 1)..i];
      matches!(next, Some(b' ' | b'\t' | b'\n' | b'(' | b'"' | b'\''))
        && line.iter().all(|c| c.is_ascii_whitespace())
    })
    .count()
}

/// The imports in a file's tree, in visiting order.
fn collect_imports<'a>(n: Node<'a>, out: &mut Vec<&'a Import<'a>>, depth: u32) {
  if depth > 64 {
    return;
  }
  let each = |rules: &[Node<'a>], out: &mut Vec<&'a Import<'a>>| {
    for r in rules {
      collect_imports(*r, out, depth + 1);
    }
  };
  match n {
    Node::Import(imp) => out.push(imp),
    Node::Ruleset(r) | Node::MixinDefinition(r) => {
      each(r.rules.borrow().as_deref().unwrap_or(&[]), out)
    }
    Node::Declaration(d) => {
      if let Node::DetachedRuleset(dr) = d.value.get() {
        collect_imports(dr.ruleset, out, depth + 1);
      }
    }
    Node::AtRule(at) => each(at.rules.get().unwrap_or(&[]), out),
    Node::Media(m) | Node::Container(m) => each(m.rules.get().unwrap_or(&[]), out),
    _ => {}
  }
}

/// The path of a variable import from plain string variables in `frames`
/// (no evaluation, no caches touched), or `None`.
fn guess_import_path<'a>(imp: &Import<'a>, frames: &[Frame<'a>]) -> Option<String> {
  let q = match imp.path {
    Node::Quoted(q) => q,
    Node::Url(u) => match u.value {
      Node::Quoted(q) => q,
      _ => return None,
    },
    _ => return None,
  };
  let v = q.value;
  let mut out = String::with_capacity(v.len() + 32);
  let mut rest = v;
  while let Some(i) = rest.find("@{") {
    out.push_str(&rest[..i]);
    let after = &rest[i + 2..];
    let end = after.find('}')?;
    let name = &after[..end];
    if name.is_empty()
      || !name
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
      return None;
    }
    out.push_str(lookup_plain(frames, name, 0)?);
    rest = &after[end + 1..];
  }
  out.push_str(rest);
  if out.contains('$') {
    return None;
  }
  Some(out)
}

/// The value of `@name` as a plain string, following `@a` / `@@a`
/// references.
fn lookup_plain<'a>(frames: &[Frame<'a>], name: &str, depth: u32) -> Option<&'a str> {
  if depth > 8 {
    return None;
  }
  let value = frames.iter().find_map(|f| match f {
    Frame::Ruleset(r) => find_variable(r, name, 0),
    Frame::Other => None,
  })?;
  plain_string(value, frames, depth)
}

/// The value of the last `@name` declaration in `rs` (or its loaded imports).
fn find_variable<'a>(rs: &Ruleset<'a>, name: &str, depth: u32) -> Option<Node<'a>> {
  if depth > 16 {
    return None;
  }
  let rules = rs.rules.borrow();
  for n in rules.iter().flat_map(|v| v.iter()).rev() {
    match n {
      Node::Declaration(d)
        if d.variable && d.name_str().and_then(|s| s.strip_prefix('@')) == Some(name) =>
      {
        return Some(d.value.get());
      }
      Node::Import(imp) => {
        if let Some(ImportRoot::Tree(root)) = imp.root.get()
          && let Some(v) = find_variable(root, name, depth + 1)
        {
          return Some(v);
        }
      }
      _ => {}
    }
  }
  None
}

fn plain_string<'a>(n: Node<'a>, frames: &[Frame<'a>], depth: u32) -> Option<&'a str> {
  let ident = |t: &str| {
    !t.is_empty()
      && t
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
  };
  match n {
    Node::Quoted(q) if !q.value.contains("@{") => Some(q.value),
    Node::Keyword(k) => Some(k.value),
    Node::Variable(v) => {
      if let Some(inner) = v.name.strip_prefix("@@") {
        // `@@a`: the variable named by the value of `@a`
        let target = lookup_plain(frames, inner, depth + 1)?;
        lookup_plain(frames, target, depth + 1)
      } else {
        lookup_plain(frames, v.name.strip_prefix('@')?, depth + 1)
      }
    }
    Node::Anonymous(a) => {
      let t = a.value.trim();
      let b = t.as_bytes();
      if b.len() >= 2 && (b[0] == b'"' || b[0] == b'\'') && b[b.len() - 1] == b[0] {
        let inner = &t[1..t.len() - 1];
        (!inner.contains(['@', '$', '"', '\''])).then_some(inner)
      } else if let Some(inner) = t.strip_prefix("@@").filter(|x| ident(x)) {
        let target = lookup_plain(frames, inner, depth + 1)?;
        lookup_plain(frames, target, depth + 1)
      } else if let Some(inner) = t.strip_prefix('@').filter(|x| ident(x)) {
        lookup_plain(frames, inner, depth + 1)
      } else if t
        .bytes()
        .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_' || c == b'.' || c == b'/')
        && !t.is_empty()
      {
        Some(t)
      } else {
        None
      }
    }
    Node::Value(v) if v.value.len() == 1 => plain_string(v.value[0], frames, depth),
    Node::Expression(e) if e.value.len() == 1 => plain_string(e.value[0], frames, depth),
    _ => None,
  }
}

/// The first path `load_file` tries for a relative or absolute import
/// (`None` when resolution starts with node module lookup).
fn first_candidate(
  s: &Session<'_>,
  filename: &str,
  current_directory: &str,
  ext: &str,
) -> Option<String> {
  if s
    .import_resolver
    .is_some_and(|resolver| !resolver.use_less_resolver(filename))
  {
    return None;
  }
  let p = if is_path_absolute(filename) {
    ""
  } else {
    current_directory
  };
  if p == "." {
    return None;
  }
  let mut full = url_path(filename).into_owned();
  if !p.is_empty() {
    full = match p.strip_prefix('#') {
      Some(rest) => format!("{}{}", rest, full),
      None => path_join(p, &full),
    };
  }
  Some(try_append_extension(&full, ext))
}

// ---------------------------------------------------------------------------
// ImportVisitor
// ---------------------------------------------------------------------------

#[derive(Copy, Clone)]
enum Frame<'a> {
  Ruleset(&'a Ruleset<'a>),
  Other,
}

struct Ctx<'a> {
  frames: Vec<Frame<'a>>,
  import_multiple: bool,
}

/// An import waiting in the sequencer. less.js starts loading at discovery
/// and completes loads in that order; here the load runs when the item comes
/// up, in the same order (and the parser threads have it ready by then).
enum Queued<'a> {
  Load {
    node: &'a Import<'a>,
    ctx: Ctx<'a>,
    try_append: bool,
  },
}

struct ImportResult<'a> {
  error: Option<Box<LessError>>,
  /// `error` is `importManager.error` (one object in less.js: it gets its
  /// position where its own import is sequenced).
  shared_error: bool,
  root: Option<ImportRoot<'a>>,
  imported_at_root: bool,
  full_path: Option<String>,
  file_id: Option<u32>,
}

struct VarImport<'a> {
  node: &'a Import<'a>,
  ctx: Ctx<'a>,
  parent: Frame<'a>,
}

pub(crate) struct ImportVisitor<'a, 's> {
  s: &'s Session<'a>,
  ctx: Ctx<'a>,
  recursion_detector: FxHashSet<String>,
  error: Option<Box<LessError>>,
  queue: VecDeque<Queued<'a>>,
  variable_imports: VecDeque<VarImport<'a>>,
  /// Pending variable imports already considered for parsing ahead.
  var_considered: usize,
  /// Jobs for the parser threads (see [`run_imports`]).
  pool: Option<Pool<'a>>,
  root_filename: String,
  /// Parsed files (`importManager.files`): name → (root, multiple option);
  /// no root for a file whose parse reported an error.
  files: Vec<(String, Option<&'a Ruleset<'a>>, bool)>,
  /// `importManager.error`: the first failed load. A file parsed after it
  /// reports it instead of its tree (`Parser.parse`'s `finish` takes
  /// `imports.error`); less.js completes loads in the order they start.
  manager_error: Option<Box<LessError>>,
  /// The import whose load failed first.
  manager_from: Option<*const Import<'a>>,
  /// The root is being visited or the sequencer is running. less.js loads
  /// imports asynchronously, so no `onImported` runs before the root visit
  /// ends, and imports found while visiting a file run after those queued
  /// before them (breadth first): a nested `tryRun` finds nothing ready.
  sequencing: bool,
}

impl<'a, 's> ImportVisitor<'a, 's> {
  pub(crate) fn new(s: &'s Session<'a>, root_filename: &str) -> Self {
    ImportVisitor {
      s,
      ctx: Ctx {
        frames: Vec::new(),
        import_multiple: false,
      },
      recursion_detector: FxHashSet::default(),
      error: None,
      queue: VecDeque::new(),
      variable_imports: VecDeque::new(),
      var_considered: 0,
      pool: None,
      root_filename: root_filename.to_string(),
      files: Vec::new(),
      manager_error: None,
      manager_from: None,
      sequencing: false,
    }
  }

  pub(crate) fn run(&mut self, root: &'a Ruleset<'a>) -> LResult<()> {
    self.prefetch(root);
    self.sequencing = true;
    if let Err(e) = self.visit_ruleset(root) {
      self.error = Some(e);
    }
    self.sequencing = false;
    self.try_run();
    match self.error.take() {
      Some(e) => Err(e),
      None => Ok(()),
    }
  }

  fn visit(&mut self, n: Node<'a>) -> LResult<()> {
    match n {
      Node::Import(i) => self.visit_import(i),
      Node::Ruleset(r) => self.visit_ruleset(r),
      Node::MixinDefinition(r) => {
        self.ctx.frames.insert(0, Frame::Ruleset(r));
        let res = self.visit_rules(r);
        self.ctx.frames.remove(0);
        res
      }
      Node::Declaration(d) => {
        if let Node::DetachedRuleset(dr) = d.value.get() {
          self.ctx.frames.insert(0, Frame::Other);
          let res = self.visit(dr.ruleset);
          self.ctx.frames.remove(0);
          return res;
        }
        Ok(())
      }
      Node::AtRule(at) => {
        let push = at.value.is_some()
          || at
            .declarations
            .get()
            .map(|d| !d.is_empty())
            .unwrap_or(false)
          || at.rules.get().map(|r| !r.is_empty()).unwrap_or(false);
        if push {
          self.ctx.frames.insert(0, Frame::Other);
        }
        let mut res = Ok(());
        if let Some(rules) = at.rules.get() {
          for r in rules {
            res = self.visit(*r);
            if res.is_err() {
              break;
            }
          }
        }
        if push {
          self.ctx.frames.remove(0);
        }
        res
      }
      Node::Media(m) | Node::Container(m) => {
        let body = m.body();
        if m.kind == AtRuleKind::Media
          && let Some(b) = body
        {
          self.ctx.frames.insert(0, Frame::Ruleset(b));
        }
        let mut res = Ok(());
        if let Some(rules) = m.rules.get() {
          for r in rules {
            res = self.visit(*r);
            if res.is_err() {
              break;
            }
          }
        }
        if m.kind == AtRuleKind::Media && body.is_some() {
          self.ctx.frames.remove(0);
        }
        res
      }
      Node::DetachedRuleset(d) => self.visit(d.ruleset),
      _ => Ok(()),
    }
  }

  fn visit_rules(&mut self, r: &'a Ruleset<'a>) -> LResult<()> {
    // `visitArray`: the length is read once; only the visited import
    // itself is replaced while it is visited.
    let n = r.rules_len();
    for k in 0..n {
      if let Some(rule) = r.rule_at(k) {
        self.visit(rule)?;
      }
    }
    Ok(())
  }

  fn visit_ruleset(&mut self, r: &'a Ruleset<'a>) -> LResult<()> {
    self.ctx.frames.insert(0, Frame::Ruleset(r));
    let res = self.visit_rules(r);
    self.ctx.frames.remove(0);
    res
  }

  /// Reads the files of the static imports of a file's tree and parses them on
  /// worker threads while the next file is read (less.js's asynchronous
  /// reads overlap too). Results go to the read cache and
  /// `Session::preparsed`; the imports themselves are still resolved,
  /// registered and adopted sequentially in `load`, so ordering, warnings
  /// and errors are unchanged.
  fn prefetch(&self, r: &'a Ruleset<'a>) {
    let s = self.s;
    if self.pool.is_none() {
      return;
    }
    let mut paths: Vec<String> = Vec::new();
    {
      let mut imports: Vec<&'a Import<'a>> = Vec::new();
      collect_imports(Node::Ruleset(r), &mut imports, 0);
      let preparsed = s.preparsed.borrow();
      for imp in imports {
        if imp.css.get().is_some()
          || imp.options.inline
          || imp.options.is_plugin
          || is_variable_import(imp)
        {
          continue;
        }
        let path = match imp.get_path() {
          Some(p) if !is_url(p) => p,
          _ => continue,
        };
        let dir = s
          .pos_file_info(imp.pos)
          .map(|i| i.current_directory)
          .unwrap_or("");
        if is_url(dir) {
          continue;
        }
        if let Some(full) = first_candidate(s, path, dir, ".less") {
          // each `multiple` import parses the file again (one job
          // per import); others parse it once
          let wanted = if imp.options.multiple == Some(true) {
            true
          } else {
            s.file_id(&full).is_none() && !preparsed.contains_key(&full) && !paths.contains(&full)
          };
          if wanted {
            paths.push(full);
          }
        }
      }
    }
    // loads run when the sequencer reaches them, after this file's visit:
    // there is time to parse even a single import ahead
    if paths.is_empty() {
      return;
    }
    self.parse_ahead(paths);
  }

  /// Queues `paths` for the parser threads, which read (when the size is
  /// known up front) and parse them; `load_file` and `load` wait for a
  /// file's result only when they need it.
  fn parse_ahead(&self, paths: Vec<String>) {
    let s = self.s;
    let Some(pool) = &self.pool else { return };
    for p in paths {
      let cached = cached_read(&s.read_cache, &p);
      let queued = s
        .preparsed
        .borrow()
        .get(&p)
        .and_then(|q| q.front().map(|pp| (pp.base, pp.max_len, pp.slot.clone())));
      // the file's length (or an upper bound) and contents if known
      let (len, contents) = match (cached, &queued) {
        (Some(None), _) => continue,
        (Some(Some(c)), _) => (import_text(s.a, c).len(), Some(c)),
        (None, Some((_, max_len, _))) => (*max_len, None),
        // unread: a generous position range (larger files are parsed
        // on this thread later), while position space lasts
        (None, None) if s.next_base() < (1 << 31) => (AHEAD_RANGE, None),
        (None, None) => continue,
      };
      // the base `load` would use: the registered file's (a `multiple`
      // re-import reuses it) or a reserved one
      let (base, max_len) = match (s.file_id(&p), &queued) {
        (Some(id), _) => (s.file_base(id), s.file_contents(id).len()),
        (None, Some((b, m, _))) => (*b, *m),
        (None, None) => (s.reserve_base(len), len),
      };
      let slot = std::sync::Arc::new(Slot::default());
      let name = s.a.str(&p);
      let job = ParseJob {
        slot: slot.clone(),
        path: p.clone(),
        contents,
        name,
        base,
        max_len,
      };
      if pool.send(job).is_err() {
        return;
      }
      s.ahead_queue
        .borrow_mut()
        .push_back((p.clone(), base, max_len, slot.clone()));
      s.preparsed
        .borrow_mut()
        .entry(p)
        .or_default()
        .push_back(Preparsed {
          base,
          max_len,
          slot,
        });
    }
  }

  fn visit_import(&mut self, imp: &'a Import<'a>) -> LResult<()> {
    let inline = imp.options.inline;
    if imp.css.get() != Some(true) || inline {
      let ctx = Ctx {
        frames: self.ctx.frames.clone(),
        import_multiple: self.ctx.import_multiple,
      };
      let parent = ctx.frames.first().copied().unwrap_or(Frame::Other);
      if is_variable_import(imp) {
        self.variable_imports.push_back(VarImport {
          node: imp,
          ctx,
          parent,
        });
      } else {
        self.process_import_node(imp, ctx, parent);
      }
    }
    Ok(())
  }

  fn eval_ctx(&self, frames: &[Frame<'a>]) -> EvalCtx<'a, 's> {
    let rs: Vec<&'a Ruleset<'a>> = frames
      .iter()
      .filter_map(|f| match f {
        Frame::Ruleset(r) => Some(*r),
        Frame::Other => None,
      })
      .collect();
    EvalCtx::new(self.s, rs)
  }

  fn process_import_node(&mut self, imp: &'a Import<'a>, mut ctx: Ctx<'a>, parent: Frame<'a>) {
    let inline = imp.options.inline;
    let mut ec = self.eval_ctx(&ctx.frames);
    let evald = match ec.eval_for_import(imp) {
      Ok(e) => Some(e),
      Err(mut e) => {
        if e.pos.is_none() && e.file.is_none() {
          e.set_pos(imp.pos);
        }
        imp.css.set(Some(true));
        let id = self.s.import_errors.borrow().len() as u32;
        self.s.import_errors.borrow_mut().push(e);
        imp.error.set(Some(id));
        None
      }
    };
    let evald = match evald {
      Some(e) if e.css.get() != Some(true) || inline => e,
      _ => {
        self.try_run();
        return;
      }
    };
    if evald.options.multiple == Some(true) {
      ctx.import_multiple = true;
    }
    let try_append = evald.css.get().is_none();
    if let Frame::Ruleset(p) = parent {
      p.bump_generation();
      if let Some(rules) = p.rules.borrow_mut().as_mut() {
        for r in rules.iter_mut() {
          if let Node::Import(x) = r
            && std::ptr::eq(*x, imp)
          {
            *r = Node::Import(evald);
            break;
          }
        }
      }
    }
    self.queue.push_back(Queued::Load {
      node: evald,
      ctx,
      try_append,
    });
    self.try_run();
  }

  /// `importManager.push(path, tryAppendExtension, fileInfo, options, callback)`
  fn load(&mut self, imp: &'a Import<'a>, try_append_ext: bool) -> ImportResult<'a> {
    let path = imp.get_path().unwrap_or("");
    let current_info = self.s.pos_file_info(imp.pos);
    let current_directory = current_info
      .as_ref()
      .map(|i| i.current_directory)
      .unwrap_or("");
    let ext = if try_append_ext {
      Some(if imp.options.is_plugin {
        ".js"
      } else {
        ".less"
      })
    } else {
      None
    };
    let loaded = load_file(self.s, path, current_directory, ext);
    let file_parsed = |s: &mut Self,
                       e: Option<Box<LessError>>,
                       root: Option<ImportRoot<'a>>,
                       full_path: Option<String>,
                       fid: Option<u32>|
     -> ImportResult<'a> {
      let imported_equals_root = full_path.as_deref() == Some(s.root_filename.as_str());
      if imp.options.optional && e.is_some() {
        return ImportResult {
          error: None,
          shared_error: false,
          root: Some(ImportRoot::Tree(s.s.a.ruleset(None, Some(s.s.a.vec())))),
          imported_at_root: false,
          full_path: None,
          file_id: None,
        };
      }
      if let Some(fp) = &full_path {
        let r = match root {
          Some(ImportRoot::Tree(r)) => Some(r),
          _ => None,
        };
        if !imp.options.inline
          && !matches!(root, Some(ImportRoot::Text(_)))
          && !s.files.iter().any(|(n, _, _)| n == fp)
        {
          s.files
            .push((fp.clone(), r, imp.options.multiple == Some(true)));
        }
      }
      if e.is_some() && s.manager_error.is_none() {
        s.manager_error = e.clone();
        s.manager_from = Some(imp as *const Import<'a>);
      }
      ImportResult {
        error: e,
        shared_error: false,
        root,
        imported_at_root: imported_equals_root,
        full_path,
        file_id: fid,
      }
    };
    let loaded = match loaded {
      Ok(l) => l,
      Err(e) => return file_parsed(self, Some(e), None, None, None),
    };
    if imp.options.is_plugin {
      let e = LessError::new(
        "Plugin",
        format!(
          "JavaScript plugins are not supported by less-rs: @plugin \"{}\"",
          path
        ),
      );
      return file_parsed(self, Some(e), None, Some(loaded.filename), None);
    }
    let a = self.s.a;
    let body: &'a str = match loaded.contents {
      std::borrow::Cow::Borrowed(b) => b,
      std::borrow::Cow::Owned(o) => a.str(&o),
    };
    let mut contents: &'a str = body.strip_prefix('\u{feff}').unwrap_or(body);
    if !imp.options.inline
      && let Some(p) = self.s.functions.preprocess(contents, &loaded.filename)
    {
      contents = a.str(&p);
    }
    let resolved = loaded.filename;
    let current_dir = get_path(&resolved);
    let mut rootpath = current_info
      .as_ref()
      .map(|i| i.rootpath.to_string())
      .unwrap_or_default();
    let entry_path = current_info.as_ref().map(|i| i.entry_path).unwrap_or("");
    if self.s.opts.rewrite_urls != RewriteUrls::Off {
      rootpath = format!(
        "{}{}",
        self.s.opts.rootpath,
        path_diff(&current_dir, entry_path)
      );
      if self.s.opts.rootpath.is_empty() {
        rootpath = path_diff(&current_dir, entry_path);
      }
    }
    let reference =
      current_info.as_ref().map(|i| i.reference).unwrap_or(false) || imp.options.reference;
    let info = FileInfo {
      filename: a.str(&resolved),
      rootpath: a.str(&rootpath),
      current_directory: a.str(&current_dir),
      entry_path,
      root_filename: current_info.as_ref().map(|i| i.root_filename).unwrap_or(""),
      reference,
      rewrite_urls: self.s.opts.rewrite_urls,
    };
    let text = if contents.contains('\r') {
      a.str(&contents.cow_replace("\r\n", "\n").cow_replace('\r', "\n"))
    } else {
      contents
    };
    if imp.options.inline {
      let fid = self.s.add_file(contents, info, 0);
      self.note_import(&resolved);
      return file_parsed(
        self,
        None,
        Some(ImportRoot::Text(contents)),
        Some(resolved),
        Some(fid),
      );
    }
    // reuse an already parsed file unless `multiple`
    if let Some((_, root, multiple)) = self.files.iter().find(|(n, _, _)| *n == resolved)
      && !*multiple
      && imp.options.multiple != Some(true)
    {
      let root = *root;
      let fid = self.s.file_id(&resolved);
      return file_parsed(self, None, root.map(ImportRoot::Tree), Some(resolved), fid);
    }
    // parsed ahead on a worker thread?
    let pp = self
      .s
      .preparsed
      .borrow_mut()
      .get_mut(&resolved)
      .and_then(|q| q.pop_front());
    let parsed = pp.and_then(|pp| {
      help_while_running(self.s, &pp.slot, false);
      let (ptext, parsed) = pp.slot.take_parsed()?;
      if ptext != Some(text) {
        return None;
      }
      match self.s.file_id(&resolved) {
        None => Some((pp.base, parsed, true)),
        // a `multiple` re-import: `add_file` keeps the id and base
        Some(id)
          if self.s.file_contents(id).len() >= text.len() && self.s.file_base(id) == pp.base =>
        {
          Some((pp.base, parsed, false))
        }
        Some(_) => None,
      }
    });
    if let Some((base, (root, warnings), new)) = parsed {
      {
        let fid = if new {
          self.s.add_file_at(text, info, 0, base)
        } else {
          self.s.add_file(text, info, 0)
        };
        if std::env::var_os("LESS_RS_LDLOG").is_some() {
          eprintln!("LD adopt {}", resolved.rsplit('/').next().unwrap_or(""));
        }
        self.note_import(&resolved);
        for (w, id) in warnings {
          self.s.warn(w, id);
        }
        return match root {
          Ok(_) if self.manager_error.is_some() => {
            let e = self.manager_error.clone();
            ImportResult {
              shared_error: true,
              ..file_parsed(self, e, None, Some(resolved), Some(fid))
            }
          }
          Ok(root) => file_parsed(
            self,
            None,
            Some(ImportRoot::Tree(root)),
            Some(resolved),
            Some(fid),
          ),
          Err(mut e) => {
            if e.file == Some(crate::syntax::parser::DETACHED_FILE) {
              e.file = Some(fid);
            }
            file_parsed(self, Some(e), None, Some(resolved), Some(fid))
          }
        };
      }
    }
    if std::env::var_os("LESS_RS_LDLOG").is_some() {
      eprintln!("LD main {}", resolved.rsplit('/').next().unwrap_or(""));
    }
    let fid = self.s.add_file(text, info, 0);
    self.note_import(&resolved);
    let mut p = Parser::new(self.s, fid, text, 0);
    match p.parse_root() {
      Ok(_) if self.manager_error.is_some() => {
        let e = self.manager_error.clone();
        ImportResult {
          shared_error: true,
          ..file_parsed(self, e, None, Some(resolved), Some(fid))
        }
      }
      Ok(root) => file_parsed(
        self,
        None,
        Some(ImportRoot::Tree(root)),
        Some(resolved),
        Some(fid),
      ),
      Err(e) => file_parsed(self, Some(e), None, Some(resolved), Some(fid)),
    }
  }

  fn note_import(&self, name: &str) {
    let mut files = self.s.imported_files.borrow_mut();
    if !files.iter().any(|f| f == name) {
      files.push(name.to_string());
    }
  }

  /// `ImportSequencer.tryRun`
  fn try_run(&mut self) {
    if self.sequencing {
      return;
    }
    self.sequencing = true;
    loop {
      while let Some(item) = self.queue.pop_front() {
        match item {
          Queued::Load {
            node,
            ctx,
            try_append,
          } => {
            let result = self.load(node, try_append);
            self.on_imported(node, ctx, result);
            // variable imports wait for the queue to drain:
            // parse their files ahead meanwhile
            if self.variable_imports.len() > self.var_considered {
              self.prefetch_variable_imports();
            }
          }
        }
      }
      if self.variable_imports.len() > self.var_considered {
        self.prefetch_variable_imports();
      }
      match self.variable_imports.pop_front() {
        Some(v) => {
          self.var_considered = self.var_considered.saturating_sub(1);
          self.process_import_node(v.node, v.ctx, v.parent)
        }
        None => break,
      }
    }
    self.sequencing = false;
  }

  /// Parses ahead the files of the pending variable imports whose paths can
  /// be guessed without evaluation (a wrong guess is never adopted).
  fn prefetch_variable_imports(&mut self) {
    let s = self.s;
    let start = self.var_considered;
    self.var_considered = self.variable_imports.len();
    if self.pool.is_none() {
      return;
    }
    let mut paths: Vec<String> = Vec::new();
    for v in self.variable_imports.iter().skip(start) {
      let imp = v.node;
      if imp.options.inline || imp.options.is_plugin {
        continue;
      }
      let Some(path) = guess_import_path(imp, &v.ctx.frames) else {
        continue;
      };
      if is_url(&path) || path.ends_with(".css") {
        continue;
      }
      let dir = s
        .pos_file_info(imp.pos)
        .map(|i| i.current_directory)
        .unwrap_or("");
      if is_url(dir) {
        continue;
      }
      if let Some(full) = first_candidate(s, &path, dir, ".less")
        && (s.file_id(&full).is_none() || imp.options.multiple == Some(true))
      {
        paths.push(full);
      }
    }
    // drop what is already queued
    {
      let pre = s.preparsed.borrow();
      let mut seen: FxHashMap<String, usize> = FxHashMap::default();
      paths.retain(|p| {
        let n = seen.entry(p.clone()).or_insert(0);
        *n += 1;
        *n > pre.get(p).map_or(0, |q| q.len())
      });
    }
    if !paths.is_empty() {
      self.parse_ahead(paths);
    }
  }

  fn on_imported(&mut self, imp: &'a Import<'a>, ctx: Ctx<'a>, r: ImportResult<'a>) {
    if let Some(mut e) = r.error {
      if r.shared_error
        && let Some(m) = &self.manager_error
      {
        e = m.clone();
      }
      if e.pos.is_none() && e.file.is_none() {
        e.set_pos(imp.pos);
      }
      if self.manager_from == Some(imp as *const Import<'a>) {
        self.manager_error = Some(e.clone());
      }
      self.error = Some(e);
    }
    let inline = imp.options.inline;
    let is_plugin = imp.options.is_plugin;
    let is_optional = imp.options.optional;
    let duplicate = r.imported_at_root
      || r
        .full_path
        .as_ref()
        .map(|p| self.recursion_detector.contains(p))
        .unwrap_or(false);
    if !ctx.import_multiple {
      if duplicate {
        imp.skip.set(Skip::Yes);
      } else if let Some(fid) = r.file_id {
        imp.skip.set(Skip::Once(fid));
      } else {
        imp
          .skip
          .set(Skip::Once(u32::MAX - imp.pos.0.min(u32::MAX - 1)));
      }
    }
    if r.full_path.is_none() && is_optional {
      imp.skip.set(Skip::Yes);
    }
    if let Some(root) = r.root {
      imp.root.set(Some(root));
      if let Some(fp) = &r.full_path {
        imp.imported_filename.set(Some(self.s.a.str(fp)));
      }
      imp.imported_file.set(r.file_id);
      if !inline && !is_plugin && (ctx.import_multiple || !duplicate) {
        if let Some(fp) = &r.full_path {
          self.recursion_detector.insert(fp.clone());
        }
        let old = std::mem::replace(&mut self.ctx, ctx);
        if let ImportRoot::Tree(rs) = root
          && let Err(e) = self.visit_ruleset_root(rs)
        {
          self.error = Some(e);
        }
        self.ctx = old;
      }
    }
  }

  fn visit_ruleset_root(&mut self, rs: &'a Ruleset<'a>) -> LResult<()> {
    self.prefetch(rs);
    self.visit_ruleset(rs)
  }
}

/// `Import.isVariableImport()`
fn is_variable_import(imp: &Import<'_>) -> bool {
  let path = match imp.path {
    Node::Url(u) => u.value,
    p => p,
  };
  match path {
    Node::Quoted(q) => {
      let b = q.value.as_bytes();
      (0..b.len())
        .any(|i| b[i] == b'@' && crate::syntax::matchers::interpolation_at(b, i).is_some())
    }
    _ => true,
  }
}
