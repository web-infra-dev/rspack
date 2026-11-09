use std::{
  hash::Hasher,
  io::{self, Write},
};

use crate::{
  CachedSource, ConcatSource, DevToolCodeSource, DevToolMapSource, ReplaceSource, Source,
};

/// Whether this is a text devtool asset, possibly behind edit/cache wrappers.
/// Other source graphs retain their existing content and equality semantics.
pub fn is_devtool_text_source(source: &dyn Source) -> bool {
  let any = source.as_any();
  if any.is::<DevToolMapSource>() {
    return true;
  }
  if let Some(code) = any.downcast_ref::<DevToolCodeSource>() {
    return code.is_text();
  }
  if let Some(source) = any.downcast_ref::<ReplaceSource>() {
    return is_devtool_text_source(source.inner().as_ref());
  }
  if let Some(source) = any.downcast_ref::<CachedSource>() {
    return is_devtool_text_source(source.inner().as_ref());
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

/// Compare rendered bytes, streaming edits through the existing source writer.
/// Unknown and binary sources retain buffer semantics.
pub fn source_content_eq(a: &dyn Source, b: &dyn Source) -> bool {
  if std::ptr::eq(a, b) {
    return true;
  }
  let (stream, other) = match (streaming_source(a), streaming_source(b)) {
    // Prefer a map on the iterator side so both sources can stream.
    (Some(a), Some(b)) if a.as_any().is::<DevToolMapSource>() => (b, a),
    (Some(a), Some(b)) => (a, b),
    (Some(a), None) => (a, b),
    (None, Some(b)) => (b, a),
    (None, None) => return a.buffer() == b.buffer(),
  };
  if std::ptr::eq(stream, other) {
    return true;
  }
  if let Some(map) = other.as_any().downcast_ref::<DevToolMapSource>() {
    if stream.as_any().downcast_ref::<DevToolMapSource>() == Some(map) {
      return true;
    }
    return matches_chunks(stream, map.json_chunks().map(str::as_bytes));
  }
  matches_chunks(stream, std::iter::once(other.buffer().as_ref()))
}

// Follow wrappers that delegate buffer() to a single source. Stream only when
// to_writer() emits exactly those bytes (an unedited ReplaceSource need not).
fn streaming_source(source: &dyn Source) -> Option<&dyn Source> {
  let any = source.as_any();
  if let Some(cached) = any.downcast_ref::<CachedSource>() {
    return streaming_source(cached.inner().as_ref());
  }
  if let Some(code) = any.downcast_ref::<DevToolCodeSource>() {
    return if code.suffix().size() == 0 {
      streaming_source(code.inner().as_ref())
    } else {
      Some(source)
    };
  }
  if let Some(concat) = any.downcast_ref::<ConcatSource>() {
    return if let [child] = concat.children() {
      streaming_source(child.as_ref())
    } else {
      Some(source)
    };
  }
  (any.is::<DevToolMapSource>()
    || any
      .downcast_ref::<ReplaceSource>()
      .is_some_and(|s| !s.replacements().is_empty()))
  .then_some(source)
}

fn matches_chunks<'a>(source: &dyn Source, chunks: impl Iterator<Item = &'a [u8]>) -> bool {
  let mut writer = CompareWriter {
    chunks,
    pending: &[],
  };
  source.to_writer(&mut writer).is_ok()
    && writer.pending.is_empty()
    && writer.chunks.all(|chunk| chunk.is_empty())
}

struct CompareWriter<'a, I> {
  chunks: I,
  pending: &'a [u8],
}
impl<'a, I: Iterator<Item = &'a [u8]>> Write for CompareWriter<'a, I> {
  fn write(&mut self, mut bytes: &[u8]) -> io::Result<usize> {
    let len = bytes.len();
    while !bytes.is_empty() {
      while self.pending.is_empty() {
        self.pending = self
          .chunks
          .next()
          .ok_or_else(|| io::Error::other("source contents differ"))?;
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
