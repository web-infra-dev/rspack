use rspack_paths::{RspackPath, RspackResource, Utf8Path};

#[test]
fn parses_posix_absolute_path_as_file_url() {
  let path = RspackPath::from_utf8_path(Utf8Path::new("/tmp/rspack path/a.js")).expect("file URL");

  assert_eq!(path.to_request_string(), "file:///tmp/rspack%20path/a.js");
  assert_eq!(
    path.as_file_path().expect("file path").as_str(),
    "/tmp/rspack path/a.js"
  );
}

#[test]
fn parses_windows_drive_path_as_file_url() {
  let path = RspackPath::from_request(r"C:\repo\src\index.js", None).expect("windows path");

  assert_eq!(path.to_cache_key(), "file:///C:/repo/src/index.js");
}

#[test]
fn parses_unc_path_as_file_url() {
  let path = RspackPath::from_request(r"\\server\share\index.js", None).expect("UNC path");

  assert_eq!(path.to_cache_key(), "file://server/share/index.js");
}

#[test]
fn preserves_query_and_fragment_as_resource_parts() {
  let resource =
    RspackResource::from_request("https://example.com/a%20b.js?raw#frag", None).expect("resource");

  assert_eq!(
    resource.path.as_url().expect("url").as_str(),
    "https://example.com/a%20b.js"
  );
  assert_eq!(resource.query.as_deref(), Some("?raw"));
  assert_eq!(resource.fragment.as_deref(), Some("#frag"));
  assert_eq!(
    resource.to_request_string(),
    "https://example.com/a%20b.js?raw#frag"
  );
}

#[test]
fn joins_relative_request_against_absolute_url_base() {
  let base = RspackPath::from_request("https://example.com/pkg/index.js", None).expect("base");
  let path = RspackPath::from_request("./dep.js", Some(&base)).expect("joined path");

  assert_eq!(path.to_request_string(), "https://example.com/pkg/dep.js");
}

#[test]
fn keeps_relative_requests_compact() {
  let resource = RspackResource::from_request("./style.css?module#layer", None).expect("resource");

  assert_eq!(resource.path.to_request_string(), "./style.css");
  assert_eq!(resource.to_cache_key(), "./style.css?module#layer");
}

#[test]
fn shared_absolute_path_helpers_cover_posix_drive_and_unc() {
  assert!(RspackPath::is_absolute_request("/repo/src/index.js"));
  assert!(RspackPath::is_absolute_request(r"C:\repo\src\index.js"));
  assert!(RspackPath::is_absolute_request(r"\\server\share\index.js"));
  assert!(!RspackPath::is_absolute_request("C:drive-relative.js"));
  assert_eq!(
    RspackPath::from_path_str(r"C:\repo\src\index.js")
      .expect("path")
      .to_request_path_string(),
    "C:/repo/src/index.js"
  );
}
