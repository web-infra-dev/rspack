pub fn rspack_version() -> String {
  // Use the version set by the build script from workspace root package.json
  // This ensures the version is read from the correct package.json file
  // regardless of the build environment (regular build or publish)
  env!("RSPACK_VERSION").to_string()
}
