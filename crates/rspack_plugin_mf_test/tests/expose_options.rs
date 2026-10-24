use rspack_plugin_mf::EnhancedExposeOptions;

#[test]
fn expose_options_serialize_to_the_webpack_identifier_payload() {
  let plain = (
    "./a",
    EnhancedExposeOptions {
      name: None,
      import: vec!["./a.js".into()],
      layer: None,
    },
  );
  assert_eq!(
    simd_json::to_string(&[&plain]).expect("valid JSON"),
    r#"[["./a",{"name":null,"import":["./a.js"]}]]"#
  );
  let layered = (
    "./b",
    EnhancedExposeOptions {
      name: Some("b".into()),
      import: vec!["./b.js".into()],
      layer: Some("server".into()),
    },
  );
  assert_eq!(
    simd_json::to_string(&[&layered]).expect("valid JSON"),
    r#"[["./b",{"name":"b","import":["./b.js"],"layer":"server"}]]"#
  );
}
