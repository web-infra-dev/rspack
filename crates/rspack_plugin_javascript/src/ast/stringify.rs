use std::sync::Arc;

use rspack_core::{ast::javascript::Ast, Devtool};
use rspack_error::{internal_error, Result};
use swc_core::{
  base::TransformOutput,
  common::{
    collections::AHashMap, comments::Comments, source_map::SourceMapGenConfig, BytePos, FileName,
    SourceMap,
  },
  ecma::{
    ast::{EsVersion, Program as SwcProgram},
    atoms::JsWord,
    codegen::{
      self,
      text_writer::{self, WriteJs},
      Emitter, Node,
    },
  },
};

pub fn stringify(ast: &Ast, devtool: &Devtool) -> Result<TransformOutput> {
  ast.visit(|program, context| {
    print(
      program.get_inner_program(),
      context.source_map.clone(),
      EsVersion::Es2022,
      SourceMapConfig {
        enable: devtool.source_map(),
        inline_sources_content: !devtool.no_sources(),
        emit_columns: !devtool.cheap(),
        names: Default::default(),
      },
      false,
      program.comments.as_ref().map(|c| c as &dyn Comments),
      false,
    )
  })
}

pub fn print(
  node: &SwcProgram,
  source_map: Arc<SourceMap>,
  target: EsVersion,
  source_map_config: SourceMapConfig,
  minify: bool,
  comments: Option<&dyn Comments>,
  ascii_only: bool,
) -> Result<TransformOutput> {
  let mut src_map_buf = vec![];

  let src = {
    let mut buf = vec![];
    {
      let mut wr = Box::new(text_writer::JsWriter::new(
        source_map.clone(),
        "\n",
        &mut buf,
        if source_map_config.enable {
          Some(&mut src_map_buf)
        } else {
          None
        },
      )) as Box<dyn WriteJs>;

      if minify {
        wr = Box::new(text_writer::omit_trailing_semi(wr));
      }

      let mut emitter = Emitter {
        cfg: codegen::Config {
          minify,
          target,
          ascii_only,
          ..Default::default()
        },
        comments,
        cm: source_map.clone(),
        wr,
      };

      node.emit_with(&mut emitter)?;
    }
    // SAFETY: SWC will emit valid utf8 for sure
    unsafe { String::from_utf8_unchecked(buf) }
  };

  let map = if source_map_config.enable {
    let mut buf = vec![];

    source_map
      .build_source_map_with_config(&src_map_buf, None, source_map_config)
      .to_writer(&mut buf)
      .map_err(|e| internal_error!(e.to_string()))?;
    // SAFETY: This buffer is already sanitized
    Some(unsafe { String::from_utf8_unchecked(buf) })
  } else {
    None
  };
  Ok(TransformOutput { code: src, map })
}

pub struct SourceMapConfig {
  pub enable: bool,
  pub inline_sources_content: bool,
  pub emit_columns: bool,
  pub names: AHashMap<BytePos, JsWord>,
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
