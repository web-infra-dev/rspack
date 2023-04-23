use std::{
  collections::HashMap,
  sync::{mpsc, Arc, Mutex},
};

use rspack_core::{
  rspack_sources::{ConcatSource, RawSource, SourceExt},
  ModuleType,
};
use rspack_error::{internal_error, DiagnosticKind, Error, Result, TraceableError};
use swc_core::{
  base::{
    config::{IsModule, JsMinifyCommentOption, JsMinifyOptions, SourceMapsConfig},
    BoolOr, TransformOutput,
  },
  common::{
    collections::AHashMap,
    comments::{Comment, Comments, SingleThreadedComments},
    errors::{Emitter, Handler, HANDLER},
    BytePos, FileName, Mark, SourceMap, GLOBALS,
  },
  ecma::{
    ast::Ident,
    atoms::JsWord,
    minifier::{
      self,
      option::{MinifyOptions, TopLevelOptions},
    },
    parser::{EsConfig, Syntax},
    transforms::base::{
      fixer::fixer,
      helpers::{self, Helpers},
      hygiene::hygiene,
      resolver,
    },
    visit::{noop_visit_type, FoldWith, Visit, VisitMutWith, VisitWith},
  },
};

use super::parse::parse_js;
use super::stringify::{print, SourceMapConfig};
use crate::{utils::ecma_parse_error_to_rspack_error, ExtractedCommentsInfo};

pub fn minify(
  opts: &JsMinifyOptions,
  input: String,
  filename: &str,
  all_extract_comments: &Mutex<HashMap<String, ExtractedCommentsInfo>>,
  extract_comments: Option<String>,
) -> Result<TransformOutput> {
  let cm: Arc<SourceMap> = Default::default();
  GLOBALS.set(&Default::default(), || -> Result<TransformOutput> {
    with_rspack_error_handler(
      "Minify Error".to_string(),
      DiagnosticKind::JavaScript,
      cm.clone(),
      |handler| {
        println!("input: {}", input);
        let fm = cm.new_source_file(FileName::Custom(filename.to_string()), input);
        let target = opts.ecma.clone().into();

        let (source_map, _) = opts
          .source_map
          .as_ref()
          .map(|obj| -> std::result::Result<_, anyhow::Error> {
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
          .expect("TODO:")?;

        let mut min_opts = MinifyOptions {
          compress: opts
            .compress
            .clone()
            .unwrap_as_option(|default| match default {
              Some(true) | None => Some(Default::default()),
              _ => None,
            })
            .map(|v| v.into_config(cm.clone())),
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
            opts.top_level = Some(true);
          }
        }

        let comments = SingleThreadedComments::default();

        let module = parse_js(
          fm.clone(),
          target,
          Syntax::Es(EsConfig {
            jsx: true,
            decorators: true,
            decorators_before_export: true,
            import_assertions: true,
            ..Default::default()
          }),
          IsModule::Bool(true),
          Some(&comments),
        )
        .map_err(|errs| {
          Error::BatchErrors(
            errs
              .into_iter()
              .map(|err| ecma_parse_error_to_rspack_error(err, &fm, &ModuleType::Js))
              .collect::<Vec<_>>(),
          )
        })?;

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

        let module = helpers::HELPERS.set(&Helpers::new(false), || {
          HANDLER.set(handler, || {
            let module = module.fold_with(&mut resolver(unresolved_mark, top_level_mark, false));

            let mut module = minifier::optimize(
              module,
              cm.clone(),
              Some(&comments),
              None,
              &min_opts,
              &minifier::option::ExtraOptions {
                unresolved_mark,
                top_level_mark,
              },
            );

            if !is_mangler_enabled {
              module.visit_mut_with(&mut hygiene())
            }
            module.fold_with(&mut fixer(Some(&comments as &dyn Comments)))
          })
        });

        let preserve_comments = opts
          .format
          .comments
          .clone()
          .into_inner()
          .unwrap_or(BoolOr::Data(JsMinifyCommentOption::PreserveSomeComments));

        minify_file_comments(&comments, preserve_comments);
        if let Some(_) = extract_comments {
          let comments_file_name = filename.to_string() + ".License.txt";
          let (l, t) = comments.borrow_all();

          let mut source = ConcatSource::default();
          source.add(RawSource::from("license"));
          l.iter().for_each(|(_, vc)| {
            vc.iter().for_each(|c| {
              source.add(RawSource::from(&*c.text));
            });
          });
          t.iter().for_each(|(_, vc)| {
            vc.iter().for_each(|c| {
              source.add(RawSource::from(&*c.text));
            });
          });
          all_extract_comments.lock().unwrap().insert(
            filename.to_string(),
            ExtractedCommentsInfo {
              source: source.boxed(),
              comments_file_name,
            },
          );
        }

        print(
          &module,
          cm.clone(),
          target,
          SourceMapConfig {
            enable: source_map.enabled(),
            inline_sources_content: opts.inline_sources_content,
            emit_columns: opts.emit_source_map_columns,
            names: source_map_names,
          },
          true,
          Some(&comments),
          opts.format.ascii_only,
        )
      },
    )
  })
}

pub struct IdentCollector {
  names: AHashMap<BytePos, JsWord>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}

fn minify_file_comments(
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

      println!("l: {:?}", l);
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

// keep this private to make sure with_rspack_error_handler is safety
struct RspackErrorEmitter {
  tx: mpsc::Sender<rspack_error::Error>,
  source_map: Arc<SourceMap>,
  title: String,
  kind: DiagnosticKind,
}

impl Emitter for RspackErrorEmitter {
  fn emit(&mut self, db: &swc_core::common::errors::DiagnosticBuilder<'_>) {
    let source_file_and_byte_pos = db
      .span
      .primary_span()
      .map(|s| self.source_map.lookup_byte_offset(s.lo()));
    if let Some(source_file_and_byte_pos) = source_file_and_byte_pos {
      self
        .tx
        .send(Error::TraceableError(
          TraceableError::from_source_file(
            &source_file_and_byte_pos.sf,
            source_file_and_byte_pos.pos.0 as usize,
            source_file_and_byte_pos.pos.0 as usize,
            self.title.to_string(),
            db.message(),
          )
          .with_kind(self.kind),
        ))
        .expect("Sender should drop after emit called");
    } else {
      self
        .tx
        .send(internal_error!(db.message()))
        .expect("Sender should drop after emit called");
    }
  }
}

pub fn with_rspack_error_handler<F, Ret>(
  title: String,
  kind: DiagnosticKind,
  cm: Arc<SourceMap>,
  op: F,
) -> Result<Ret>
where
  F: FnOnce(&Handler) -> Result<Ret>,
{
  let (tx, rx) = mpsc::channel();
  let emitter = RspackErrorEmitter {
    title,
    kind,
    source_map: cm,
    tx,
  };
  let handler = Handler::with_emitter(true, false, Box::new(emitter));

  let ret = HANDLER.set(&handler, || op(&handler));

  if handler.has_errors() {
    drop(handler);
    Err(rspack_error::Error::BatchErrors(rx.into_iter().collect()))
  } else {
    ret
  }
}
