use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  io::{self, Write},
  sync::{Arc, OnceLock},
};

use crate::{
  MapOptions, Source, SourceMap, SourceValue,
  helpers::{Chunks, RawRopeChunks, StreamChunks},
  source::is_all_none,
};

#[cfg_attr(feature = "allocative", derive(rspack_allocative::Allocative))]
pub(crate) struct MapData {
  pub(crate) map: SourceMap,
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

/// A complete external source map JSON asset emitted by DevTool. Retains its
/// original SourceMap instead of another complete JSON string.
///
/// Content APIs serialize on demand; streaming APIs borrow fragments without
/// flattening the JSON asset. RCH wraps edited JSON in ReplaceSource without retaining a flat string.
/// The JSON asset itself has no source map.
#[derive(Clone)]
#[cfg_attr(feature = "allocative", derive(rspack_allocative::Allocative))]
pub struct DevToolMapSource(Arc<MapData>);

impl DevToolMapSource {
  /// Create a JSON source map asset.
  pub fn new(map: SourceMap) -> Self {
    let version = map.version.to_string();
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

  pub(crate) fn original_map(&self) -> &SourceMap {
    &self.0.map
  }
}

struct CountingWriter<W> {
  inner: W,
  written: usize,
}

impl<W: Write> Write for CountingWriter<W> {
  fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
    let n = self.inner.write(bytes)?;
    self.written += n;
    Ok(n)
  }

  fn flush(&mut self) -> io::Result<()> {
    self.inner.flush()
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
      self
        .write_counted(io::sink())
        .expect("count map JSON bytes")
    })
  }

  fn write_counted(&self, inner: impl Write) -> io::Result<usize> {
    let mut writer = CountingWriter { inner, written: 0 };
    simd_json::to_writer(&mut writer as &mut dyn Write, &self.map).map_err(io::Error::other)?;
    Ok(writer.written)
  }

  fn json_parts(&self) -> impl Iterator<Item = Part<'_>> {
    use std::iter::once;

    use Part::{Raw, String};

    fn string_field<'a>(
      key: &'static str,
      value: Option<&'a str>,
    ) -> impl Iterator<Item = Part<'a>> {
      value.into_iter().flat_map(move |s| [Raw(key), String(s)])
    }

    fn array<'a>(items: impl Iterator<Item = Part<'a>>) -> impl Iterator<Item = Part<'a>> {
      items
        .enumerate()
        .flat_map(|(i, item)| (i != 0).then_some(Raw(",")).into_iter().chain(once(item)))
    }

    let map = &self.map;
    let sources_content = (!is_all_none(&map.sources_content)).then(|| {
      once(Raw(",\"sourcesContent\":["))
        .chain(array(
          map
            .sources_content()
            .iter()
            .map(|s| s.as_deref().map_or(Raw("null"), String)),
        ))
        .chain(once(Raw("]")))
    });
    [Raw("{\"version\":"), Raw(&self.version)]
      .into_iter()
      .chain(string_field(",\"file\":", map.file.as_deref()))
      .chain(once(Raw(",\"sources\":[")))
      .chain(array(map.sources().iter().map(|s| String(s))))
      .chain(once(Raw("]")))
      .chain(sources_content.into_iter().flatten())
      .chain(once(Raw(",\"names\":[")))
      .chain(array(map.names().iter().map(|s| String(s))))
      .chain([Raw("],\"mappings\":"), String(&map.mappings)])
      .chain(string_field(",\"sourceRoot\":", map.source_root.as_deref()))
      .chain(string_field(",\"debugId\":", map.debug_id.as_deref()))
      .chain(
        self
          .ignore_list
          .as_deref()
          .into_iter()
          .flat_map(|s| [Raw(",\"ignoreList\":"), Raw(s)]),
      )
      .chain(once(Raw("}")))
  }
}

fn needs_escape(b: u8) -> bool {
  b < 32 || b == b'"' || b == b'\\'
}

// Return the high bit of each matching byte, with byte 0 in the lowest lane.
// Masking each lane to 7 bits prevents these additions carrying into its neighbor.
fn escape_mask(word: u64) -> u64 {
  const LOW: u64 = u64::from_le_bytes([0x7f; 8]);
  let zero = |v: u64| !(((v & LOW) + LOW) | v);
  let control = !(((word & LOW) + u64::from_le_bytes([0x60; 8])) | word);
  (control
    | zero(word ^ u64::from_le_bytes([b'"'; 8]))
    | zero(word ^ u64::from_le_bytes([b'\\'; 8])))
    & !LOW
}

fn quoted_json<'a>(value: &'a str, on_chunk: &mut dyn FnMut(&'a str)) {
  // Match simd_json's compact spelling while borrowing all unescaped text.
  const CONTROLS: [&str; 32] = [
    "\\u0000", "\\u0001", "\\u0002", "\\u0003", "\\u0004", "\\u0005", "\\u0006", "\\u0007", "\\b",
    "\\t", "\\n", "\\u000b", "\\f", "\\r", "\\u000e", "\\u000f", "\\u0010", "\\u0011", "\\u0012",
    "\\u0013", "\\u0014", "\\u0015", "\\u0016", "\\u0017", "\\u0018", "\\u0019", "\\u001a",
    "\\u001b", "\\u001c", "\\u001d", "\\u001e", "\\u001f",
  ];
  on_chunk("\"");
  let mut start = 0;
  let mut emit_escape = |i: usize, byte: u8| {
    let escape = match byte {
      b'"' => "\\\"",
      b'\\' => "\\\\",
      _ => CONTROLS[usize::from(byte)],
    };
    if start < i {
      on_chunk(&value[start..i]);
    }
    on_chunk(escape);
    start = i + 1;
  };
  for (i, chunk) in value.as_bytes().chunks(64).enumerate() {
    // A fixed-width reduction vectorizes. Most mappings and long spans of code
    // have no escapes; leave these spans borrowed, even across block boundaries.
    if let Ok(block) = <&[u8; 64]>::try_from(chunk)
      && block
        .iter()
        .fold(0u8, |mask, &b| mask | u8::from(needs_escape(b)))
        == 0
    {
      continue;
    }
    let base = i * 64;
    let (words, tail) = chunk.as_chunks::<8>();
    for (j, word) in words.iter().enumerate() {
      let mut mask = escape_mask(u64::from_le_bytes(*word));
      while mask != 0 {
        let lane = mask.trailing_zeros() as usize / 8;
        emit_escape(base + j * 8 + lane, word[lane]);
        mask &= mask - 1;
      }
    }
    for (j, &b) in tail.iter().enumerate() {
      if needs_escape(b) {
        emit_escape(base + words.len() * 8 + j, b);
      }
    }
  }
  if start < value.len() {
    on_chunk(&value[start..]);
  }
  on_chunk("\"");
}

impl Source for DevToolMapSource {
  fn mapped(&self) -> bool {
    false
  }

  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Owned(self.0.to_json()))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    for part in self.0.json_parts() {
      match part {
        Part::Raw(s) => on_chunk(s),
        Part::String(value) => quoted_json(value, on_chunk),
      }
    }
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    Cow::Owned(self.0.to_json().into_bytes())
  }

  fn size(&self) -> usize {
    self.0.size()
  }

  fn map(&self, _: &MapOptions) -> Option<SourceMap> {
    None
  }

  fn to_writer(&self, writer: &mut dyn Write) -> io::Result<()> {
    if self.0.size.get().is_some() {
      return simd_json::to_writer(writer, &self.0.map).map_err(io::Error::other);
    }
    let written = self.0.write_counted(writer)?;
    let _ = self.0.size.set(written);
    Ok(())
  }
}

impl PartialEq for DevToolMapSource {
  fn eq(&self, other: &Self) -> bool {
    Arc::ptr_eq(&self.0, &other.0) || self.0 == other.0
  }
}
impl Eq for DevToolMapSource {}

impl Hash for DevToolMapSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "DevToolMapSource".hash(state);
    self.0.hash(state);
  }
}

impl std::fmt::Debug for DevToolMapSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("DevToolMapSource")
      .field("size", &self.size())
      .field("sources", &self.original_map().sources().len())
      .finish()
  }
}

impl StreamChunks for DevToolMapSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(RawRopeChunks(self))
  }
}
