use std::{
  env, fs,
  path::{Path, PathBuf},
};

use anyhow::Result;

fn get_workspace_root() -> PathBuf {
  Path::new(
    &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
  )
  .ancestors()
  .nth(2)
  .unwrap()
  .to_path_buf()
}

fn main() -> Result<()> {
  let mut args = env::args().skip(1);
  let task = args.next().expect("task not specified");
  let out_path = args.next().expect("out_path not specified");

  match task.as_str() {
    "swc-version" => generate_swc_version(&out_path)?,
    "rspack-version" => generate_rspack_version(&out_path)?,
    "workspace-versions" => generate_workspace_versions(&out_path)?,
    _ => panic!("unknown task"),
  }
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
    "
pub fn rspack_swc_core_version() -> &'static str {{
  \"{swc_core_version}\"
}}

pub fn rspack_version() -> &'static str {{
  \"{rspack_version}\"
}}
"
  );
  fs::write(out_path, content)?;
  Ok(())
}

fn generate_swc_version(out_path: &str) -> Result<()> {
  let workspace_root = get_workspace_root();
  let cargo_toml_path = workspace_root.join("Cargo.toml");

  let cargo_toml_content = fs::read_to_string(&cargo_toml_path)?;

  let manifest =
    cargo_toml::Manifest::from_str(&cargo_toml_content).expect("Should parse cargo toml");

  let workspace = manifest.workspace.unwrap();
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

  let content = format!(
    "
pub fn rspack_swc_core_version() -> &'static str {{
  \"{swc_core_version}\"
}}
"
  );
  fs::write(out_path, content)?;
  Ok(())
}

fn generate_rspack_version(out_path: &str) -> Result<()> {
  let workspace_root = get_workspace_root();
  let package_json_path = workspace_root.join("package.json");
  let package_json_content = fs::read_to_string(&package_json_path)?;
  let package_json: serde_json::Value = serde_json::from_str(&package_json_content)?;

  let version = package_json["version"]
    .as_str()
    .expect("version field in package.json is not a string");

  let content = format!(
    "
pub fn rspack_version() -> &'static str {{
  \"{version}\"
}}
"
  );
  fs::write(out_path, content)?;

  Ok(())
}
