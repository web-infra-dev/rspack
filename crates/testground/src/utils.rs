pub fn assert_inline_sourcemap_in_pos(
  dist_code: &str,
  line_in_dist: u32,
  column_in_dist: u32,
  expected_in_source: &str,
) {
  const DATA_PREAMBLE: &str = "data:application/json;charset=utf-8;base64,";
  // TODO: should find last DATA_PREAMBLE.
  let index = dist_code.find(DATA_PREAMBLE).unwrap();
  let data_b64 = &dist_code[index + DATA_PREAMBLE.len()..];
  let data = base64::decode(data_b64).unwrap();
  let decoded_map = sourcemap::decode_slice(&data).unwrap();
  let token = decoded_map
    .lookup_token(line_in_dist, column_in_dist)
    .unwrap();
  let source_view = token.get_source_view().unwrap();
  let actual = source_view
    .get_line_slice(
      token.get_src_line(),
      token.get_src_col(),
      expected_in_source.len() as u32,
    )
    .expect("failed to assert sourcemap");
  assert_eq!(actual, expected_in_source);
}

pub fn get_inline_source_map(dist_code: &str) -> sourcemap::SourceMap {
  const DATA_PREAMBLE: &str = "data:application/json;charset=utf-8;base64,";
  // TODO: should find last DATA_PREAMBLE.
  let index = dist_code.find(DATA_PREAMBLE).unwrap();
  let data_b64 = &dist_code[index + DATA_PREAMBLE.len()..];
  let data = base64::decode(data_b64).unwrap();

  sourcemap::SourceMap::from_slice(&data).unwrap()
}
