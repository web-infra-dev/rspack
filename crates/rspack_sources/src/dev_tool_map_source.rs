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

  fn json_chunks(&self) -> impl Iterator<Item = &str> {
    let mut parts = self.0.json_parts();
    let mut quoted = None;
    std::iter::from_fn(move || {
      loop {
        if let Some(chunk) = quoted.as_mut().and_then(Iterator::next) {
          return Some(chunk);
        }
        match parts.next()? {
          Part::Raw(chunk) => return Some(chunk),
          Part::String(value) => quoted = Some(quoted_json(value)),
        }
      }
    })
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

fn quoted_json(mut value: &str) -> impl Iterator<Item = &str> {
  // Match simd_json's compact spelling while borrowing all unescaped text.
  const CONTROLS: [&str; 32] = [
    "\\u0000", "\\u0001", "\\u0002", "\\u0003", "\\u0004", "\\u0005", "\\u0006", "\\u0007", "\\b",
    "\\t", "\\n", "\\u000b", "\\f", "\\r", "\\u000e", "\\u000f", "\\u0010", "\\u0011", "\\u0012",
    "\\u0013", "\\u0014", "\\u0015", "\\u0016", "\\u0017", "\\u0018", "\\u0019", "\\u001a",
    "\\u001b", "\\u001c", "\\u001d", "\\u001e", "\\u001f",
  ];
  let content = std::iter::from_fn(move || {
    if value.is_empty() {
      return None;
    }
    for (i, byte) in value.bytes().enumerate() {
      let escape = match byte {
        b'"' => "\\\"",
        b'\\' => "\\\\",
        0..=31 => CONTROLS[usize::from(byte)],
        _ => continue,
      };
      if i == 0 {
        value = &value[1..];
        return Some(escape);
      }
      let (prefix, rest) = value.split_at(i);
      value = rest;
      return Some(prefix);
    }
    Some(std::mem::take(&mut value))
  });
  std::iter::once("\"")
    .chain(content)
    .chain(std::iter::once("\""))
}

impl Source for DevToolMapSource {
  fn mapped(&self) -> bool {
    false
  }

  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Owned(self.0.to_json()))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    for chunk in self.json_chunks() {
      on_chunk(chunk);
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
