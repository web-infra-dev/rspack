use std::{mem::size_of, sync::Arc};

use rspack_sources::{
  CachedSource, Content, ContentSlice, CowContent, MapOptions, ObjectPool, OriginalSource,
  RawBufferSource, RawStringSource, ReplaceSource, Source, SourceMap, SourceMapSource, SourceValue,
  WithoutOriginalOptions, stream_chunks::StreamChunks,
};

#[test]
fn raw_source_and_source_value_share_the_ascii_cache() {
  for source in [
    RawStringSource::from("éabc"),
    RawStringSource::from_static("éabc"),
  ] {
    match source.source() {
      SourceValue::String(CowContent::Borrowed(value)) => {
        assert_eq!(value.cached_is_ascii(), None);
        assert!(!value.is_ascii());
      }
      _ => panic!("raw string source should borrow its content"),
    }
    match source.source() {
      SourceValue::String(value) => assert_eq!(value.cached_is_ascii(), Some(false)),
      _ => panic!("raw string source should return text"),
    }
  }
}

#[test]
fn rope_preserves_owner_tags_and_keeps_substring_tags_local() {
  let raw = RawStringSource::from("plain");
  raw.rope(&mut |chunk| assert!(chunk.is_ascii()));
  match raw.source() {
    SourceValue::String(value) => assert_eq!(value.cached_is_ascii(), Some(true)),
    _ => panic!("raw string source should return text"),
  }

  let mut replaced = ReplaceSource::new(OriginalSource::new("éabc", "example.js"));
  replaced.replace_static(0, 2, "", None);
  let original = replaced
    .inner()
    .as_ref()
    .as_any()
    .downcast_ref::<OriginalSource>()
    .unwrap();
  let mut saw_substring = false;
  replaced.rope(&mut |chunk| {
    if chunk.as_str() == "abc" {
      saw_substring = true;
      assert_eq!(chunk.cached_is_ascii(), None);
      assert!(chunk.is_ascii());
      assert_eq!(chunk.cached_is_ascii(), Some(true));
    }
  });
  assert!(saw_substring);
  assert_eq!(original.value().cached_is_ascii(), None);
}

#[test]
fn cached_rope_preserves_the_inner_content_tag() {
  let cached = CachedSource::new(RawStringSource::from("plain"));
  cached.rope(&mut |chunk| assert!(chunk.is_ascii()));
  match cached.inner().source() {
    SourceValue::String(value) => assert_eq!(value.cached_is_ascii(), Some(true)),
    _ => panic!("cached inner source should return text"),
  }
}

#[test]
fn lossy_buffer_rope_reuses_its_content_tag() {
  let source = RawBufferSource::from(vec![0xff]);
  source.rope(&mut |chunk| {
    assert_eq!(chunk.cached_is_ascii(), None);
    assert!(!chunk.is_ascii());
  });
  source.rope(&mut |chunk| assert_eq!(chunk.cached_is_ascii(), Some(false)));
}

#[test]
fn cached_source_returns_content_with_known_ascii_status() {
  let ascii = CachedSource::new(RawStringSource::from("const value = 42;"));
  let non_ascii = CachedSource::new(RawStringSource::from("const café = 42;"));
  for (source, expected) in [(&ascii, true), (&non_ascii, false)] {
    match source.source() {
      SourceValue::String(value) => assert_eq!(value.cached_is_ascii(), Some(expected)),
      _ => panic!("cached string source should return text"),
    }
  }
}

#[test]
fn content_from_iter_collects_text_and_ascii_status() {
  let first = Content::from("abc");
  let second = Content::from("é");
  let joined = Content::from_iter([first.as_ref(), second.as_ref()], 5);
  assert_eq!(joined.as_str(), "abcé");
  assert_eq!(joined.cached_is_ascii(), Some(false));
  assert_eq!(first.cached_is_ascii(), Some(true));
  assert_eq!(second.cached_is_ascii(), Some(false));
}

#[test]
fn owned_content_returns_its_allocation_without_copying() {
  let text: Box<str> = "hello".into();
  let ptr = text.as_ptr();
  let content = Content::from_boxed(text);
  let restored = content.into_boxed_str();
  assert_eq!(restored.as_ptr(), ptr);
  assert_eq!(&*restored, "hello");
}

#[test]
fn streamed_complete_chunk_updates_owner_but_substring_does_not() {
  let object_pool = ObjectPool::default();
  let options = MapOptions::default();
  let whole = OriginalSource::new("plain", "whole.js");
  whole.stream_chunks().stream(
    &object_pool,
    &options,
    &mut |chunk, _| assert!(chunk.unwrap().is_ascii()),
    &mut |_, _, _| {},
    &mut |_, _| {},
  );
  assert_eq!(whole.value().cached_is_ascii(), Some(true));

  let split = OriginalSource::new("é;abc", "split.js");
  split.stream_chunks().stream(
    &object_pool,
    &options,
    &mut |chunk, _| {
      let chunk = chunk.unwrap();
      if chunk.as_str() == "abc" {
        assert!(chunk.is_ascii());
      }
    },
    &mut |_, _, _| {},
    &mut |_, _| {},
  );
  assert_eq!(split.value().cached_is_ascii(), None);

  let mapped = SourceMapSource::new(WithoutOriginalOptions {
    value: "plain".to_string(),
    name: "mapped.js".to_string(),
    source_map: SourceMap::new(
      "AAAA",
      vec!["mapped.js".into()],
      vec!["plain".into()],
      vec![],
    ),
  });
  mapped.stream_chunks().stream(
    &object_pool,
    &options,
    &mut |chunk, _| assert!(chunk.unwrap().is_ascii()),
    &mut |_, _, _| {},
    &mut |_, _| {},
  );
  assert_eq!(mapped.value().cached_is_ascii(), Some(true));
}

#[test]
fn complete_view_updates_owner_and_substring_keeps_its_own_cache() {
  let owner = Content::from_boxed("éabc".into());
  assert_eq!(owner.cached_is_ascii(), None);

  let complete = owner.slice(0..owner.len()).unwrap();
  assert!(!complete.is_ascii());
  assert_eq!(owner.cached_is_ascii(), Some(false));

  let substring = owner.slice(2..owner.len()).unwrap();
  assert_eq!(substring.cached_is_ascii(), None);
  assert!(substring.is_ascii());
  assert_eq!(substring.cached_is_ascii(), Some(true));
  assert_eq!(owner.cached_is_ascii(), Some(false));
  assert!(owner.slice(1..2).is_none());

  let borrowed = CowContent::from_content(&owner);
  assert_eq!(borrowed.cached_is_ascii(), Some(false));

  let ascii_owner = Content::from_boxed("hello".into());
  let complete = ascii_owner.as_ref().clone();
  assert!(complete.is_ascii());
  assert_eq!(ascii_owner.cached_is_ascii(), Some(true));
  assert_eq!(
    ascii_owner.slice(1..4).unwrap().cached_is_ascii(),
    Some(true)
  );
}

#[test]
fn source_map_contents_retain_a_tag() {
  let source = OriginalSource::new("const value = 'é';\n", "example.js");
  let map = source
    .map(&ObjectPool::default(), &MapOptions::default())
    .unwrap();
  let content = map.get_source_content(0).unwrap();
  assert_eq!(content.cached_is_ascii(), None);
  assert_eq!(source.value().cached_is_ascii(), None);
  assert!(!content.is_ascii());
  assert_eq!(content.cached_is_ascii(), Some(false));
  assert_eq!(source.value().cached_is_ascii(), Some(false));

  let parsed = SourceMap::from_json(map.to_json()).unwrap();
  let parsed_content = parsed.get_source_content(0).unwrap();
  assert_eq!(parsed_content.cached_is_ascii(), None);
  assert!(!parsed_content.is_ascii());
  assert_eq!(parsed_content.cached_is_ascii(), Some(false));
}

#[test]
fn source_content_callback_updates_the_owner_tag() {
  let source = OriginalSource::new("const café = 1;\n", "example.js");
  assert_eq!(source.value().cached_is_ascii(), None);

  source.stream_chunks().stream(
    &ObjectPool::default(),
    &MapOptions::default(),
    &mut |_, _| {},
    &mut |_, _, content| assert!(!content.unwrap().is_ascii()),
    &mut |_, _| {},
  );

  assert_eq!(source.value().cached_is_ascii(), Some(false));

  let mapped = SourceMapSource::new(WithoutOriginalOptions {
    value: "generated".to_string(),
    name: "generated.js".to_string(),
    source_map: SourceMap::new(
      "AAAA",
      vec!["example.js".into()],
      vec!["const café = 1;\n".into()],
      vec![],
    ),
  });
  assert_eq!(
    mapped
      .source_map()
      .get_source_content(0)
      .unwrap()
      .cached_is_ascii(),
    None
  );
  mapped.stream_chunks().stream(
    &ObjectPool::default(),
    &MapOptions::default(),
    &mut |_, _| {},
    &mut |_, _, content| assert!(!content.unwrap().is_ascii()),
    &mut |_, _| {},
  );
  assert_eq!(
    mapped
      .source_map()
      .get_source_content(0)
      .unwrap()
      .cached_is_ascii(),
    Some(false)
  );
}

#[test]
fn static_source_map_retains_the_tag_owner() {
  let source = Arc::new(OriginalSource::new("const café = 1;\n", "example.js"));
  let map = source
    .clone()
    .map_static(&ObjectPool::default(), &MapOptions::default())
    .unwrap();
  drop(source);

  let content = map.get_source_content(0).unwrap();
  assert_eq!(content.cached_is_ascii(), None);
  assert!(!content.is_ascii());
  assert_eq!(content.cached_is_ascii(), Some(false));
}

#[test]
fn local_views_do_not_change_an_unrelated_cache() {
  let original = ContentSlice::from_str("café");
  let copy = original.clone();
  assert!(!copy.is_ascii());
  assert_eq!(copy.cached_is_ascii(), Some(false));
  assert_eq!(original.cached_is_ascii(), None);
}

#[test]
fn source_map_accepts_owned_content() {
  let content = Content::from("let a = 1;");
  assert!(content.is_ascii());
  let map = SourceMap::new("AAAA", vec!["a.js".into()], vec![content.into()], vec![]);
  assert_eq!(
    map.get_source_content(0).unwrap().cached_is_ascii(),
    Some(true)
  );
}

#[test]
fn compact_layout_and_thread_safety() {
  fn assert_send_sync<T: Send + Sync>() {}
  assert_send_sync::<Content>();
  assert_send_sync::<ContentSlice<'_>>();
  assert_send_sync::<CowContent<'_>>();
  if cfg!(target_pointer_width = "64") {
    assert_eq!(size_of::<Content>(), size_of::<Box<str>>());
    assert_eq!(size_of::<Content>(), 16);
    assert_eq!(size_of::<ContentSlice<'_>>(), 16);
    assert_eq!(size_of::<CowContent<'_>>(), 24);
  }
}
