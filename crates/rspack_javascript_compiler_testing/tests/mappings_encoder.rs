use rspack_sources::{FullMappingsEncoder, Mapping, OriginalLocation, encode_mappings};

#[test]
fn incremental_encoding_matches_iterator_encoding() {
  for length in [0, 1, 100, 10_000] {
    let mappings = (0..length).map(|index| Mapping {
      generated_line: 1 + index / 32,
      generated_column: (index % 32) * 2,
      original: (index % 5 != 0).then_some(OriginalLocation {
        source_index: index % 3,
        original_line: 1 + index % 50,
        original_column: index % 20,
        name_index: (index % 7 == 0).then_some(index % 11),
      }),
    });
    let expected = encode_mappings(mappings.clone());
    let mut encoder = FullMappingsEncoder::default();
    for mapping in mappings {
      encoder.encode(&mapping);
    }
    assert_eq!(encoder.finish(), expected);
  }
}
