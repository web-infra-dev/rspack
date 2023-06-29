use std::hash::Hash;

use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{
  ChunkGroupOptions, CodeGeneratableDependency, ConstDependency, EntryOptions, ModuleDependency,
  ModuleIdentifier, OutputOptions, SpanExt,
};
use rspack_hash::RspackHash;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{Id, ImportDecl, ModuleExportName, ObjectLit};
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::{
  ast::{Expr, ExprOrSpread, Ident, Lit, NewExpr},
  visit::{noop_visit_type, Visit, VisitWith},
};

use super::url_scanner::match_new_url;
use crate::dependency::WorkerDependency;

static WORKER_FROM_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r"^(.+?)(\(\))?\s+from\s+(.+)$").expect("invalid regex"));

#[derive(Debug, PartialEq, Eq)]
enum WorkerSyntax {
  New(WorkerName),
  // Call(WorkerName),
}

#[derive(Debug, PartialEq, Eq)]
enum WorkerName {
  Global(JsWord),
  Var(Id),
}

pub struct WorkerSyntaxScanner<'a> {
  result: Vec<WorkerSyntax>,
  caps: Vec<(&'a str, &'a str)>,
}

impl<'a> WorkerSyntaxScanner<'a> {
  pub fn new(syntax: &'a [&'a str]) -> Self {
    let mut result = Vec::new();
    let mut caps = Vec::new();
    for s in syntax {
      if let Some(captures) = WORKER_FROM_REGEX.captures(s)
      && let Some(ids) = captures.get(1)
      && let Some(source) = captures.get(3) {
        caps.push((ids.as_str(), source.as_str()));
      } else {
        result.push(WorkerSyntax::New(WorkerName::Global(JsWord::from(*s))))
      }
    }
    Self { result, caps }
  }
}

impl Visit for WorkerSyntaxScanner<'_> {
  fn visit_import_decl(&mut self, decl: &ImportDecl) {
    let source = &*decl.src.value;
    let found = self
      .caps
      .iter()
      .filter(|cap| cap.1 == source)
      .flat_map(|cap| {
        if cap.0 == "default" {
          decl
            .specifiers
            .iter()
            .filter_map(|spec| spec.as_default())
            .map(|spec| spec.local.to_id())
            .collect::<Vec<Id>>()
        } else {
          decl
            .specifiers
            .iter()
            .filter_map(|spec| {
              spec.as_named().filter(|named| {
                named
                  .imported
                  .as_ref()
                  .map(|name| match name {
                    ModuleExportName::Ident(s) => &s.sym,
                    ModuleExportName::Str(s) => &s.value,
                  })
                  .filter(|s| *s == cap.0)
                  .is_some()
              })
            })
            .map(|spec| spec.local.to_id())
            .collect::<Vec<Id>>()
        }
      })
      .map(|pair| WorkerSyntax::New(WorkerName::Var(pair)));
    self.result.extend(found);
  }
}

// TODO: should created by WorkerPlugin
pub struct WorkerScanner<'a> {
  pub presentational_dependencies: Vec<Box<dyn CodeGeneratableDependency>>,
  pub dependencies: Vec<Box<dyn ModuleDependency>>,
  index: usize,
  module_identifier: &'a ModuleIdentifier,
  output_options: &'a OutputOptions,
  syntax: Vec<WorkerSyntax>,
}

// new Worker(new URL("./foo.worker.js", import.meta.url));
impl<'a> WorkerScanner<'a> {
  pub fn new(
    module_identifier: &'a ModuleIdentifier,
    output_options: &'a OutputOptions,
    syntax_scanner: WorkerSyntaxScanner,
  ) -> Self {
    Self {
      presentational_dependencies: Vec::new(),
      dependencies: Vec::new(),
      index: 0,
      module_identifier,
      output_options,
      syntax: syntax_scanner.result,
    }
  }

  fn add_dependencies(
    &mut self,
    new_expr: &NewExpr,
    parsed_path: ParsedNewWorkerPath,
    parsed_options: Option<ParsedNewWorkerOptions>,
  ) {
    let mut hasher = RspackHash::from(self.output_options);
    self.module_identifier.hash(&mut hasher);
    self.index.hash(&mut hasher);
    self.index += 1;
    let digest = hasher.digest(&self.output_options.hash_digest);
    let runtime = digest
      .rendered(self.output_options.hash_digest_length)
      .to_owned();
    let range = parsed_options.as_ref().map(|options| options.range);
    let name = parsed_options.and_then(|options| options.name);
    let output_module = self.output_options.module;
    self.dependencies.push(Box::new(WorkerDependency::new(
      parsed_path.range.0,
      parsed_path.range.1,
      parsed_path.value,
      self.output_options.worker_public_path.clone(),
      Some(new_expr.span.into()),
      ChunkGroupOptions {
        name,
        entry_options: Some(EntryOptions {
          runtime: Some(runtime),
          chunk_loading: Some(self.output_options.worker_chunk_loading.clone()),
          async_chunks: None,
          public_path: None,
          base_uri: None,
        }),
      },
    )));
    if let Some(range) = range {
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          range.0,
          range.0,
          "Object.assign({}, ".into(),
          None,
        )));
      self
        .presentational_dependencies
        .push(Box::new(ConstDependency::new(
          range.1,
          range.1,
          format!(
            ", {{ type: {} }})",
            if output_module {
              "\"module\""
            } else {
              "undefined"
            }
          )
          .into(),
          None,
        )));
    }
  }

  fn parse_new_worker(
    &self,
    new_expr: &NewExpr,
  ) -> Option<(ParsedNewWorkerPath, Option<ParsedNewWorkerOptions>)> {
    if self.match_new_worker(new_expr)
    && let Some(args) = &new_expr.args
    && let Some(expr_or_spread) = args.first()
    && let ExprOrSpread { spread: None, expr: box Expr::New(new_url_expr) } = expr_or_spread
    && let Some((start, end, request)) = match_new_url(new_url_expr) {
      let path = ParsedNewWorkerPath {
        range: (start, end),
        value: request,
      };
      let options = args.get(1).map(parse_new_worker_options);
      Some((path, options))
    } else {
      None
    }
  }

  fn find_worker_syntax(&self, ident: &Ident) -> Option<&WorkerSyntax> {
    self.syntax.iter().find(|s| match s {
      WorkerSyntax::New(name) => match name {
        WorkerName::Global(n) => n == &ident.sym,
        WorkerName::Var(id) => *id == ident.to_id(),
      },
    })
  }

  pub fn match_new_worker(&self, new_expr: &NewExpr) -> bool {
    matches!(&*new_expr.callee, Expr::Ident(ident) if self.find_worker_syntax(ident).is_some())
  }
}

impl Visit for WorkerScanner<'_> {
  noop_visit_type!();

  fn visit_new_expr(&mut self, new_expr: &NewExpr) {
    if let Some((parsed_path, parsed_options)) = self.parse_new_worker(new_expr) {
      self.add_dependencies(new_expr, parsed_path, parsed_options);
    } else {
      new_expr.visit_children_with(self);
    }
  }
}

#[derive(Debug)]
struct ParsedNewWorkerPath {
  pub range: (u32, u32),
  pub value: String,
}

#[derive(Debug)]
struct ParsedNewWorkerOptions {
  pub range: (u32, u32),
  pub name: Option<String>,
}

fn parse_new_worker_options(arg: &ExprOrSpread) -> ParsedNewWorkerOptions {
  fn get_prop_literal_str(obj: &ObjectLit, key: &str) -> Option<String> {
    obj
      .props
      .iter()
      .filter_map(|p| {
        p.as_prop()
          .and_then(|p| p.as_key_value())
          .filter(|kv| {
            kv.key.as_str().filter(|k| &*k.value == key).is_some()
              || kv.key.as_ident().filter(|k| &*k.sym == key).is_some()
          })
          .and_then(|name| name.value.as_lit())
          .and_then(|lit| match lit {
            Lit::Str(s) => Some(s.value.to_string()),
            _ => None,
          })
      })
      .next()
  }

  let obj = arg.expr.as_object();
  let name = obj.and_then(|obj| get_prop_literal_str(obj, "name"));
  let span = arg.span();
  ParsedNewWorkerOptions {
    range: (span.real_lo(), span.real_hi()),
    name,
  }
}
