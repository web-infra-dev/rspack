use std::sync::Arc;

use rspack_error::{miette::IntoDiagnostic, Error};
use rspack_sources::{encode_mappings, Mapping, OriginalLocation};
use rustc_hash::FxHashMap;
use swc_core::{
  base::{config::JsMinifyFormatOptions, sourcemap},
  common::{
    comments::Comments, source_map::SourceMapGenConfig, BytePos, FileName,
    SourceMap as SwcSourceMap,
  },
  ecma::{
    ast::{EsVersion, Ident, Program as SwcProgram},
    atoms::Atom,
    codegen::{
      self,
      text_writer::{self, WriteJs},
      Emitter, Node,
    },
    visit::{noop_visit_type, Visit, VisitWith},
  },
};

use super::{JavaScriptCompiler, TransformOutput};
use crate::ast::Ast;

#[derive(Default, Clone, Debug)]
pub struct CodegenOptions<'a> {
  pub target: Option<EsVersion>,
  pub source_map_config: SourceMapConfig,
  pub input_source_map: Option<&'a sourcemap::SourceMap>,
  pub keep_comments: Option<bool>,
  pub minify: Option<bool>,
  pub ascii_only: Option<bool>,
  pub inline_script: Option<bool>,
  pub emit_assert_for_import_attributes: Option<bool>,
}

#[derive(Default, Clone, Debug)]
pub struct SourceMapConfig {
  pub enable: bool,
  pub inline_sources_content: bool,
  pub emit_columns: bool,
  pub names: FxHashMap<BytePos, Atom>,
}

impl SourceMapGenConfig for SourceMapConfig {
  fn file_name_to_source(&self, f: &FileName) -> String {
    let f = f.to_string();
    if f.starts_with('<') && f.ends_with('>') {
      f[1..f.len() - 1].to_string()
    } else {
      f
    }
  }

  fn inline_sources_content(&self, _: &FileName) -> bool {
    self.inline_sources_content
  }

  fn emit_columns(&self, _f: &FileName) -> bool {
    self.emit_columns
  }

  fn name_for_bytepos(&self, pos: BytePos) -> Option<&str> {
    self.names.get(&pos).map(|v| &**v)
  }
}

pub struct PrintOptions<'a> {
  pub source_map: Arc<SwcSourceMap>,
  pub target: EsVersion,
  pub source_map_config: SourceMapConfig,
  pub input_source_map: Option<&'a sourcemap::SourceMap>,
  pub minify: bool,
  pub comments: Option<&'a dyn Comments>,
  pub format: &'a JsMinifyFormatOptions,
}

impl JavaScriptCompiler {
  pub fn stringify(&self, ast: &Ast, options: CodegenOptions) -> Result<TransformOutput, Error> {
    ast.visit(|program, context| {
      let keep_comments = options.keep_comments;
      let target = options.target.unwrap_or(EsVersion::latest());
      let source_map_config = options.source_map_config;
      let minify = options.minify.unwrap_or_default();
      let format_opt = JsMinifyFormatOptions {
        inline_script: options.inline_script.unwrap_or(true),
        ascii_only: options.ascii_only.unwrap_or_default(),
        emit_assert_for_import_attributes: options
          .emit_assert_for_import_attributes
          .unwrap_or_default(),
        ..Default::default()
      };
      let print_options = PrintOptions {
        source_map: context.source_map.clone(),
        target,
        source_map_config,
        input_source_map: options.input_source_map,
        minify,
        comments: keep_comments
          .unwrap_or_default()
          .then(|| program.comments.as_ref().map(|c| c as &dyn Comments))
          .flatten(),
        format: &format_opt,
      };
      self.print(program.get_inner_program(), print_options)
    })
  }

  pub fn print(
    &self,
    node: &SwcProgram,
    options: PrintOptions<'_>,
  ) -> Result<TransformOutput, Error> {
    let PrintOptions {
      source_map,
      target,
      mut source_map_config,
      input_source_map,
      minify,
      comments,
      format,
    } = options;
    let mut src_map_buf = vec![];

    if source_map_config.enable {
      let mut v = IdentCollector {
        names: Default::default(),
      };

      node.visit_with(&mut v);

      source_map_config.names = v.names;
    }

    let src = {
      let mut buf = vec![];
      {
        let mut wr = Box::new(text_writer::JsWriter::new(
          source_map.clone(),
          "\n",
          &mut buf,
          source_map_config.enable.then_some(&mut src_map_buf),
        )) as Box<dyn WriteJs>;

        if minify {
          wr = Box::new(text_writer::omit_trailing_semi(wr));
        }

        let mut emitter = Emitter {
          cfg: codegen::Config::default()
            .with_minify(minify)
            .with_target(target)
            .with_ascii_only(format.ascii_only)
            .with_inline_script(format.inline_script),
          comments,
          cm: source_map.clone(),
          wr,
        };
        node.emit_with(&mut emitter).into_diagnostic()?;
      }
      // SAFETY: SWC will emit valid utf8 for sure
      unsafe { String::from_utf8_unchecked(buf) }
    };

    let map = if source_map_config.enable {
      let combined_source_map =
        source_map.build_source_map(&src_map_buf, input_source_map.cloned(), source_map_config);

      let mappings = encode_mappings(combined_source_map.tokens().map(|token| Mapping {
        generated_line: token.get_dst_line() + 1,
        generated_column: token.get_dst_col(),
        original: if token.has_source() {
          Some(OriginalLocation {
            source_index: token.get_src_id(),
            original_line: token.get_src_line() + 1,
            original_column: token.get_src_col(),
            name_index: if token.has_name() {
              Some(token.get_name_id())
            } else {
              None
            },
          })
        } else {
          None
        },
      }));

      let mut rspack_source_map = rspack_sources::SourceMap::new(
        mappings,
        combined_source_map
          .sources()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
        combined_source_map
          .source_contents()
          .flatten()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
        combined_source_map
          .names()
          .map(ToString::to_string)
          .collect::<Vec<_>>(),
      );
      rspack_source_map.set_file(combined_source_map.get_file().map(ToString::to_string));

      Some(rspack_source_map)
    } else {
      None
    };

    Ok(TransformOutput {
      code: src,
      map,
      diagnostics: Default::default(),
    })
  }
}

struct IdentCollector {
  pub names: FxHashMap<BytePos, Atom>,
}

impl Visit for IdentCollector {
  noop_visit_type!();

  fn visit_ident(&mut self, ident: &Ident) {
    self.names.insert(ident.span.lo, ident.sym.clone());
  }
}
