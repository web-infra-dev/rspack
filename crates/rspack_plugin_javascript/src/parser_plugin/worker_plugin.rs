use std::hash::Hash;

use itertools::Itertools;
use rspack_core::{
  AsyncDependenciesBlock, ConstDependency, DependencyRange, EntryOptions, GroupOptions,
};
use rspack_hash::RspackHash;
use rspack_util::SpanExt;
use rustc_hash::{FxHashMap, FxHashSet};
use swc_core::atoms::Atom;
use swc_experimental_ecma_ast::{
  CallExpr, ExprOrSpread, Ident, NewExpr, Span, Spanned, TypedSubRange, VarDeclarator,
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

fn parse_new_worker_options(
  parser: &JavascriptParser,
  arg: ExprOrSpread,
) -> ParsedNewWorkerOptions {
  let obj = arg.expr(&parser.ast).as_object();
  let name = obj
    .and_then(|obj| get_literal_str_by_obj_prop(&parser.ast, obj, "name"))
    .map(|str| {
      parser
        .ast
        .get_wtf8(str.value(&parser.ast))
        .to_string_lossy()
        .into()
    });
  let span = arg.span(&parser.ast);
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
  first_arg: ExprOrSpread,
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
  let range = parsed_options.as_ref().and_then(|options| options.range);
  let name = parsed_options.and_then(|options| options.name);
  let output_module = output_options.module;
  let dep = Box::new(WorkerDependency::new(
    parsed_path.value,
    output_options.worker_public_path.clone(),
    span.into(),
    parsed_path.range.into(),
    need_new_url,
  ));
  let mut block = AsyncDependenciesBlock::new(
    *parser.module_identifier,
    Into::<DependencyRange>::into(span).to_loc(Some(parser.source())),
    None,
    vec![dep],
    None,
  );
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
      first_arg.span(&parser.ast).into(),
    )));
  }

  if let Some(range) = range {
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (range.0, range.0).into(),
      "Object.assign({}, ".into(),
    )));
    parser.add_presentational_dependency(Box::new(ConstDependency::new(
      (range.1, range.1).into(),
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

fn handle_worker(
  parser: &mut JavascriptParser,
  args: TypedSubRange<ExprOrSpread>,
  span: Span,
) -> Option<(
  ParsedNewWorkerPath,
  Option<ParsedNewWorkerOptions>,
  ExprOrSpread,
  bool,
)> {
  if let Some(expr_or_spread) = args.first() {
    let expr_or_spread = parser.ast.get_node_in_sub_range(expr_or_spread);
    if expr_or_spread.spread(&parser.ast).is_some() {
      return None;
    }
    let expr_box = expr_or_spread.expr(&parser.ast);

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
        range: (
          member_expr.span(&parser.ast).real_lo(),
          member_expr.span(&parser.ast).real_hi(),
        ),
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
          .args(&parser.ast)
          .and_then(|args| args.first())
          .and_then(|n| {
            // new Worker(new URL(/* options */ "worker.js"))
            parse_new_worker_options_from_comments(parser, n.span(), new_url_expr.span(&parser.ast))
          })
      })
      .or_else(|| {
        // new Worker(/* options */ new URL("worker.js"))
        parse_new_worker_options_from_comments(parser, expr_or_spread.span(&parser.ast), span)
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
  new_syntax: FxHashSet<String>,
  call_syntax: FxHashSet<String>,
  from_new_syntax: FxHashSet<(String, String)>,
  from_call_syntax: FxHashSet<(String, String)>,
  pattern_syntax: FxHashMap<String, FxHashSet<String>>,
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

impl WorkerPlugin {
  pub fn new(syntax_list: &[String]) -> Self {
    let mut this = Self {
      new_syntax: FxHashSet::default(),
      call_syntax: FxHashSet::default(),
      from_new_syntax: FxHashSet::default(),
      from_call_syntax: FxHashSet::default(),
      pattern_syntax: FxHashMap::default(),
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

  fn handle_syntax(&mut self, syntax: &str) {
    if let Some(syntax) = syntax.strip_prefix('*')
      && let Some(first_dot) = syntax.find('.')
      && let Some(syntax) = syntax.strip_suffix("()")
    {
      let pattern = &syntax[0..first_dot];
      let members = &syntax[first_dot + 1..];
      if let Some(value) = self.pattern_syntax.get_mut(pattern) {
        value.insert(members.to_string());
      } else {
        self.pattern_syntax.insert(
          pattern.to_string(),
          FxHashSet::from_iter([members.to_string()]),
        );
      }
    } else if let Some(syntax) = syntax.strip_suffix("()") {
      self.call_syntax.insert(syntax.to_string());
    } else if let Ok(((ids, is_call), source)) = worker_from(syntax) {
      if is_call {
        self
          .from_call_syntax
          .insert((ids.to_string(), source.to_string()));
      } else {
        self
          .from_new_syntax
          .insert((ids.to_string(), source.to_string()));
      }
    } else {
      self.new_syntax.insert(syntax.to_string());
    }
  }
}

impl JavascriptParserPlugin for WorkerPlugin {
  fn pre_declarator(
    &self,
    parser: &mut JavascriptParser,
    decl: VarDeclarator,
    _statement: VariableDeclaration,
  ) -> Option<bool> {
    if let Some(ident) = decl.name(&parser.ast).as_ident()
      && self.pattern_syntax.contains_key(ident.sym.as_str())
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

  fn pattern(&self, parser: &mut JavascriptParser, ident: Ident, for_name: &str) -> Option<bool> {
    if self.pattern_syntax.contains_key(for_name) {
      parser.tag_variable(
        parser.ast.get_atom(ident.sym(&parser.ast)),
        WORKER_SPECIFIER_TAG,
        Some(WorkerSpecifierData {
          key: parser.ast.get_atom(ident.sym(&parser.ast)),
        }),
      );
      return Some(true);
    }
    None
  }

  fn call_member_chain(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
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
    if let Some(value) = self.pattern_syntax.get(data.key.as_str())
      && value.contains(&members.iter().map(|id| id.as_str()).join("."))
    {
      return handle_worker(
        parser,
        call_expr.args(&parser.ast),
        call_expr.span(&parser.ast),
      )
      .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
        add_dependencies(
          parser,
          call_expr.span(&parser.ast),
          first_arg,
          parsed_path,
          parsed_options,
          need_new_url,
        );
        if let Some(callee) = call_expr.callee(&parser.ast).as_expr() {
          parser.walk_expression(callee);
        }
        if let Some(arg) = call_expr.args(&parser.ast).get(1) {
          let arg = parser.ast.get_node_in_sub_range(arg);
          parser.walk_expression(arg.expr(&parser.ast));
        }
        true
      });
    }
    None
  }

  fn call(
    &self,
    parser: &mut JavascriptParser,
    call_expr: CallExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == ESM_SPECIFIER_TAG {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(parser.current_tag_info?);
      let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);
      let ids = settings.ids.iter().map(|id| id.as_str()).join(".");
      if self
        .from_call_syntax
        .contains(&(ids, settings.source.to_string()))
      {
        return handle_worker(
          parser,
          call_expr.args(&parser.ast),
          call_expr.span(&parser.ast),
        )
        .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
          add_dependencies(
            parser,
            call_expr.span(&parser.ast),
            first_arg,
            parsed_path,
            parsed_options,
            need_new_url,
          );
          if let Some(callee) = call_expr.callee(&parser.ast).as_expr() {
            parser.walk_expression(callee);
          }
          if let Some(arg) = call_expr.args(&parser.ast).get(1) {
            parser.walk_expression(arg.expr(&parser.ast));
          }
          true
        });
      }
      return None;
    }
    if !self.call_syntax.contains(for_name) {
      return None;
    }
    handle_worker(
      parser,
      call_expr.args(&parser.ast),
      call_expr.span(&parser.ast),
    )
    .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
      add_dependencies(
        parser,
        call_expr.span(&parser.ast),
        first_arg,
        parsed_path,
        parsed_options,
        need_new_url,
      );
      if let Some(callee) = call_expr.callee(&parser.ast).as_expr() {
        parser.walk_expression(callee);
      }
      if let Some(arg) = call_expr.args(&parser.ast).get(1) {
        let arg = parser.ast.get_node_in_sub_range(arg);
        parser.walk_expression(arg.expr(&parser.ast));
      }
      true
    })
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    new_expr: NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name == ESM_SPECIFIER_TAG {
      let tag_info = parser
        .definitions_db
        .expect_get_tag_info(parser.current_tag_info?);
      let settings = ESMSpecifierData::downcast(tag_info.data.clone()?);
      let ids = settings.ids.iter().map(|id| id.as_str()).join(".");
      if self
        .from_new_syntax
        .contains(&(ids, settings.source.to_string()))
      {
        return new_expr
          .args(&parser.ast)
          .and_then(|args| handle_worker(parser, args, new_expr.span(&parser.ast)))
          .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
            add_dependencies(
              parser,
              new_expr.span(&parser.ast),
              first_arg,
              parsed_path,
              parsed_options,
              need_new_url,
            );
            parser.walk_expression(new_expr.callee(&parser.ast));
            if let Some(args) = new_expr.args(&parser.ast)
              && let Some(arg) = args.get(1)
            {
              let arg = parser.ast.get_node_in_sub_range(arg);
              parser.walk_expression(arg.expr(&parser.ast));
            }
            true
          });
      }
      return None;
    }
    if !self.new_syntax.contains(for_name) {
      return None;
    }
    new_expr
      .args(&parser.ast)
      .and_then(|args| handle_worker(parser, args, new_expr.span(&parser.ast)))
      .map(|(parsed_path, parsed_options, first_arg, need_new_url)| {
        add_dependencies(
          parser,
          new_expr.span(&parser.ast),
          first_arg,
          parsed_path,
          parsed_options,
          need_new_url,
        );
        parser.walk_expression(new_expr.callee(&parser.ast));
        if let Some(args) = new_expr.args(&parser.ast)
          && let Some(arg) = args.get(1)
        {
          let arg = parser.ast.get_node_in_sub_range(arg);
          parser.walk_expression(arg.expr(&parser.ast));
        }
        true
      })
  }
}

fn worker_from(mut input: &str) -> winnow::ModalResult<((&str, bool), &str)> {
  use winnow::{ascii, combinator::separated_pair, prelude::*, stream::AsChar, token};

  let ident_and_call = token::take_while(1.., |c: char| !c.is_space()).map(|ident: &str| {
    ident
      .strip_suffix("()")
      .map_or((ident, false), |ident| (ident, true))
  });

  let mut parser = separated_pair(
    ident_and_call,
    (ascii::multispace1, "from", ascii::multispace1),
    token::take_while(1.., |_| true),
  );

  parser.parse_next(&mut input)
}

#[test]
fn test_worker_from() {
  assert_eq!(
    worker_from("Worker from worker_threads").ok(),
    Some((("Worker", false), "worker_threads"))
  );

  assert_eq!(
    worker_from("worker() from worker_threads").ok(),
    Some((("worker", true), "worker_threads"))
  );

  assert_eq!(
    worker_from("()() from worker_threads").ok(),
    Some((("()", true), "worker_threads"))
  );
}
