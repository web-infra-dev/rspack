use rspack_sources::{CachedSource, ConcatSource, RawStringSource, Source, SourceExt};

#[test]
fn released_rope_cache_is_rebuilt_on_demand() {
  let source = CachedSource::new(ConcatSource::new([
    RawStringSource::from("first\n".to_string()).boxed(),
    RawStringSource::from("second\n".to_string()).boxed(),
  ]));

  assert_eq!(source.source().into_string_lossy(), "first\nsecond\n");

  source.release_rope_caches();

  assert_eq!(source.source().into_string_lossy(), "first\nsecond\n");
  let mut chunks = Vec::new();
  source.rope(&mut |chunk| chunks.push(chunk));
  assert_eq!(chunks.concat(), "first\nsecond\n");
}
