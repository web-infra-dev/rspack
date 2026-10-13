use std::{
  borrow::Cow,
  cell::OnceCell,
  hash::{Hash, Hasher},
  io::{self, Cursor, Read, Write},
  sync::{Arc, OnceLock},
};

use xxhash_rust::xxh3::Xxh3;

use crate::{
  MapOptions, ObjectPool, Source, SourceMap, SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, TextSpan, stream_chunks_of_raw_source},
};

/// A final text asset retained as a Zstandard frame.
///
/// Flat reads allocate temporary uncompressed content. Writers and readers decompress
/// incrementally. Hashing uses a digest of the original bytes, independent of the frame.
/// Only the borrowed `rope` API requires retaining a materialized text view.
#[cfg_attr(feature = "allocative", derive(allocative::Allocative))]
pub struct AssetSource {
  compressed: Arc<[u8]>,
  size: usize,
  content_hash: u64,
  text: OnceLock<String>,
}

impl AssetSource {
  /// Compress a UTF-8 text asset at Zstandard level 1.
  pub fn new(text: &str) -> io::Result<Self> {
    Self::from_writer(|writer| writer.write_all(text.as_bytes()), |_| {})
  }

  /// Serialize JSON directly into a compressor without retaining an intermediate JSON string.
  pub fn from_json(value: &impl serde::Serialize) -> crate::Result<Self> {
    Self::from_json_with_observer(value, |_| {})
  }

  /// Observe the original JSON bytes while compressing, for caller-specific content hashes.
  pub fn from_json_with_observer(
    value: &impl serde::Serialize,
    observe: impl FnMut(&[u8]),
  ) -> crate::Result<Self> {
    Ok(Self::from_writer(
      |writer| simd_json::to_writer(writer, value).map_err(io::Error::other),
      observe,
    )?)
  }

  fn from_writer(
    write: impl FnOnce(&mut dyn Write) -> io::Result<()>,
    observe: impl FnMut(&[u8]),
  ) -> io::Result<Self> {
    let mut writer = AssetWriter {
      encoder: zstd::stream::write::Encoder::new(Vec::new(), 1)?,
      hash: Xxh3::new(),
      size: 0,
      observe,
    };
    writer.encoder.include_checksum(true)?;
    write(&mut writer)?;
    Ok(Self::from_parts(
      writer.encoder.finish()?.into(),
      writer.size,
      writer.hash.digest(),
    ))
  }

  pub(crate) fn from_parts(compressed: Arc<[u8]>, size: usize, content_hash: u64) -> Self {
    Self {
      compressed,
      size,
      content_hash,
      text: OnceLock::new(),
    }
  }

  /// The encoded frame, suitable for an HTTP `Content-Encoding: zstd` response.
  pub fn compressed_buffer(&self) -> &[u8] {
    &self.compressed
  }

  /// The digest of the uncompressed bytes, computed while compressing.
  pub fn content_hash(&self) -> u64 {
    self.content_hash
  }

  /// Create an independent streaming decoder retaining shared ownership of the frame.
  pub fn reader(&self) -> io::Result<impl Read + Send + 'static> {
    zstd::stream::read::Decoder::with_buffer(Cursor::new(Arc::clone(&self.compressed)))
  }

  /// Compare decoded bytes with a reader using bounded temporary buffers.
  pub fn matches_reader(&self, mut other: impl Read) -> io::Result<bool> {
    let mut reader = self.reader()?;
    let mut decoded = vec![0; 64 * 1024].into_boxed_slice();
    let mut existing = vec![0; 64 * 1024].into_boxed_slice();
    loop {
      let size = reader.read(&mut decoded)?;
      if size == 0 {
        return Ok(other.read(&mut existing[..1])? == 0);
      }
      match other.read_exact(&mut existing[..size]) {
        Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return Ok(false),
        result => result?,
      }
      if decoded[..size] != existing[..size] {
        return Ok(false);
      }
    }
  }

  #[cfg(feature = "rspack_cacheable")]
  pub(crate) fn validate(&self) -> io::Result<()> {
    let mut reader = self.reader()?;
    let mut bytes = vec![0; 64 * 1024 + 3].into_boxed_slice();
    let mut pending = 0;
    let mut size = 0;
    let mut hash = Xxh3::new();
    loop {
      let count = reader.read(&mut bytes[pending..])?;
      if count == 0 {
        break;
      }
      hash.update(&bytes[pending..pending + count]);
      size += count;
      if size > self.size {
        return Err(io::Error::new(
          io::ErrorKind::InvalidData,
          "AssetSource size mismatch",
        ));
      }
      let count = pending + count;
      pending = match std::str::from_utf8(&bytes[..count]) {
        Ok(_) => 0,
        Err(error) if error.error_len().is_none() => {
          let pending = count - error.valid_up_to();
          bytes.copy_within(error.valid_up_to()..count, 0);
          pending
        }
        Err(error) => return Err(io::Error::new(io::ErrorKind::InvalidData, error)),
      };
    }
    if pending != 0 || size != self.size || hash.digest() != self.content_hash {
      return Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "AssetSource content mismatch",
      ));
    }
    Ok(())
  }

  fn decompress(&self) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(self.size);
    self
      .reader()
      .and_then(|mut reader| reader.read_to_end(&mut bytes))
      .expect("AssetSource contains a valid Zstandard frame");
    bytes
  }

  fn decompress_text(&self) -> String {
    String::from_utf8(self.decompress()).expect("AssetSource contains UTF-8 text")
  }
}

struct AssetWriter<'a, F> {
  encoder: zstd::stream::write::Encoder<'a, Vec<u8>>,
  hash: Xxh3,
  size: usize,
  observe: F,
}

impl<F: FnMut(&[u8])> Write for AssetWriter<'_, F> {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    let written = self.encoder.write(bytes)?;
    self.hash.update(&bytes[..written]);
    (self.observe)(&bytes[..written]);
    self.size += written;
    Ok(written)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.encoder.flush()
  }
}

impl Source for AssetSource {
  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Owned(self.decompress_text()))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    on_chunk(self.text.get_or_init(|| self.decompress_text()));
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    Cow::Owned(self.decompress())
  }

  fn size(&self) -> usize {
    self.size
  }

  fn map(&self, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'_>> {
    None
  }

  fn map_static(self: Arc<Self>, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'static>> {
    None
  }

  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    io::copy(&mut self.reader()?, writer)?;
    Ok(())
  }
}

impl Clone for AssetSource {
  fn clone(&self) -> Self {
    Self::from_parts(Arc::clone(&self.compressed), self.size, self.content_hash)
  }
}

impl Hash for AssetSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "AssetSource".hash(state);
    self.size.hash(state);
    self.content_hash.hash(state);
  }
}

impl PartialEq for AssetSource {
  fn eq(&self, other: &Self) -> bool {
    self.size == other.size
      && self.content_hash == other.content_hash
      && (self.compressed == other.compressed || self.buffer() == other.buffer())
  }
}

impl Eq for AssetSource {}

impl std::fmt::Debug for AssetSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("AssetSource")
      .field("size", &self.size)
      .field("compressed_size", &self.compressed.len())
      .field("content_hash", &self.content_hash)
      .finish()
  }
}

struct AssetChunks<'source> {
  source: &'source AssetSource,
  text: OnceCell<String>,
}

impl<'source> Chunks<'source> for AssetChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    _: &ObjectPool,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    on_source: crate::helpers::OnSource<'_, 'source>,
    on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    let text = self.text.get_or_init(|| self.source.decompress_text());
    stream_chunks_of_raw_source(TextSpan::new(text), options, on_chunk, on_source, on_name)
  }
}

impl StreamChunks for AssetSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(AssetChunks {
      source: self,
      text: OnceCell::new(),
    })
  }
}
