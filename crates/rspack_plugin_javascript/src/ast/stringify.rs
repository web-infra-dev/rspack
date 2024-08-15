use std::sync::Arc;

use rspack_ast::javascript::Ast;
use rspack_error::{miette::IntoDiagnostic, Result};
use swc_core::base::config::JsMinifyFormatOptions;
use swc_core::base::sourcemap;
use swc_core::{
  common::{
    collections::AHashMap, comments::Comments, source_map::SourceMapGenConfig, BytePos, FileName,
    SourceMap,
  },
  ecma::{
    ast::{EsVersion, Program as SwcProgram},
    atoms::Atom,
    codegen::{
      self,
      text_writer::{self, WriteJs},
      Emitter, Node,
    },
  },
};

use crate::TransformOutput;

#[derive(Default, Clone, Debug)]
pub struct CodegenOptions<'a> {
  pub target: Option<EsVersion>,
  pub source_map_config: SourceMapConfig,
  pub input_source_map: Option<&'a sourcemap::SourceMap>,
  pub keep_comments: Option<bool>,
  pub minify: Option<bool>,
  pub ascii_only: Option<bool>,
  pub inline_script: Option<bool>,
}

pub fn stringify(ast: &Ast, options: CodegenOptions) -> Result<TransformOutput> {
  ast.visit(|program, context| {
    let keep_comments = options.keep_comments;
    let target = options.target.unwrap_or(EsVersion::latest());
    let source_map_config = options.source_map_config;
    let minify = options.minify.unwrap_or_default();
    let format_opt = JsMinifyFormatOptions {
      inline_script: options.inline_script.unwrap_or(true),
      ascii_only: options.ascii_only.unwrap_or_default(),
      ..Default::default()
    };
    print(
      program.get_inner_program(),
      context.source_map.clone(),
      target,
      source_map_config,
      options.input_source_map,
      minify,
      keep_comments
        .unwrap_or_default()
        .then(|| program.comments.as_ref().map(|c| c as &dyn Comments))
        .flatten(),
      &format_opt,
    )
  })
}

#[allow(clippy::too_many_arguments)]
pub fn print(
  node: &SwcProgram,
  source_map: Arc<SourceMap>,
  target: EsVersion,
  source_map_config: SourceMapConfig,
  input_source_map: Option<&sourcemap::SourceMap>,
  minify: bool,
  comments: Option<&dyn Comments>,
  format: &JsMinifyFormatOptions,
) -> Result<TransformOutput> {
  let mut src_map_buf = vec![];

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
    let mut buf = vec![];

    source_map
      .build_source_map_with_config(&src_map_buf, input_source_map, source_map_config)
      .to_writer(&mut buf)
      .unwrap_or_else(|e| panic!("{}", e.to_string()));
    // SAFETY: This buffer is already sanitized
    Some(unsafe { String::from_utf8_unchecked(buf) })
  } else {
    None
  };
  Ok(TransformOutput { code: src, map })
}

#[derive(Default, Clone, Debug)]
pub struct SourceMapConfig {
  pub enable: bool,
  pub inline_sources_content: bool,
  pub emit_columns: bool,
  pub names: AHashMap<BytePos, Atom>,
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
