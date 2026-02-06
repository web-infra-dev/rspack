use std::{fs, path::Path};

use clap::Args;
use toml::Value;

#[derive(Debug, Args)]
pub(crate) struct ReleaseCheckCmd;
impl ReleaseCheckCmd {
  pub(crate) fn run(self) -> anyhow::Result<()> {
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
        let cargo_toml: Value = toml::from_str(&content)?;
        if let Some(package) = cargo_toml.get("package") {
          let mut has_optional = false;
          for field in &required_fields {
            if optional_fields.contains(field) {
              if package.get(field).is_some() {
                has_optional = true
              }
            } else if package.get(field).is_none() {
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
