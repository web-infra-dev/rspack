use std::hash::Hash;

use itertools::Itertools;
use rspack_core::{
  AsyncDependenciesBlock, ConstDependency, DependencyLocation, EntryOptions, ErrorSpan,
  GroupOptions, SpanExt,
};
use rspack_hash::RspackHash;
use rustc_hash::FxHashSet;
use swc_core::{
  common::{Span, Spanned},
  ecma::ast::{Expr, ExprOrSpread, NewExpr},
};

use super::{
  harmony_import_dependency_parser_plugin::{HarmonySpecifierData, HARMONY_SPECIFIER_TAG},
  url_plugin::get_url_request,
  JavascriptParserPlugin,
};
use crate::{
  dependency::WorkerDependency,
  utils::get_literal_str_by_obj_prop,
  visitors::{JavascriptParser, TagInfoData},
  webpack_comment::try_extract_webpack_magic_comment,
};

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

fn add_dependencies(
  parser: &mut JavascriptParser,
  new_expr: &NewExpr,
  parsed_path: ParsedNewWorkerPath,
  parsed_options: Option<ParsedNewWorkerOptions>,
) {
  let output_options = &parser.compiler_options.output;
  let mut hasher = RspackHash::from(output_options);
  parser.module_identifier.hash(&mut hasher);
  parser.worker_index.hash(&mut hasher);
  parser.worker_index += 1;
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
    *parser.module_identifier,
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

  parser.blocks.push(block);
  if let Some(range) = range {
    parser
      .presentational_dependencies
      .push(Box::new(ConstDependency::new(
        range.0,
        range.0,
        "Object.assign({}, ".into(),
        None,
      )));
    parser
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
  parser: &mut JavascriptParser,
  new_expr: &NewExpr,
) -> Option<(ParsedNewWorkerPath, Option<ParsedNewWorkerOptions>)> {
  if let Some(args) = &new_expr.args
    && let Some(expr_or_spread) = args.first()
    && let ExprOrSpread {
      spread: None,
      expr: box Expr::New(new_url_expr),
    } = expr_or_spread
    && let Some((request, start, end)) = get_url_request(parser, new_url_expr)
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
            parse_new_worker_options_from_comments(parser, n.span(), new_url_expr.span())
          })
      })
      .or_else(|| {
        // new Worker(/* options */ new URL("worker.js"))
        parse_new_worker_options_from_comments(parser, expr_or_spread.span(), new_expr.span())
      });
    Some((path, options))
  } else {
    None
  }
}

pub struct WorkerPlugin {
  new_syntax: FxHashSet<String>,
  // call_syntax: FxHashSet<String>,
  from_new_syntax: FxHashSet<(String, String)>,
  // from_call_syntax: FxHashSet<(String, String)>,
}

impl WorkerPlugin {
  pub fn new(/* syntax_list: &[&str] */) -> Self {
    Self {
      new_syntax: FxHashSet::from_iter(["Worker".into(), "SharedWorker".into()]),
      // call_syntax: FxHashSet::default(),
      from_new_syntax: FxHashSet::from_iter([("Worker".into(), "worker_threads".into())]),
      // from_call_syntax: FxHashSet::default(),
    }
  }
}

impl JavascriptParserPlugin for WorkerPlugin {
  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    new_expr: &swc_core::ecma::ast::NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == HARMONY_SPECIFIER_TAG {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(&parser.current_tag_info?);
      let settings = HarmonySpecifierData::downcast(tag_info.data.clone()?);
      let ids = settings.ids.iter().map(|id| id.as_str()).join(".");
      if self
        .from_new_syntax
        .contains(&(ids, settings.source.to_string()))
      {
        return parse_new_worker(parser, new_expr).map(|(parsed_path, parsed_options)| {
          add_dependencies(parser, new_expr, parsed_path, parsed_options);
          parser.walk_expression(&new_expr.callee);
          true
        });
      }
      return None;
    }
    if !self.new_syntax.contains(for_name) {
      return None;
    }
    parse_new_worker(parser, new_expr).map(|(parsed_path, parsed_options)| {
      add_dependencies(parser, new_expr, parsed_path, parsed_options);
      parser.walk_expression(&new_expr.callee);
      true
    })
  }
}
