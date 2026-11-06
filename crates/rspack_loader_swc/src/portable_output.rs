//! Audited subset of SWC options for portable output. Unknown fields decline certification.
//!
//! The compiler uses only the resource extension for its fallback config, never a config-file
//! search. The allowed parser/transpiler/React JSX passes have no filename-derived output
//! except the JSX development filename literal. Explicit `env.targets` versions select passes
//! without reading the filename or a browserslist config. Plugins, other env fields, module
//! conversion, custom refresh options, import rewriting, and custom paths are intentionally
//! excluded.

use std::{io, path::Path, sync::Arc};

use rspack_core::RunnerContext;
use rspack_fs::{FsResultToIoResultExt, ReadableFileSystem};
use rspack_javascript_compiler::transform::{
  InputSourceMapRequest, InputSourceMapResolver, InputSourceMapResult,
};
use rspack_loader_runner::LoaderContext;
use rspack_paths::{Utf8Path, Utf8PathBuf};
use rustc_hash::FxHashSet;
use serde_json::{Map, Value};
use swc_core::{
  base::config::InputSourceMap,
  common::FileName,
  ecma::{
    ast::{BigInt, Bool, JSXText, Null, Number, Program, Regex, Str, TplElement, TsEnumMember},
    utils::number::ToJsString,
    visit::{Visit, VisitWith},
  },
};

/// Records the I/O that selected the map, including failed higher-priority candidates.
/// Never turns a failed metadata operation into proof of absence unless it is NotFound.
pub(crate) struct InputSourceMapAcquisition {
  fs: Arc<dyn ReadableFileSystem>,
  files: Vec<Utf8PathBuf>,
  missing: Vec<Utf8PathBuf>,
  automatic_no_marker: bool,
  uncertain: bool,
}

impl InputSourceMapAcquisition {
  pub(crate) fn new(fs: Arc<dyn ReadableFileSystem>) -> Self {
    Self {
      fs,
      files: Vec::new(),
      missing: Vec::new(),
      automatic_no_marker: false,
      uncertain: false,
    }
  }

  fn path<'a>(&mut self, path: &'a Path) -> io::Result<&'a Utf8Path> {
    Utf8Path::from_path(path).ok_or_else(|| {
      self.uncertain = true;
      io::Error::new(
        io::ErrorKind::InvalidInput,
        "input source map path is not UTF-8",
      )
    })
  }

  fn observe<T>(&mut self, path: &Utf8Path, result: &io::Result<T>) {
    match result {
      Ok(_) => self.files.push(path.to_path_buf()),
      Err(error) if error.kind() == io::ErrorKind::NotFound => {
        self.missing.push(path.to_path_buf());
      }
      Err(_) => {
        self.files.push(path.to_path_buf());
        self.uncertain = true;
      }
    }
  }

  pub(crate) async fn acquire(
    &mut self,
    source: &str,
    name: &FileName,
    option: &InputSourceMap,
  ) -> InputSourceMapResult {
    let mut resolver = InputSourceMapResolver::new(source, name, option);
    self.automatic_no_marker = resolver.automatic_lookup() == Some(false);
    while let Some(request) = resolver.next_request() {
      match request {
        InputSourceMapRequest::Metadata(path) => {
          let found = self.metadata(&path).await.is_ok();
          resolver.metadata_result(path, found);
        }
        InputSourceMapRequest::Read(path) => resolver.read_result(self.read(&path).await),
      }
    }
    resolver.finish()
  }

  pub(crate) fn register_dependencies(&self, context: &mut LoaderContext<RunnerContext>) {
    for path in &self.files {
      context.add_file_dependency(path.clone());
    }
    for path in &self.missing {
      context.add_missing_dependency(path.clone());
    }
    if self.uncertain {
      // An unreadable probe can affect fallback selection without a reliable snapshot.
      context.cacheable = false;
    }
  }

  pub(crate) fn is_automatic_absence(&self, resource: &Utf8Path) -> bool {
    self.automatic_no_marker
      && !self.uncertain
      && self.files.is_empty()
      && self.missing.len() == 1
      && self.missing[0].as_str() == format!("{resource}.map")
  }

  pub(crate) async fn absence_is_unambiguous(&self) -> bool {
    // metadata follows symlinks. A dangling symlink is not a definitely absent entry.
    // This refines the actual negative observation; map selection is never repeated.
    matches!(
      self.fs.symlink_metadata(&self.missing[0]).await.to_io_result(),
      Err(error) if error.kind() == io::ErrorKind::NotFound
    )
  }

  async fn metadata(&mut self, path: &Path) -> io::Result<()> {
    let path = self.path(path)?;
    let result = self.fs.metadata(path).await.to_io_result().map(|_| ());
    self.observe(path, &result);
    result
  }

  async fn read(&mut self, path: &Path) -> io::Result<Vec<u8>> {
    let path = self.path(path)?;
    let result = self.fs.read(path).await.to_io_result();
    self.observe(path, &result);
    result
  }
}

const NUMBER_TEXT: [&str; 16] = [
  "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", ".", "-", "+", "e", "Infinity", "NaN",
];

/// Only SWC-generated filename literals belong to this invocation's declaration. An authored
/// value equal to the resource must stay fixed when a checkout moves, and transforms can build
/// that value from several literals. So the question covers every concatenation of literal text
/// rather than each transform that concatenates.
pub(crate) fn input_can_spell_resource_path(program: &Program, resource: &str) -> bool {
  let mut visitor = ResourceFragments {
    resource,
    fragments: FxHashSet::default(),
    declined: false,
  };
  program.visit_with(&mut visitor);
  visitor.declined || visitor.spell_resource()
}

struct ResourceFragments<'a> {
  resource: &'a str,
  /// Literal text occurring in the resource, as slices of it because a visitor can't keep AST
  /// borrows.
  fragments: FxHashSet<&'a str>,
  declined: bool,
}

impl ResourceFragments<'_> {
  fn add(&mut self, text: &str) {
    if !text.is_empty()
      && let Some(start) = self.resource.find(text)
    {
      self
        .fragments
        .insert(&self.resource[start..start + text.len()]);
    }
  }

  fn add_cooked(&mut self, text: Option<&str>) {
    match text {
      Some(text) => self.add(text),
      // Two lone surrogates join into one astral character, which no byte offset can split.
      None => self.declined = true,
    }
  }

  fn spell_resource(&self) -> bool {
    let mut reachable = vec![false; self.resource.len() + 1];
    reachable[0] = true;
    for start in 0..self.resource.len() {
      if reachable[start] {
        for fragment in &self.fragments {
          if self.resource[start..].starts_with(fragment) {
            reachable[start + fragment.len()] = true;
          }
        }
      }
    }
    reachable[self.resource.len()]
  }
}

impl Visit for ResourceFragments<'_> {
  fn visit_str(&mut self, literal: &Str) {
    // This includes JSX attributes, whose entities have already been decoded by the parser.
    self.add_cooked(literal.value.as_str());
  }

  fn visit_tpl_element(&mut self, element: &TplElement) {
    // Downlevel tagged templates also emit an array of raw strings.
    self.add(element.raw.as_str());
    if let Some(cooked) = &element.cooked {
      self.add_cooked(cooked.as_str());
    }
  }

  fn visit_number(&mut self, literal: &Number) {
    if let Some(raw) = &literal.raw {
      self.add(raw.as_str());
    }
    self.add(&literal.value.to_js_string());
  }

  fn visit_big_int(&mut self, literal: &BigInt) {
    if let Some(raw) = &literal.raw {
      self.add(raw.as_str());
    }
    self.add(&literal.value.to_string());
  }

  fn visit_bool(&mut self, literal: &Bool) {
    self.add(if literal.value { "true" } else { "false" });
  }

  fn visit_null(&mut self, _literal: &Null) {
    self.add("null");
  }

  fn visit_regex(&mut self, literal: &Regex) {
    // A sticky regex lowers to `new RegExp("pattern")`. A Windows path can be a valid pattern.
    self.add(literal.exp.as_str());
  }

  fn visit_ts_enum_member(&mut self, member: &TsEnumMember) {
    // The enum evaluator prints computed numbers into strings, including arithmetic results and
    // auto-incremented members that appear in no literal.
    if member.init.as_deref().is_some_and(|init| !init.is_lit()) {
      for text in NUMBER_TEXT {
        self.add(text);
      }
    }
    member.visit_children_with(self);
  }

  fn visit_jsx_text(&mut self, text: &JSXText) {
    // JSX text additionally normalizes whitespace. Conservatively decline path-shaped text.
    self.declined |= text.value.to_string_lossy().contains(['/', '\\']);
  }
}

fn object(value: &Value, allowed: fn(&str, &Value) -> bool) -> bool {
  value
    .as_object()
    .is_some_and(|fields| fields.iter().all(|(key, value)| allowed(key, value)))
}

fn parser(key: &str, value: &Value) -> bool {
  match key {
    "syntax" => matches!(value.as_str(), Some("ecmascript" | "typescript")),
    "jsx" | "tsx" | "decorators" => value.is_boolean(),
    _ => false,
  }
}

fn react(key: &str, value: &Value) -> bool {
  match key {
    "development" => value.is_boolean(),
    "runtime" => matches!(value.as_str(), Some("classic" | "automatic")),
    // The refresh pass reads source text for its signatures, never the filename.
    "refresh" => value.is_boolean(),
    _ => false,
  }
}

fn jsc(key: &str, value: &Value) -> bool {
  match key {
    "parser" => {
      object(value, parser)
        && (value.get("decorators").and_then(Value::as_bool) != Some(true)
          || value.get("syntax").and_then(Value::as_str) == Some("typescript"))
    }
    "target" => value.is_string(), // SWC's option deserializer validates the enum.
    "externalHelpers" => value.is_boolean(),
    "experimental" => object(value, |key, value| {
      key == "disableAllLints" && value.is_boolean()
    }),
    "transform" => object(value, |key, value| key == "react" && object(value, react)),
    _ => false,
  }
}

pub(crate) fn supports_portable_output(options: &Map<String, Value>) -> bool {
  options.iter().all(|(key, value)| match key.as_str() {
    "jsc" => object(value, jsc),
    // Omitted or query targets would read a browserslist config from the checkout.
    "env" => {
      value.get("targets").is_some()
        && object(value, |key, value| {
          key == "targets" && object(value, |_, version| version.is_string())
        })
    }
    "rspackExperiments" => object(value, |key, value| {
      key == "portableOutput" && value.as_bool() == Some(true)
    }),
    "detectSyntax" => value.as_bool() == Some(false) || value.as_str() == Some("auto"),
    "sourceMaps" | "sourceMap" | "inlineSourcesContent" | "emitSourceMapColumns" => {
      value.is_boolean()
    }
    // A candidate only: actual acquisition and producer authority decide certification.
    "inputSourceMap" => value.is_boolean(),
    "isModule" => value.is_boolean() || value.as_str() == Some("unknown"),
    _ => false,
  })
}
