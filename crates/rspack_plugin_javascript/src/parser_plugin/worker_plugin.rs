use std::hash::Hash;

use rspack_core::{
  AsyncDependenciesBlock, ConstDependency, DependencyLocation, EntryOptions, ErrorSpan,
  GroupOptions, SpanExt,
};
use rspack_hash::RspackHash;
use swc_core::common::{Span, Spanned};
use swc_core::ecma::ast::{Expr, ExprOrSpread, NewExpr};

use super::JavascriptParserPlugin;
use crate::dependency::WorkerDependency;
use crate::utils::get_literal_str_by_obj_prop;
use crate::visitors::JavascriptParser;
use crate::webpack_comment::try_extract_webpack_magic_comment;

#[derive(Debug)]
struct ParsedNewWorkerPath {
  pub range: (u32, u32),
  pub value: String,
}

#[derive(Debug)]
struct ParsedNewWorkerOptions {
  pub range: Option<(u32, u32)>,
  pub name: Option<String>,
}

fn parse_new_worker_options(arg: &ExprOrSpread) -> ParsedNewWorkerOptions {
  let obj = arg.expr.as_object();
  let name = obj
    .and_then(|obj| get_literal_str_by_obj_prop(obj, "name"))
    .map(|str| str.value.to_string());
  let span = arg.span();
  ParsedNewWorkerOptions {
    range: Some((span.real_lo(), span.real_hi())),
    name,
  }
}

fn parse_new_worker_options_from_comments(
  parser: &mut JavascriptParser,
  span: Span,
  diagnostic_span: Span,
) -> Option<ParsedNewWorkerOptions> {
  let comments = try_extract_webpack_magic_comment(
    parser.source_file,
    &parser.comments,
    diagnostic_span,
    span,
    &mut parser.warning_diagnostics,
  );
  comments
    .get_webpack_chunk_name()
    .map(|name| ParsedNewWorkerOptions {
      range: None,
      name: Some(name.to_string()),
    })
}

impl<'parser> JavascriptParser<'parser> {
  fn add_dependencies(
    &mut self,
    new_expr: &NewExpr,
    parsed_path: ParsedNewWorkerPath,
    parsed_options: Option<ParsedNewWorkerOptions>,
  ) {
    let output_options = &self.compiler_options.output;
    let mut hasher = RspackHash::from(output_options);
    self.module_identifier.hash(&mut hasher);
    self.worker_index.hash(&mut hasher);
    self.worker_index += 1;
    let digest = hasher.digest(&output_options.hash_digest);
    let runtime = digest
      .rendered(output_options.hash_digest_length)
      .to_owned();
    let range = parsed_options.as_ref().and_then(|options| options.range);
    let name = parsed_options.and_then(|options| options.name);
    let output_module = output_options.module;
    let span = ErrorSpan::from(new_expr.span);
    let dep = Box::new(WorkerDependency::new(
      parsed_path.range.0,
      parsed_path.range.1,
      parsed_path.value,
      output_options.worker_public_path.clone(),
      Some(span),
    ));
    let mut block = AsyncDependenciesBlock::new(
      *self.module_identifier,
      Some(DependencyLocation::new(span.start, span.end)),
      None,
      vec![dep],
    );
    block.set_group_options(GroupOptions::Entrypoint(Box::new(EntryOptions {
      name,
      runtime: Some(runtime.into()),
      chunk_loading: Some(output_options.worker_chunk_loading.clone()),
      async_chunks: None,
      public_path: None,
      base_uri: None,
      filename: None,
      library: None,
      depend_on: None,
    })));

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
    &mut self,
    new_expr: &NewExpr,
  ) -> Option<(ParsedNewWorkerPath, Option<ParsedNewWorkerOptions>)> {
    if self.worker_syntax_list.match_new_worker(new_expr)
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
      let options = args
        .get(1)
        // new Worker(new URL("worker.js"), options)
        .map(parse_new_worker_options)
        .or_else(|| {
          // new Worker(new URL(/* options */ "worker.js"))
          new_url_expr
            .args
            .as_ref()
            .and_then(|args| args.first())
            .and_then(|n| {
              parse_new_worker_options_from_comments(self, n.span(), new_url_expr.span())
            })
        })
        .or_else(|| {
          // new Worker(/* options */ new URL("worker.js"))
          parse_new_worker_options_from_comments(self, expr_or_spread.span(), new_expr.span())
        });
      Some((path, options))
    } else {
      None
    }
  }
}

/// `new Worker(new URL("./foo.worker.js", import.meta.url));`
pub struct WorkerPlugin;

impl JavascriptParserPlugin for WorkerPlugin {
  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    new_expr: &swc_core::ecma::ast::NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    parser
      .parse_new_worker(new_expr)
      .map(|(parsed_path, parsed_options)| {
        parser.add_dependencies(new_expr, parsed_path, parsed_options);
        parser.walk_expression(&new_expr.callee);
        true
      })
  }
}
