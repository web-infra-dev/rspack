use std::{
  hash::Hasher,
  io::{self, Write},
};

use crate::{CachedSource, DevToolCodeSource, DevToolMapSource, ReplaceSource, Source};

/// Whether this is a text devtool asset, possibly behind edit/cache wrappers.
/// Other source graphs retain their existing content and equality semantics.
pub fn is_devtool_source(source: &dyn Source) -> bool {
  devtool_source(source).is_some()
}

// Ignore cache wrappers for comparison, but keep edits on the writer side.
fn devtool_source(source: &dyn Source) -> Option<&dyn Source> {
  let any = source.as_any();
  if any.is::<DevToolCodeSource>() || any.is::<DevToolMapSource>() {
    return Some(source);
  }
  if let Some(edited) = any.downcast_ref::<ReplaceSource>() {
    return is_devtool_source(edited.inner().as_ref()).then_some(source);
  }
  if let Some(cached) = any.downcast_ref::<CachedSource>() {
    return devtool_source(cached.inner().as_ref());
  }
  None
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
  let (writer, other) = match (devtool_source(a), devtool_source(b)) {
    (Some(a), Some(b)) => {
      if std::ptr::eq(a, b) {
        return true;
      }
      for (source, other) in [(a, b), (b, a)] {
        if let Some(map) = source.as_any().downcast_ref::<DevToolMapSource>() {
          return map.content_eq(other);
        }
      }
      (a, b)
    }
    (Some(a), None) => (a, b),
    (None, Some(b)) => (b, a),
    (None, None) => return a.buffer() == b.buffer(),
  };
  matches_chunks(writer, std::iter::once(other.buffer().as_ref()))
}

pub(super) fn matches_chunks<'a>(
  source: &dyn Source,
  chunks: impl Iterator<Item = &'a [u8]>,
) -> bool {
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
