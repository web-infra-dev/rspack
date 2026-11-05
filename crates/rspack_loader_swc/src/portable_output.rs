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
use serde_json::{Map, Value};
use swc_core::{
  base::config::InputSourceMap,
  common::FileName,
  ecma::{
    ast::{Expr, JSXText, Lit, Program, Str, TplElement, TsEnumDecl},
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

/// Only SWC-generated filename literals belong to this invocation's declaration. An authored
/// value equal to the resource must stay fixed when a checkout moves, including escaped input.
///
/// The transform can also synthesize such a value from fragments: TypeScript folds enum member
/// initializers, so `"/checkout/src/" + "a.ts"` becomes the resource path without any input
/// literal equal to it. Output whose value matches cannot show where it came from, so a program
/// that can synthesize a string declines certification.
pub(crate) fn input_has_resource_path(program: &Program, resource: &str) -> bool {
  let mut visitor = InputResourcePath {
    resource,
    found: false,
  };
  program.visit_with(&mut visitor);
  visitor.found
}

struct InputResourcePath<'a> {
  resource: &'a str,
  found: bool,
}

impl Visit for InputResourcePath<'_> {
  fn visit_str(&mut self, literal: &Str) {
    // This includes JSX attributes, whose entities have already been decoded by the parser.
    self.found |= literal.value.as_str() == Some(self.resource);
  }

  fn visit_tpl_element(&mut self, element: &TplElement) {
    // Downlevel tagged templates also emit an array of raw strings.
    self.found |= element.raw.as_str() == self.resource
      || element
        .cooked
        .as_ref()
        .is_some_and(|value| value.as_str() == Some(self.resource));
  }

  fn visit_jsx_text(&mut self, text: &JSXText) {
    // JSX text additionally normalizes whitespace. Conservatively decline path-shaped text.
    self.found |= text.value.to_string_lossy().contains(['/', '\\']);
  }

  fn visit_ts_enum_decl(&mut self, declaration: &TsEnumDecl) {
    // Member initializers are folded. Number arithmetic cannot produce a path; anything that
    // can produce a string from more than one literal can.
    self.found |= declaration
      .members
      .iter()
      .filter_map(|member| member.init.as_deref())
      .any(|init| {
        !matches!(init, Expr::Lit(_) | Expr::Ident(_) | Expr::Member(_)) && !is_numeric(init)
      });
    declaration.visit_children_with(self);
  }
}

/// An expression tree of numeric literals and arithmetic alone.
fn is_numeric(expr: &Expr) -> bool {
  match expr {
    Expr::Lit(Lit::Num(_)) => true,
    Expr::Paren(paren) => is_numeric(&paren.expr),
    Expr::Unary(unary) => is_numeric(&unary.arg),
    Expr::Bin(binary) => is_numeric(&binary.left) && is_numeric(&binary.right),
    _ => false,
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
