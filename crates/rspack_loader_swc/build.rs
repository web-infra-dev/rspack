use std::{env, fs, path::Path};

fn main() {
  let workspace_root =
    env::var("CARGO_WORKSPACE_DIR").expect("CARGO_WORKSPACE_DIR environment variable not set");

  let cargo_toml_path = Path::new(&workspace_root).join("Cargo.toml");

  // Read the Cargo.toml file from workspace root
  let cargo_toml_content =
    fs::read_to_string(&cargo_toml_path).expect("Should read workspace Cargo.toml");

  let workspace_toml = cargo_toml::Manifest::from_str(&cargo_toml_content)
    .expect("Should parse cargo toml")
    .workspace;
  let swc_core_version = workspace_toml
    .as_ref()
    .and_then(|ws| {
      ws.dependencies.get("swc_core").and_then(|dep| match dep {
        cargo_toml::Dependency::Simple(s) => Some(&**s),
        cargo_toml::Dependency::Inherited(_) => unreachable!(),
        cargo_toml::Dependency::Detailed(d) => d.version.as_deref(),
      })
    })
    .expect("Should have `swc_core` version")
    .to_owned();
  println!("cargo::rustc-env=RSPACK_SWC_CORE_VERSION={swc_core_version}");

  // Tell cargo to rerun this build script if Cargo.toml changes
  println!("cargo:rerun-if-changed={}", cargo_toml_path.display());
}
