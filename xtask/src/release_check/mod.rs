use std::{fs, path::Path};

use cargo_toml::{Inheritable, Manifest};
use clap::Args;

#[derive(Debug, Args)]
pub struct ReleaseCheckCmd;
impl ReleaseCheckCmd {
  pub fn run(self) -> anyhow::Result<()> {
    run_inner()
  }
}
fn run_inner() -> anyhow::Result<()> {
  let required_fields = vec![
    "description",
    "version",
    "license",
    "repository",
    "homepage",
    "documentation",
  ];
  let optional_fields = ["repository", "homepage", "documentation"];
  let mut diagnostics: Vec<anyhow::Error> = vec![];
  // iterate all crate in crates folder
  let crates_dir = Path::new("crates");

  for entry in fs::read_dir(crates_dir)? {
    let entry = entry?;
    let path = entry.path();

    if path.is_dir() {
      let cargo_toml_path = path.join("Cargo.toml");
      if cargo_toml_path.exists() {
        let content = fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml = Manifest::from_str(&content)?;
        if let Some(package) = cargo_toml.package {
          let mut has_optional = false;
          for field in &required_fields {
            if optional_fields.contains(field) {
              if has_package_field(&package, field) {
                has_optional = true
              }
            } else if !has_package_field(&package, field) {
              diagnostics.push(anyhow::anyhow!(
                "Missing required field: {} in {}",
                field,
                cargo_toml_path.display(),
              ));
            }
          }
          if !has_optional {
            let msg = optional_fields.join(",");
            let error = anyhow::anyhow!(
              "You have at least have one of these fields {} section in {}",
              msg,
              cargo_toml_path.display()
            );
            diagnostics.push(error);
          }
        } else {
          diagnostics.push(anyhow::anyhow!("Missing [package] section"));
        }
      }
    }
  }
  if diagnostics.is_empty() {
    Ok(())
  } else {
    let combined = diagnostics
      .into_iter()
      .map(|e| format!("{e:?}"))
      .collect::<Vec<_>>()
      .join("\n");
    Err(anyhow::anyhow!("Multiple errors occurred:\n{combined}"))
  }
}

fn has_package_field(package: &cargo_toml::Package, field: &str) -> bool {
  match field {
    "description" => package.description.is_some(),
    "version" => {
      matches!(package.version, Inheritable::Inherited)
        || package
          .version
          .get()
          .is_ok_and(|version| version != "0.0.0")
    }
    "license" => package.license.is_some(),
    "repository" => package.repository.is_some(),
    "homepage" => package.homepage.is_some(),
    "documentation" => package.documentation.is_some(),
    _ => false,
  }
}
