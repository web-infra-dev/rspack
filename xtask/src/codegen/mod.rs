#![allow(clippy::unwrap_used)]

use std::{
  env, fs,
  path::{Path, PathBuf},
};

use anyhow::Result;
use clap::Args;

fn get_workspace_root() -> PathBuf {
  Path::new(
    &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
  )
  .ancestors()
  .nth(1)
  .unwrap()
  .to_path_buf()
}

fn run_impl() -> Result<()> {
  let workspace_root = get_workspace_root();
  let out_path = workspace_root.join("crates/rspack_workspace/src/generated.rs");

  generate_workspace_versions(&out_path.to_string_lossy())?;
  Ok(())
}

fn generate_workspace_versions(out_path: &str) -> Result<()> {
  let workspace_root = get_workspace_root();

  // Get SWC version
  let cargo_toml_path = workspace_root.join("Cargo.toml");
  let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;
  let manifest =
    cargo_toml::Manifest::from_str(&cargo_toml_content).expect("Should parse cargo toml");
  let workspace = manifest.workspace.unwrap();

  let workspace_version = workspace.package.unwrap().version.unwrap();

  let swc_core_version = workspace
    .dependencies
    .get("swc_core")
    .and_then(|dep| match dep {
      cargo_toml::Dependency::Simple(s) => Some(s.to_owned()),
      // In workspace Cargo.toml, dependencies should be Simple type, not Inherited
      cargo_toml::Dependency::Inherited(_) => unreachable!(),
      cargo_toml::Dependency::Detailed(d) => {
        // Remove leading '=' from version string if present
        d.version
          .as_deref()
          .map(|s| s.trim_start_matches('=').to_string())
      }
    })
    .expect("Should have `swc_core` version");

  // Get Rspack version
  let package_json_path = workspace_root.join("package.json");
  let package_json_content = fs::read_to_string(&package_json_path)?;
  let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;
  let rspack_version = package_json["version"]
    .as_str()
    .expect("version field in package.json is not a string");

  // Generate combined content
  let content = format!(
    r#"//! This is a generated file. Don't modify it by hand! Run 'cargo codegen' to re-generate the file.
/// The version of the `swc_core` package used in the current workspace.
pub const fn rspack_swc_core_version() -> &'static str {{
  "{swc_core_version}"
}}

/// The version of the JavaScript `@rspack/core` package.
pub const fn rspack_pkg_version() -> &'static str {{
  "{rspack_version}"
}}

/// The version of the Rust workspace in the root `Cargo.toml` of the repository.
pub const fn rspack_workspace_version() -> &'static str {{
  "{workspace_version}"
}}
"#
  );
  fs::write(out_path, content)?;
  Ok(())
}

#[derive(Debug, Args)]
pub(crate) struct CodegenCmd;

impl CodegenCmd {
  pub(crate) fn run(self) -> Result<()> {
    run_impl()
  }
}
