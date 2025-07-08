use std::{env, fs, path::Path};

fn main() {
  let workspace_root =
    env::var("CARGO_WORKSPACE_DIR").expect("CARGO_WORKSPACE_DIR environment variable not set");

  let package_json_path = Path::new(&workspace_root).join("package.json");

  if !package_json_path.exists() {
    panic!(
      "package.json not found at workspace root: {}",
      package_json_path.display()
    );
  }

  let package_json_content =
    fs::read_to_string(&package_json_path).expect("Could not read package.json");

  let package_json: serde_json::Value =
    serde_json::from_str(&package_json_content).expect("Could not parse package.json");

  let version = package_json["version"]
    .as_str()
    .expect("version field in package.json is not a string");

  println!("cargo:rustc-env=RSPACK_VERSION={version}");
  println!("cargo:rerun-if-changed={}", package_json_path.display());
}
