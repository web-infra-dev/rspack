pub fn rspack_version() -> String {
  // package.json in project root directory
  let package_json = include_str!("../../../package.json");
  let mut pkg =
    json::parse(package_json).expect("can not parse package.json in project root directory");
  let Some(version) = pkg["version"].take_string() else {
    panic!("version field in package.json is not a string");
  };
  version
}
