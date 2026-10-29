//! # less-rs
//!
//! A Rust port of the [Less](https://lesscss.org) CSS pre-processor with
//! less.js 4.9.1 semantics: byte-identical CSS output, the same error
//! messages, and the same option names.
//!
//! ```
//! let out = less_rs::compile("@c: red; a { color: @c }", &less_rs::Options::default()).unwrap();
//! assert_eq!(out.css, "a {\n  color: red;\n}\n");
//! ```
//!
//! For repeated compilations, configure a [`Compiler`] once:
//!
//! ```
//! use less_rs::{Compiler, Math};
//!
//! let compiler = Compiler::builder()
//!   .math(Math::Always)
//!   .global_var("brand", "#f01")
//!   .build();
//! let out = compiler
//!   .compile_str(".a { color: @brand; width: 10px / 2 }", "inline.less")
//!   .unwrap();
//! assert_eq!(out.css, ".a {\n  color: #f01;\n  width: 5px;\n}\n");
//! ```
//!
//! Everything a host configures, in one place (the plan's API sketch):
//!
//! ```
//! use less_rs::{
//!   Compiler, ErrorKind, Math, MemFs, RewriteUrls, SourceMapOptions, Value, ValueArgs,
//! };
//!
//! struct MyPlugin;
//! impl less_rs::Plugin for MyPlugin {
//!   fn install(&self, r: &mut less_rs::functions::Registry) {
//!     r.add_function(
//!       "triple",
//!       std::sync::Arc::new(|args: &[Value]| -> less_rs::FnResult {
//!         let d = args.dimension(0)?;
//!         Ok(Value::dimension(d.value * 3.0, d.unit))
//!       }),
//!     );
//!   }
//! }
//!
//! let compiler = Compiler::builder()
//!   .include_paths(["node_modules", "src/styles"])
//!   .math(Math::ParensDivision)
//!   .rewrite_urls(RewriteUrls::Local)
//!   .source_map(SourceMapOptions::external("app.css.map").with_sources_content())
//!   .global_var("brand", "#f01")
//!   .modify_var("radius", "4px")
//!   .function("double", |args: &[Value]| -> less_rs::FnResult {
//!     let d = args.dimension(0)?;
//!     Ok(Value::dimension(d.value * 2.0, d.unit))
//!   })
//!   .plugin(MyPlugin)
//!   .file_system(MemFs::from([(
//!     "src/styles/a.less",
//!     "@radius: 0; .a { r: @radius; w: double(3px); h: triple(1em) }",
//!   )]))
//!   .on_warning(|w| eprintln!("{w}"))
//!   .build();
//!
//! let out = compiler
//!   .compile_str("@import 'a'; .b { color: @brand }", "app.less")
//!   .unwrap();
//! assert!(
//!   out
//!     .css
//!     .starts_with(".a {\n  r: 4px;\n  w: 6px;\n  h: 3em;\n}\n.b {\n  color: #f01;\n}\n")
//! );
//! assert!(out.map.is_some());
//! assert_eq!(out.imports, ["src/styles/a.less"]);
//!
//! match Compiler::default().compile_str(".a { color: red;\n.b {}", "broken.less") {
//!   Err(e) => {
//!     assert_eq!(e.kind(), ErrorKind::Parse);
//!     let (line, _col) = e.location().unwrap();
//!     assert_eq!(line, 2);
//!   }
//!   Ok(_) => unreachable!(),
//! }
//! ```
//!
//! The arena that holds the syntax tree never leaks into the public API:
//! [`Output`] and [`Error`] are owned values, and [`Compiler`] is
//! `Send + Sync`.

#![allow(clippy::too_many_arguments, clippy::new_ret_no_self)]
// The tree mirrors less.js field-for-field; a few fields and helpers exist
// for parity even where this implementation does not read them.
#![allow(dead_code)]
#![allow(
  clippy::result_large_err,
  clippy::needless_range_loop,
  clippy::type_complexity,
  clippy::inherent_to_string,
  clippy::wrong_self_convention,
  clippy::unnecessary_unwrap,
  clippy::manual_clamp,
  clippy::while_let_loop
)]

mod color;
mod css;
mod data;
mod error;
mod eval;
pub mod fs;
pub mod functions;
mod imports;
mod num;
mod options;
pub mod plugin;
mod session;
#[cfg(feature = "sourcemap")]
mod sourcemap;
mod syntax;
mod tree;
mod visitors;

use std::sync::Arc;

use cow_utils::CowUtils;
pub use error::{Error, ErrorKind, Warning};

/// Internals exposed for differential testing (`cargo xtask jsdiff`); not a
/// stable API.
#[doc(hidden)]
pub mod __private {
  pub use crate::{
    num::{fround, to_fixed, to_js_string},
    syntax::matchers::*,
  };
}
pub use fs::{FileSystem, MemFs, StdFs, UrlError};
pub use options::{DumpLineNumbers, Math, Options, RewriteUrls, SourceMapOptions};
pub use plugin::{
  DimensionArg, FnResult, LessFunction, Plugin, PostProcessor, PreProcessor, Value, ValueArgs,
};

use crate::{
  css::CssCtx,
  session::{FileInfo, Session},
  tree::Alloc,
};

/// The result of a successful compilation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Output {
  /// The generated CSS.
  pub css: String,
  /// The source map JSON, when source maps are enabled.
  pub map: Option<String>,
  /// Files imported during the compilation (excluding the root file).
  pub imports: Vec<String>,
  /// Warnings (deprecations, unmatched extends, ...).
  pub warnings: Vec<Warning>,
  /// Compilation statistics.
  pub stats: Stats,
}

/// Statistics about a compilation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Stats {
  /// Bytes used in the compilation arena (syntax trees, evaluated tree,
  /// strings); released in one go when the compilation ends.
  pub arena_bytes: usize,
  /// Number of source files read (root plus imports).
  pub files: usize,
  /// Arena bytes in use once all files are parsed (imports included).
  pub parse_arena_bytes: usize,
  /// Arena bytes in use once the tree is evaluated.
  pub eval_arena_bytes: usize,
}

/// The result of [`parse`]: the file was syntactically valid and all its
/// imports resolved.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ParsedFile {
  imports: Vec<String>,
  warnings: Vec<Warning>,
  source: String,
  filename: String,
}

impl ParsedFile {
  /// Files imported (transitively), excluding the root file.
  pub fn imports(&self) -> &[String] {
    &self.imports
  }
  /// Parse-time warnings (deprecations).
  pub fn warnings(&self) -> &[Warning] {
    &self.warnings
  }
  /// Compiles the parsed file with `compiler`'s configuration.
  ///
  /// ```
  /// let parsed = less_rs::parse("@w: 2px; .a { width: @w * 2 }", "a.less").unwrap();
  /// let out = parsed.compile(&less_rs::Compiler::default()).unwrap();
  /// assert_eq!(out.css, ".a {\n  width: 4px;\n}\n");
  /// ```
  pub fn compile(&self, compiler: &Compiler) -> Result<Output, Error> {
    compiler.compile_str(&self.source, &self.filename)
  }
}

/// Parses `input` and resolves its imports without evaluating it (syntax
/// validation for linters and dependency scanners).
///
/// ```
/// let parsed = less_rs::parse(".a { color: red }", "a.less").unwrap();
/// assert!(parsed.imports().is_empty());
/// assert!(less_rs::parse(".a { color: red", "a.less").is_err());
/// ```
pub fn parse(input: &str, filename: &str) -> Result<ParsedFile, Error> {
  Compiler::default().parse_str(input, filename)
}

/// Compiles Less source with the given options.
pub fn compile(input: &str, options: &Options) -> Result<Output, Error> {
  Compiler::with_options(options.clone())
    .compile_str(input, options.filename.as_deref().unwrap_or("input"))
}

type WarningSink = Arc<dyn Fn(&Warning) + Send + Sync>;

/// A configured, reusable and thread-safe compiler.
#[derive(Clone)]
pub struct Compiler {
  options: Options,
  fs: Arc<dyn FileSystem>,
  registry: Arc<functions::Registry>,
  on_warning: Option<WarningSink>,
  /// Reset arenas kept for reuse (avoids re-faulting fresh memory on every
  /// compilation; each compilation still starts from an empty arena).
  arenas: Arc<std::sync::Mutex<Vec<bumpalo::Bump>>>,
  /// Reset per-thread arenas for parsing imports in parallel.
  worker_arenas: Arc<std::sync::Mutex<Vec<bumpalo::Bump>>>,
}

/// Threads used to parse imported files in parallel.
fn parse_threads() -> usize {
  static N: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
  *N.get_or_init(|| {
    if cfg!(target_arch = "wasm32") {
      return 1;
    }
    if let Some(n) = std::env::var("LESS_RS_THREADS")
      .ok()
      .and_then(|v| v.parse::<usize>().ok())
    {
      return n.max(1);
    }
    // reads stay sequential (they do not scale), so a couple of parser
    // threads keep up with them
    std::thread::available_parallelism()
      .map(|n| n.get())
      .unwrap_or(1)
      .min(3)
  })
}

impl std::fmt::Debug for Compiler {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Compiler")
      .field("options", &self.options)
      .finish()
  }
}

impl Default for Compiler {
  fn default() -> Compiler {
    Compiler::with_options(Options::default())
  }
}

impl Compiler {
  pub fn builder() -> CompilerBuilder {
    CompilerBuilder::default()
  }

  pub fn with_options(options: Options) -> Compiler {
    Compiler {
      options,
      fs: Arc::new(StdFs),
      registry: Arc::new(functions::Registry::with_builtins()),
      on_warning: None,
      arenas: Default::default(),
      worker_arenas: Default::default(),
    }
  }

  pub fn options(&self) -> &Options {
    &self.options
  }

  /// Reads and compiles a file.
  pub fn compile_file(&self, path: impl AsRef<std::path::Path>) -> Result<Output, Error> {
    let p = path.as_ref();
    let name = p.to_string_lossy().to_string();
    let src = self.fs.read(p).map_err(|e| {
      Error::simple(
        ErrorKind::Io,
        "File",
        format!("'{}' wasn't found. {}", name, e),
      )
    })?;
    self.compile_str(&src, &name)
  }

  /// Parses and resolves imports without evaluating (see [`parse`]).
  pub fn parse_str(&self, input: &str, filename: &str) -> Result<ParsedFile, Error> {
    let mut opts = self.options.clone();
    opts.filename = Some(filename.to_string());
    let bump = bumpalo::Bump::with_capacity(input.len() * 4);
    let a = Alloc { bump: &bump };
    let s = Session::new(
      a,
      &opts,
      &*self.fs,
      &self.registry,
      self.on_warning.as_deref(),
      &[],
    );
    let r = run_stage(&s, input, Stage::Parse);
    let warnings = s.warnings.borrow().clone();
    match r {
      Ok(_) => Ok(ParsedFile {
        imports: s.imported_files.borrow().clone(),
        warnings,
        source: input.to_string(),
        filename: filename.to_string(),
      }),
      Err(e) => Err(s.to_public_error(&e)),
    }
  }

  /// Compiles a string; `filename` is used for errors, relative imports and
  /// source maps.
  pub fn compile_str(&self, input: &str, filename: &str) -> Result<Output, Error> {
    let mut opts = self.options.clone();
    opts.filename = Some(filename.to_string());
    let pooled = self.arenas.lock().ok().and_then(|mut p| p.pop());
    let mut bump =
      pooled.unwrap_or_else(|| bumpalo::Bump::with_capacity((input.len() * 26).max(64 * 1024)));
    let sink = self.on_warning.clone();
    let sink_ref: Option<&(dyn Fn(&Warning) + Send + Sync)> = sink.as_deref();
    let threads = parse_threads();
    let mut workers: Vec<bumpalo::Bump> = Vec::new();
    if threads > 1 {
      if let Ok(mut p) = self.worker_arenas.lock() {
        let len = p.len();
        workers = p.split_off(len - len.min(threads));
      }
      workers.resize_with(threads, bumpalo::Bump::new);
    }
    let result = run(
      &bump,
      &workers,
      &opts,
      &*self.fs,
      &self.registry,
      sink_ref,
      input,
    );
    bump.reset();
    if let Ok(mut p) = self.arenas.lock()
      && p.len() < 2
    {
      p.push(bump);
    }
    if !workers.is_empty() {
      for w in workers.iter_mut() {
        w.reset();
      }
      if let Ok(mut p) = self.worker_arenas.lock()
        && p.len() < 2 * threads
      {
        p.extend(workers);
      }
    }
    result
  }
}

/// Builder for [`Compiler`].
#[derive(Default)]
pub struct CompilerBuilder {
  options: Options,
  fs: Option<Arc<dyn FileSystem>>,
  registry: Option<functions::Registry>,
  on_warning: Option<WarningSink>,
}

impl CompilerBuilder {
  pub fn options(mut self, o: Options) -> Self {
    self.options = o;
    self
  }
  pub fn include_paths<I, S>(mut self, paths: I) -> Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self.options.include_paths = paths.into_iter().map(Into::into).collect();
    self
  }
  pub fn math(mut self, m: Math) -> Self {
    self.options.math = m;
    self
  }
  pub fn compress(mut self, c: bool) -> Self {
    self.options.compress = c;
    self
  }
  pub fn strict_units(mut self, s: bool) -> Self {
    self.options.strict_units = s;
    self
  }
  pub fn rewrite_urls(mut self, r: RewriteUrls) -> Self {
    self.options.rewrite_urls = r;
    self
  }
  pub fn rootpath(mut self, r: impl Into<String>) -> Self {
    self.options.rootpath = r.into();
    self
  }
  pub fn url_args(mut self, a: impl Into<String>) -> Self {
    self.options.url_args = a.into();
    self
  }
  pub fn source_map(mut self, s: SourceMapOptions) -> Self {
    self.options.source_map = Some(s);
    self
  }
  pub fn global_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
    self.options.global_vars.push((name.into(), value.into()));
    self
  }
  pub fn modify_var(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
    self.options.modify_vars.push((name.into(), value.into()));
    self
  }
  pub fn banner(mut self, b: impl Into<String>) -> Self {
    self.options.banner = Some(b.into());
    self
  }
  /// Registers a function callable from Less code.
  pub fn function<F>(mut self, name: &str, f: F) -> Self
  where
    F: Fn(&[Value]) -> FnResult + Send + Sync + 'static,
  {
    self
      .registry
      .get_or_insert_with(functions::Registry::with_builtins)
      .add_function(name, Arc::new(f));
    self
  }
  pub fn plugin(mut self, p: impl Plugin) -> Self {
    p.install(
      self
        .registry
        .get_or_insert_with(functions::Registry::with_builtins),
    );
    self
  }
  pub fn file_system(mut self, fs: impl FileSystem + 'static) -> Self {
    self.fs = Some(Arc::new(fs));
    self
  }
  pub fn pre_processor(mut self, p: impl PreProcessor + 'static) -> Self {
    self
      .registry
      .get_or_insert_with(functions::Registry::with_builtins)
      .add_pre_processor(Arc::new(p));
    self
  }
  pub fn post_processor(mut self, p: impl PostProcessor + 'static) -> Self {
    self
      .registry
      .get_or_insert_with(functions::Registry::with_builtins)
      .add_post_processor(Arc::new(p));
    self
  }
  pub fn on_warning(mut self, f: impl Fn(&Warning) + Send + Sync + 'static) -> Self {
    self.on_warning = Some(Arc::new(f));
    self
  }
  pub fn build(self) -> Compiler {
    Compiler {
      options: self.options,
      fs: self.fs.unwrap_or_else(|| Arc::new(StdFs)),
      registry: Arc::new(
        self
          .registry
          .unwrap_or_else(functions::Registry::with_builtins),
      ),
      on_warning: self.on_warning,
      arenas: Default::default(),
      worker_arenas: Default::default(),
    }
  }
}

fn run<'a>(
  bump: &'a bumpalo::Bump,
  workers: &'a [bumpalo::Bump],
  opts: &'a Options,
  fs: &'a dyn FileSystem,
  registry: &'a functions::Registry,
  sink: Option<&'a (dyn Fn(&Warning) + Send + Sync)>,
  input: &'a str,
) -> Result<Output, Error> {
  let a = Alloc { bump };
  let s = Session::new(a, opts, fs, registry, sink, workers);
  let result = run_inner(&s, input);
  let warnings = s.warnings.borrow().clone();
  match result {
    Ok((mut css, map)) => {
      for p in &registry.post {
        css = p.process(css);
      }
      let imports = s.imported_files.borrow().clone();
      let (parse_arena_bytes, eval_arena_bytes) = s.arena_phases.get();
      let stats = Stats {
        arena_bytes: s.arena_used(),
        files: s.file_count(),
        parse_arena_bytes,
        eval_arena_bytes,
      };
      Ok(Output {
        css,
        map,
        imports,
        warnings,
        stats,
      })
    }
    Err(e) => Err(s.to_public_error(&e)),
  }
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum Stage {
  Parse,
  Full,
}

fn run_inner<'a>(s: &Session<'a>, input: &'a str) -> error::LResult<(String, Option<String>)> {
  run_stage(s, input, Stage::Full)
}

/// A timestamp for `LESS_RS_TIMING`, taken only when timing (there is no
/// clock on `wasm32-unknown-unknown`, where `Instant::now` panics).
fn stamp(timing: bool) -> Option<std::time::Instant> {
  timing.then(std::time::Instant::now)
}

fn span(a: Option<std::time::Instant>, b: Option<std::time::Instant>) -> std::time::Duration {
  match (a, b) {
    (Some(a), Some(b)) => b - a,
    _ => std::time::Duration::ZERO,
  }
}

fn run_stage<'a>(
  s: &Session<'a>,
  input: &'a str,
  stage: Stage,
) -> error::LResult<(String, Option<String>)> {
  let a = s.a;
  let opts = s.opts;
  let filename = opts.filename.clone().unwrap_or_else(|| "input".to_string());
  let entry_path = match filename.rfind(['/', '\\']) {
    Some(p) => filename[..p + 1].to_string(),
    None => String::new(),
  };
  let mut rootpath = opts.rootpath.clone();
  if !rootpath.is_empty() && !rootpath.ends_with('/') {
    rootpath.push('/');
  }
  let info = FileInfo {
    filename: a.str(&filename),
    rootpath: a.str(&rootpath),
    current_directory: a.str(&entry_path),
    entry_path: a.str(&entry_path),
    root_filename: a.str(&filename),
    reference: false,
    rewrite_urls: opts.rewrite_urls,
  };
  // globalVars / banner are prepended, modifyVars appended.
  let global_vars = if opts.global_vars.is_empty() {
    String::new()
  } else {
    format!("{}\n", options::serialize_vars(&opts.global_vars))
  };
  let modify_vars = if opts.modify_vars.is_empty() {
    String::new()
  } else {
    format!("\n{}", options::serialize_vars(&opts.modify_vars))
  };
  let mut pre_text = String::new();
  if !global_vars.is_empty() || opts.banner.is_some() {
    pre_text = format!("{}{}", opts.banner.clone().unwrap_or_default(), global_vars);
  }
  let original: &'a str = input;
  let preprocessed = s.functions.preprocess(input, &filename);
  let input: &str = preprocessed.as_deref().unwrap_or(input);
  // the caller's text is parsed in place when nothing is rewritten
  let borrowed: Option<&'a str> = match &preprocessed {
    None if !original.contains('\r') => Some(original.strip_prefix('\u{feff}').unwrap_or(original)),
    _ => None,
  };
  let normalized;
  let body: &str = if input.contains('\r') {
    normalized = input
      .cow_replace("\r\n", "\n")
      .cow_replace('\r', "\n")
      .into_owned();
    &normalized
  } else {
    input
  };
  let body = body.strip_prefix('\u{feff}').unwrap_or(body);
  let text: &'a str = match (pre_text.is_empty() && modify_vars.is_empty(), borrowed) {
    (true, Some(b)) => b,
    (true, None) => a.str(body),
    (false, _) => a.str(&format!("{}{}{}", pre_text, body, modify_vars)),
  };
  let timing = std::env::var_os("LESS_RS_TIMING").is_some();
  let t0 = stamp(timing);
  let fid = s.add_file(text, info, pre_text.len() as u32);
  let mut parser = syntax::parser::Parser::new(s, fid, text, 0);
  let root = parser.parse_root()?;
  if opts.process_imports {
    imports::run_imports(s, root, &filename)?;
  }
  if stage == Stage::Parse {
    return Ok((String::new(), None));
  }
  let t1 = stamp(timing);
  let m1 = a.bump.allocated_bytes();
  s.arena_phases.set((s.arena_used(), 0));
  #[cfg(feature = "arena-stats")]
  if timing {
    tree::stats::dump("parse");
    eprintln!(
      "arena used after parse {} KB",
      (a.bump.allocated_bytes() - a.bump.chunk_capacity()) / 1024
    );
  }
  // transformTree
  let mut ctx = eval::EvalCtx::new(s, Vec::new());
  let evald = ctx.eval_ruleset(root)?;
  let t2 = stamp(timing);
  let m2 = a.bump.allocated_bytes();
  s.arena_phases.set((s.arena_phases.get().0, s.arena_used()));
  #[cfg(feature = "arena-stats")]
  if timing {
    tree::stats::dump("eval");
    eprintln!(
      "arena used after eval {} KB",
      (a.bump.allocated_bytes() - a.bump.chunk_capacity()) / 1024
    );
  }
  if s.vis_blocked.get() {
    visitors::unshare_blocked(a, tree::Node::Ruleset(evald), false);
  }
  visitors::join_selectors(a, evald, error::arena_budget(s.source_len()))?;
  visitors::mark_visible(evald);
  let t3 = stamp(timing);
  let mut ev = visitors::ExtendVisitor::new(s);
  ev.run(evald)?;
  let t4 = stamp(timing);
  // its buffers are not needed while the CSS is written
  visitors::ToCssVisitor::new(s).run(evald)?;
  let t5 = stamp(timing);
  #[cfg(feature = "arena-stats")]
  if timing {
    tree::stats::dump("join+extend+tocss");
    eprintln!(
      "arena used after tocss {} KB",
      (a.bump.allocated_bytes() - a.bump.chunk_capacity()) / 1024
    );
  }
  if timing {
    eprintln!(
      "parse+imports {:?} eval {:?} join+vis {:?} extend {:?} tocss {:?} arena {} KB (parse {} KB, eval {} KB)",
      span(t0, t1),
      span(t1, t2),
      span(t2, t3),
      span(t3, t4),
      span(t4, t5),
      (a.bump.allocated_bytes() - a.bump.chunk_capacity()) / 1024,
      m1 / 1024,
      (m2 - m1) / 1024
    );
  }
  if opts.compress {
    s.warn(
            "The compress option has been deprecated. We recommend you use a dedicated css minifier, for instance see less-plugin-clean-css."
                .to_string(),
            None,
        );
  }
  let mut cctx = CssCtx {
    compress: opts.compress,
    strict_units: opts.strict_units,
    num_precision: Some(8),
    dump_line_numbers: opts.dump_line_numbers,
    output_hint: {
      let n = s.source_len().max(input.len());
      n + n / 4
    },
    ..CssCtx::default()
  };
  match &opts.source_map {
    #[cfg(feature = "sourcemap")]
    Some(sm) => sourcemap::generate(s, evald, &mut cctx, sm, &filename),
    #[cfg(not(feature = "sourcemap"))]
    Some(_) => Err(error::LessError::new(
      "Runtime",
      "source maps need less-rs's `sourcemap` feature",
    )),
    None => {
      let css = css::to_css(tree::Node::Ruleset(evald), &mut cctx)?;
      Ok((css, None))
    }
  }
}
