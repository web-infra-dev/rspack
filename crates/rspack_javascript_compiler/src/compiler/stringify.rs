use std::sync::Arc;

use rspack_error::Result;
use rspack_sources::{Mapping, OriginalLocation, encode_mappings};
use rustc_hash::FxHashMap;
use swc_core::{
  base::sourcemap,
  common::{
    BytePos, FileName, SourceMap as SwcSourceMap, comments::Comments,
    source_map::SourceMapGenConfig,
  },
  ecma::{
    ast::{EsVersion, Ident, Program as SwcProgram},
    atoms::Atom,
    codegen::{
      self, Emitter, Node,
      text_writer::{self, WriteJs},
    },
    visit::{Visit, VisitWith, noop_visit_type},
  },
};

use super::{JavaScriptCompiler, TransformOutput};

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
  pub source_len: u32,
  pub source_map: Arc<SwcSourceMap>,
  pub target: EsVersion,
  pub source_map_config: SourceMapConfig,
  pub input_source_map: Option<&'a sourcemap::SourceMap>,
  pub minify: bool,
  pub comments: Option<&'a dyn Comments>,
  pub preamble: &'a str,
  pub ascii_only: bool,
  pub inline_script: bool,
}

impl JavaScriptCompiler {
  pub fn print(&self, node: &SwcProgram, options: PrintOptions<'_>) -> Result<TransformOutput> {
    let PrintOptions {
      source_len,
      source_map,
      target,
      mut source_map_config,
      input_source_map,
      minify,
      comments,
      preamble,
      ascii_only,
      inline_script,
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
      let mut buf = Vec::with_capacity(source_len as usize);
      {
        let mut w = text_writer::JsWriter::new(
          source_map.clone(),
          "\n",
          &mut buf,
          source_map_config.enable.then_some(&mut src_map_buf),
        );

        w.preamble(preamble)?;
        let mut wr = Box::new(w) as Box<dyn WriteJs>;

        if minify {
          wr = Box::new(text_writer::omit_trailing_semi(wr));
        }

        let mut emitter = Emitter {
          cfg: codegen::Config::default()
            .with_minify(minify)
            .with_target(target)
            .with_ascii_only(ascii_only)
            .with_inline_script(inline_script),
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
          .map(|byte_str| Arc::from(byte_str.to_string()))
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
