use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
  sync::{Arc, OnceLock},
};

use crate::{
  MapOptions, ObjectPool, Source, SourceMap, SourceValue,
  helpers::{Chunks, GeneratedInfo, StreamChunks, stream_chunks_of_raw_rope},
};

#[cfg_attr(feature = "allocative", derive(allocative::Allocative))]
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

#[cfg_attr(feature = "allocative", derive(allocative::Allocative))]
#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Data {
  Map(Box<MapData>),
  Edited {
    inner: MappingSource,
    replacements: Vec<(usize, usize, String)>,
    size: usize,
  },
}

/// An external source map asset which retains its original SourceMap instead
/// of retaining another complete JSON string.
///
/// Content APIs serialize on demand. Edits retain their input and byte ranges,
/// and all streaming APIs borrow fragments without flattening the JSON asset.
/// The JSON asset itself has no source map.
#[derive(Clone)]
#[cfg_attr(feature = "allocative", derive(allocative::Allocative))]
pub struct MappingSource(Arc<Data>);

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
    Self(Arc::new(Data::Map(Box::new(data))))
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

  /// Return an immutable view with sorted, non-overlapping UTF-8 byte edits.
  /// Coordinates refer to this view's serialized JSON, including earlier edits.
  pub fn with_replacements(&self, replacements: Vec<(usize, usize, String)>) -> Self {
    if replacements.is_empty() {
      return self.clone();
    }
    let mut end = 0;
    let mut size = self.size();
    for (start, next_end, content) in &replacements {
      assert!(end <= *start && start <= next_end && *next_end <= self.size());
      size = size - (next_end - start) + content.len();
      end = *next_end;
    }
    Self(Arc::new(Data::Edited {
      inner: self.clone(),
      replacements,
      size,
    }))
  }

  /// Compare rendered bytes without allocating the complete JSON. Only borrowed
  /// fragment references are retained while the other source writes its bytes.
  pub fn content_eq(&self, other: &dyn Source) -> bool {
    if self.size() != other.size() {
      return false;
    }
    if let Some(other) = other.as_any().downcast_ref::<Self>()
      && self == other
    {
      return true;
    }
    let mut chunks = Vec::new();
    self.rope(&mut |chunk| {
      if !chunk.is_empty() {
        chunks.push(chunk.as_bytes());
      }
    });
    let mut writer = CompareWriter {
      chunks,
      index: 0,
      offset: 0,
    };
    other.to_writer(&mut writer).is_ok() && writer.index == writer.chunks.len()
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
    match self.0.as_ref() {
      Data::Map(data) => &data.map,
      Data::Edited { inner, .. } => inner.original_map(),
    }
  }

  #[cfg(feature = "rspack_cacheable")]
  pub(crate) fn data(&self) -> &Data {
    &self.0
  }
}

struct CompareWriter<'a> {
  chunks: Vec<&'a [u8]>,
  index: usize,
  offset: usize,
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
      let Some(chunk) = self.chunks.get(self.index) else {
        return Err(io::Error::other("source contents differ"));
      };
      let n = bytes.len().min(chunk.len() - self.offset);
      if bytes[..n] != chunk[self.offset..self.offset + n] {
        return Err(io::Error::other("source contents differ"));
      }
      self.offset += n;
      if self.offset == chunk.len() {
        self.index += 1;
        self.offset = 0;
      }
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
      simd_json::to_writer(&mut writer, self.map.fields()).expect("count map JSON bytes");
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
    if fields
      .sources_content
      .iter()
      .any(|content| !content.is_empty())
    {
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

// These spellings match simd_json's compact serializer, including lowercase hex
// for control characters. Every emitted slice borrows the immutable input or a
// static escape, so rope callers may retain the slices for the source lifetime.
fn json_string_rope<'a>(value: &'a str, visit: &mut dyn FnMut(&'a str)) {
  const CONTROLS: [&str; 32] = [
    "\\u0000", "\\u0001", "\\u0002", "\\u0003", "\\u0004", "\\u0005", "\\u0006", "\\u0007", "\\b",
    "\\t", "\\n", "\\u000b", "\\f", "\\r", "\\u000e", "\\u000f", "\\u0010", "\\u0011", "\\u0012",
    "\\u0013", "\\u0014", "\\u0015", "\\u0016", "\\u0017", "\\u0018", "\\u0019", "\\u001a",
    "\\u001b", "\\u001c", "\\u001d", "\\u001e", "\\u001f",
  ];
  visit("\"");
  let mut start = 0;
  for (i, byte) in value.bytes().enumerate() {
    let escaped = match byte {
      b'"' => "\\\"",
      b'\\' => "\\\\",
      0..=31 => CONTROLS[usize::from(byte)],
      _ => continue,
    };
    if start < i {
      visit(&value[start..i]);
    }
    visit(escaped);
    start = i + 1;
  }
  if start < value.len() {
    visit(&value[start..]);
  }
  visit("\"");
}

impl Source for MappingSource {
  fn source(&self) -> SourceValue<'_> {
    let value = match self.0.as_ref() {
      Data::Map(data) => data.to_json(),
      Data::Edited { .. } => {
        let mut value = String::with_capacity(self.size());
        self.rope(&mut |chunk| value.push_str(chunk));
        value
      }
    };
    SourceValue::String(Cow::Owned(value))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    match self.0.as_ref() {
      Data::Map(data) => data.visit(&mut |part| match part {
        Part::Raw(s) => on_chunk(s),
        Part::String(s) => json_string_rope(s, on_chunk),
      }),
      Data::Edited {
        inner,
        replacements,
        ..
      } => {
        let mut position = 0;
        let mut consumed = 0;
        let mut edits = replacements.iter().peekable();
        inner.rope(&mut |chunk| {
          let end = position + chunk.len();
          while let Some((start, stop, content)) = edits.peek()
            && *start <= end
          {
            if *start > consumed.max(position) {
              on_chunk(&chunk[consumed.max(position) - position..start - position]);
            }
            on_chunk(content);
            consumed = *stop;
            edits.next();
          }
          if consumed < end {
            on_chunk(&chunk[consumed.max(position) - position..]);
            consumed = end;
          }
          position = end;
        });
        // An edited view may be empty, in which case its rope has no fragments.
        for (_, _, content) in edits {
          on_chunk(content);
        }
      }
    }
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    let buffer = match self.0.as_ref() {
      Data::Map(data) => data.to_json().into_bytes(),
      Data::Edited { size, .. } => {
        let mut buffer = Vec::with_capacity(*size);
        self.to_writer(&mut buffer).expect("write JSON to Vec");
        buffer
      }
    };
    Cow::Owned(buffer)
  }

  fn size(&self) -> usize {
    match self.0.as_ref() {
      Data::Map(data) => data.size(),
      Data::Edited { size, .. } => *size,
    }
  }

  fn map(&self, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'_>> {
    None
  }

  fn map_static(self: Arc<Self>, _: &ObjectPool, _: &MapOptions) -> Option<SourceMap<'static>> {
    None
  }

  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    match self.0.as_ref() {
      Data::Map(data) => simd_json::to_writer(writer, data.map.fields()).map_err(io::Error::other),
      Data::Edited { .. } => {
        let mut result = Ok(());
        self.rope(&mut |chunk| {
          if result.is_ok() {
            result = writer.write_all(chunk.as_bytes());
          }
        });
        result
      }
    }
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
    _: &ObjectPool,
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
