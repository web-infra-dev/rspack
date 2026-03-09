use std::{fs, path::Path};

use anyhow::Context;
use cargo_toml::{Dependency, DepsSet, Inheritable, LintGroups, Manifest};
use clap::Args;
/// check every workspace dependencies has default-features=false
fn check_setting_default_features_false(workspace_deps: &DepsSet) -> Vec<String> {
  let mut errors = Vec::new();
  // Check each dependency for default-features=false
  for (dep_name, dep_value) in workspace_deps {
    match dep_value {
      Dependency::Detailed(detail) if !detail.default_features => {}
      _ => {
        errors.push(format!(
          "Dependency '{dep_name}' does not have default-features=false",
        ));
      }
    }
  }
  errors
}
/// enforce every workspace member to use workspace=true for their dependencies
fn enforce_workspace_version() -> anyhow::Result<()> {
  let workspace_root = find_workspace_root()?;
  let workspace_manifest_path = workspace_root.join("Cargo.toml");

  // Read and parse the workspace Cargo.toml
  let workspace_content = fs::read_to_string(&workspace_manifest_path)?;
  let workspace_toml = Manifest::from_str(&workspace_content)?;
  let mut errors = Vec::new();
  let workspace = workspace_toml
    .workspace
    .with_context(|| "No workspace section found")?;
  let workspace_deps = &workspace.dependencies;
  let default_features_errors = check_setting_default_features_false(workspace_deps);
  errors.extend(default_features_errors);
  let workspace_members = &workspace.members;

  let mut checked_crates = 0;
  let mut total_dependencies = 0;

  // Check each workspace member
  for member in workspace_members {
    // Skip if it's a glob pattern, we need to resolve it
    if member.contains('*') {
      let pattern_parts: Vec<&str> = member.split('/').collect();
      if pattern_parts.len() == 2 && pattern_parts[1] == "*" {
        // Handle patterns like "crates/*"
        let base_dir = workspace_root.join(pattern_parts[0]);
        if base_dir.is_dir() {
          for entry in fs::read_dir(&base_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
              let crate_path = entry.path();
              let manifest_path = crate_path.join("Cargo.toml");
              if manifest_path.exists() {
                let deps_count =
                  check_crate_dependencies(&manifest_path, workspace_deps, &mut errors)?;
                checked_crates += 1;
                total_dependencies += deps_count;
              }
            }
          }
        }
      }
    } else {
      // Direct member path
      let manifest_path = workspace_root.join(member).join("Cargo.toml");
      if manifest_path.exists() {
        let deps_count = check_crate_dependencies(&manifest_path, workspace_deps, &mut errors)?;
        checked_crates += 1;
        total_dependencies += deps_count;
      }
    }
  }

  if !errors.is_empty() {
    eprintln!("Found dependency version violations:");
    for error in errors {
      eprintln!("  ❌ {error}");
    }
    eprintln!(
      "\nSummary: {checked_crates} crates checked, {total_dependencies} dependencies analyzed"
    );
    anyhow::bail!("Some workspace dependency or lint settings are invalid");
  }

  println!("Summary: {checked_crates} crates checked, {total_dependencies} dependencies analyzed");
  println!("All workspace members correctly use workspace=true for dependencies and lints");
  Ok(())
}

fn check_crate_dependencies(
  manifest_path: &Path,
  workspace_deps: &DepsSet,
  errors: &mut Vec<String>,
) -> anyhow::Result<usize> {
  let content = fs::read_to_string(manifest_path)?;
  let manifest = Manifest::from_str(&content)?;

  let crate_name = manifest_path
    .parent()
    .and_then(|p| p.file_name())
    .and_then(|n| n.to_str())
    .unwrap_or("unknown");

  let mut deps_count = 0;

  check_crate_workspace_lints(&manifest.lints, crate_name, errors);

  // Check regular dependencies
  deps_count += check_dependency_section(
    &manifest.dependencies,
    workspace_deps,
    crate_name,
    "dependencies",
    errors,
  );

  // Check dev-dependencies
  deps_count += check_dependency_section(
    &manifest.dev_dependencies,
    workspace_deps,
    crate_name,
    "dev-dependencies",
    errors,
  );

  // Check build-dependencies
  deps_count += check_dependency_section(
    &manifest.build_dependencies,
    workspace_deps,
    crate_name,
    "build-dependencies",
    errors,
  );

  Ok(deps_count)
}

fn check_crate_workspace_lints(
  lints: &Inheritable<LintGroups>,
  crate_name: &str,
  errors: &mut Vec<String>,
) {
  match lints {
    Inheritable::Inherited => {}
    Inheritable::Set(groups) if groups.is_empty() => {
      errors.push(format!(
        "{crate_name}: missing [lints] section, expected [lints] workspace = true to enable clippy workspace lints"
      ));
    }
    Inheritable::Set(_) => {
      errors.push(format!(
        "{crate_name}: [lints] does not use workspace = true, expected workspace lints inheritance"
      ));
    }
  }
}

fn check_dependency_section(
  deps: &DepsSet,
  workspace_deps: &DepsSet,
  crate_name: &str,
  section_name: &str,
  errors: &mut Vec<String>,
) -> usize {
  let mut count = 0;

  for (dep_name, dep_value) in deps {
    // Skip if this dependency is not in workspace dependencies
    if !workspace_deps.contains_key(dep_name) {
      continue;
    }

    count += 1;

    match dep_value {
      Dependency::Simple(_) => {
        // Simple version string - should use workspace=true
        errors.push(format!(
          "{crate_name} [{section_name}]: dependency '{dep_name}' uses version string instead of workspace=true"
        ));
      }
      Dependency::Detailed(detail) => {
        if detail.version.is_some() {
          // Has version but no workspace=true
          errors.push(format!(
            "{crate_name} [{section_name}]: dependency '{dep_name}' specifies version instead of workspace=true"
          ));
        }
        // If it has path but no version, it might be a local path dependency, which is OK
      }
      Dependency::Inherited(_) => {}
    }
  }

  count
}

fn find_workspace_root() -> anyhow::Result<std::path::PathBuf> {
  let mut current_dir = std::env::current_dir()?;
  loop {
    let manifest_path = current_dir.join("Cargo.toml");
    if manifest_path.exists() {
      let content = fs::read_to_string(&manifest_path)?;
      let manifest = Manifest::from_str(&content)?;

      // Check if this is a workspace root
      if manifest.workspace.is_some() {
        return Ok(current_dir);
      }
    }

    // Move to parent directory
    if let Some(parent) = current_dir.parent() {
      current_dir = parent.to_path_buf();
    } else {
      break;
    }
  }

  anyhow::bail!("Could not find workspace root")
}

#[derive(Debug, Args)]
pub struct DenyExtCmd;

impl DenyExtCmd {
  pub fn run(self) -> anyhow::Result<()> {
    println!("Checking workspace dependencies...");

    enforce_workspace_version()?;

    println!("✅ All checks passed!");
    Ok(())
  }
}
