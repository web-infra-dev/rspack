use std::hash::Hash;

use rspack_core::{
  AsyncDependenciesBlock, ConstDependency, DependencyRange, EntryOptions, GroupOptions,
};
use rspack_hash::RspackHash;
use rspack_util::SpanExt;
use smallvec::SmallVec;
use swc_core::{
  atoms::Atom,
  common::{Span, Spanned},
  ecma::ast::{CallExpr, ExprOrSpread, Ident, NewExpr, VarDeclarator},
};
use url::Url;

use super::{
  JavascriptParserPlugin,
  esm_import_dependency_parser_plugin::{ESM_SPECIFIER_TAG, ESMSpecifierData},
  url_plugin::get_url_request,
};
use crate::{
  dependency::{CreateScriptUrlDependency, WorkerDependency},
  magic_comment::try_extract_magic_comment,
  parser_plugin::url_plugin::is_meta_url,
  utils::object_properties::get_literal_str_by_obj_prop,
  visitors::{JavascriptParser, TagInfoData, VariableDeclaration},
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

#[derive(Debug)]
struct ParsedNewWorkerImportOptions {
  pub name: Option<String>,
  pub ignored: Option<bool>,
}

fn parse_new_worker_options(arg: &ExprOrSpread) -> ParsedNewWorkerOptions {
  let obj = arg.expr.as_object();
  let name = obj
    .and_then(|obj| get_literal_str_by_obj_prop(obj, "name"))
    .map(|str| str.value.to_string_lossy().into());
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
) -> Option<ParsedNewWorkerImportOptions> {
  let comments = try_extract_magic_comment(parser, diagnostic_span, span);
  if comments.get_ignore().is_some() || comments.get_chunk_name().is_some() {
    Some(ParsedNewWorkerImportOptions {
      name: comments.get_chunk_name().cloned(),
      ignored: comments.get_ignore(),
    })
  } else {
    None
  }
}

fn add_dependencies(
  parser: &mut JavascriptParser,
  span: Span,
  first_arg: &ExprOrSpread,
  parsed_path: ParsedNewWorkerPath,
  parsed_options: Option<ParsedNewWorkerOptions>,
  need_new_url: bool,
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
  let options_range = parsed_options.as_ref().and_then(|options| options.range);
  let name = parsed_options.and_then(|options| options.name);
  let output_module = output_options.module;
  let dep = Box::new(WorkerDependency::new(
    parsed_path.value,
    output_options.worker_public_path.clone(),
    span.into(),
    parsed_path.range.into(),
    need_new_url,
  ));
  let range = DependencyRange::from(span);
  let loc = parser.to_dependency_location(range);
  let mut block =
    AsyncDependenciesBlock::new(*parser.module_identifier, loc, None, vec![dep], None);
  block.set_group_options(GroupOptions::Entrypoint(Box::new(EntryOptions {
    name,
    runtime: Some(runtime.into()),
    chunk_loading: Some(output_options.worker_chunk_loading.clone()),
    wasm_loading: Some(output_options.worker_wasm_loading.clone()),
    async_chunks: None,
    public_path: None,
    base_uri: None,
    filename: None,
    library: None,
    depend_on: None,
    layer: None,
  })));

  parser.add_block(Box::new(block));

  if parser.compiler_options.output.trusted_types.is_some() {
    parser.add_dependency(Box::new(CreateScriptUrlDependency::new(
      span.into(),
      first_arg.span().into(),
    )));
  }

  if let Some(options_range) = options_range {
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (options_range.0, options_range.0).into(),
      "Object.assign({}, ".into(),
    )));
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (options_range.1, options_range.1).into(),
      format!(
        ", {{ type: {} }})",
        if output_module {
          "\"module\""
        } else {
          "undefined"
        }
      )
      .into(),
    )));
  }
}

fn handle_worker<'a>(
  parser: &mut JavascriptParser,
  args: &'a [ExprOrSpread],
  span: Span,
) -> Option<(
  ParsedNewWorkerPath,
  Option<ParsedNewWorkerOptions>,
  &'a ExprOrSpread,
  bool,
)> {
  if let Some(expr_or_spread) = args.first()
    && let ExprOrSpread {
      spread: None,
      expr: expr_box,
    } = expr_or_spread
  {
    let mut need_new_url = false;
    let path = if let Some(new_url_expr) = expr_box.as_new()
      && let Some((request, start, end)) = get_url_request(parser, new_url_expr)
    {
      ParsedNewWorkerPath {
        range: (start, end),
        value: request,
      }
    } else if let Some(member_expr) = expr_box.as_member()
      && is_meta_url(parser, member_expr)
    {
      need_new_url = true;
      ParsedNewWorkerPath {
        range: (member_expr.span().real_lo(), member_expr.span().real_hi()),
        value: Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_string(),
      }
    } else {
      return None;
    };
    let mut options = args
      .get(1)
      // new Worker(new URL("worker.js"), options)
      .map(parse_new_worker_options);

    let import_options = expr_box
      .as_new()
      .and_then(|new_url_expr| {
        new_url_expr
          .args
          .as_ref()
          .and_then(|args| args.first())
          .and_then(|n| {
            // new Worker(new URL(/* options */ "worker.js"))
            parse_new_worker_options_from_comments(parser, n.span(), new_url_expr.span())
          })
      })
      .or_else(|| {
        // new Worker(/* options */ new URL("worker.js"))
        parse_new_worker_options_from_comments(parser, expr_or_spread.span(), span)
      });

    if import_options
      .as_ref()
      .and_then(|options| options.ignored)
      .is_some_and(|i| i)
    {
      return None;
    }

    if let Some(name) = import_options.and_then(|options| options.name) {
      if let Some(options) = &mut options {
        options.name = Some(name);
      } else {
        options = Some(ParsedNewWorkerOptions {
          range: None,
          name: Some(name),
        });
      }
    }

    Some((path, options, expr_or_spread, need_new_url))
  } else {
    None
  }
}

pub struct WorkerPlugin {
  new_syntax: SyntaxEntries,
  call_syntax: SyntaxEntries,
  from_new_syntax: FromSyntaxEntries,
  from_call_syntax: FromSyntaxEntries,
  pattern_syntax: PatternSyntaxEntries,
}

const WORKER_SPECIFIER_TAG: &str = "_identifier__worker_specifier_tag__";
const DEFAULT_SYNTAX: [&str; 4] = [
  "Worker",
  "SharedWorker",
  "navigator.serviceWorker.register()",
  "Worker from worker_threads",
];

#[derive(Debug, Clone)]
struct WorkerSpecifierData {
  key: Atom,
}

type SyntaxEntries = SmallVec<[Box<str>; 2]>;
type FromSyntaxEntries = SmallVec<[(Box<str>, Box<str>); 1]>;
type PatternSyntaxEntries = SmallVec<[PatternSyntax; 1]>;
type PatternMembers = SmallVec<[Box<str>; 1]>;

#[derive(Debug)]
struct PatternSyntax {
  pattern: Box<str>,
  members: PatternMembers,
}

impl WorkerPlugin {
  pub fn new(syntax_list: &[String]) -> Self {
    let mut this = Self {
      new_syntax: SyntaxEntries::new(),
      call_syntax: SyntaxEntries::new(),
      from_new_syntax: FromSyntaxEntries::new(),
      from_call_syntax: FromSyntaxEntries::new(),
      pattern_syntax: PatternSyntaxEntries::new(),
    };
    for syntax in syntax_list {
      if syntax == "..." {
        for syntax in DEFAULT_SYNTAX {
          this.handle_syntax(syntax);
        }
      } else {
        this.handle_syntax(syntax);
      }
    }
    this
  }

  fn insert_pattern_syntax(&mut self, pattern: &str, members: &str) {
    if let Some(entry) = self
      .pattern_syntax
      .iter_mut()
      .find(|entry| entry.pattern.as_ref() == pattern)
    {
      insert_unique_syntax(&mut entry.members, members);
    } else {
      let mut pattern_members = PatternMembers::new();
      pattern_members.push(members.into());
      self.pattern_syntax.push(PatternSyntax {
        pattern: pattern.into(),
        members: pattern_members,
      });
    }
  }

  fn has_pattern_syntax(&self, pattern: &str) -> bool {
    self
      .pattern_syntax
      .iter()
      .any(|entry| entry.pattern.as_ref() == pattern)
  }

  fn matches_pattern_members(&self, pattern: &str, members: &[Atom]) -> bool {
    self
      .pattern_syntax
      .iter()
      .find(|entry| entry.pattern.as_ref() == pattern)
      .is_some_and(|entry| {
        entry
          .members
          .iter()
          .any(|candidate| dotted_members_match(candidate, members.iter().map(Atom::as_str)))
      })
  }

  fn contains_from_syntax(&self, entries: &FromSyntaxEntries, ids: &[Atom], source: &str) -> bool {
    entries.iter().any(|(candidate_ids, candidate_source)| {
      candidate_source.as_ref() == source
        && dotted_members_match(candidate_ids, ids.iter().map(Atom::as_str))
    })
  }

  fn handle_syntax(&mut self, syntax: &str) {
    if let Some(syntax) = syntax.strip_prefix('*')
      && let Some(first_dot) = syntax.find('.')
      && let Some(syntax) = syntax.strip_suffix("()")
    {
      let pattern = &syntax[0..first_dot];
      let members = &syntax[first_dot + 1..];
      self.insert_pattern_syntax(pattern, members);
    } else if let Some(syntax) = syntax.strip_suffix("()") {
      insert_unique_syntax(&mut self.call_syntax, syntax);
    } else if let Some(((ids, is_call), source)) = worker_from(syntax) {
      if is_call {
        insert_unique_from_syntax(&mut self.from_call_syntax, ids, source);
      } else {
        insert_unique_from_syntax(&mut self.from_new_syntax, ids, source);
      }
    } else {
      insert_unique_syntax(&mut self.new_syntax, syntax);
    }
  }
}

#[rspack_macros::implemented_javascript_parser_hooks]
impl JavascriptParserPlugin for WorkerPlugin {
  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    decl: &VarDeclarator,
    _statement: VariableDeclaration<'_>,
  ) -> Option<bool> {
    if let Some(ident) = decl.name.as_ident()
      && self.has_pattern_syntax(ident.sym.as_str())
    {
      parser.tag_variable(
        ident.sym.clone(),
        WORKER_SPECIFIER_TAG,
        Some(WorkerSpecifierData {
          key: ident.sym.clone(),
        }),
      );
      return Some(true);
    }
    None
  }

  fn pattern(&self, parser: &mut JavascriptParser, ident: &Ident, for_name: &str) -> Option<bool> {
    if self.has_pattern_syntax(for_name) {
      parser.tag_variable(
        ident.sym.clone(),
        WORKER_SPECIFIER_TAG,
        Some(WorkerSpecifierData {
          key: ident.sym.clone(),
        }),
      );
      return Some(true);
    }
    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
    members: &[Atom],
    _members_optionals: &[bool],
    _member_ranges: &[Span],
  ) -> Option<bool> {
    if for_name != WORKER_SPECIFIER_TAG {
      return None;
    }
    let tag_info = parser
      .definitions_db
      .expect_get_tag_info(parser.current_tag_info?);
    let data = WorkerSpecifierData::downcast(tag_info.data.clone()?);
    if self.matches_pattern_members(data.key.as_str(), members) {
      return handle_worker(parser, &call_expr.args, call_expr.span).map(
        |(parsed_path, parsed_options, first_arg, need_new_url)| {
          add_dependencies(
            parser,
            call_expr.span,
            first_arg,
            parsed_path,
            parsed_options,
            need_new_url,
          );
          if let Some(callee) = call_expr.callee.as_expr() {
            parser.walk_expression(callee);
          }
          if let Some(arg) = call_expr.args.get(1) {
            parser.walk_expression(&arg.expr);
          }
          true
        },
      );
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: &CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == ESM_SPECIFIER_TAG {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(parser.current_tag_info?);
      let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);
      if self.contains_from_syntax(
        &self.from_call_syntax,
        &settings.ids,
        settings.source.as_str(),
      ) {
        return handle_worker(parser, &call_expr.args, call_expr.span).map(
          |(parsed_path, parsed_options, first_arg, need_new_url)| {
            add_dependencies(
              parser,
              call_expr.span,
              first_arg,
              parsed_path,
              parsed_options,
              need_new_url,
            );
            if let Some(callee) = call_expr.callee.as_expr() {
              parser.walk_expression(callee);
            }
            if let Some(arg) = call_expr.args.get(1) {
              parser.walk_expression(&arg.expr);
            }
            true
          },
        );
      }
      return None;
    }
    if !contains_syntax(&self.call_syntax, for_name) {
      return None;
    }
    handle_worker(parser, &call_expr.args, call_expr.span).map(
      |(parsed_path, parsed_options, first_arg, need_new_url)| {
        add_dependencies(
          parser,
          call_expr.span,
          first_arg,
          parsed_path,
          parsed_options,
          need_new_url,
        );
        if let Some(callee) = call_expr.callee.as_expr() {
          parser.walk_expression(callee);
        }
        if let Some(arg) = call_expr.args.get(1) {
          parser.walk_expression(&arg.expr);
        }
        true
      },
    )
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    new_expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == ESM_SPECIFIER_TAG {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(parser.current_tag_info?);
      let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);
      if self.contains_from_syntax(
        &self.from_new_syntax,
        &settings.ids,
        settings.source.as_str(),
      ) {
        return new_expr
          .args
          .as_ref()
          .and_then(|args| handle_worker(parser, args, new_expr.span))
          .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
            add_dependencies(
              parser,
              new_expr.span,
              first_arg,
              parsed_path,
              parsed_options,
              need_new_url,
            );
            parser.walk_expression(&new_expr.callee);
            if let Some(args) = &new_expr.args
              && let Some(arg) = args.get(1)
            {
              parser.walk_expression(&arg.expr);
            }
            true
          });
      }
      return None;
    }
    if !contains_syntax(&self.new_syntax, for_name) {
      return None;
    }
    new_expr
      .args
      .as_ref()
      .and_then(|args| handle_worker(parser, args, new_expr.span))
      .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
        add_dependencies(
          parser,
          new_expr.span,
          first_arg,
          parsed_path,
          parsed_options,
          need_new_url,
        );
        parser.walk_expression(&new_expr.callee);
        if let Some(args) = &new_expr.args
          && let Some(arg) = args.get(1)
        {
          parser.walk_expression(&arg.expr);
        }
        true
      })
  }
}

fn insert_unique_syntax<const N: usize>(entries: &mut SmallVec<[Box<str>; N]>, value: &str) {
  if !contains_syntax(entries, value) {
    entries.push(value.into());
  }
}

fn contains_syntax(entries: &[Box<str>], value: &str) -> bool {
  entries.iter().any(|entry| entry.as_ref() == value)
}

fn insert_unique_from_syntax(entries: &mut FromSyntaxEntries, ids: &str, source: &str) {
  if !entries
    .iter()
    .any(|(entry_ids, entry_source)| entry_ids.as_ref() == ids && entry_source.as_ref() == source)
  {
    entries.push((ids.into(), source.into()));
  }
}

fn dotted_members_match<'a>(candidate: &str, mut members: impl Iterator<Item = &'a str>) -> bool {
  for candidate_member in candidate.split('.') {
    let Some(member) = members.next() else {
      return false;
    };
    if candidate_member != member {
      return false;
    }
  }
  members.next().is_none()
}

fn worker_from(input: &str) -> Option<((&str, bool), &str)> {
  let ident_end = input.find(char::is_whitespace)?;
  if ident_end == 0 {
    return None;
  }

  let ident = &input[..ident_end];
  let mut rest = input[ident_end..].trim_start_matches(char::is_whitespace);
  rest = rest.strip_prefix("from")?;

  let source = rest.trim_start_matches(char::is_whitespace);
  if source.len() == rest.len() || source.is_empty() {
    return None;
  }

  Some(
    ident
      .strip_suffix("()")
      .map_or(((ident, false), source), |ident| ((ident, true), source)),
  )
}

#[test]
fn test_worker_from() {
  assert_eq!(
    worker_from("Worker from worker_threads"),
    Some((("Worker", false), "worker_threads"))
  );

  assert_eq!(
    worker_from("worker() from worker_threads"),
    Some((("worker", true), "worker_threads"))
  );

  assert_eq!(
    worker_from("()() from worker_threads"),
    Some((("()", true), "worker_threads"))
  );
}
