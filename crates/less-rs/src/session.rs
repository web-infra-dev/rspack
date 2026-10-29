//! Per-compilation state: the arena, the source files and their positions,
//! warnings, and the bits of mutable global state less.js keeps in modules
//! (`default()` state, `importantScope`).

use std::cell::{Cell, RefCell};

use rustc_hash::FxHashMap;

use crate::{
  error::{Error, ErrorKind, LessError, Warning, format_less_error},
  fs::FileSystem,
  options::{Options, RewriteUrls},
  tree::{Alloc, Pos},
};

/// less.js `fileInfo`.
#[derive(Clone, Debug)]
pub struct FileInfo<'a> {
  pub filename: &'a str,
  pub rootpath: &'a str,
  pub current_directory: &'a str,
  pub entry_path: &'a str,
  pub root_filename: &'a str,
  pub reference: bool,
  pub rewrite_urls: RewriteUrls,
}

pub struct SourceFile<'a> {
  pub name: &'a str,
  pub contents: &'a str,
  pub base: u32,
  pub ignored_chars: u32,
  pub info: FileInfo<'a>,
}

/// State of the `default()` function (functions/default.js).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DefaultState {
  Unset,
  Value(bool),
  Error,
}

/// A file parsed ahead of its import (see `ImportVisitor::prefetch`).
/// Path → contents (`None` inside: not readable; `None` outside: being read).
pub(crate) type ReadCache<'a> = std::sync::Mutex<FxHashMap<String, Option<Option<&'a str>>>>;

pub(crate) struct Preparsed<'a> {
  pub base: u32,
  /// The reserved length at `base` (longer texts are not parsed ahead).
  pub max_len: usize,
  pub slot: std::sync::Arc<crate::imports::Slot<'a>>,
}

pub struct Session<'a> {
  pub a: Alloc<'a>,
  /// Stands in for the throwaway `new Ruleset(null, rules)` that
  /// `MixinDefinition.evalCall` evaluates: only its identity (unequal to
  /// every mixin) and its default visibility are ever observed.
  pub plain_original: std::cell::OnceCell<&'a crate::tree::Ruleset<'a>>,
  pub opts: &'a Options,
  pub fs: &'a dyn FileSystem,
  pub functions: &'a crate::functions::Registry,
  files: RefCell<Vec<SourceFile<'a>>>,
  file_ids: RefCell<FxHashMap<String, u32>>,
  /// (base, file id) sorted by base: bases may be reserved before a file
  /// is registered (parallel parsing), so ids are not in base order.
  by_base: RefCell<Vec<(u32, u32)>>,
  next_base: Cell<u32>,
  /// Arenas for parsing imported files on worker threads (one per thread).
  pub(crate) worker_bumps: &'a [bumpalo::Bump],
  /// Files parsed ahead on worker threads, adopted by the first load.
  pub(crate) preparsed: RefCell<FxHashMap<String, std::collections::VecDeque<Preparsed<'a>>>>,
  /// Queued parse-ahead jobs in order, for the main thread to help with
  /// while it waits: (path, base, max length, slot).
  pub(crate) ahead_queue: RefCell<
    std::collections::VecDeque<(String, u32, usize, std::sync::Arc<crate::imports::Slot<'a>>)>,
  >,
  pub warnings: RefCell<Vec<Warning>>,
  pub warning_sink: Option<&'a (dyn Fn(&Warning) + Send + Sync)>,
  // Keep the boxed error representation returned by LResult.
  #[expect(clippy::vec_box)]
  pub import_errors: RefCell<Vec<Box<LessError>>>,
  pub default_state: Cell<DefaultState>,
  pub important_scope: RefCell<Vec<Option<&'a str>>>,
  /// Files imported (for `Output::imports`), in order of first load.
  pub imported_files: RefCell<Vec<String>>,
  /// File contents read during this compilation (`None`: not readable),
  /// shared with the parser threads: each file is read once, and
  /// `@import (multiple)` re-imports share one arena copy.
  pub(crate) read_cache: ReadCache<'a>,
  /// Bumped whenever evaluation captures the frame list (detached
  /// rulesets, mixin definitions): a mixin call's frame and result
  /// containers are dead afterwards unless this changed during the call.
  pub(crate) frames_captured: Cell<u64>,
  /// Evaluation steps, for the arena budget check (`EvalCtx::enter`).
  pub(crate) eval_steps: Cell<u32>,
  /// Arena bytes used after parsing (with imports) and after evaluation.
  pub(crate) arena_phases: Cell<(usize, usize)>,
  /// A visibility block was added; shared declarations inside blocked
  /// subtrees get copies after evaluation (see `unshare_blocked`).
  pub(crate) vis_blocked: Cell<bool>,
  /// Dead `new Ruleset(null, …)` containers (mixin frames and results) for
  /// reuse.
  pub(crate) ruleset_pool: RefCell<Vec<&'a crate::tree::Ruleset<'a>>>,
  /// Interned `DeclName::Str` names (declarations share them).
  decl_names: RefCell<FxHashMap<&'a str, &'a crate::tree::DeclName<'a>>>,
  /// Line start offsets per file id (built on the first warning/error).
  line_index: RefCell<FxHashMap<u32, Vec<u32>>>,
  pub extend_warn_indices: RefCell<rustc_hash::FxHashSet<String>>,
  /// `onceFileDetectionMap` (by file id).
  pub once_files: RefCell<rustc_hash::FxHashSet<u32>>,
}

impl<'a> Session<'a> {
  /// Bytes used in this compilation's arenas (main and parser threads').
  pub(crate) fn arena_used(&self) -> usize {
    let used = |b: &bumpalo::Bump| b.allocated_bytes() - b.chunk_capacity();
    used(self.a.bump) + self.worker_bumps.iter().map(used).sum::<usize>()
  }

  pub fn new(
    a: Alloc<'a>,
    opts: &'a Options,
    fs: &'a dyn FileSystem,
    functions: &'a crate::functions::Registry,
    warning_sink: Option<&'a (dyn Fn(&Warning) + Send + Sync)>,
    worker_bumps: &'a [bumpalo::Bump],
  ) -> Session<'a> {
    Session {
      a,
      plain_original: std::cell::OnceCell::new(),
      opts,
      fs,
      functions,
      files: RefCell::new(Vec::new()),
      file_ids: RefCell::new(FxHashMap::default()),
      by_base: RefCell::new(Vec::new()),
      next_base: Cell::new(0),
      worker_bumps,
      preparsed: RefCell::new(FxHashMap::default()),
      ahead_queue: RefCell::new(std::collections::VecDeque::new()),
      warnings: RefCell::new(Vec::new()),
      warning_sink,
      import_errors: RefCell::new(Vec::new()),
      default_state: Cell::new(DefaultState::Unset),
      important_scope: RefCell::new(Vec::new()),
      imported_files: RefCell::new(Vec::new()),
      read_cache: ReadCache::default(),
      decl_names: RefCell::new(FxHashMap::default()),
      frames_captured: Cell::new(0),
      eval_steps: Cell::new(0),
      arena_phases: Cell::new((0, 0)),
      vis_blocked: Cell::new(false),
      ruleset_pool: RefCell::new(Vec::new()),
      line_index: RefCell::new(FxHashMap::default()),
      extend_warn_indices: RefCell::new(Default::default()),
      once_files: RefCell::new(Default::default()),
    }
  }

  /// Registers (or re-registers) a file; returns its id.
  pub fn add_file(&self, contents: &'a str, info: FileInfo<'a>, ignored_chars: u32) -> u32 {
    let name = info.filename;
    if let Some(&id) = self.file_ids.borrow().get(name) {
      let mut files = self.files.borrow_mut();
      let f = &mut files[id as usize];
      if f.contents.len() >= contents.len() {
        f.contents = contents;
        f.info = info;
        f.ignored_chars = ignored_chars;
        return id;
      }
    }
    let base = self.reserve_base(contents.len());
    self.add_file_at(contents, info, ignored_chars, base)
  }

  pub fn next_base(&self) -> u32 {
    self.next_base.get()
  }

  /// Reserves a position range for a file of `len` bytes.
  pub fn reserve_base(&self, len: usize) -> u32 {
    let base = self.next_base.get();
    self.next_base.set(base + len as u32 + 1);
    base
  }

  /// Registers a new file at a reserved base.
  pub fn add_file_at(
    &self,
    contents: &'a str,
    info: FileInfo<'a>,
    ignored_chars: u32,
    base: u32,
  ) -> u32 {
    let name = info.filename;
    let mut files = self.files.borrow_mut();
    let id = files.len() as u32;
    files.push(SourceFile {
      name,
      contents,
      base,
      ignored_chars,
      info,
    });
    self.file_ids.borrow_mut().insert(name.to_string(), id);
    let mut by_base = self.by_base.borrow_mut();
    let at = by_base.partition_point(|&(b, _)| b < base);
    by_base.insert(at, (base, id));
    id
  }

  pub fn file_id(&self, name: &str) -> Option<u32> {
    self.file_ids.borrow().get(name).copied()
  }

  /// The shared `DeclName::Str(name)`.
  pub fn decl_name(&self, name: &'a str) -> &'a crate::tree::DeclName<'a> {
    if let Some(n) = self.decl_names.borrow().get(name) {
      return n;
    }
    let n: &'a crate::tree::DeclName<'a> = self.a.alloc(crate::tree::DeclName::Str(name));
    self.decl_names.borrow_mut().insert(name, n);
    n
  }

  /// Total length of the distinct source files (an output size hint).
  pub fn source_len(&self) -> usize {
    self.files.borrow().iter().map(|f| f.contents.len()).sum()
  }
  pub fn file_count(&self) -> usize {
    self.files.borrow().len()
  }

  pub fn file_base(&self, id: u32) -> u32 {
    self.files.borrow()[id as usize].base
  }
  pub fn file_info(&self, id: u32) -> FileInfo<'a> {
    self.files.borrow()[id as usize].info.clone()
  }
  pub fn file_contents(&self, id: u32) -> &'a str {
    self.files.borrow()[id as usize].contents
  }
  pub fn file_name(&self, id: u32) -> &'a str {
    self.files.borrow()[id as usize].name
  }
  pub fn file_ignored_chars(&self, id: u32) -> u32 {
    self.files.borrow()[id as usize].ignored_chars
  }

  /// Global position → (file id, file-local byte index).
  pub fn locate(&self, pos: Pos) -> Option<(u32, u32)> {
    if pos.is_none() {
      return None;
    }
    let by_base = self.by_base.borrow();
    let i = match by_base.binary_search_by(|&(b, _)| b.cmp(&pos.0)) {
      Ok(i) => i,
      Err(0) => return None,
      Err(i) => i - 1,
    };
    let (base, id) = by_base[i];
    Some((id, pos.0 - base))
  }

  pub fn pos_file_info(&self, pos: Pos) -> Option<FileInfo<'a>> {
    self.locate(pos).map(|(f, _)| self.file_info(f))
  }

  pub fn warn(&self, message: String, id: Option<&'static str>) {
    let w = Warning { id, message };
    if let Some(sink) = self.warning_sink {
      sink(&w);
    }
    self.warnings.borrow_mut().push(w);
  }

  /// `new LessError({...}).toString()` for a warning at a position.
  pub fn format_positioned(&self, typ: &str, message: &str, pos: Pos) -> String {
    let (filename, line, col, extract) = self.position_details(Some(pos), None);
    format_less_error(typ, message, filename.as_deref(), line, col, &extract)
  }

  fn position_details(
    &self,
    pos: Option<Pos>,
    file: Option<u32>,
  ) -> (
    Option<String>,
    Option<u32>,
    Option<u32>,
    [Option<String>; 3],
  ) {
    let mut filename = None;
    let mut line = None;
    let mut col = None;
    let mut extract = [None, None, None];
    let loc = pos.and_then(|p| self.locate(p));
    let fid = loc.map(|l| l.0).or(file);
    if let Some(fid) = fid {
      filename = Some(self.file_name(fid).to_string());
      let text = self.file_contents(fid);
      if let Some((_, idx)) = loc {
        let mut index = self.line_index.borrow_mut();
        let starts = index.entry(fid).or_insert_with(|| line_starts(text));
        let (l, c, x) = details_in(text, starts, idx as usize);
        line = Some(l);
        col = Some(c);
        extract = x;
      }
    }
    (filename, line, col, extract)
  }

  /// Converts an internal error into the public error type.
  pub fn to_public_error(&self, e: &LessError) -> Error {
    let (filename, line, col, extract) = self.position_details(e.pos, e.file);
    Error {
      kind: ErrorKind::from_type(if e.typ.is_empty() { "Syntax" } else { &e.typ }),
      type_name: if e.typ.is_empty() {
        "Syntax".to_string()
      } else {
        e.typ.clone()
      },
      message: e.message.clone(),
      filename,
      line,
      column: col,
      index: e.pos.and_then(|p| self.locate(p)).map(|(_, i)| i),
      extract,
    }
  }
}

/// `utils.getLocation` on a byte index; the column is measured in UTF-16
/// code units.
/// `format_positioned` for a file that is not registered (yet): `idx` is the
/// byte index in `text`.
pub(crate) fn format_positioned_in(
  typ: &str,
  message: &str,
  name: &str,
  text: &str,
  idx: usize,
) -> String {
  format_positioned_with(typ, message, name, text, &line_starts(text), idx)
}

/// [`format_positioned_in`] with the text's [`line_starts`].
pub(crate) fn format_positioned_with(
  typ: &str,
  message: &str,
  name: &str,
  text: &str,
  starts: &[u32],
  idx: usize,
) -> String {
  let (line, col, extract) = details_in(text, starts, idx);
  format_less_error(typ, message, Some(name), Some(line), Some(col), &extract)
}

pub(crate) fn line_starts(text: &str) -> Vec<u32> {
  let mut v = vec![0u32];
  v.extend(memchr_iter_nl(text));
  v
}

/// 1-based line, UTF-16 column and the 3-line extract at `idx`.
fn details_in(text: &str, starts: &[u32], idx: usize) -> (u32, u32, [Option<String>; 3]) {
  let idx = floor_char_boundary(text, idx.min(text.len()));
  let li = match starts.binary_search(&(idx as u32)) {
    Ok(i) => i,
    Err(i) => i - 1,
  };
  let c = text[starts[li] as usize..idx].encode_utf16().count() as u32;
  let l = li as i64 + 1;
  // `text.split('\n')[i]`
  let get = |i: i64| -> Option<String> {
    if i >= 0 && (i as usize) < starts.len() {
      let start = starts[i as usize] as usize;
      let end = starts
        .get(i as usize + 1)
        .map_or(text.len(), |&e| e as usize - 1);
      Some(text[start..end].to_string())
    } else {
      None
    }
  };
  (li as u32 + 1, c, [get(l - 2), get(l - 1), get(l)])
}

/// Offsets just past each `\n`.
fn memchr_iter_nl(text: &str) -> impl Iterator<Item = u32> + '_ {
  memchr::memchr_iter(b'\n', text.as_bytes()).map(|i| i as u32 + 1)
}

pub fn line_col_in(text: &str, idx: usize) -> (u32, u32) {
  let before = &text[..floor_char_boundary(text, idx)];
  let line = before.bytes().filter(|&b| b == b'\n').count() as u32;
  let line_start = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
  let col = before[line_start..].encode_utf16().count() as u32;
  (line, col)
}

pub fn floor_char_boundary(s: &str, mut i: usize) -> usize {
  if i >= s.len() {
    return s.len();
  }
  while !s.is_char_boundary(i) {
    i -= 1;
  }
  i
}
