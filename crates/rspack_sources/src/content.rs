use std::{
  hash::Hasher,
  io::{self, Write},
};

use crate::{RawString, Source};

/// Hash emitted bytes, independently of source type or rope boundaries.
/// Unlike Source::update_hash, this is a content hash, not a graph hash.
pub fn update_content_hash(source: &dyn Source, hasher: &mut dyn Hasher) {
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
  let mut writer = HashWriter(hasher);
  match source.raw_string() {
    // Coalesce small writes from JSON serialization and edits.
    Some(RawString::Rendered(_)) => {
      let mut writer = io::BufWriter::with_capacity(64 * 1024, writer);
      source.to_writer(&mut writer).expect("hash source contents");
      writer.flush().expect("flush source content to hasher");
    }
    _ => source.to_writer(&mut writer).expect("hash source contents"),
  }
}

/// Compare emitted bytes, materializing at most one side of the comparison.
pub fn source_content_eq(a: &dyn Source, b: &dyn Source) -> bool {
  if std::ptr::eq(a, b) {
    return true;
  }
  struct CompareWriter<'a>(&'a [u8]);

  impl Write for CompareWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      self.0 = self
        .0
        .strip_prefix(bytes)
        .ok_or(io::ErrorKind::InvalidData)?;
      Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }

  let bytes = b.buffer();
  let mut writer = CompareWriter(&bytes);
  a.to_writer(&mut writer).is_ok() && writer.0.is_empty()
}
