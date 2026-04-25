use rspack_plugin_split_chunks::{ModuleTypeFilter, create_default_module_type_filter};

#[test]
fn default_module_type_filter_is_sync_and_matches_everything() {
  let filter = create_default_module_type_filter();
  assert!(filter.test_internal("javascript/auto"));
  assert!(filter.test_internal("css"));
}

#[test]
fn string_and_regex_module_type_filters_match_synchronously() {
  let string_filter = ModuleTypeFilter::String("css".to_string());
  assert!(string_filter.test_internal("css"));
  assert!(!string_filter.test_internal("javascript/auto"));

  let regex_filter = ModuleTypeFilter::Regex(
    rspack_regex::RspackRegex::with_flags("^javascript", "").expect("regex should compile"),
  );
  assert!(regex_filter.test_internal("javascript/auto"));
  assert!(!regex_filter.test_internal("css"));
}
