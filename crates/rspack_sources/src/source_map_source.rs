use std::{
  borrow::Cow,
  hash::{Hash, Hasher},
  sync::Arc,
};

use crate::{
  MapOptions, Source, SourceMap, SourceValue,
  helpers::{
    Chunks, StreamChunks, TextSpan, get_map, stream_chunks_of_combined_source_map,
    stream_chunks_of_source_map,
  },
  object_pool::ObjectPool,
};

/// Options for [SourceMapSource::new].
#[derive(Debug)]
pub struct SourceMapSourceOptions<V, N> {
  /// The source code.
  pub value: V,
  /// Name of the file.
  pub name: N,
  /// The source map of the source code.
  pub source_map: SourceMap<'static>,
  /// The original source code.
  pub original_source: Option<Box<str>>,
  /// The original source map.
  pub inner_source_map: Option<SourceMap<'static>>,
  /// Whether remove the original source.
  pub remove_original_source: bool,
}

/// An convenient options for [SourceMapSourceOptions], `original_source` and
/// `inner_source_map` will be `None`, `remove_original_source` will be false.
#[derive(Debug)]
pub struct WithoutOriginalOptions<V, N> {
  /// The source code.
  pub value: V,
  /// Name of the file.
  pub name: N,
  /// The source map of the source code.
  pub source_map: SourceMap<'static>,
}

impl<V, N> From<WithoutOriginalOptions<V, N>> for SourceMapSourceOptions<V, N> {
  fn from(options: WithoutOriginalOptions<V, N>) -> Self {
    Self {
      value: options.value,
      name: options.name,
      source_map: options.source_map,
      original_source: None,
      inner_source_map: None,
      remove_original_source: false,
    }
  }
}

/// Represents source code with source map, optionally having an additional
/// source map for the original source.
///
/// - [webpack-sources docs](https://github.com/webpack/webpack-sources/#sourcemapsource).
#[derive(Eq)]
pub struct SourceMapSource {
  value: Box<str>,
  name: Box<str>,
  source_map: SourceMap<'static>,
  original_source: Option<Box<str>>,
  inner_source_map: Option<SourceMap<'static>>,
  remove_original_source: bool,
}

impl SourceMapSource {
  /// Create a [SourceMapSource] with [SourceMapSourceOptions].
  pub fn new<V, N, O>(options: O) -> Self
  where
    V: Into<String>,
    N: Into<String>,
    O: Into<SourceMapSourceOptions<V, N>>,
  {
    let options = options.into();
    Self {
      value: Box::from(options.value.into()),
      name: Box::from(options.name.into()),
      source_map: options.source_map,
      original_source: options.original_source,
      inner_source_map: options.inner_source_map,
      remove_original_source: options.remove_original_source,
    }
  }

  /// Get the value as a shared string reference.
  pub fn value(&self) -> &str {
    &self.value
  }

  /// Get the name of the source file.
  pub fn name(&self) -> &str {
    &self.name
  }

  /// Get the source map.
  pub fn source_map(&self) -> &SourceMap<'static> {
    &self.source_map
  }

  /// Get the original source code.
  pub fn original_source(&self) -> Option<&str> {
    self.original_source.as_deref()
  }

  /// Get the inner source map.
  pub fn inner_source_map(&self) -> Option<&SourceMap<'static>> {
    self.inner_source_map.as_ref()
  }

  /// Whether to remove the original source.
  pub fn remove_original_source(&self) -> bool {
    self.remove_original_source
  }
}

impl Source for SourceMapSource {
  fn source(&self) -> SourceValue<'_> {
    SourceValue::String(Cow::Borrowed(&self.value))
  }

  fn rope<'a>(&'a self, on_chunk: &mut dyn FnMut(&'a str)) {
    on_chunk(&self.value)
  }

  fn buffer(&self) -> Cow<'_, [u8]> {
    Cow::Borrowed(self.value.as_bytes())
  }

  fn size(&self) -> usize {
    self.value.len()
  }

  fn map<'a>(&'a self, object_pool: &ObjectPool, options: &MapOptions) -> Option<SourceMap<'a>> {
    if self.inner_source_map.is_none() {
      return Some(self.source_map.as_borrowed());
    }
    let chunks = self.stream_chunks();
    get_map(object_pool, chunks.as_ref(), options).map(SourceMap::from_fields)
  }

  fn map_static(
    self: Arc<Self>,
    object_pool: &ObjectPool,
    options: &MapOptions,
  ) -> Option<SourceMap<'static>> {
    let owner = self.clone();
    self
      .as_ref()
      .map(object_pool, options)
      .map(|map| map.into_static(owner))
  }

  fn to_writer(&self, writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    writer.write_all(self.value.as_bytes())
  }
}

impl Hash for SourceMapSource {
  fn hash<H: Hasher>(&self, state: &mut H) {
    "SourceMapSource".hash(state);
    self.buffer().hash(state);
    self.source_map.hash(state);
    self.original_source.hash(state);
    self.inner_source_map.hash(state);
    self.remove_original_source.hash(state);
  }
}

impl PartialEq for SourceMapSource {
  fn eq(&self, other: &Self) -> bool {
    self.value == other.value
      && self.name == other.name
      && self.source_map == other.source_map
      && self.original_source == other.original_source
      && self.inner_source_map == other.inner_source_map
      && self.remove_original_source == other.remove_original_source
  }
}

impl std::fmt::Debug for SourceMapSource {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    let indent = f.width().unwrap_or(0);
    let indent_str = format!("{:indent$}", "", indent = indent);

    writeln!(
      f,
      "{indent_str}SourceMapSource::new(SourceMapSourceOptions {{"
    )?;
    writeln!(f, "{indent_str}  value: {:?},", self.value)?;
    writeln!(f, "{indent_str}  name: {:?},", self.name)?;
    writeln!(f, "{indent_str}  source_map: {:?},", self.source_map)?;
    match &self.original_source {
      Some(original_source) => {
        writeln!(
          f,
          "{indent_str}  original_source: Some({:?}.to_string()),",
          original_source
        )?;
      }
      None => {
        writeln!(f, "{indent_str}  original_source: None,")?;
      }
    }
    writeln!(
      f,
      "{indent_str}  inner_source_map: {:?},",
      self.inner_source_map
    )?;
    writeln!(
      f,
      "{indent_str}  remove_original_source: {:?},",
      self.remove_original_source
    )?;
    write!(f, "{indent_str}}}).boxed()")?;

    Ok(())
  }
}

struct SourceMapSourceChunks<'source>(&'source SourceMapSource);

impl<'source> Chunks<'source> for SourceMapSourceChunks<'source> {
  fn stream<'chunk>(
    &'chunk self,
    object_pool: &ObjectPool,
    options: &MapOptions,
    on_chunk: crate::helpers::OnChunk<'_, 'chunk>,
    on_source: crate::helpers::OnSource<'_, 'source>,
    on_name: crate::helpers::OnName<'_, 'source>,
  ) -> crate::helpers::GeneratedInfo {
    if let Some(inner_source_map) = &self.0.inner_source_map {
      stream_chunks_of_combined_source_map(
        options,
        object_pool,
        &self.0.value,
        self.0.source_map.fields(),
        &self.0.name,
        self.0.original_source.as_deref(),
        inner_source_map.fields(),
        self.0.remove_original_source,
        on_chunk,
        on_source,
        on_name,
      )
    } else {
      stream_chunks_of_source_map(
        options,
        object_pool,
        TextSpan::new(self.0.value.as_ref()),
        self.0.source_map.fields(),
        on_chunk,
        on_source,
        on_name,
      )
    }
  }
}

impl StreamChunks for SourceMapSource {
  fn stream_chunks<'a>(&'a self) -> Box<dyn Chunks<'a> + 'a> {
    Box::new(SourceMapSourceChunks(self))
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    BoxSource, CachedSource, ConcatSource, OriginalSource, RawStringSource, ReplaceSource,
    SourceExt,
  };

  #[test]
  fn map_correctly() {
    let inner_source_code = "Hello World\nis a test string\n";
    let inner_source = ConcatSource::new([
      OriginalSource::new(inner_source_code, "hello-world.txt").boxed(),
      OriginalSource::new("Translate: ", "header.txt").boxed(),
      RawStringSource::from("Other text").boxed(),
    ])
    .boxed();
    let source_r_code = "Translated: Hallo Welt\nist ein test Text\nAnderer Text";
    let source_map_str = r#"{
      "version": 3,
      "sources": [ "text" ],
      "names": [ "Hello", "World", "nope" ],
      "mappings": "YAAAA,K,CAAMC;AACNC,O,MAAU;AACC,O,CAAM",
      "file": "translated.txt",
      "sourcesContent": [ "Hello World\nis a test string\n" ]
    }"#;
    let sms1 = SourceMapSource::new(SourceMapSourceOptions {
      value: source_r_code,
      name: "text",
      source_map: SourceMap::from_json(source_map_str.to_string()).unwrap(),
      original_source: Some(inner_source.source().into_string_lossy().into()),
      inner_source_map: inner_source
        .clone()
        .map_static(&ObjectPool::default(), &MapOptions::default()),
      remove_original_source: false,
    });
    let sms2 = SourceMapSource::new(SourceMapSourceOptions {
      value: source_r_code,
      name: "text",
      source_map: SourceMap::from_json(source_map_str.to_string()).unwrap(),
      original_source: Some(inner_source.source().into_string_lossy().into()),
      inner_source_map: inner_source
        .clone()
        .map_static(&ObjectPool::default(), &MapOptions::default()),
      remove_original_source: true,
    });
    let expected_content = "Translated: Hallo Welt\nist ein test Text\nAnderer Text";
    assert_eq!(sms1.source().into_string_lossy(), expected_content);
    assert_eq!(sms2.source().into_string_lossy(), expected_content);
    assert_eq!(
      sms1
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "YAAAA,K,CAAMC;AACN,O,MAAU;ACCC,O,CAAM",
          "names": ["Hello", "World"],
          "sources": ["hello-world.txt", "text"],
          "sourcesContent": [
            "Hello World\nis a test string\n",
            "Hello World\nis a test string\nTranslate: Other text"
          ],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap(),
    );
    assert_eq!(
      sms2
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "YAAAA,K,CAAMC;AACN,O,MAAU",
          "names": ["Hello", "World"],
          "sources": ["hello-world.txt"],
          "sourcesContent": ["Hello World\nis a test string\n"],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap(),
    );

    let mut hasher = twox_hash::XxHash64::default();
    sms1.hash(&mut hasher);
    assert_eq!(format!("{:x}", hasher.finish()), "736934c6e249aa6e");
  }

  #[test]
  fn should_handle_null_sources_and_sources_content() {
    let a = SourceMapSource::new(WithoutOriginalOptions {
      value: "hello world\n",
      name: "hello.txt",
      source_map: SourceMap::new("AAAA".to_string(), vec!["".into()], vec!["".into()], vec![]),
    });
    let b = SourceMapSource::new(WithoutOriginalOptions {
      value: "hello world\n",
      name: "hello.txt",
      source_map: SourceMap::new("AAAA".to_string(), vec![], vec![], vec![]),
    });
    let c = SourceMapSource::new(WithoutOriginalOptions {
      value: "hello world\n",
      name: "hello.txt",
      source_map: SourceMap::new(
        "AAAA".to_string(),
        vec!["hello-source.txt".into()],
        vec!["hello world\n".into()],
        vec![],
      ),
    });
    let sources = [a, b, c].into_iter().map(|s| {
      let mut r = ReplaceSource::new(s);
      r.replace_static(1, 5, "i", None);
      r
    });
    let source = ConcatSource::new(sources);
    assert_eq!(
      source.source().into_string_lossy(),
      "hi world\nhi world\nhi world\n"
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAA;;ACAA,CAAC,CAAI",
          "names": [],
          "sources": [null, "hello-source.txt"],
          "sourcesContent": [null,"hello world\n"],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap()
    );
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAA;;ACAA",
          "names": [],
          "sources": [null, "hello-source.txt"],
          "sourcesContent": [null,"hello world\n"],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap()
    );
  }

  #[test]
  fn should_handle_es6_promise_correctly() {
    let code = include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/tests/fixtures/es6-promise.js"
    ));
    let map = SourceMap::from_json(
      include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/es6-promise.map"
      ))
      .to_string(),
    )
    .unwrap();
    let inner = SourceMapSource::new(WithoutOriginalOptions {
      value: code,
      name: "es6-promise.js",
      source_map: map,
    })
    .boxed();
    let source = ConcatSource::new([inner.clone(), inner]);
    assert_eq!(source.source().into_string_lossy(), format!("{code}{code}"));
  }

  #[test]
  fn should_not_emit_zero_sizes_mappings_when_ending_with_empty_mapping() {
    let a = SourceMapSource::new(WithoutOriginalOptions {
      value: "hello\n",
      name: "a",
      source_map: SourceMap::new(
        "AAAA;AACA".to_string(),
        vec!["hello1".into()],
        vec![],
        vec![],
      ),
    })
    .boxed();
    let b = SourceMapSource::new(WithoutOriginalOptions {
      value: "hi",
      name: "b",
      source_map: SourceMap::new(
        "AAAA,EAAE".to_string(),
        vec!["hello2".into()],
        vec![],
        vec![],
      ),
    })
    .boxed();
    let b2 = SourceMapSource::new(WithoutOriginalOptions {
      value: "hi",
      name: "b",
      source_map: SourceMap::new(
        "AAAA,EAAE".to_string(),
        vec!["hello3".into()],
        vec![],
        vec![],
      ),
    })
    .boxed();
    let c = SourceMapSource::new(WithoutOriginalOptions {
      value: "",
      name: "c",
      source_map: SourceMap::new("AAAA".to_string(), vec!["hello4".into()], vec![], vec![]),
    })
    .boxed();
    let source = ConcatSource::new([
      a.clone(),
      a.clone(),
      b.clone(),
      b.clone(),
      b2.clone(),
      b.clone(),
      c.clone(),
      c.clone(),
      b2.clone(),
      a.clone(),
      b2,
      c,
      a,
      b,
    ]);
    let map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    assert_eq!(
      map.mappings(),
      "AAAA;AAAA;ACAA,ICAA,EDAA,ECAA,EFAA;AEAA,EFAA;ACAA",
    );

    macro_rules! test_cached {
      ($s:expr, $fn:expr) => {{
        let s = $s.clone().boxed();
        let c = CachedSource::new(s.clone()).boxed();
        let o = $fn(s.clone());
        let a = $fn(c.clone());
        assert_eq!(a, o);
        let b = $fn(c.clone());
        assert_eq!(b, o);
      }};
    }

    test_cached!(source, |s: BoxSource| s
      .source()
      .into_string_lossy()
      .into_owned());
    test_cached!(source, |s: BoxSource| Source::map_static(
      s,
      &ObjectPool::default(),
      &MapOptions::default()
    ));
    test_cached!(source, |s: BoxSource| Source::map_static(
      s,
      &ObjectPool::default(),
      &MapOptions::new(false)
    ));
  }

  #[test]
  fn should_not_crash_without_original_source_when_mapping_names() {
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: "h",
      name: "hello.txt",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["hello.txt"],
          "mappings": "AAAAA",
          "names": ["hello"]
        }"#
          .to_string(),
      )
      .unwrap(),
      original_source: Some("hello".into()),
      inner_source_map: Some(
        SourceMap::from_json(
          r#"{
          "version": 3,
          "sources": ["hello world.txt"],
          "mappings": "AAAA"
        }"#
            .to_string(),
        )
        .unwrap(),
      ),
      remove_original_source: false,
    });
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAA",
          "names": [],
          "sources": ["hello world.txt"],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap()
    );
  }

  #[test]
  fn should_map_generated_lines_to_the_inner_source() {
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: "Message: H W!",
      name: "HELLO_WORLD.txt",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["messages.txt", "HELLO_WORLD.txt"],
          "mappings": "AAAAA,SCAAC,EAAMC,C",
          "names": ["Message", "hello", "world"]
        }"#
          .to_string(),
      )
      .unwrap(),
      original_source: Some("HELLO WORLD".into()),
      inner_source_map: Some(
        SourceMap::from_json(
          r#"{
            "version": 3,
            "mappings": "AAAAA,M",
            "sources": ["hello world.txt"],
            "sourcesContent": ["hello world"]
          }"#
            .to_string(),
        )
        .unwrap(),
      ),
      remove_original_source: false,
    });
    assert_eq!(
      source.source().into_string_lossy().into_owned(),
      "Message: H W!"
    );
    assert_eq!(source.size(), 13);
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAAA,SCAA,ECAMC,C",
          "names": ["Message", "world"],
          "sources": ["messages.txt", "hello world.txt", "HELLO_WORLD.txt"],
          "sourcesContent": [null, "hello world", "HELLO WORLD"],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap()
    );
  }

  #[test]
  fn should_map_generated_with_correct_inner_source_index() {
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: r#"(()=>{function n(){b1("*b0*")}function b(){n("*a0*")}b()})();"#,
      name: "main.js",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["main.js"],
          "mappings": "CAAC,IAAM,CAEL,SAASA,GAAK,CACZ,GAAG,MAAM,CACX,CAGA,SAASC,GAAK,CACZD,EAAG,MAAM,CACX,CACAC,EAAG,CACL,GAAG",
          "names": ["b0", "a0"]
        }"#.to_string(),
      ).unwrap(),
      original_source: Some(r#"(() => {
  // b.js
  function b0() {
    b1("*b0*");
  }

  // a.js
  function a0() {
    b0("*a0*");
  }
  a0();
})();
"#.into()),
      inner_source_map: Some(SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["b.js", "a.js"],
          "sourcesContent": ["export function b0() {\n\tb1(\"*b0*\");\n}\n", "import { b0 } from \"./b.js\";\nfunction a0() {\n\tb0(\"*a0*\");\n}\na0()\n"],
          "mappings": ";;AAAO,WAAS,KAAK;AACpB,OAAG,MAAM;AAAA,EACV;;;ACDA,WAAS,KAAK;AACb,OAAG,MAAM;AAAA,EACV;AACA,KAAG;",
          "names": []
        }"#.to_string()
      ).unwrap()),
      remove_original_source: true,
    });
    let map = source
      .map(&ObjectPool::default(), &MapOptions::default())
      .unwrap();
    assert_eq!(
      map,
      SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["b.js", "a.js"],
          "sourcesContent": ["export function b0() {\n\tb1(\"*b0*\");\n}\n", "import { b0 } from \"./b.js\";\nfunction a0() {\n\tb0(\"*a0*\");\n}\na0()\n"],
          "names": ["b0", "a0"],
          "mappings": "MAAO,SAASA,GAAK,CACpB,GAAG,MAAM,CACV,CCDA,SAASC,GAAK,CACbD,EAAG,MAAM,CACV,CACAC,EAAG,C"
        }"#.to_string()
      ).unwrap()
    );
  }

  #[test]
  fn should_have_map_when_columns_is_false_and_last_line_start_is_none() {
    let source = SourceMapSource::new(WithoutOriginalOptions {
      value: "console.log('a')\n",
      name: "a.js",
      source_map: OriginalSource::new("console.log('a')\n", "a.js")
        .boxed()
        .map_static(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap(),
    });
    let source = ConcatSource::new([
      RawStringSource::from("\n").boxed(),
      RawStringSource::from("\n").boxed(),
      RawStringSource::from("\n").boxed(),
      source.boxed(),
    ]);
    let map = source
      .map(&ObjectPool::default(), &MapOptions::new(false))
      .unwrap();
    assert_eq!(map.mappings(), ";;;AAAA");
  }

  #[test]
  fn source_root_is_correctly_applied_to_mappings() {
    let inner_source_code = "Hello World\nis a test string\n";
    let inner_source = ConcatSource::new([
      OriginalSource::new(inner_source_code, "hello-world.txt").boxed(),
      OriginalSource::new("Translate: ", "header.txt").boxed(),
      RawStringSource::from("Other text").boxed(),
    ])
    .boxed();
    let source_r_code = "Translated: Hallo Welt\nist ein test Text\nAnderer Text";
    let source_r_map = SourceMap::from_json(
      r#"{
        "version": 3,
        "sources": [ "text" ],
        "names": [ "Hello", "World", "nope" ],
        "mappings": "YAAAA,K,CAAMC;AACNC,O,MAAU;AACC,O,CAAM",
        "file": "translated.txt",
        "sourcesContent": [ "Hello World\nis a test string\n" ]
      }"#
        .to_string(),
    )
    .unwrap();
    let inner_source_map = inner_source
      .clone()
      .map_static(&ObjectPool::default(), &MapOptions::default())
      .map(|mut map| {
        map.set_source_root(Some("/path/to/folder/".to_string().into()));
        map
      });
    let sms = SourceMapSource::new(SourceMapSourceOptions {
      value: source_r_code,
      name: "text",
      source_map: source_r_map,
      original_source: Some(inner_source.source().into_string_lossy().into()),
      inner_source_map,
      remove_original_source: false,
    });
    assert_eq!(
      sms
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "YAAAA,K,CAAMC;AACN,O,MAAU;ACCC,O,CAAM",
          "names": ["Hello", "World"],
          "sources": ["/path/to/folder/hello-world.txt", "text"],
          "sourcesContent": [
            "Hello World\nis a test string\n",
            "Hello World\nis a test string\nTranslate: Other text"
          ],
          "version": 3
        }"#
          .to_string()
      )
      .unwrap(),
    );
  }

  #[test]
  fn should_ignores_names_without_columns() {
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: "h",
      name: "hello.txt",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["hello.txt"],
          "mappings": "AAAAA",
          "names": ["hello"]
        }"#
          .to_string(),
      )
      .unwrap(),
      original_source: Some("hello".into()),
      inner_source_map: Some(
        SourceMap::from_json(
          r#"{
          "version": 3,
          "sources": ["hello world.txt"],
          "mappings": "AAAA",
          "names": [],
          "sourcesContent": ["hello, world!"]
        }"#
            .to_string(),
        )
        .unwrap(),
      ),
      remove_original_source: false,
    });
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::new(false))
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "mappings": "AAAA",
          "names": [],
          "sources": ["hello world.txt"],
          "version": 3,
          "sourcesContent": ["hello, world!"]
        }"#
          .to_string()
      )
      .unwrap()
    );
  }

  #[test]
  fn should_not_panic_when_check_for_an_identity_mapping() {
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: "hello world",
      name: "hello.txt",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["hello.txt"],
          "mappings": "AAAA,MAAG"
        }"#
          .to_string(),
      )
      .unwrap(),
      original_source: Some("你好 世界".into()),
      inner_source_map: Some(
        SourceMap::from_json(
          r#"{
          "version": 3,
          "sources": ["hello world.txt"],
          "mappings": "AAAA,EAAE",
          "sourcesContent": ["你好✋世界"]
        }"#
            .to_string(),
        )
        .unwrap(),
      ),
      remove_original_source: false,
    });
    assert_eq!(
      source
        .map(&ObjectPool::default(), &MapOptions::default())
        .unwrap(),
      SourceMap::from_json(
        r#"{
          "version": 3,
          "mappings": "AAAA,MAAE",
          "sources": ["hello world.txt"],
          "sourcesContent": ["你好✋世界"]
        }"#
          .to_string()
      )
      .unwrap()
    );
  }

  #[test]
  fn test_debug_output() {
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: "hello world",
      name: "hello.txt",
      source_map: SourceMap::from_json(
        r#"{
          "version": 3,
          "sources": ["hello.txt"],
          "mappings": "AAAA,MAAG"
        }"#
          .to_string(),
      )
      .unwrap(),
      original_source: Some("你好 世界".into()),
      inner_source_map: Some(
        SourceMap::from_json(
          r#"{
          "version": 3,
          "sources": ["hello world.txt"],
          "mappings": "AAAA,EAAE",
          "sourcesContent": ["你好✋世界"]
        }"#
            .to_string(),
        )
        .unwrap(),
      ),
      remove_original_source: false,
    });

    assert_eq!(
      format!("{:?}", source),
      r#"SourceMapSource::new(SourceMapSourceOptions {
  value: "hello world",
  name: "hello.txt",
  source_map: SourceMap::from_json("{\"version\":3,\"sources\":[\"hello.txt\"],\"names\":[],\"mappings\":\"AAAA,MAAG\"}".to_string()).unwrap(),
  original_source: Some("你好 世界".to_string()),
  inner_source_map: Some(SourceMap::from_json("{\"version\":3,\"sources\":[\"hello world.txt\"],\"sourcesContent\":[\"你好✋世界\"],\"names\":[],\"mappings\":\"AAAA,EAAE\"}".to_string()).unwrap()),
  remove_original_source: false,
}).boxed()"#
    );
  }
}
