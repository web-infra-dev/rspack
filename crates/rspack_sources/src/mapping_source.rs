use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
  sync::{Arc, OnceLock},
};

use crate::{
  MapOptions, RawBufferSource, RawStringSource, Source, SourceMap, SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, stream_chunks_of_raw_rope},
  source::is_all_empty,
};

#[cfg_attr(feature = "allocative", derive(rspack_allocative::Allocative))]
pub(crate) struct MapData {
  pub(crate) map: SourceMap<'static>,
  version: String,
  ignore_list: Option<String>,
  size: OnceLock<usize>,
}

// The serialized length is a cache, not part of the source's identity.
impl PartialEq for MapData {
  fn eq(&self, other: &Self) -> bool {
    self.map == other.map
  }
}
impl Eq for MapData {}

impl Hash for MapData {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.map.hash(state);
  }
}

/// An external source map asset which retains its original SourceMap instead
/// of retaining another complete JSON string.
///
/// Content APIs serialize on demand. Edits retain their input and byte ranges,
/// and all streaming APIs borrow fragments without flattening the JSON asset.
/// The JSON asset itself has no source map.
#[derive(Clone)]
#[cfg_attr(feature = "allocative", derive(rspack_allocative::Allocative))]
pub struct MappingSource(Arc<MapData>);

impl MappingSource {
  /// Create a JSON source map asset.
  pub fn new(map: SourceMap<'static>) -> Self {
    let version = map.fields().version.to_string();
    let ignore_list = map
      .ignore_list()
      .map(|list| simd_json::to_string(list).expect("serialize ignoreList"));
    let data = MapData {
      map,
      version,
      ignore_list,
      size: OnceLock::new(),
    };
    Self(Arc::new(data))
  }

  /// Visit borrowed JSON fragments. Boundaries may occur within encoded values.
  pub fn json_chunks(&self, on_chunk: &mut dyn FnMut(&str)) {
    self.rope(&mut |chunk| on_chunk(chunk));
  }

  /// Hash the rendered bytes, coalescing small escaped fragments. Unlike the
  /// structural Source hash, this matches hashing the complete output buffer.
  pub fn update_content_hash(&self, hasher: &mut dyn Hasher) {
    let mut writer = io::BufWriter::with_capacity(64 * 1024, HashWriter(hasher));
    self
      .to_writer(&mut writer)
      .expect("write map JSON to hasher");
    writer.flush().expect("flush map JSON to hasher");
  }

  /// Compare rendered bytes without allocating the complete JSON or collecting
  /// a fragment reference for every escaped character.
  pub fn content_eq(&self, other: &dyn Source) -> bool {
    if let Some(other) = other.as_any().downcast_ref::<Self>()
      && self == other
    {
      return true;
    }
    if self.size() != other.size() {
      return false;
    }
    if other.as_any().is::<RawStringSource>() || other.as_any().is::<RawBufferSource>() {
      let buffer = other.buffer();
      let mut writer = CompareWriter {
        chunks: None,
        chunk: buffer.as_ref(),
      };
      return self.to_writer(&mut writer).is_ok() && writer.is_finished();
    }
    let mut writer = CompareWriter {
      chunks: Some(Box::new(self.json_chunks_iter().map(str::as_bytes))),
      chunk: &[],
    };
    other.to_writer(&mut writer).is_ok() && writer.is_finished()
  }

  /// Iterate borrowed serialized JSON fragments without collecting escaped chunks.
  pub fn json_chunks_iter(&self) -> impl Iterator<Item = &str> {
    JsonChunks::new(&self.0)
  }

  /// Visit the original, unescaped source contents without copying them.
  pub fn sources_content(&self) -> impl Iterator<Item = &str> {
    self
      .original_map()
      .fields()
      .sources_content
      .iter()
      .map(AsRef::as_ref)
  }

  fn original_map(&self) -> &SourceMap<'static> {
    &self.0.map
  }

  #[cfg(feature = "rspack_cacheable")]
  pub(crate) fn data(&self) -> &MapData {
    &self.0
  }
}

struct CompareWriter<'a> {
  chunks: Option<Box<dyn Iterator<Item = &'a [u8]> + 'a>>,
  chunk: &'a [u8],
}

impl CompareWriter<'_> {
  fn is_finished(&mut self) -> bool {
    self.chunk.is_empty()
      && self
        .chunks
        .as_mut()
        .is_none_or(|chunks| chunks.all(|chunk| chunk.is_empty()))
  }
}

struct HashWriter<'a>(&'a mut dyn Hasher);

struct SizeWriter(usize);

impl Write for SizeWriter {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    self.0 += bytes.len();
    Ok(bytes.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl Write for HashWriter<'_> {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    self.0.write(bytes);
    Ok(bytes.len())
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

impl Write for CompareWriter<'_> {
  fn write(&mut self, mut bytes: &[u8]) -> io::Result<usize> {
    let len = bytes.len();
    while !bytes.is_empty() {
      while self.chunk.is_empty() {
        self.chunk = self
          .chunks
          .as_mut()
          .and_then(|chunks| chunks.next())
          .ok_or_else(|| io::Error::other("source contents differ"))?;
      }
      let n = bytes.len().min(self.chunk.len());
      if bytes[..n] != self.chunk[..n] {
        return Err(io::Error::other("source contents differ"));
      }
      self.chunk = &self.chunk[n..];
      bytes = &bytes[n..];
    }
    Ok(len)
  }

  fn flush(&mut self) -> io::Result<()> {
    Ok(())
  }
}

enum Part<'a> {
  Raw(&'a str),
  String(&'a str),
}

struct JsonChunks<'a> {
  parts: std::vec::IntoIter<Part<'a>>,
  string: Option<JsonStringChunks<'a>>,
}

impl<'a> JsonChunks<'a> {
  fn new(data: &'a MapData) -> Self {
    // Keep one reference per field rather than per escape. The string cursor
    // scans each field only as a consumer needs its fragments.
    let mut parts = Vec::new();
    data.visit(&mut |part| parts.push(part));
    Self {
      parts: parts.into_iter(),
      string: None,
    }
  }
}

impl<'a> Iterator for JsonChunks<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if let Some(string) = &mut self.string {
        if let Some(chunk) = string.next() {
          return Some(chunk);
        }
        self.string = None;
      }
      match self.parts.next()? {
        Part::Raw(chunk) => return Some(chunk),
        Part::String(value) => self.string = Some(JsonStringChunks::new(value)),
      }
    }
  }
}

impl MapData {
  fn to_json(&self) -> String {
    // Reuse the capacity estimate and concrete Vec writer so emission needs
    // only one SIMD serialization pass. Retain the length, never the JSON.
    let json = self.map.to_json();
    let _ = self.size.set(json.len());
    json
  }

  fn size(&self) -> usize {
    *self.size.get_or_init(|| {
      let mut writer = SizeWriter(0);
      simd_json::to_writer(&mut writer as &mut dyn Write, self.map.fields())
        .expect("count map JSON bytes");
      writer.0
    })
  }

  fn visit<'a>(&'a self, visit: &mut impl FnMut(Part<'a>)) {
    let fields = self.map.fields();
    visit(Part::Raw("{\"version\":"));
    visit(Part::Raw(&self.version));
    if let Some(file) = fields.file.as_deref() {
      visit(Part::Raw(",\"file\":"));
      visit(Part::String(file));
    }
    visit(Part::Raw(",\"sources\":["));
    for (i, source) in fields.sources.iter().enumerate() {
      if i != 0 {
        visit(Part::Raw(","));
      }
      visit(Part::String(source));
    }
    visit(Part::Raw("]"));
    if !is_all_empty(&fields.sources_content) {
      visit(Part::Raw(",\"sourcesContent\":["));
      for (i, content) in fields.sources_content.iter().enumerate() {
        if i != 0 {
          visit(Part::Raw(","));
        }
        visit(Part::String(content));
      }
      visit(Part::Raw("]"));
    }
    visit(Part::Raw(",\"names\":["));
    for (i, name) in fields.names.iter().enumerate() {
      if i != 0 {
        visit(Part::Raw(","));
      }
      visit(Part::String(name));
    }
    visit(Part::Raw("],\"mappings\":"));
    visit(Part::String(&fields.mappings));
    if let Some(root) = fields.source_root.as_deref() {
      visit(Part::Raw(",\"sourceRoot\":"));
      visit(Part::String(root));
    }
    if let Some(id) = fields.debug_id.as_deref() {
      visit(Part::Raw(",\"debugId\":"));
      visit(Part::String(id));
    }
    if let Some(list) = &self.ignore_list {
      visit(Part::Raw(",\"ignoreList\":"));
      visit(Part::Raw(list));
    }
    visit(Part::Raw("}"));
  }
}

struct JsonStringChunks<'a> {
  remaining: &'a str,
  pending: Option<&'static str>,
  finished: bool,
}

impl<'a> JsonStringChunks<'a> {
  fn new(value: &'a str) -> Self {
    Self {
      remaining: value,
      pending: Some("\""),
      finished: false,
    }
  }
}

impl<'a> Iterator for JsonStringChunks<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    // These spellings match simd_json's compact serializer. Every slice
    // borrows the immutable input or a static escape, including for rope().
    const CONTROLS: [&str; 32] = [
      "\\u0000", "\\u0001", "\\u0002", "\\u0003", "\\u0004", "\\u0005", "\\u0006", "\\u0007",
      "\\b", "\\t", "\\n", "\\u000b", "\\f", "\\r", "\\u000e", "\\u000f", "\\u0010", "\\u0011",
      "\\u0012", "\\u0013", "\\u0014", "\\u0015", "\\u0016", "\\u0017", "\\u0018", "\\u0019",
      "\\u001a", "\\u001b", "\\u001c", "\\u001d", "\\u001e", "\\u001f",
    ];
    if let Some(chunk) = self.pending.take() {
      return Some(chunk);
    }
    if self.finished {
      return None;
    }
    let value = self.remaining;
    for (i, byte) in value.bytes().enumerate() {
      let escaped = match byte {
        b'"' => "\\\"",
        b'\\' => "\\\\",
        0..=31 => CONTROLS[usize::from(byte)],
        _ => continue,
      };
      self.remaining = &value[i + 1..];
      if i == 0 {
        return Some(escaped);
      }
      self.pending = Some(escaped);
      return Some(&value[..i]);
    }
    self.remaining = "";
    self.finished = true;
    if value.is_empty() {
      Some("\"")
    } else {
      self.pending = Some("\"");
      Some(value)
    }
  }
}

impl Source for MappingSource {
  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Owned(self.0.to_json()))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    for chunk in self.json_chunks_iter() {
      on_chunk(chunk);
    }
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    Cow::Owned(self.0.to_json().into_bytes())
  }

  fn size(&self) -> usize {
    self.0.size()
  }

  fn map<'a>(&'a self, _: &MapOptions) -> Option<SourceMap<'a>> {
    None
  }

  fn map_static(self: Arc<Self>, _: &MapOptions) -> Option<SourceMap<'static>> {
    None
  }

  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    simd_json::to_writer(writer, self.0.map.fields()).map_err(io::Error::other)
  }
}

impl PartialEq for MappingSource {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
  }
}
impl Eq for MappingSource {}

impl Hash for MappingSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "MappingSource".hash(state);
    self.0.hash(state);
  }
}

impl std::fmt::Debug for MappingSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MappingSource")
      .field("size", &self.size())
      .field("sources", &self.original_map().sources().len())
      .finish()
  }
}

struct MappingChunks<'a>(&'a MappingSource);
impl<'source> Chunks<'source> for MappingChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    on_source: crate::helpers::OnSource<'_, 'source>,
    on_name: crate::helpers::OnName<'_, 'source>,
  ) -> GeneratedInfo {
    let _ = (on_source, on_name);
    stream_chunks_of_raw_rope(self.0, options, on_chunk)
  }
}
impl StreamChunks for MappingSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(MappingChunks(self))
  }
}
