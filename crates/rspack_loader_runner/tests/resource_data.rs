use rspack_loader_runner::ResourceData;

#[test]
fn resource_data_populates_typed_path_for_absolute_path() {
  let resource = ResourceData::new_with_resource("/tmp/rspack path/a.js?raw#frag".to_string());
  let typed = resource.typed_path().expect("typed path");

  assert_eq!(
    typed.to_request_string(),
    "file:///tmp/rspack%20path/a.js?raw#frag"
  );
  assert_eq!(resource.resource(), "/tmp/rspack path/a.js?raw#frag");
  assert_eq!(resource.query(), Some("?raw"));
  assert_eq!(resource.fragment(), Some("#frag"));
}

#[test]
fn resource_data_populates_typed_path_for_http_url() {
  let resource =
    ResourceData::new_with_resource("https://example.com/pkg/a.js?raw#frag".to_string());
  let typed = resource.typed_path().expect("typed path");

  assert_eq!(
    typed.to_request_string(),
    "https://example.com/pkg/a.js?raw#frag"
  );
  assert_eq!(
    resource.path().expect("path").as_str(),
    "https://example.com/pkg/a.js"
  );
  assert_eq!(resource.query(), Some("?raw"));
  assert_eq!(resource.fragment(), Some("#frag"));
}
