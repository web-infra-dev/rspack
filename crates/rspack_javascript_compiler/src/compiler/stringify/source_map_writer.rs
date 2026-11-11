// SPDX-License-Identifier: Apache-2.0
// Adapted from swc_ecma_codegen 33.0.0's JsWriter, originally ported from
// TypeScript's createTextWriter. Copyright the SWC contributors.
// https://docs.rs/crate/swc_ecma_codegen/33.0.0/source/src/text_writer/basic_impl.rs
// Changes: mapping deduplication is scoped to the current output position;
// indentation and newlines use Rspack's fixed settings; scope tracking is omitted.
// See ../../../LICENSE-APACHE for the license.

use std::io::{self, Write};

use memchr::memchr2;
use rustc_hash::FxHashSet;
use swc_core::{
  common::{BytePos, LineCol, Span},
  ecma::codegen::text_writer::WriteJs,
};

pub(super) struct SourceMapWriter<W, F> {
  writer: W,
  on_mapping: F,
  indent: usize,
  line_start: bool,
  line: usize,
  column: usize,
  pending_mapping: Option<BytePos>,
  mapping_position: Option<(usize, usize)>,
  seen_positions: FxHashSet<BytePos>,
}

impl<W: Write, F: FnMut(BytePos, LineCol)> SourceMapWriter<W, F> {
  pub(super) fn new(writer: W, on_mapping: F) -> Self {
    Self {
      writer,
      on_mapping,
      indent: 0,
      line_start: true,
      line: 0,
      column: 0,
      pending_mapping: None,
      mapping_position: None,
      seen_positions: Default::default(),
    }
  }

  pub(super) fn preamble(&mut self, text: &str) -> io::Result<()> {
    self.writer.write_all(text.as_bytes())?;
    self.update_position(text);
    Ok(())
  }

  fn write(&mut self, span: Option<Span>, text: &str) -> io::Result<()> {
    if text.is_empty() {
      return Ok(());
    }
    if self.line_start {
      for _ in 0..self.indent {
        self.writer.write_all(b"    ")?;
      }
      self.column += self.indent * 4;
      self.line_start = false;
      if let Some(pending) = self.pending_mapping.take() {
        self.record_mapping(pending);
      }
    }
    if let Some(span) = span {
      self.record_mapping(span.lo());
    }
    self.writer.write_all(text.as_bytes())?;
    self.update_position(text);
    if let Some(span) = span {
      self.record_mapping(span.hi());
    }
    Ok(())
  }

  fn update_position(&mut self, text: &str) {
    if text.is_empty() {
      return;
    }
    let bytes = text.as_bytes();
    if bytes.len() == 1 {
      match bytes[0] {
        b'\r' | b'\n' => {
          self.line += 1;
          self.column = 0;
        }
        _ => self.column += 1,
      }
      return;
    }
    if memchr2(b'\r', b'\n', bytes).is_none() {
      self.column += if text.is_ascii() {
        bytes.len()
      } else {
        text.encode_utf16().count()
      };
      return;
    }

    let mut position = 0;
    let mut line_start = 0;
    while position < bytes.len() {
      match bytes[position] {
        b'\r' => {
          self.line += 1;
          position += 1;
          if position < bytes.len() && bytes[position] == b'\n' {
            position += 1;
          }
          line_start = position;
        }
        b'\n' => {
          self.line += 1;
          position += 1;
          line_start = position;
        }
        _ => position += 1,
      }
    }
    self.column = if text.is_ascii() {
      bytes.len() - line_start
    } else {
      text[line_start..].encode_utf16().count()
    };
  }

  fn record_mapping(&mut self, source: BytePos) {
    if source.is_dummy() && source != BytePos(u32::MAX) {
      return;
    }
    let position = (self.line, self.column);
    if self.mapping_position != Some(position) {
      // Generated positions only advance, so an earlier position cannot be a duplicate.
      self.seen_positions.clear();
      self.mapping_position = Some(position);
    }
    if self.seen_positions.insert(source) {
      (self.on_mapping)(
        source,
        LineCol {
          line: self.line as u32,
          col: self.column as u32,
        },
      );
    }
  }
}

impl<W: Write, F: FnMut(BytePos, LineCol)> WriteJs for SourceMapWriter<W, F> {
  fn increase_indent(&mut self) -> io::Result<()> {
    self.indent += 1;
    Ok(())
  }

  fn decrease_indent(&mut self) -> io::Result<()> {
    self.indent -= 1;
    Ok(())
  }

  fn write_semi(&mut self, span: Option<Span>) -> io::Result<()> {
    self.write(span, ";")
  }

  fn write_space(&mut self) -> io::Result<()> {
    self.write(None, " ")
  }

  fn write_keyword(&mut self, span: Option<Span>, text: &'static str) -> io::Result<()> {
    self.write(span, text)
  }

  fn write_operator(&mut self, span: Option<Span>, text: &str) -> io::Result<()> {
    self.write(span, text)
  }

  fn write_param(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  fn write_property(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  fn write_line(&mut self) -> io::Result<()> {
    let pending = self.pending_mapping.take();
    if !self.line_start {
      self.writer.write_all(b"\n")?;
      self.line += 1;
      self.column = 0;
      self.line_start = true;
      if let Some(pending) = pending {
        self.record_mapping(pending);
      }
    }
    Ok(())
  }

  fn write_lit(&mut self, span: Span, text: &str) -> io::Result<()> {
    if !text.is_empty() {
      self.record_mapping(span.lo());
      self.write(None, text)?;
      self.record_mapping(span.hi());
    }
    Ok(())
  }

  fn write_comment(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  fn write_str_lit(&mut self, span: Span, text: &str) -> io::Result<()> {
    self.write_lit(span, text)
  }

  fn write_str(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  fn write_symbol(&mut self, span: Span, text: &str) -> io::Result<()> {
    self.write(Some(span), text)
  }

  fn write_punct(
    &mut self,
    span: Option<Span>,
    text: &'static str,
    _commit_pending_semi: bool,
  ) -> io::Result<()> {
    self.write(span, text)
  }

  fn care_about_srcmap(&self) -> bool {
    true
  }

  fn add_srcmap(&mut self, position: BytePos) -> io::Result<()> {
    if self.line_start {
      self.pending_mapping = Some(position);
    } else {
      self.record_mapping(position);
    }
    Ok(())
  }

  fn commit_pending_semi(&mut self) -> io::Result<()> {
    Ok(())
  }
}
