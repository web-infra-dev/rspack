use rspack_loader_runner::parse_resource;

#[test]
fn parses_windows_dos_device_paths() {
  let cases = [
    (r"\\?\C:\long\resource.js", r"\\?\C:\long\resource.js"),
    (r"\\.\C:\device\resource.js", r"\\.\C:\device\resource.js"),
    (
      r"\\?\UNC\server\share\resource.js",
      r"\\?\UNC\server\share\resource.js",
    ),
    (
      "\\\\?\\C:\\escaped\u{200b}#name.js",
      r"\\?\C:\escaped#name.js",
    ),
  ];

  for (resource_path, expected_path) in cases {
    let resource = format!("{resource_path}?resource-query#fragment");
    let parsed = parse_resource(&resource).expect("resource should parse");
    assert_eq!(parsed.path.as_str(), expected_path);
    assert_eq!(parsed.query.as_deref(), Some("?resource-query"));
    assert_eq!(parsed.fragment.as_deref(), Some("#fragment"));
  }
}
