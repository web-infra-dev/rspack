pub fn rspack_version() -> String {
  let re = regex::Regex::new(r#""version": ?"([0-9a-zA-Z\.-]+)""#).expect("should create regex");
  // package.json in project root directory
  let package_json = include_str!("../../../package.json");
  let version = re
    .captures(package_json)
    .expect("can not found version field in project package.json");
  version[1].to_string()
}
