use std::sync::{
  Arc,
  atomic::{AtomicBool, Ordering},
};

use bytes::Bytes;
use bytes_str::BytesStr;
use rspack_sources::{
  BoxSource, CachedSource, ConcatSource, MapOptions, Source, SourceExt, SourceMap, SourceMapSource,
  SourceMapSourceOptions, stream_chunks::StreamChunks,
};

struct OuterStorage {
  text: String,
  released: Arc<AtomicBool>,
}

impl AsRef<[u8]> for OuterStorage {
  fn as_ref(&self) -> &[u8] {
    self.text.as_bytes()
  }
}

impl Drop for OuterStorage {
  fn drop(&mut self) {
    self.released.store(true, Ordering::SeqCst);
  }
}

#[test]
fn combined_maps_share_independently_owned_outer_contents() {
  let content = BytesStr::from_string("external();\n".repeat(100_000));
  let content_ptr = content.as_ptr();
  let source = SourceMapSource::new(SourceMapSourceOptions {
    value: "first();\nsecond();\n",
    name: "intermediate.js",
    source_map: SourceMap::new(
      "AAAA;ACAA".into(),
      vec!["intermediate.js".into(), "external.js".into()],
      vec![Some("intermediate();\n".into()), Some(content)],
      vec![],
    ),
    original_source: None,
    inner_source_map: Some(SourceMap::new(
      "AAAA".into(),
      vec!["original.js".into()],
      vec![Some("original();\n".into())],
      vec![],
    )),
    remove_original_source: true,
  });
  let map = source.map(&MapOptions::default()).unwrap();
  drop(source);
  let index = map
    .sources()
    .iter()
    .position(|s| &**s == "external.js")
    .unwrap();
  assert_eq!(map.get_source_content(index).unwrap().as_ptr(), content_ptr);
}

#[test]
fn combined_maps_release_unused_outer_storage_and_share_inner_contents() {
  for columns in [true, false] {
    for path in ["direct", "cached-map", "cached-stream", "concat"] {
      let released = Arc::new(AtomicBool::new(false));
      let storage = BytesStr::from_utf8(Bytes::from_owner(OuterStorage {
        text: format!(
          "intermediate.js\0{}\0external.js\0external();\0innerName\0outerName",
          "unused intermediate text".repeat(400_000)
        ),
        released: released.clone(),
      }))
      .unwrap();
      let fields: Vec<_> = storage
        .split('\0')
        .map(|field| storage.slice_ref(field))
        .collect();
      let outer = SourceMap::new(
        "AAAAA;ACAAC".into(),
        vec![fields[0].clone(), fields[2].clone()],
        vec![Some(fields[1].clone()), Some(fields[3].clone())],
        vec![fields[4].clone(), fields[5].clone()],
      );
      drop(fields);
      drop(storage);
      let inner_content = BytesStr::from_string("original();\n".repeat(100_000));
      let inner_content_ptr = inner_content.as_ptr();
      let source = SourceMapSource::new(SourceMapSourceOptions {
        value: "first();\nsecond();\n",
        name: "intermediate.js",
        source_map: outer,
        original_source: None,
        inner_source_map: Some(SourceMap::new(
          "AAAA".into(),
          vec!["original.js".into()],
          vec![Some(inner_content)],
          vec![],
        )),
        remove_original_source: true,
      })
      .boxed();
      let source: BoxSource = match path {
        "cached-map" | "cached-stream" => CachedSource::new(source).boxed(),
        "concat" => ConcatSource::new([source]).boxed(),
        _ => source,
      };
      let options = MapOptions::new(columns);
      if path == "cached-stream" {
        source
          .stream_chunks()
          .stream(&options, &mut |_, _| {}, &mut |_, _, _| {}, &mut |_, _| {});
      }
      let map = source.map(&options).unwrap();
      assert!(!released.load(Ordering::SeqCst));
      drop(source);

      assert!(
        released.load(Ordering::SeqCst),
        "{path}, columns={columns}: the map retains unused outer storage"
      );
      let original = map
        .sources()
        .iter()
        .position(|s| &**s == "original.js")
        .unwrap();
      let external = map
        .sources()
        .iter()
        .position(|s| &**s == "external.js")
        .unwrap();
      assert_eq!(
        map.get_source_content(original).unwrap().as_ptr(),
        inner_content_ptr
      );
      assert_eq!(
        map.get_source_content(external).unwrap().as_str(),
        "external();"
      );
      let mappings: Vec<_> = map.decoded_mappings().collect();
      assert_eq!(mappings.len(), 2);
      for (mapping, line, file) in [
        (&mappings[0], 1, "original.js"),
        (&mappings[1], 2, "external.js"),
      ] {
        assert_eq!(mapping.generated_line, line);
        let original = mapping.original.unwrap();
        assert_eq!(map.get_source(original.source_index as usize), Some(file));
        assert_eq!(original.original_line, 1);
        assert_eq!(original.original_column, 0);
      }
      assert_eq!(
        map.names().iter().any(|name| &**name == "outerName"),
        columns
      );
    }
  }
}

#[test]
fn combined_maps_release_outer_storage_retained_only_by_a_name() {
  let released = Arc::new(AtomicBool::new(false));
  let storage = BytesStr::from_utf8(Bytes::from_owner(OuterStorage {
    text: format!("outerName\0{}", "unused text".repeat(100_000)),
    released: released.clone(),
  }))
  .unwrap();
  let name = storage.slice_ref(storage.split('\0').next().unwrap());
  let unused = storage.slice_ref(storage.split('\0').nth(1).unwrap());
  let source = SourceMapSource::new(SourceMapSourceOptions {
    value: "generated();\n",
    name: "intermediate.js",
    source_map: SourceMap::new(
      "AAAAA".into(),
      vec!["intermediate.js".into()],
      vec![Some(unused)],
      vec![name],
    ),
    original_source: None,
    inner_source_map: Some(SourceMap::new(
      "AAAA".into(),
      vec!["original.js".into()],
      vec![Some("outerName();\n".into())],
      vec![],
    )),
    remove_original_source: true,
  });
  drop(storage);
  let map = source.map(&MapOptions::default()).unwrap();
  drop(source);

  assert!(released.load(Ordering::SeqCst));
  assert_eq!(map.get_name(0), Some("outerName"));
}

#[test]
fn unmapped_intermediate_contents_release_outer_storage() {
  for provide_original in [false, true] {
    let released = Arc::new(AtomicBool::new(false));
    let original = "intermediate();\nunmapped();\n";
    let storage = BytesStr::from_utf8(Bytes::from_owner(OuterStorage {
      text: format!("{original}\0AAAA;AACA{}", ";".repeat(100_000)),
      released: released.clone(),
    }))
    .unwrap();
    let mut fields = storage.split('\0');
    let content = storage.slice_ref(fields.next().unwrap());
    let mappings = storage.slice_ref(fields.next().unwrap());
    let source = SourceMapSource::new(SourceMapSourceOptions {
      value: "first();\nsecond();\n",
      name: "intermediate.js",
      source_map: SourceMap::new(
        mappings,
        vec!["intermediate.js".into()],
        vec![Some(content)],
        vec![],
      ),
      original_source: provide_original.then(|| original.into()),
      inner_source_map: Some(SourceMap::new(
        "AAAA".into(),
        vec!["original.js".into()],
        vec![Some("original();\n".into())],
        vec![],
      )),
      remove_original_source: false,
    });
    let original_ptr = source.original_source().map(str::as_ptr);
    drop(storage);
    let map = source.map(&MapOptions::default()).unwrap();
    drop(source);

    assert!(released.load(Ordering::SeqCst));
    let index = map
      .sources()
      .iter()
      .position(|s| &**s == "intermediate.js")
      .unwrap();
    let content = map.get_source_content(index).unwrap();
    assert_eq!(content.as_str(), original);
    if let Some(original_ptr) = original_ptr {
      assert_eq!(content.as_ptr(), original_ptr);
    }
  }
}
