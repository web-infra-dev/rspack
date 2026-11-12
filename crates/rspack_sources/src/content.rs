use std::{
  hash::Hasher,
  io::{self, Write},
};

use crate::{RawBufferSource, RawStringSource, Source};

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
  // Raw assets already write one contiguous slice; buffering only allocates and copies it.
  let any = source.as_any();
  if let Some(raw) = any.downcast_ref::<RawStringSource>() {
    return raw.to_writer(&mut writer).expect("hash source contents");
  }
  if let Some(raw) = any.downcast_ref::<RawBufferSource>() {
    return raw.to_writer(&mut writer).expect("hash source contents");
  }
  // Coalesce small writes from JSON serialization and edits.
  let mut writer = io::BufWriter::new(writer);
  source.to_writer(&mut writer).expect("hash source contents");
  writer.flush().expect("flush source content to hasher");
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
