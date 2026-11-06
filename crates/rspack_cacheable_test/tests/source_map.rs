use std::{
  borrow::Cow,
  collections::hash_map::DefaultHasher,
  hash::{Hash, Hasher},
};

use rspack_sources::{MapOptions, OriginalSource, Source, SourceMap};

fn json() -> String {
  r#"{"version":3,"file":"out.js","sources":["src/你好.js","src\/b.js",null,""],"sourcesContent":["line1\n\"quote\"\\tail","\u4F60\u597D \uD83D\uDE00 \u0000",null,""],"names":["alpha","\u0062eta"],"mappings":"AAAA;AACA","sourceRoot":"root","debugId":"debug-id","ignoreList":[0,2]}"#.to_owned()
}

#[test]
fn parsed_strings_share_the_input_allocation() {
  for spare_capacity in [false, true] {
    let mut input = json();
    if spare_capacity {
      input.reserve(4096);
    }
    let allocation = input.as_ptr() as usize..input.as_ptr() as usize + input.len();
    let map = SourceMap::from_json(input).unwrap();
    for text in map
      .sources()
      .iter()
      .chain(map.sources_content().iter().flatten())
      .chain(map.names())
    {
      if !text.is_empty() {
        assert!(allocation.contains(&(text.as_ptr() as usize)));
        assert!(text.as_ptr() as usize + text.len() <= allocation.end);
      }
    }
    for text in [
      map.file(),
      Some(map.mappings()),
      map.source_root(),
      map.get_debug_id(),
    ]
    .into_iter()
    .flatten()
    {
      assert!(allocation.contains(&(text.as_ptr() as usize)));
    }
    assert_eq!(map.get_source(1), Some("src/b.js"));
    assert_eq!(
      map.get_source_content(0).unwrap().as_str(),
      "line1\n\"quote\"\\tail"
    );
    assert_eq!(map.get_source_content(1).unwrap().as_str(), "你好 😀 \0");
    assert!(map.get_source_content(2).is_none());
    assert_eq!(map.get_source_content(3).unwrap().as_str(), "");
    assert_eq!(map.ignore_list(), Some([0, 2].as_slice()));
    // Independently owned fields keep the decoded input alive after dropping the map.
    let content = map.get_source_content(1).unwrap().clone();
    drop(map);
    assert_eq!(content.as_str(), "你好 😀 \0");
  }
}

#[test]
fn parsed_fields_serialize_as_strings_and_preserve_empty_entries() {
  let map = SourceMap::from_json(json()).unwrap();
  let serialized = map.to_json();
  let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();
  assert_eq!(value["sourcesContent"][1], "你好 😀 \0");
  assert!(value["sourcesContent"][2].is_null());
  assert_eq!(value["sourcesContent"][3], "");
  assert_eq!(value["sources"][0], "src/你好.js");
  assert_eq!(value["names"][1], "beta");
  assert_eq!(map, SourceMap::from_json(serialized).unwrap());
  let duplicate = format!(
    "{},\"debugId\":\"debug-id\"}}",
    json().trim_end_matches('}')
  );
  assert_eq!(map, SourceMap::from_json(duplicate).unwrap());
}

#[test]
fn removing_contents_detaches_the_remaining_json_fields() {
  let input = json();
  let allocation = input.as_ptr() as usize..input.as_ptr() as usize + input.len();
  let mut map = SourceMap::from_json(input).unwrap();
  // Keep the old allocation occupied so a detached allocation cannot reuse its address.
  let original = map.get_source_content(0).unwrap().clone();
  map.remove_sources_content();
  assert!(map.sources_content().is_empty());
  for text in map.sources().iter().chain(map.names()) {
    if !text.is_empty() {
      assert!(!allocation.contains(&(text.as_ptr() as usize)));
    }
  }
  for text in [
    map.file(),
    Some(map.mappings()),
    map.source_root(),
    map.get_debug_id(),
  ]
  .into_iter()
  .flatten()
  {
    assert!(!allocation.contains(&(text.as_ptr() as usize)));
  }
  assert_eq!(original.as_str(), "line1\n\"quote\"\\tail");
  assert!(!map.to_json().contains("sourcesContent"));
}

#[test]
fn generated_maps_share_source_contents() {
  let source = OriginalSource::new("const value = '你好';\n".to_owned(), "input.js".to_owned());
  let text = source.source();
  let map = source.map(&MapOptions::default()).unwrap();
  let content = map.get_source_content(0).unwrap();
  assert_eq!(content.as_ptr(), text.as_bytes().as_ptr());
}

#[test]
fn shared_fields_preserve_text_hashing() {
  fn hash(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
  }
  let mut shared = SourceMap::from_json(json()).unwrap();
  let borrowed = shared.as_borrowed();
  assert_eq!(shared, borrowed);
  assert_eq!(hash(&shared), hash(&borrowed));
  assert_eq!(
    hash(&shared.sources()[0]),
    hash(&Cow::Borrowed("src/你好.js"))
  );
  drop(borrowed);
  shared.set_sources(vec!["new.js"]);
  shared.set_sources_content(vec![Some("new contents".into())]);
  shared.set_names(vec!["new_name"]);
  shared.set_file(Some("new.map".into()));
  assert_eq!(shared.get_source(0), Some("new.js"));
  assert_eq!(
    shared.get_source_content(0).unwrap().as_str(),
    "new contents"
  );
  assert_eq!(shared.get_name(0), Some("new_name"));
  assert_eq!(shared.file(), Some("new.map"));
}

#[test]
fn malformed_source_map_fields_are_still_rejected() {
  for input in [
    r#"{}"#,
    r#"{"mappings":null}"#,
    r#"{"mappings":[],"sources":[]}"#,
    r#"{"mappings":"","sources":[1]}"#,
    r#"{"mappings":"","sourcesContent":[{}]}"#,
    r#"{"mappings":"","ignoreList":[-1]}"#,
    r#"{"mappings":"AAAA","file":"out.js","sources":["a.js"],"sourcesContent":["text"],"names":["name",1]}"#,
    r#"{"mappings":"AAAA","file":"out.js","sources":["a.js"],"sourcesContent":["text"],"names":["name"],"ignoreList":[0,-1]}"#,
  ] {
    assert!(SourceMap::from_json(input.to_owned()).is_err(), "{input}");
  }
  assert!(SourceMap::from_bytes(vec![0xff]).is_err());
}

#[test]
fn generated_map_outlives_its_source_without_retaining_the_graph() {
  use std::sync::Arc;
  let source = Arc::new(OriginalSource::new(
    "const value = '你好';\n".to_owned(),
    "input.js".to_owned(),
  ));
  let weak = Arc::downgrade(&source);
  let pointer = source.value().as_ptr();
  let map = source.clone().map_static(&MapOptions::default()).unwrap();
  drop(source);
  assert!(weak.upgrade().is_none());
  assert_eq!(map.get_source_content(0).unwrap().as_ptr(), pointer);
  assert_eq!(
    map.get_source_content(0).unwrap().as_str(),
    "const value = '你好';\n"
  );
}

#[test]
fn cached_map_releases_its_source_graph() {
  use std::sync::Arc;

  use rspack_sources::{CachedSource, ConcatSource, ReplaceSource, SourceExt};

  let original = Arc::new(OriginalSource::new("const value = 1;\n", "input.js"));
  let weak_original = Arc::downgrade(&original);
  let pointer = original.value().as_ptr();
  let mut replacement = ReplaceSource::new(original as rspack_sources::BoxSource);
  replacement.replace_static(14, 15, "2", Some("value"));
  let source = CachedSource::new(ConcatSource::new([replacement.boxed()])).boxed();
  let weak_source = Arc::downgrade(&source);
  let map = source.clone().map_static(&MapOptions::default()).unwrap();
  drop(source);
  assert!(weak_source.upgrade().is_none());
  assert!(weak_original.upgrade().is_none());
  assert_eq!(map.get_source_content(0).unwrap().as_ptr(), pointer);
  assert_eq!(
    map.get_source_content(0).unwrap().as_str(),
    "const value = 1;\n"
  );
}

#[test]
fn cloned_maps_apply_setters_without_copying_unchanged_text() {
  let original = SourceMap::from_json(json()).unwrap();
  let pointer = original.get_source_content(0).unwrap().as_ptr();
  let mut edited = original.clone();
  edited.set_file(Some("edited.map".into()));
  assert_eq!(original.file(), Some("out.js"));
  assert_eq!(edited.file(), Some("edited.map"));
  assert_eq!(edited.get_source_content(0).unwrap().as_ptr(), pointer);
  edited.remove_sources_content();
  assert!(!original.sources_content().is_empty());
  assert!(edited.sources_content().is_empty());
}

#[test]
fn structured_cache_round_trip_preserves_map_text_and_hash() {
  use rspack_sources::cacheable::CacheableSourceMap;
  let original = SourceMap::from_json(json()).unwrap();
  let restored = SourceMap::from(CacheableSourceMap::from(&original));
  assert_eq!(original.to_json(), restored.to_json());
  assert_eq!(original, restored);
  let mut original_hash = DefaultHasher::new();
  let mut restored_hash = DefaultHasher::new();
  original.hash(&mut original_hash);
  restored.hash(&mut restored_hash);
  assert_eq!(original_hash.finish(), restored_hash.finish());
}
