use std::sync::Arc;

use anyhow::{Context, Error};
use swc_atoms::JsWord;
use swc_common::{
  collections::AHashMap,
  comments::{Comment, Comments},
  errors::Handler,
  BytePos, Mark, SourceFile,
};
use swc_ecma_ast::Ident;
use swc_ecma_minifier::option::{MinifyOptions, TopLevelOptions};
use swc_ecma_parser::{EsConfig, Syntax};
use swc_ecma_transforms::{fixer, hygiene, resolver};
use swc_ecma_visit::{noop_visit_type, FoldWith, Visit, VisitMutWith, VisitWith};

use swc::{
  config::{IsModule, JsMinifyCommentOption, JsMinifyOptions, SourceMapsConfig},
  BoolOr, TransformOutput,
};
use swc_common::comments::SingleThreadedComments;

use crate::utils::get_swc_compiler;

pub fn minify(
  fm: Arc<SourceFile>,
  handler: &Handler,
  opts: &JsMinifyOptions,
  minify: bool,
) -> Result<TransformOutput, Error> {
  let compiler = get_swc_compiler();
  let target = opts.ecma.clone().into();

  let (source_map, orig) = opts
    .source_map
    .as_ref()
    .map(|obj| -> Result<_, Error> {
      let orig = obj
        .content
        .as_ref()
        .map(|s| sourcemap::SourceMap::from_slice(s.as_bytes()));
      let orig = match orig {
        Some(v) => Some(v?),
        None => None,
      };
      Ok((SourceMapsConfig::Bool(true), orig))
    })
    .unwrap_as_option(|v| {
      Some(Ok(match v {
        Some(true) => (SourceMapsConfig::Bool(true), None),
        _ => (SourceMapsConfig::Bool(false), None),
      }))
    })
    .unwrap()?;

  let mut min_opts = MinifyOptions {
    compress: opts
      .compress
      .clone()
      .unwrap_as_option(|default| match default {
        Some(true) | None => Some(Default::default()),
        _ => None,
      })
      .map(|v| v.into_config(compiler.cm.clone())),
    mangle: opts
      .mangle
      .clone()
      .unwrap_as_option(|default| match default {
        Some(true) | None => Some(Default::default()),
        _ => None,
      }),
    ..Default::default()
  };

  // top_level defaults to true if module is true

  // https://github.com/swc-project/swc/issues/2254

  if opts.module {
    if let Some(opts) = &mut min_opts.compress {
      if opts.top_level.is_none() {
        opts.top_level = Some(TopLevelOptions { functions: true });
      }
    }

    if let Some(opts) = &mut min_opts.mangle {
      opts.top_level = true;
    }
  }

  let comments = SingleThreadedComments::default();

  let module = compiler
    .parse_js(
      fm.clone(),
      handler,
      target,
      Syntax::Es(EsConfig {
        jsx: true,
        decorators: true,
        decorators_before_export: true,
        import_assertions: true,
        private_in_object: true,
        ..Default::default()
      }),
      IsModule::Bool(true),
      Some(&comments),
    )
    .context("failed to parse input file")?;

  let source_map_names = if source_map.enabled() {
    let mut v = IdentCollector {
      names: Default::default(),
    };

    module.visit_with(&mut v);

    v.names
  } else {
    Default::default()
  };

  let unresolved_mark = Mark::new();
  let top_level_mark = Mark::new();

  let is_mangler_enabled = min_opts.mangle.is_some();

  let module = compiler.run_transform(handler, false, || {
    let module = module.fold_with(&mut resolver(unresolved_mark, top_level_mark, false));

    let mut module = swc_ecma_minifier::optimize(
      module,
      compiler.cm.clone(),
      Some(&comments),
      None,
      &min_opts,
      &swc_ecma_minifier::option::ExtraOptions {
        unresolved_mark,
        top_level_mark,
      },
    );

    if !is_mangler_enabled {
      module.visit_mut_with(&mut hygiene())
    }
    module.fold_with(&mut fixer(Some(&comments as &dyn Comments)))
  });

  let preserve_comments = opts
    .format
    .comments
    .clone()
    .into_inner()
    .unwrap_or(BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments));
  minify_file_comments(&comments, preserve_comments);

  compiler.print(
    &module,
    Some(&fm.name.to_string()),
    opts.output_path.clone().map(From::from),
    opts.inline_sources_content,
    target,
    source_map,
    &source_map_names,
    orig.as_ref(),
    minify,
    Some(&comments),
    opts.emit_source_map_columns,
    opts.format.ascii_only,
  )
}

pub(crate) fn minify_file_comments(
  comments: &SingleThreadedComments,
  preserve_comments: BoolOr<JsMinifyCommentOption>,
) {
  match preserve_comments {
    BoolOr::Bool(true) | BoolOr::Data(JsMinifyCommentOption::PreserveAllComments) => {}

    BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments) => {
      let preserve_excl = |_: &BytePos, vc: &mut Vec<Comment>| -> bool {
        // Preserve license comments.
        vc.retain(|c: &Comment| c.text.contains("@license") || c.text.starts_with('!'));
        !vc.is_empty()
      };
      let (mut l, mut t) = comments.borrow_all_mut();

      l.retain(preserve_excl);
      t.retain(preserve_excl);
    }

    BoolOr::Bool(false) => {
      let (mut l, mut t) = comments.borrow_all_mut();
      l.clear();
      t.clear();
    }
  }
}

pub struct IdentCollector {
  pub names: AHashMap<BytePos, JsWord>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
