use std::hash::Hash;

use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, BoxDependencyTemplate, ConstDependency,
  DependencyLocation, EntryOptions, ErrorSpan, GroupOptions, ModuleIdentifier, OutputOptions,
  SpanExt,
};
use rspack_hash::RspackHash;
use rustc_hash::FxHashSet;
use swc_core::common::Spanned;
use swc_core::ecma::ast::{Expr, ExprOrSpread, NewExpr};
use swc_core::ecma::visit::{noop_visit_type, Visit, VisitWith};

use crate::dependency::WorkerDependency;
use crate::no_visit_ignored_stmt;
use crate::utils::get_literal_str_by_obj_prop;

// TODO: should created by WorkerPlugin
pub struct WorkerScanner<'a> {
  pub presentational_dependencies: Vec<BoxDependencyTemplate>,
  pub dependencies: Vec<BoxDependency>,
  pub blocks: Vec<AsyncDependenciesBlock>,
  index: usize,
  module_identifier: &'a ModuleIdentifier,
  output_options: &'a OutputOptions,
  syntax_list: &'a rspack_core::needs_refactor::WorkerSyntaxList,
  pub ignored: &'a mut FxHashSet<DependencyLocation>,
}

// new Worker(new URL("./foo.worker.js", import.meta.url));
impl<'a> WorkerScanner<'a> {
  pub fn new(
    module_identifier: &'a ModuleIdentifier,
    output_options: &'a OutputOptions,
    syntax_list: &'a rspack_core::needs_refactor::WorkerSyntaxList,
    ignored: &'a mut FxHashSet<DependencyLocation>,
  ) -> Self {
    Self {
      presentational_dependencies: Vec::new(),
      dependencies: Vec::new(),
      blocks: Vec::new(),
      index: 0,
      module_identifier,
      output_options,
      syntax_list,
      ignored,
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
    let span = ErrorSpan::from(new_expr.span);
    let dep = Box::new(WorkerDependency::new(
      parsed_path.range.0,
      parsed_path.range.1,
      parsed_path.value,
      self.output_options.worker_public_path.clone(),
      Some(span),
    ));
    let mut block = AsyncDependenciesBlock::new(
      *self.module_identifier,
      Some(DependencyLocation::new(span.start, span.end)),
    );
    block.set_group_options(GroupOptions::Entrypoint(Box::new(EntryOptions {
      name,
      runtime: Some(runtime),
      chunk_loading: Some(self.output_options.worker_chunk_loading.clone()),
      async_chunks: None,
      public_path: None,
      base_uri: None,
      filename: None,
      library: None,
    })));
    block.add_dependency(dep);
    self.blocks.push(block);

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
    if self.syntax_list.match_new_worker(new_expr)
      && let Some(args) = &new_expr.args
      && let Some(expr_or_spread) = args.first()
      && let ExprOrSpread {
        spread: None,
        expr: box Expr::New(new_url_expr),
      } = expr_or_spread
      && let Some((start, end, request)) = rspack_core::needs_refactor::match_new_url(new_url_expr)
    {
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
}

impl Visit for WorkerScanner<'_> {
  noop_visit_type!();
  no_visit_ignored_stmt!();

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
  let obj = arg.expr.as_object();
  let name = obj
    .and_then(|obj| get_literal_str_by_obj_prop(obj, "name"))
    .map(|str| str.value.to_string());
  let span = arg.span();
  ParsedNewWorkerOptions {
    range: (span.real_lo(), span.real_hi()),
    name,
  }
}
