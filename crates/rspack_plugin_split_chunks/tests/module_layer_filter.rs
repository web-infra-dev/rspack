use rspack_plugin_split_chunks::{ModuleLayerFilter, create_default_module_layer_filter};

#[test]
fn default_layer_filter_is_sync_and_matches_everything() {
  let filter = create_default_module_layer_filter();
  assert!(!filter.is_func());
  assert!(filter.test_internal(None));
  assert!(filter.test_internal(Some("any-layer")));
}

#[test]
fn string_and_regex_layer_filters_match_without_async() {
  let string_filter = ModuleLayerFilter::String("app".to_string());
  assert!(!string_filter.is_func());
  assert!(string_filter.test_internal(Some("app/shared")));
  assert!(!string_filter.test_internal(Some("other/shared")));

  let regex_filter = ModuleLayerFilter::Regex(
    rspack_regex::RspackRegex::with_flags("^app", "").expect("regex should compile"),
  );
  assert!(!regex_filter.is_func());
  assert!(regex_filter.test_internal(Some("app/shared")));
  assert!(!regex_filter.test_internal(Some("other/shared")));
}
