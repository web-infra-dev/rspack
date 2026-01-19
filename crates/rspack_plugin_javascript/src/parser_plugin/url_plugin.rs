use std::hash::Hash;

use rspack_core::{
  AsyncDependenciesBlock, ChunkLoading, ConstDependency, ContextDependency, ContextMode,
  ContextNameSpaceObject, ContextOptions, DependencyCategory, DependencyRange, EntryOptions,
  GroupOptions, JavascriptParserUrl, RuntimeGlobals, SharedSourceMap,
};
use rspack_hash::RspackHash;
use rspack_util::SpanExt;
use swc_core::{
  common::Spanned,
  ecma::{
    ast::{Expr, ExprOrSpread, MemberExpr, MetaPropKind, NewExpr},
    visit::{Visit, VisitWith},
  },
};
use url::Url;

use super::JavascriptParserPlugin;
use crate::{
  InnerGraphPlugin,
  dependency::{URLBundleDependency, URLContextDependency, URLDependency},
  magic_comment::try_extract_magic_comment,
  parser_plugin::inner_graph::state::InnerGraphUsageOperation,
  visitors::{ExprRef, JavascriptParser, context_reg_exp, create_context_dependency},
};

#[derive(Default)]
struct NestedNewUrlVisitor {
  has_nested_new_url: bool,
}

impl Visit for NestedNewUrlVisitor {
  fn visit_new_expr(&mut self, expr: &NewExpr) {
    if expr
      .callee
      .as_ident()
      .is_some_and(|ident| ident.sym.eq("URL"))
    {
      self.has_nested_new_url = true;
    }
  }
}

pub fn is_meta_url(parser: &mut JavascriptParser, expr: &MemberExpr) -> bool {
  let chain = parser.extract_member_expression_chain(ExprRef::Member(expr));
  if let ExprRef::MetaProp(meta) = chain.object {
    return meta.kind == MetaPropKind::ImportMeta
      && chain.members.len() == 1
      && chain.members.first().is_some_and(|member| member == "url");
  }
  false
}

pub fn get_url_request(
  parser: &mut JavascriptParser,
  expr: &NewExpr,
) -> Option<(String, u32, u32)> {
  let args = expr.args.as_ref()?;
  let ExprOrSpread {
    spread: None,
    expr: arg1,
  } = args.first()?
  else {
    return None;
  };
  let arg2 = args.get(1);

  if let Some(arg2) = arg2 {
    // new URL(xx, import.meta.url)
    let ExprOrSpread {
      spread: None,
      expr: arg2,
    } = arg2
    else {
      return None;
    };
    let Expr::Member(arg2) = &**arg2 else {
      return None;
    };
    if is_meta_url(parser, arg2) {
      return parser
        .evaluate_expression(arg1)
        .as_string()
        .map(|req| (req, arg1.span().real_lo(), arg2.span().real_hi()));
    }
  } else {
    // new URL(import.meta.url)
    let Expr::Member(arg1) = &**arg1 else {
      return None;
    };
    if is_meta_url(parser, arg1) {
      return Some((
        Url::from_file_path(parser.resource_data.resource())
          .expect("should be a path")
          .to_string(),
        arg1.span().real_lo(),
        arg1.span().real_hi(),
      ));
    }
  }

  None
}

pub struct URLPlugin {
  pub mode: Option<JavascriptParserUrl>,
  /// Whether to bundle new URL() targets as separate entry chunks
  pub bundle_new_url: Option<bool>,
}

impl JavascriptParserPlugin for URLPlugin {
  fn can_rename(&self, _parser: &mut JavascriptParser, for_name: &str) -> Option<bool> {
    (for_name == "URL").then_some(true)
  }

  fn new_expression(
    &self,
    parser: &mut JavascriptParser,
    expr: &NewExpr,
    for_name: &str,
  ) -> Option<bool> {
    if for_name != "URL" {
      return None;
    }

    let args = expr.args.as_ref()?;

    let arg = args.first()?;
    let magic_comment_options = try_extract_magic_comment(parser, expr.span, arg.span());
    if magic_comment_options.get_ignore().unwrap_or_default() {
      if args.len() != 2 {
        return None;
      }
      let arg2 = args.get(1)?;
      if let ExprOrSpread {
        spread: None,
        expr: arg2_expr,
      } = arg2
        && let Expr::Member(arg2) = &**arg2_expr
        && !is_meta_url(parser, arg2)
      {
        return None;
      }
      parser.add_presentational_dependency(Box::new(ConstDependency::new(
        arg2.span().into(),
        parser
          .runtime_template
          .render_runtime_globals(&RuntimeGlobals::BASE_URI)
          .into(),
        Some(RuntimeGlobals::BASE_URI),
      )));
      return Some(true);
    }

    // should not parse new URL(import.meta.url)
    if expr.args.as_ref().is_some_and(|args| {
      args.len() == 1
        && args[0]
          .expr
          .as_member()
          .is_some_and(|member| is_meta_url(parser, member))
    }) {
      return None;
    }

    if let Some((request, start, end)) = get_url_request(parser, expr) {
      // Handle Bundle mode - create AsyncDependenciesBlock with special EntryOptions
      if self.bundle_new_url == Some(true) {
        return self.handle_bundle_mode(parser, expr, request, start, end);
      }

      let dep = URLDependency::new(
        request.into(),
        expr.span.into(),
        (start, end).into(),
        self.mode,
      );
      let dep_idx = parser.next_dependency_idx();
      parser.add_dependency(Box::new(dep));
      InnerGraphPlugin::on_usage(parser, InnerGraphUsageOperation::URLDependency(dep_idx));
      return Some(true);
    }

    let mut nested_new_url_visitor = NestedNewUrlVisitor::default();
    arg.expr.visit_with(&mut nested_new_url_visitor);
    if nested_new_url_visitor.has_nested_new_url {
      return None;
    }

    let arg2 = args.get(1)?;
    if !arg2
      .expr
      .as_member()
      .is_some_and(|member| is_meta_url(parser, member))
    {
      return None;
    }

    let param = parser.evaluate_expression(&arg.expr);
    let result = create_context_dependency(&param, parser);
    let options = ContextOptions {
      mode: ContextMode::Sync,
      recursive: true,
      reg_exp: context_reg_exp(&result.reg, "", None, parser),
      include: magic_comment_options.get_include(),
      exclude: magic_comment_options.get_exclude(),
      category: DependencyCategory::Url,
      request: format!("{}{}{}", result.context, result.query, result.fragment),
      context: result.context,
      namespace_object: ContextNameSpaceObject::Unset,
      group_options: None,
      replaces: result.replaces,
      start: expr.span().real_lo(),
      end: expr.span().real_hi(),
      referenced_exports: None,
      attributes: None,
    };

    let mut dep = URLContextDependency::new(
      options,
      expr.span().into(),
      param.range().into(),
      parser.in_try,
    );
    *dep.critical_mut() = result.critical;
    parser.add_dependency(Box::new(dep));

    Some(true)
  }

  fn is_pure(&self, parser: &mut JavascriptParser, expr: &Expr) -> Option<bool> {
    let expr = expr.as_new()?;
    let callee = expr.callee.as_ident()?;
    if parser.get_free_info_from_variable(&callee.sym).is_none() || !callee.sym.eq("URL") {
      return None;
    }
    get_url_request(parser, expr)?;
    Some(true)
  }
}

impl URLPlugin {
  /// Handle Bundle mode: creates an AsyncDependenciesBlock with EntryOptions
  /// that force single-file output (no chunk loading, no async chunks)
  fn handle_bundle_mode(
    &self,
    parser: &mut JavascriptParser,
    expr: &NewExpr,
    request: String,
    start: u32,
    end: u32,
  ) -> Option<bool> {
    let output_options = &parser.compiler_options.output;
    let mut hasher = RspackHash::from(output_options);
    parser.module_identifier.hash(&mut hasher);
    parser.worker_index.hash(&mut hasher);
    parser.worker_index += 1;
    let digest = hasher.digest(&output_options.hash_digest);
    let runtime = digest
      .rendered(output_options.hash_digest_length)
      .to_owned();

    // Use empty public_path - it will be resolved at runtime via RuntimeGlobals::PUBLIC_PATH
    let public_path = String::new();

    // Use relative URL when mode is NewUrlRelative
    let use_relative = matches!(self.mode, Some(JavascriptParserUrl::NewUrlRelative));

    let dep = Box::new(URLBundleDependency::new(
      request.clone(),
      public_path,
      expr.span.into(),
      (start, end).into(),
      use_relative,
    ));

    let source_map: SharedSourceMap = parser.source_rope().clone();
    let mut block = AsyncDependenciesBlock::new(
      *parser.module_identifier,
      Into::<DependencyRange>::into(expr.span).to_loc(Some(&source_map)),
      None,
      vec![dep],
      Some(request),
    );

    // Set EntryOptions with:
    // - chunk_loading: Disable (no chunk loading runtime needed)
    // - async_chunks: false (prevent further async chunk splitting)
    // This ensures the bundle is a single self-contained file
    block.set_group_options(GroupOptions::Entrypoint(Box::new(EntryOptions {
      name: None, // Anonymous entry - will get auto-generated chunk name
      runtime: Some(runtime.into()),
      chunk_loading: Some(ChunkLoading::Disable),
      wasm_loading: None,
      async_chunks: Some(false),
      public_path: None,
      base_uri: None,
      filename: None,
      library: None,
      depend_on: None,
      layer: None,
    })));

    parser.add_block(Box::new(block));

    Some(true)
  }
}
