/**
 * Some code is modified based on
 * https://github.com/swc-project/swc/blob/v33.0.0/crates/swc_ecma_codegen/src/text_writer/basic_impl.rs
 * Apache-2.0 licensed
 * Author Donny/강동윤
 * Copyright (c)
 */
use std::io::{self, Write};

use memchr::memchr2;
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
  last_mapping_loc: LineCol,
  last_source: BytePos,
  extra_sources: Vec<BytePos>,
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
      last_mapping_loc: LineCol {
        line: u32::MAX,
        col: u32::MAX,
      },
      last_source: BytePos::DUMMY,
      extra_sources: Vec::new(),
    }
  }

  pub(super) fn preamble(&mut self, text: &str) -> io::Result<()> {
    self.writer.write_all(text.as_bytes())?;
    self.update_position(text);
    Ok(())
  }

  #[inline]
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

  #[inline]
  fn update_position(&mut self, text: &str) {
    let bytes = text.as_bytes();
    match bytes {
      [] => return,
      [b'\r' | b'\n'] => {
        self.line += 1;
        self.column = 0;
        return;
      }
      [_] => {
        self.column += 1;
        return;
      }
      _ => {}
    }

    let mut line_start = 0;
    while let Some(offset) = memchr2(b'\r', b'\n', &bytes[line_start..]) {
      let pos = line_start + offset;
      self.line += 1;
      line_start = if bytes[pos] == b'\r' && bytes.get(pos + 1) == Some(&b'\n') {
        pos + 2
      } else {
        pos + 1
      };
    }

    let tail = &text[line_start..];
    if line_start == 0 {
      self.column += utf16_columns(tail);
    } else {
      self.column = utf16_columns(tail);
    }
  }

  #[inline]
  fn record_mapping(&mut self, source: BytePos) {
    if source.is_dummy() && source != BytePos(u32::MAX) {
      return;
    }
    let loc = LineCol {
      line: self.line as u32,
      col: self.column as u32,
    };
    if self.last_mapping_loc != loc {
      // Generated positions only advance, so earlier positions cannot recur.
      self.last_mapping_loc = loc;
      self.last_source = source;
      self.extra_sources.clear();
      (self.on_mapping)(source, loc);
      return;
    }
    if self.last_source == source || self.extra_sources.contains(&source) {
      return;
    }
    self.extra_sources.push(source);
    (self.on_mapping)(source, loc);
  }
}

#[inline]
fn utf16_columns(text: &str) -> usize {
  if text.is_ascii() {
    text.len()
  } else {
    rspack_sources::utf16_len(text)
  }
}

impl<W: Write, F: FnMut(BytePos, LineCol)> WriteJs for SourceMapWriter<W, F> {
  #[inline]
  fn increase_indent(&mut self) -> io::Result<()> {
    self.indent += 1;
    Ok(())
  }

  #[inline]
  fn decrease_indent(&mut self) -> io::Result<()> {
    self.indent -= 1;
    Ok(())
  }

  #[inline]
  fn write_semi(&mut self, span: Option<Span>) -> io::Result<()> {
    self.write(span, ";")
  }

  #[inline]
  fn write_space(&mut self) -> io::Result<()> {
    self.write(None, " ")
  }

  #[inline]
  fn write_keyword(&mut self, span: Option<Span>, text: &'static str) -> io::Result<()> {
    self.write(span, text)
  }

  #[inline]
  fn write_operator(&mut self, span: Option<Span>, text: &str) -> io::Result<()> {
    self.write(span, text)
  }

  #[inline]
  fn write_param(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  #[inline]
  fn write_property(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  #[inline]
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

  #[inline]
  fn write_lit(&mut self, span: Span, text: &str) -> io::Result<()> {
    if !text.is_empty() {
      self.record_mapping(span.lo());
      self.write(None, text)?;
      self.record_mapping(span.hi());
    }
    Ok(())
  }

  #[inline]
  fn write_comment(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  #[inline]
  fn write_str_lit(&mut self, span: Span, text: &str) -> io::Result<()> {
    self.write_lit(span, text)
  }

  #[inline]
  fn write_str(&mut self, text: &str) -> io::Result<()> {
    self.write(None, text)
  }

  #[inline]
  fn write_symbol(&mut self, span: Span, text: &str) -> io::Result<()> {
    self.write(Some(span), text)
  }

  #[inline]
  fn write_punct(
    &mut self,
    span: Option<Span>,
    text: &'static str,
    _commit_pending_semi: bool,
  ) -> io::Result<()> {
    self.write(span, text)
  }

  #[inline]
  fn care_about_srcmap(&self) -> bool {
    true
  }

  #[inline]
  fn add_srcmap(&mut self, position: BytePos) -> io::Result<()> {
    if self.line_start {
      self.pending_mapping = Some(position);
    } else {
      self.record_mapping(position);
    }
    Ok(())
  }

  #[inline]
  fn commit_pending_semi(&mut self) -> io::Result<()> {
    Ok(())
  }
}
