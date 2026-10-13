use std::{
  hash::{Hash, Hasher},
  io::{self, Read, Write},
};

use rspack_cacheable::{enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset};
use rspack_sources::{AssetSource, BoxSource, RawBufferSource, RawStringSource, Source, SourceExt};

#[cacheable]
#[derive(Debug)]
struct Data(#[cacheable(with=AsPreset)] BoxSource);

#[test]
fn test_rspack_source() {
  fn test_data(data: Data) {
    let bytes = to_bytes(&data, &()).unwrap();
    let new_data: Data = from_bytes(&bytes, &()).unwrap();
    assert_eq!(data.0.buffer(), new_data.0.buffer());
    assert_eq!(
      data.0.map(&Default::default()),
      new_data.0.map(&Default::default())
    );
  }

  test_data(Data(RawBufferSource::from("123".as_bytes()).boxed()));
  test_data(Data(RawStringSource::from_static("123").boxed()));
  test_data(Data(
    AssetSource::new("😀原始内容\n".repeat(20_000).as_str())
      .unwrap()
      .boxed(),
  ));
}

#[test]
fn test_asset_content_hash_and_streaming_cache_roundtrip() {
  let value = serde_json::json!({ "sourcesContent": ["😀原始内容\n".repeat(20_000)] });
  let mut observed = Vec::new();
  let source =
    AssetSource::from_json_with_observer(&value, |bytes| observed.extend_from_slice(bytes))
      .unwrap();
  let same_content = AssetSource::new(std::str::from_utf8(&observed).unwrap()).unwrap();
  assert_eq!(source.content_hash(), same_content.content_hash());
  assert_eq!(source, same_content);
  assert!(source.compressed_buffer().len() < source.size() / 10);

  let data = Data(source.boxed());
  let cached = to_bytes(&data, &()).unwrap();
  let restored: Data = from_bytes(&cached, &()).unwrap();
  let restored_asset = restored
    .0
    .as_ref()
    .as_any()
    .downcast_ref::<AssetSource>()
    .unwrap();
  let mut original_hash = rustc_hash::FxHasher::default();
  let mut restored_hash = rustc_hash::FxHasher::default();
  data.0.hash(&mut original_hash);
  restored.0.hash(&mut restored_hash);
  assert_eq!(original_hash.finish(), restored_hash.finish());
  assert_eq!(restored_asset.content_hash(), same_content.content_hash());
  assert_eq!(
    restored_asset.compressed_buffer(),
    same_content.compressed_buffer()
  );

  struct ShortWriter(Vec<u8>);
  impl Write for ShortWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
      let count = bytes.len().min(17);
      self.0.extend_from_slice(&bytes[..count]);
      Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
      Ok(())
    }
  }
  let mut output = ShortWriter(Vec::new());
  restored.0.to_writer(&mut output).unwrap();
  assert_eq!(output.0, observed);
  assert!(restored_asset.matches_reader(observed.as_slice()).unwrap());
  assert!(
    !restored_asset
      .matches_reader(&observed[..observed.len() - 1])
      .unwrap()
  );
  observed.push(b' ');
  assert!(!restored_asset.matches_reader(observed.as_slice()).unwrap());
  let mut reader = restored_asset.reader().unwrap();
  let mut prefix = [0; 7];
  reader.read_exact(&mut prefix).unwrap();
  assert_eq!(prefix, output.0[..7]);
}

#[test]
fn test_asset_corrupted_cache_is_rejected() {
  use rspack_sources::cacheable::{CacheableSource, from_cacheable, to_cacheable};
  let source = AssetSource::new("cached map 😀").unwrap();
  let CacheableSource::Asset {
    mut compressed,
    size,
    content_hash,
  } = to_cacheable(&source)
  else {
    unreachable!()
  };
  // Every truncated prefix must fail, even if the decoded text is complete and
  // only the zlib checksum is missing.
  for end in 0..compressed.len() {
    assert!(
      from_cacheable(CacheableSource::Asset {
        compressed: compressed[..end].to_vec(),
        size,
        content_hash,
      })
      .is_err()
    );
  }
  let last = compressed.last_mut().unwrap();
  *last ^= 1;
  assert!(
    from_cacheable(CacheableSource::Asset {
      compressed,
      size,
      content_hash,
    })
    .is_err()
  );
  let mut trailing = source.compressed_buffer().to_vec();
  trailing.push(0);
  assert!(
    from_cacheable(CacheableSource::Asset {
      compressed: trailing,
      size,
      content_hash,
    })
    .is_err()
  );
  assert!(
    from_cacheable(CacheableSource::Asset {
      compressed: source.compressed_buffer().to_vec(),
      size: size + 1,
      content_hash,
    })
    .is_err()
  );
}

#[test]
fn test_asset_preserves_mapping_source_json() {
  use rspack_sources::{MappingSource, SourceMap};

  let map = SourceMap::from_json(
    r#"{"version":3,"sources":["😀\".js"],"sourcesContent":["a\nb"],"names":[],"mappings":"AAAA"}"#
      .to_owned(),
  )
  .unwrap();
  let mapping = MappingSource::new(map);
  let mut observed = Vec::new();
  let asset =
    AssetSource::from_source_with_observer(&mapping, |bytes| observed.extend_from_slice(bytes))
      .unwrap();
  assert_eq!(observed, mapping.buffer().as_ref());
  assert_eq!(asset.buffer(), mapping.buffer());
  assert_eq!(asset.size(), mapping.size());

  let empty = AssetSource::new("").unwrap();
  let mut reader = empty.reader().unwrap();
  assert_eq!(reader.read(&mut [0; 1]).unwrap(), 0);
  let cached = to_bytes(&Data(empty.boxed()), &()).unwrap();
  let restored: Data = from_bytes(&cached, &()).unwrap();
  assert_eq!(restored.0.size(), 0);
}
