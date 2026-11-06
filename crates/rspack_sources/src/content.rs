use std::{
  hash::Hasher,
  io::{self, Write},
};

use crate::{
  CachedSource, ConcatSource, DevToolCodeSource, DevToolMapSource, ReplaceSource, Replacement,
  Source, dev_tool_code_source::has_utf8_text,
};

/// Whether this is a text devtool asset, possibly behind edit/cache wrappers.
/// Other source graphs retain their existing content and equality semantics.
pub fn is_source_map_text(source: &dyn Source) -> bool {
  let any = source.as_any();
  if any.is::<DevToolMapSource>() {
    return true;
  }
  if let Some(code) = any.downcast_ref::<DevToolCodeSource>() {
    return code.is_text();
  }
  if let Some(source) = any.downcast_ref::<ReplaceSource>() {
    return is_source_map_text(source.inner().as_ref());
  }
  if let Some(source) = any.downcast_ref::<CachedSource>() {
    return is_source_map_text(source.inner().as_ref());
  }
  false
}

/// Hash emitted UTF-8 bytes, independently of source type or rope boundaries.
/// Unlike Source::update_hash, this is a content hash, not a graph hash.
pub fn update_text_content_hash(source: &dyn Source, hasher: &mut dyn Hasher) {
  struct HashWriter<'a>(&'a mut dyn Hasher);
  impl Write for HashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.0.write(bytes);
      Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }
  // Coalesce escaped JSON fragments; bare maps retain their SIMD writer path.
  let mut writer = io::BufWriter::with_capacity(64 * 1024, HashWriter(hasher));
  source
    .to_writer(&mut writer)
    .expect("write source content to hasher");
  writer.flush().expect("flush source content to hasher");
}

/// Compare rendered bytes without flattening known UTF-8 source graphs.
/// Unknown and binary sources retain buffer semantics.
pub fn source_content_eq(a: &dyn Source, b: &dyn Source) -> bool {
  if std::ptr::eq(a, b) {
    return true;
  }
  if let (Some(a), Some(b)) = (
    a.as_any().downcast_ref::<DevToolMapSource>(),
    b.as_any().downcast_ref::<DevToolMapSource>(),
  ) && a == b
  {
    return true;
  }
  if !has_utf8_text(a) || !has_utf8_text(b) {
    return a.buffer() == b.buffer();
  }
  let mut writer = CompareWriter {
    chunks: text_chunks(a),
    pending: &[],
  };
  b.to_writer(&mut writer).is_ok()
    && writer.pending.is_empty()
    && writer.chunks.all(|chunk| chunk.is_empty())
}

type TextChunks<'a> = Box<dyn Iterator<Item = &'a str> + 'a>;

fn text_chunks(source: &dyn Source) -> TextChunks<'_> {
  let any = source.as_any();
  if let Some(map) = any.downcast_ref::<DevToolMapSource>() {
    return Box::new(map.json_chunks_iter());
  }
  if let Some(code) = any.downcast_ref::<DevToolCodeSource>() {
    return Box::new(text_chunks(code.inner().as_ref()).chain(text_chunks(code.suffix().as_ref())));
  }
  if let Some(source) = any.downcast_ref::<CachedSource>() {
    return text_chunks(source.inner().as_ref());
  }
  if let Some(source) = any.downcast_ref::<ConcatSource>() {
    return Box::new(
      source
        .children()
        .iter()
        .flat_map(|child| text_chunks(child.as_ref())),
    );
  }
  if let Some(source) = any.downcast_ref::<ReplaceSource>() {
    return Box::new(ReplacedChunks {
      chunks: text_chunks(source.inner().as_ref()),
      replacements: source.replacements().iter().peekable(),
      pending: "",
      position: 0,
      skip_until: 0,
    });
  }
  // Known text leaves expose borrowed chunks. Map escapes and graph wrappers
  // above stay lazy instead of collecting a reference per JSON escape.
  let mut chunks = Vec::new();
  source.rope(&mut |chunk| chunks.push(chunk));
  Box::new(chunks.into_iter())
}

// Pull counterpart of ReplaceSource::rope for equality's left-hand cursor.
// Preserve its ordering, overlap, insertion-at-end and beyond-end semantics.
struct ReplacedChunks<'a> {
  chunks: TextChunks<'a>,
  replacements: std::iter::Peekable<std::slice::Iter<'a, Replacement>>,
  pending: &'a str,
  position: usize,
  skip_until: usize,
}

impl<'a> Iterator for ReplacedChunks<'a> {
  type Item = &'a str;
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.pending.is_empty() {
        match self.chunks.next() {
          Some(chunk) => {
            self.pending = chunk;
            continue;
          }
          None => return self.replacements.next().map(Replacement::content),
        }
      }
      if self.position < self.skip_until {
        let n = (self.skip_until - self.position).min(self.pending.len());
        self.pending = &self.pending[n..];
        self.position += n;
        continue;
      }
      if let Some(replacement) = self.replacements.peek()
        && (replacement.start() as usize) < self.position + self.pending.len()
      {
        if replacement.start() as usize > self.position {
          let n = replacement.start() as usize - self.position;
          let chunk = &self.pending[..n];
          self.pending = &self.pending[n..];
          self.position += n;
          return Some(chunk);
        }
        let replacement = self.replacements.next().expect("peeked replacement");
        self.skip_until = self.skip_until.max(replacement.end() as usize);
        return Some(replacement.content());
      }
      let chunk = self.pending;
      self.pending = "";
      self.position += chunk.len();
      return Some(chunk);
    }
  }
}

struct CompareWriter<'a> {
  chunks: TextChunks<'a>,
  pending: &'a [u8],
}
impl Write for CompareWriter<'_> {
  fn write(&mut self, mut bytes: &[u8]) -> io::Result<usize> {
    let len = bytes.len();
    while !bytes.is_empty() {
      while self.pending.is_empty() {
        self.pending = self
          .chunks
          .next()
          .ok_or_else(|| io::Error::other("source contents differ"))?
          .as_bytes();
      }
      let n = bytes.len().min(self.pending.len());
      if bytes[..n] != self.pending[..n] {
        return Err(io::Error::other("source contents differ"));
      }
      self.pending = &self.pending[n..];
      bytes = &bytes[n..];
    }
    Ok(len)
  }
  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}
