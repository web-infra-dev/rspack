use std::{fs, path::Path};

use anyhow::Context;
use clap::Args;
/// check every workspace dependencies has default-features=false
fn check_setting_default_features_false(
  workspace_deps: &toml::map::Map<String, toml::Value>,
) -> Vec<String> {
  let mut errors = Vec::new();
  // Check each dependency for default-features=false
  for (dep_name, dep_value) in workspace_deps {
    if let Some(table) = dep_value.as_table()
      && (!table.contains_key("default-features")
        || table.get("default-features") != Some(&toml::Value::Boolean(false)))
    {
      errors.push(format!(
        "Dependency '{dep_name}' does not have default-features=false",
      ));
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
  let workspace_toml: toml::Value = toml::from_str(&workspace_content)?;
  let mut errors = Vec::new();
  // Get workspace dependencies
  let workspace_deps = workspace_toml
    .get("workspace")
    .and_then(|w| w.get("dependencies"))
    .and_then(|d| d.as_table())
    .with_context(|| "No workspace dependencies found")?;
  let default_features_errors = check_setting_default_features_false(workspace_deps);
  errors.extend(default_features_errors);
  // Get workspace members
  let workspace_members = workspace_toml
    .get("workspace")
    .and_then(|w| w.get("members"))
    .and_then(|m| m.as_array())
    .with_context(|| "No workspace members found")?;

  let mut checked_crates = 0;
  let mut total_dependencies = 0;

  // Check each workspace member
  for member in workspace_members {
    let member_path = member.as_str().with_context(|| "Invalid member path")?;

    // Skip if it's a glob pattern, we need to resolve it
    if member_path.contains('*') {
      let pattern_parts: Vec<&str> = member_path.split('/').collect();
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
      let manifest_path = workspace_root.join(member_path).join("Cargo.toml");
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
    anyhow::bail!("Some dependencies are not using workspace=true");
  }

  println!("Summary: {checked_crates} crates checked, {total_dependencies} dependencies analyzed");
  println!("All workspace members correctly use workspace=true for their dependencies");
  Ok(())
}

fn check_crate_dependencies(
  manifest_path: &Path,
  workspace_deps: &toml::map::Map<String, toml::Value>,
  errors: &mut Vec<String>,
) -> anyhow::Result<usize> {
  let content = fs::read_to_string(manifest_path)?;
  let toml_value: toml::Value = toml::from_str(&content)?;

  let crate_name = manifest_path
    .parent()
    .and_then(|p| p.file_name())
    .and_then(|n| n.to_str())
    .unwrap_or("unknown");

  let mut deps_count = 0;

  // Check regular dependencies
  if let Some(deps) = toml_value.get("dependencies").and_then(|d| d.as_table()) {
    deps_count +=
      check_dependency_section(deps, workspace_deps, crate_name, "dependencies", errors);
  }

  // Check dev-dependencies
  if let Some(dev_deps) = toml_value
    .get("dev-dependencies")
    .and_then(|d| d.as_table())
  {
    deps_count += check_dependency_section(
      dev_deps,
      workspace_deps,
      crate_name,
      "dev-dependencies",
      errors,
    );
  }

  // Check build-dependencies
  if let Some(build_deps) = toml_value
    .get("build-dependencies")
    .and_then(|d| d.as_table())
  {
    deps_count += check_dependency_section(
      build_deps,
      workspace_deps,
      crate_name,
      "build-dependencies",
      errors,
    );
  }

  Ok(deps_count)
}

fn check_dependency_section(
  deps: &toml::map::Map<String, toml::Value>,
  workspace_deps: &toml::map::Map<String, toml::Value>,
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
      toml::Value::String(_) => {
        // Simple version string - should use workspace=true
        errors.push(format!(
          "{crate_name} [{section_name}]: dependency '{dep_name}' uses version string instead of workspace=true"
        ));
      }
      toml::Value::Table(table) => {
        // Check if it uses workspace=true
        if let Some(workspace_val) = table.get("workspace") {
          if let Some(workspace_bool) = workspace_val.as_bool() {
            if !workspace_bool {
              errors.push(format!(
                "{crate_name} [{section_name}]: dependency '{dep_name}' has workspace=false, should be workspace=true"
              ));
            }
          } else {
            errors.push(format!(
              "{crate_name} [{section_name}]: dependency '{dep_name}' has invalid workspace value, should be workspace=true"
            ));
          }
        } else if table.contains_key("version") {
          // Has version but no workspace=true
          errors.push(format!(
            "{crate_name} [{section_name}]: dependency '{dep_name}' specifies version instead of workspace=true"
          ));
        }
        // If it has path but no version, it might be a local path dependency, which is OK
      }
      _ => {
        errors.push(format!(
          "{crate_name} [{section_name}]: dependency '{dep_name}' has invalid format"
        ));
      }
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
      let toml_value: toml::Value = toml::from_str(&content)?;

      // Check if this is a workspace root
      if toml_value.get("workspace").is_some() {
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
pub(crate) struct DenyExtCmd;

impl DenyExtCmd {
  pub(crate) fn run(self) -> anyhow::Result<()> {
    println!("Checking workspace dependencies...");

    enforce_workspace_version()?;

    println!("✅ All checks passed!");
    Ok(())
  }
}
