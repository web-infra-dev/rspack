use std::{fs, path::Path};

use miette::{miette, Diagnostic, IntoDiagnostic, Report, Result};
use thiserror::Error;
use toml::Value;
fn main() -> Result<()> {
  let required_fields = vec![
    "description",
    "version",
    "license",
    "repository",
    "homepage",
    "documentation",
  ];
  let optional_fields = ["repository", "homepage", "documentation"];
  let mut diagnostics: Vec<Report> = vec![];
  // iterate all crate in crates folder
  let crates_dir = Path::new("crates");

  for entry in fs::read_dir(crates_dir).into_diagnostic()? {
    let entry = entry.into_diagnostic()?;
    let path = entry.path();

    if path.is_dir() {
      let cargo_toml_path = path.join("Cargo.toml");
      if cargo_toml_path.exists() {
        let content = fs::read_to_string(&cargo_toml_path).into_diagnostic()?;
        let cargo_toml: Value = toml::from_str(&content).into_diagnostic()?;
        if let Some(package) = cargo_toml.get("package") {
          let mut has_optional = false;
          for field in &required_fields {
            if optional_fields.contains(field) {
              if package.get(field).is_some() {
                has_optional = true
              }
            } else if package.get(field).is_none() {
              diagnostics.push(miette!(
                "Missing required field: {} in {}",
                field,
                cargo_toml_path.display(),
              ));
            }
          }
          if !has_optional {
            let msg = optional_fields.join(",");
            let error = miette!(
              "You have at least have one of these fields {} section in {}",
              msg,
              cargo_toml_path.display()
            );
            diagnostics.push(error);
          }
        } else {
          diagnostics.push(miette!("Missing [package] section"));
        }
      }
    }
  }
  #[derive(Debug, Diagnostic, Error)]
  #[error("release-check failed")]
  struct ReleaseCheckError {
    #[related]
    others: Vec<Report>,
  }
  if diagnostics.is_empty() {
    Ok(())
  } else {
    let error = ReleaseCheckError {
      others: diagnostics,
    }
    .into();
    Err(error)
  }
}
